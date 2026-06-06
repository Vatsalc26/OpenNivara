# Model Provider Gateway

This document defines the thin model gateway/provider abstraction OpenNivara should introduce before serializing pending turn state for approval resume.

The goal is narrow: prevent engine state and approval resume payloads from being locked to Gemini-native `Content`, `Part`, and `function_call` structs. This is not a provider marketplace, routing layer, or local-model project yet.

Provider errors should be sanitized and classified through [Error Taxonomy](error-taxonomy.md).

## Current Engine Context

`src/engine.rs` currently defines Gemini-shaped structs directly:

- `Content`
- `Part`
- `FunctionCall`
- `FunctionResponse`
- `GeminiRequest`
- `Tool`
- `FunctionDeclaration`
- `GeminiResponse`
- `Candidate`
- `ApiErrorDetail`

It also:

- hardcodes `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent`
- creates `reqwest::Client` directly
- sends the Gemini HTTP request directly
- parses Gemini JSON directly
- stores Gemini-shaped history in the in-memory tool loop

Approval resume requires storing pending model turn state. If that state stores Gemini-native structs, the resume system becomes tightly coupled to Gemini and harder to evolve for OpenAI-compatible APIs, Ollama, local models, or future providers.

## Implementation Order

Use this order:

1. State DB schema/migrations.
2. Provider abstraction/model gateway.
3. Engine approval integration.

The state schema establishes `pending_turns.resume_payload_json`. The provider abstraction ensures the payload stores OpenNivara-native model state. Engine approval integration can then use stable native model types.

## Target Layout

```text
src/model/
  mod.rs
  types.rs
  provider.rs
  gemini.rs
  mock.rs
```

Responsibilities:

- `types.rs`: OpenNivara-native model request, response, message, part, tool declaration, tool call, and generation config types.
- `provider.rs`: thin provider trait and provider selection helper.
- `gemini.rs`: Gemini-specific request/response structs, conversion code, HTTP call, endpoint, and provider error sanitization.
- `mock.rs`: deterministic provider for engine tests.

## Native Model Types

The engine should use OpenNivara-native model types:

```rust
pub enum ModelRole {
    User,
    Assistant,
    Tool,
    System,
}

pub struct ModelMessage {
    pub role: ModelRole,
    pub parts: Vec<ModelPart>,
}

pub enum ModelPart {
    Text {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
    },
    ToolResult {
        tool_call_id: String,
        name: String,
        result: serde_json::Value,
    },
}
```

Tool calls must have IDs. If a provider does not return a tool call ID, the provider adapter must generate a stable `toolcall_<uuid>` ID before returning `ModelResponse`. The ID is required for `pending_approvals.tool_call_id` and exact resume behavior. Request and turn ID ownership is defined in [Request And Turn Envelopes](request-turn-envelopes.md).

Provider-facing tool declarations contain only model-callable schema:

```rust
pub struct ModelToolDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

Do not expose `ToolRisk`, `OperationKind`, or `OperationClassification` to provider declarations. Those are engine/policy concepts.

`ModelToolCall`:

```rust
pub struct ModelToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}
```

`GenerationConfig` starts small:

```rust
pub struct GenerationConfig {
    pub max_tool_rounds: u32,
}
```

Future fields may include temperature, top-p, max output tokens, safety settings, and system instruction mode.

`ModelRequest`:

```rust
pub struct ModelRequest {
    pub request_id: String,
    pub turn_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub messages: Vec<ModelMessage>,
    pub tools: Vec<ModelToolDeclaration>,
    pub generation: GenerationConfig,
}
```

`ModelResponse`:

```rust
pub struct ModelResponse {
    pub message: ModelMessage,
    pub text: Option<String>,
    pub tool_calls: Vec<ModelToolCall>,
    pub raw_provider_response_json: Option<serde_json::Value>,
}
```

The provider should normalize assistant text into `text`, function/tool call parts into `tool_calls`, and the provider assistant response into `message`.

## Provider Trait

Use a thin async provider trait:

```rust
#[async_trait::async_trait]
pub trait ModelProvider {
    async fn generate(&self, request: ModelRequest) -> anyhow::Result<ModelResponse>;
}
```

If avoiding `async-trait` is preferred, use boxed futures or a concrete provider enum. The recommended path is `async-trait` for readability unless compile constraints argue otherwise.

Gemini remains the default provider for now:

- `provider_id = "gemini"`
- `model_id = "gemini-2.5-flash"`

## Gemini Provider

Move Gemini-specific structs and HTTP code from `src/engine.rs` to `src/model/gemini.rs`.

`GeminiProvider` responsibilities:

1. Convert `ModelRequest` to Gemini request JSON.
2. Convert `ModelMessage` and `ModelPart` to Gemini `Content` and `Part`.
3. Convert `ModelToolDeclaration` to Gemini function declarations.
4. Send HTTP request to Gemini `generateContent`.
5. Parse Gemini response JSON.
6. Convert Gemini response to `ModelResponse`.
7. Assign `tool_call_id` if Gemini does not provide one.
8. Sanitize API keys in provider errors.

Move `sanitize_provider_error` into `src/model/gemini.rs` or a small provider error utility.

## Mock Provider

Add `src/model/mock.rs` for deterministic tests.

`MockProvider` should simulate:

- plain text response
- tool call response
- second response after a tool result
- provider error
- generated tool call ID when a provider lacks one

## Engine Shape After Gateway

The engine tool loop becomes provider-neutral:

1. Build `Vec<ModelMessage>`.
2. Build `Vec<ModelToolDeclaration>`.
3. Call `provider.generate(ModelRequest)`.
4. Push `ModelResponse.message` into history.
5. If the response contains a tool call, classify the operation.
6. If automatic, execute the tool and push a `ModelRole::Tool` message with `ModelPart::ToolResult`.
7. If approval is required, store pending turn state using native model types.
8. If the response contains final text, store the assistant message and return the answer.

Automatic tool result message:

```rust
ModelMessage {
    role: ModelRole::Tool,
    parts: vec![ModelPart::ToolResult {
        tool_call_id,
        name,
        result,
    }],
}
```

## Pending Turn State

`PendingTurnState` must store OpenNivara-native model state, not Gemini-native `Content`.

It should include:

- `provider_id`
- `model_id`
- `messages_so_far: Vec<ModelMessage>`
- `tools: Vec<ModelToolDeclaration>`
- `pending_tool_call: ModelToolCall`
- `generation: GenerationConfig`
- `current_round`
- `max_rounds`
- request/session/user message metadata
- `compiled_context_audit_id`
- `selected_skill_ids`
- `pinned_context_ids`

Approval pause stores history after the assistant tool-call message has been pushed, plus the pending tool call.

Resume reloads `PendingTurnState`, executes or denies the pending tool call, appends `ModelPart::ToolResult`, and calls the same provider again using the same provider ID, model ID, tools, and generation settings.

## Required Tests

Add tests for:

1. `ModelMessage` serializes/deserializes.
2. `ModelPart::ToolCall` has a stable ID.
3. `ModelPart::ToolResult` serializes/deserializes.
4. `ModelToolDeclaration` does not include `ToolRisk` or `OperationKind`.
5. Gemini provider converts text messages to Gemini `Content`.
6. Gemini provider converts tool declarations to Gemini function declarations.
7. Gemini provider converts Gemini `function_call` to `ModelToolCall`.
8. Gemini provider generates `tool_call_id` when Gemini does not provide one.
9. Gemini provider converts `ModelPart::ToolResult` to Gemini `function_response`.
10. Engine can get a plain answer through `ModelProvider`.
11. Engine can receive a tool call through `ModelProvider`.
12. Engine can append a tool result and call provider again.
13. `PendingTurnState` JSON round-trips with native `ModelMessage`.
14. `PendingTurnState` does not depend on Gemini `Content`.
15. `MockProvider` supports deterministic engine tests.

## Implementation Milestones

PR 1 extracts native model types:

- Add `src/model/types.rs`.
- Define `ModelRole`, `ModelMessage`, `ModelPart`, `ModelToolCall`, `ModelToolDeclaration`, `ModelRequest`, `ModelResponse`, and `GenerationConfig`.
- Add serialization tests.

PR 2 extracts Gemini provider code:

- Move Gemini request/response structs from `src/engine.rs` to `src/model/gemini.rs`.
- Implement native-to-Gemini conversion.
- Implement Gemini-to-native conversion.
- Move Gemini endpoint and error sanitization out of `src/engine.rs`.
- Preserve behavior.

PR 3 makes the engine use `ModelProvider`:

- Add provider trait.
- Add `GeminiProvider`.
- Replace direct Gemini HTTP in the engine with `provider.generate`.
- Change engine history to `Vec<ModelMessage>`.
- Use `ModelToolCall` and `ModelPart::ToolResult` in the tool loop.

PR 4 adds `MockProvider` tests:

- Add `MockProvider`.
- Test plain response.
- Test tool call response.
- Test tool result followed by final response.
- Test provider error.
- Test generated tool call ID when missing.

PR 5 prepares pending turn serialization:

- Define `PendingTurnState` using native model types.
- Ensure JSON round-trip works.
- Prove it does not depend on Gemini-native structs.
