# MockProvider Test Harness

`MockProvider` exists to make approval/resume behavior deterministic and testable without Gemini, network, or API keys.

The MVP vertical slice is:

```text
CLI + MockProvider + write_file + approval pause/resume
```

`MockProvider` is a scripted model simulator, not a fake Gemini clone.

## Purpose

`MockProvider` should test:

- approval pause
- approval denial
- exactly-once tool execution
- provider failure after tool execution
- resume without rerunning tool
- model-visible tool result delivery

It must not:

- classify tools
- execute tools
- generate realistic prose
- simulate Gemini wire format
- handle OAuth/connectors
- inspect filesystem directly

It only:

- receives `ModelRequest`
- records it
- returns scripted `ModelResponse` or `ProviderError`

## Provider Trait Target

```rust
#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate(
        &self,
        request: ModelRequest,
    ) -> Result<ModelResponse, ProviderError>;
}
```

`ModelRequest` includes:

- `request_id`
- `turn_id`
- `provider_id`
- `model_id`
- `messages: Vec<ModelMessage>`
- `tools: Vec<ModelToolDeclaration>`
- `generation: GenerationConfig`

`ModelResponse` should support:

- `Text { content }`
- `ToolCalls { calls }`

For MVP, support one tool call at a time. Multi-tool-call turns can come later.

## MockProvider Shape

```rust
pub struct MockProvider {
    steps: Mutex<VecDeque<MockProviderStep>>,
    calls: Mutex<Vec<ModelRequest>>,
}
```

`MockProviderStep`:

- `Text { content }`
- `ToolCall { id, name, args }`
- `Error { kind, message, retryable }`
- `AssertThen { assertion, next }`

Useful helper constructors:

- `MockProviderStep::text(...)`
- `MockProviderStep::tool_call(name, args)`
- `MockProviderStep::provider_timeout(...)`
- `MockProviderStep::assert_last_tool_result_then_text(tool_name, expected_ok, content)`

## Script Examples

Happy path:

```text
[
  tool_call("write_file", {
    "path": "notes.txt",
    "mode": "create_new",
    "content": "hello world"
  }),
  assert_last_tool_result_then_text(
    "write_file",
    true,
    "Created notes.txt with hello world."
  )
]
```

Provider failure after execution:

```text
[
  tool_call("write_file", {
    "path": "notes.txt",
    "mode": "create_new",
    "content": "hello world"
  }),
  provider_timeout("simulated timeout after tool execution"),
  assert_last_tool_result_then_text(
    "write_file",
    true,
    "Created notes.txt with hello world."
  )
]
```

Denial:

```text
[
  tool_call("write_file", {
    "path": "notes.txt",
    "mode": "create_new",
    "content": "hello world"
  }),
  assert_last_tool_result_then_text(
    "write_file",
    false,
    "I did not create the file because you denied approval."
  )
]
```

Recovery expectation:

1. Initial provider call returns a tool call.
2. Post-tool continuation fails.
3. Resume continuation succeeds.

`write_file` must execute exactly once.

## Recorded Calls

`MockProvider` should record every `ModelRequest`.

Expose assertions:

- `call_count()`
- `calls()`
- `last_request()`
- `last_request_contains_tool_result(tool_name, ok)`
- `count_tool_results(tool_name)`
- `last_request_contains_error_code(tool_name, code)`

## Engine Test Harness

Target shape:

```rust
pub struct EngineTestHarness {
    pub temp_dir: TempDir,
    pub engine: OpenNivaraEngine,
    pub provider: Arc<MockProvider>,
    pub state: StateHandle,
}
```

Builder:

```rust
EngineTestHarness::new()
    .with_mock_provider(script)
    .with_temp_workspace()
    .with_temp_config_dir()
    .build()
```

Harness should set:

- `OPENNIVARA_TEST_CONFIG_DIR`
- temp workspace/current cwd
- tools config with `write_file` enabled
- `MockProvider` as model provider
- isolated state DB

## Tool Execution Counter

Use a test-only counting wrapper around tool execution.

Example:

```rust
pub struct CountingToolExecutor {
    inner: ToolRegistry,
    counts: Mutex<HashMap<String, usize>>,
}
```

Assertions:

- `tool_execution_count("write_file") == 1`
- filesystem content is correct

Do not rely only on final file content, because identical duplicate writes would be invisible.

## MVP Tests

`approval_write_file_happy_path_executes_once`:

1. User asks to create `notes.txt`.
2. `MockProvider` returns `write_file`.
3. Engine returns `ApprovalRequired`.
4. File does not exist yet.
5. `approve_pending_operation`.
6. File exists with `hello world`.
7. Provider receives `ok = true` tool result.
8. Final answer is stored.
9. Approval status is `completed`.
10. Pending turn is deleted.
11. Tool execution count is 1.

`approval_write_file_denial_does_not_execute_tool`:

1. Engine returns `ApprovalRequired`.
2. `deny_pending_operation`.
3. File does not exist.
4. Provider receives `ok = false` and `approval_denied`.
5. Final denial explanation is stored.
6. Pending turn is deleted.
7. Tool execution count is 0.

`approval_write_file_provider_failure_can_continue_without_rerun`:

1. Approve.
2. `write_file` executes once.
3. Provider fails after tool result.
4. Approval status is `executed`.
5. Pending turn remains.
6. `last_resume_error` is set.
7. Continue.
8. Provider returns final answer.
9. File content is still `hello world`.
10. Tool execution count is still 1.
11. Approval status is `completed`.
12. Pending turn is deleted.

`approval_write_file_duplicate_approve_does_not_rerun`:

1. Create pending approval.
2. Approve once.
3. Attempt approve again.
4. Second result is `already_executing`, `already_executed`, or `already_completed`.
5. Tool execution count is 1.

`write_file_preview_create_new_does_not_create_file`:

1. Call preview.
2. Assert file is missing.
3. Assert preview kind is `text_write`.

## Out Of MVP Harness

- real Gemini
- Desktop React
- Telegram bot
- GitHub connector
- memory tools
- `run_command`
- binary file writes
- concurrent multi-tool-call turns

## Locked Decisions

1. `MockProvider` is scripted, not a fake Gemini clone.
2. `MockProvider` records every `ModelRequest`.
3. `MockProvider` supports text, tool call, provider error, and assertion steps.
4. MVP tests use `MockProvider`, not Gemini.
5. MVP tests use temp config dir and temp workspace.
6. MVP tests include a `write_file` execution counter.
7. First happy path uses `write_file create_new`.
8. Recovery test proves continue does not rerun `write_file`.
9. Denial test proves `approval_denied` tool result reaches provider.
10. `MockProvider` does not classify/execute tools; engine does.

## Tests

Required tests:

1. All MVP tests above pass.
2. Provider call count assertions work.
3. Tool result shape assertions work.
4. Provider failure/retry assertions work.
5. `MockProvider` records every `ModelRequest`.
