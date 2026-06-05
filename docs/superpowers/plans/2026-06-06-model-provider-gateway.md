# Model Provider Gateway Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move Gemini-native request/response handling behind a thin OpenNivara-native model provider boundary before approval resume serializes pending turn state.

**Architecture:** Add `src/model` with native model types, a thin async provider trait, a Gemini adapter, and a deterministic mock provider. The engine should use `ModelRequest`, `ModelMessage`, `ModelToolDeclaration`, and `ModelResponse`; provider adapters own provider-specific JSON. `PendingTurnState` stores native model messages and tool declarations, never Gemini `Content`.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `reqwest`, `anyhow`, optional `async-trait`, existing Gemini API integration.

---

## File Structure

- Create `src/model/mod.rs`: exports `types`, `provider`, `gemini`, and `mock`.
- Create `src/model/types.rs`: native model roles, messages, parts, tool calls, tool declarations, request, response, and generation config.
- Create `src/model/provider.rs`: `ModelProvider` trait and default provider construction.
- Create `src/model/gemini.rs`: Gemini structs, conversion code, endpoint, HTTP request, and error sanitization.
- Create `src/model/mock.rs`: deterministic provider for engine tests.
- Modify `src/lib.rs`: expose `pub mod model;`.
- Modify `src/engine.rs`: remove Gemini-native structs after provider extraction and use model-native history.
- Modify `Cargo.toml`: add `async-trait = "0.1"` only if using the async trait approach.

## Task 1: Native Model Types

**Files:**

- Create: `src/model/mod.rs`
- Create: `src/model/types.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add failing serialization tests**

Create `src/model/types.rs` with the tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_message_round_trips_tool_call() {
        let message = ModelMessage {
            role: ModelRole::Assistant,
            parts: vec![ModelPart::ToolCall {
                id: "tc_1".to_string(),
                name: "read_file".to_string(),
                args: serde_json::json!({"path": "Cargo.toml"}),
            }],
        };

        let json = serde_json::to_string(&message).unwrap();
        let decoded: ModelMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn model_tool_declaration_excludes_policy_fields() {
        let declaration = ModelToolDeclaration {
            name: "read_file".to_string(),
            description: "Read a file".to_string(),
            parameters: serde_json::json!({"type": "OBJECT"}),
        };

        let json = serde_json::to_value(declaration).unwrap();

        assert!(json.get("risk_level").is_none());
        assert!(json.get("operation_kind").is_none());
        assert!(json.get("classification").is_none());
    }
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test model_message_round_trips_tool_call model_tool_declaration_excludes_policy_fields`

Expected: fail because model types do not exist.

- [ ] **Step 3: Add model modules**

Create `src/model/mod.rs`:

```rust
pub mod gemini;
pub mod mock;
pub mod provider;
pub mod types;
```

Add to `src/lib.rs`:

```rust
pub mod model;
```

- [ ] **Step 4: Add native model types**

Create `src/model/types.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelMessage {
    pub role: ModelRole,
    pub parts: Vec<ModelPart>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelToolDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenerationConfig {
    pub max_tool_rounds: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelRequest {
    pub provider_id: String,
    pub model_id: String,
    pub messages: Vec<ModelMessage>,
    pub tools: Vec<ModelToolDeclaration>,
    pub generation: GenerationConfig,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelResponse {
    pub message: ModelMessage,
    pub text: Option<String>,
    pub tool_calls: Vec<ModelToolCall>,
    pub raw_provider_response_json: Option<serde_json::Value>,
}
```

- [ ] **Step 5: Run serialization tests**

Run: `cargo test model_message_round_trips_tool_call model_tool_declaration_excludes_policy_fields`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/model
git commit -m "feat(model): add native model types"
```

## Task 2: Provider Trait And Mock Provider

**Files:**

- Create: `src/model/provider.rs`
- Create: `src/model/mock.rs`
- Modify: `Cargo.toml` if using `async-trait`

- [ ] **Step 1: Add mock provider tests**

Create `src/model/mock.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::*;

    #[tokio::test]
    async fn mock_provider_returns_scripted_responses() {
        let provider = MockProvider::new(vec![Ok(ModelResponse {
            message: ModelMessage {
                role: ModelRole::Assistant,
                parts: vec![ModelPart::Text {
                    text: "hello".to_string(),
                }],
            },
            text: Some("hello".to_string()),
            tool_calls: vec![],
            raw_provider_response_json: None,
        })]);

        let response = provider
            .generate(ModelRequest {
                provider_id: "mock".to_string(),
                model_id: "mock".to_string(),
                messages: vec![],
                tools: vec![],
                generation: GenerationConfig { max_tool_rounds: 3 },
            })
            .await
            .unwrap();

        assert_eq!(response.text.as_deref(), Some("hello"));
    }
}
```

- [ ] **Step 2: Run test and confirm failure**

Run: `cargo test mock_provider_returns_scripted_responses`

Expected: fail because provider trait and mock provider are not implemented.

- [ ] **Step 3: Add provider trait**

Create `src/model/provider.rs`:

```rust
use crate::model::types::{ModelRequest, ModelResponse};

#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    async fn generate(&self, request: ModelRequest) -> anyhow::Result<ModelResponse>;
}
```

If `async-trait` is not already present, add to `Cargo.toml`:

```toml
async-trait = "0.1"
```

- [ ] **Step 4: Add mock provider**

Create `src/model/mock.rs`:

```rust
use crate::model::provider::ModelProvider;
use crate::model::types::{ModelRequest, ModelResponse};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockProvider {
    responses: Arc<Mutex<Vec<anyhow::Result<ModelResponse>>>>,
}

impl MockProvider {
    pub fn new(responses: Vec<anyhow::Result<ModelResponse>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }
}

#[async_trait::async_trait]
impl ModelProvider for MockProvider {
    async fn generate(&self, _request: ModelRequest) -> anyhow::Result<ModelResponse> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            anyhow::bail!("mock provider has no scripted response");
        }
        responses.remove(0)
    }
}
```

- [ ] **Step 5: Run mock provider tests**

Run: `cargo test mock_provider_returns_scripted_responses`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/model/provider.rs src/model/mock.rs
git commit -m "feat(model): add provider trait and mock provider"
```

## Task 3: Gemini Conversion Layer

**Files:**

- Create: `src/model/gemini.rs`
- Modify: `src/engine.rs`

- [ ] **Step 1: Add conversion tests**

Add tests in `src/model/gemini.rs` proving:

```rust
#[test]
fn converts_model_tool_declarations_to_gemini_function_declarations() {
    let tools = vec![ModelToolDeclaration {
        name: "read_file".to_string(),
        description: "Read a file".to_string(),
        parameters: serde_json::json!({"type": "OBJECT"}),
    }];

    let gemini_tools = gemini_tools_from_model_tools(&tools);

    assert_eq!(gemini_tools[0].function_declarations[0].name, "read_file");
}

#[test]
fn converts_gemini_function_call_to_model_tool_call_with_id() {
    let response = GeminiResponse {
        candidates: Some(vec![Candidate {
            content: Some(Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: None,
                    function_call: Some(FunctionCall {
                        name: "read_file".to_string(),
                        args: serde_json::json!({"path": "Cargo.toml"}),
                    }),
                    function_response: None,
                }],
            }),
        }]),
        error: None,
    };

    let model = model_response_from_gemini(response).unwrap();

    assert_eq!(model.tool_calls[0].name, "read_file");
    assert!(!model.tool_calls[0].id.is_empty());
}
```

- [ ] **Step 2: Run conversion tests and confirm failure**

Run: `cargo test gemini`

Expected: fail because Gemini conversion module does not exist.

- [ ] **Step 3: Move Gemini structs**

Move Gemini structs from `src/engine.rs` into `src/model/gemini.rs`, preserving serialization attributes:

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

- [ ] **Step 4: Implement conversion functions**

Implement:

```rust
pub fn gemini_request_from_model(request: ModelRequest) -> GeminiRequest;
pub fn gemini_tools_from_model_tools(tools: &[ModelToolDeclaration]) -> Vec<Tool>;
pub fn model_response_from_gemini(response: GeminiResponse) -> anyhow::Result<ModelResponse>;
pub fn sanitize_provider_error(message: &str, api_key: &str) -> String;
```

When Gemini does not provide a tool call ID, generate one with `uuid::Uuid::new_v4()`.

- [ ] **Step 5: Run conversion tests**

Run: `cargo test gemini`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/model/gemini.rs src/engine.rs
git commit -m "feat(model): add Gemini conversion layer"
```

## Task 4: Gemini Provider HTTP Adapter

**Files:**

- Modify: `src/model/gemini.rs`
- Modify: `src/engine.rs`

- [ ] **Step 1: Add provider error sanitization test**

Add:

```rust
#[test]
fn gemini_provider_error_sanitizes_api_key() {
    let sanitized = sanitize_provider_error("bad key secret-123", "secret-123");
    assert_eq!(sanitized, "bad key [REDACTED_API_KEY]");
}
```

- [ ] **Step 2: Implement `GeminiProvider`**

Add:

```rust
pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}
```

Implement `ModelProvider` by converting `ModelRequest` to Gemini JSON, sending to `gemini_generate_content_url()`, parsing response JSON, handling provider errors, and converting to `ModelResponse`.

- [ ] **Step 3: Run model tests**

Run: `cargo test model`

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add src/model/gemini.rs
git commit -m "feat(model): add Gemini provider adapter"
```

## Task 5: Engine Uses Provider Boundary

**Files:**

- Modify: `src/engine.rs`

- [ ] **Step 1: Add engine provider tests**

Add tests that use `MockProvider` to prove:

- plain answer returns final `EngineResponse`
- tool call response is observed
- tool result can be appended and provider called again
- provider error is returned

- [ ] **Step 2: Run tests and confirm failure**

Run: `cargo test engine_provider`

Expected: fail until engine accepts a provider boundary.

- [ ] **Step 3: Refactor engine history**

Replace `Vec<Content>` with `Vec<ModelMessage>`. Convert session messages into model-native messages. Convert registry declarations into `Vec<ModelToolDeclaration>`.

- [ ] **Step 4: Call provider**

Replace direct `reqwest` Gemini request code with:

```rust
let response = provider
    .generate(ModelRequest {
        provider_id: "gemini".to_string(),
        model_id: "gemini-2.5-flash".to_string(),
        messages: history.clone(),
        tools: tools_declaration.clone(),
        generation: GenerationConfig { max_tool_rounds },
    })
    .await?;
```

- [ ] **Step 5: Run engine provider tests**

Run: `cargo test engine_provider`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src/engine.rs src/model
git commit -m "refactor(engine): use model provider boundary"
```

## Final Verification

- [ ] **Step 1: Run model tests**

Run: `cargo test model`

Expected: all model tests pass.

- [ ] **Step 2: Run engine tests**

Run: `cargo test engine`

Expected: all engine tests pass.

- [ ] **Step 3: Run full Rust tests**

Run: `cargo test`

Expected: all Rust tests pass.

- [ ] **Step 4: Run docs checks**

Run: `bun run docs:check`

Expected: markdown and internal docs links pass.

- [ ] **Step 5: Search for Gemini-native engine leftovers**

Run: `rg -n "GeminiRequest|GeminiResponse|FunctionCall|FunctionResponse|generateContent|reqwest::Client::new" src/engine.rs src/model`

Expected: Gemini-native structs and HTTP construction remain in `src/model/gemini.rs`, not in `src/engine.rs`.
