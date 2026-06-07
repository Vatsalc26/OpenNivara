use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    User,
    Model,
    Tool,
}

impl ModelRole {
    pub fn as_provider_role(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Model => "model",
            Self::Tool => "function",
        }
    }

    pub fn from_provider_role(role: &str) -> Self {
        match role {
            "assistant" | "model" => Self::Model,
            "function" | "tool" => Self::Tool,
            _ => Self::User,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ModelPart {
    Text(String),
    ToolCall(ModelToolCall),
    ToolResult(ModelToolResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelToolCall {
    pub tool_call_id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelToolResult {
    pub tool_call_id: String,
    pub name: String,
    pub response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelMessage {
    pub role: ModelRole,
    pub parts: Vec<ModelPart>,
}

impl ModelMessage {
    pub fn text(role: ModelRole, text: impl Into<String>) -> Self {
        Self {
            role,
            parts: vec![ModelPart::Text(text.into())],
        }
    }

    pub fn tool_call(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        arguments: Value,
    ) -> Self {
        Self {
            role: ModelRole::Model,
            parts: vec![ModelPart::ToolCall(ModelToolCall {
                tool_call_id: tool_call_id.into(),
                name: name.into(),
                arguments,
            })],
        }
    }

    pub fn tool_result(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        response: Value,
    ) -> Self {
        Self {
            role: ModelRole::Tool,
            parts: vec![ModelPart::ToolResult(ModelToolResult {
                tool_call_id: tool_call_id.into(),
                name: name.into(),
                response,
            })],
        }
    }

    pub fn first_text(&self) -> Option<&str> {
        self.parts.iter().find_map(|part| match part {
            ModelPart::Text(text) => Some(text.as_str()),
            ModelPart::ToolCall(_) | ModelPart::ToolResult(_) => None,
        })
    }

    pub fn first_tool_call(&self) -> Option<&ModelToolCall> {
        self.parts.iter().find_map(|part| match part {
            ModelPart::ToolCall(call) => Some(call),
            ModelPart::Text(_) | ModelPart::ToolResult(_) => None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelToolDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRequest {
    pub messages: Vec<ModelMessage>,
    pub tools: Vec<ModelToolDeclaration>,
}

impl ModelRequest {
    pub fn from_user_text(text: impl Into<String>) -> Self {
        Self {
            messages: vec![ModelMessage::text(ModelRole::User, text)],
            tools: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelResponse {
    pub message: ModelMessage,
}
