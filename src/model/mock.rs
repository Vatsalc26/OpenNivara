use super::provider::{ModelProvider, ModelProviderFuture};
use super::types::{ModelMessage, ModelRequest, ModelResponse};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum MockStep {
    Text(String),
    ToolCall {
        tool_call_id: Option<String>,
        name: String,
        arguments: Value,
    },
    Failure(String),
}

impl MockStep {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn tool_call(name: impl Into<String>, arguments: Value) -> Self {
        Self::ToolCall {
            tool_call_id: None,
            name: name.into(),
            arguments,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::Failure(message.into())
    }
}

#[derive(Debug)]
pub struct MockProvider {
    script: Mutex<VecDeque<MockStep>>,
    requests: Mutex<Vec<ModelRequest>>,
}

impl MockProvider {
    pub fn new(steps: Vec<MockStep>) -> Self {
        Self {
            script: Mutex::new(steps.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    pub fn requests(&self) -> Vec<ModelRequest> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

impl ModelProvider for MockProvider {
    fn generate<'a>(&'a self, request: ModelRequest) -> ModelProviderFuture<'a> {
        Box::pin(async move {
            self.requests
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(request);

            let step = self
                .script
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .pop_front()
                .ok_or_else(|| anyhow::anyhow!("MockProvider script exhausted"))?;

            match step {
                MockStep::Text(text) => Ok(ModelResponse {
                    message: ModelMessage::text(super::types::ModelRole::Model, text),
                }),
                MockStep::ToolCall {
                    tool_call_id,
                    name,
                    arguments,
                } => Ok(ModelResponse {
                    message: ModelMessage::tool_call(
                        tool_call_id.unwrap_or_else(crate::runtime::ids::new_tool_call_id),
                        name,
                        arguments,
                    ),
                }),
                MockStep::Failure(message) => Err(anyhow::anyhow!(message)),
            }
        })
    }
}
