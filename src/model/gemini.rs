use super::provider::{ModelProvider, ModelProviderFuture};
use super::types::{
    ModelMessage, ModelPart, ModelRequest, ModelResponse, ModelRole, ModelToolCall,
    ModelToolDeclaration,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<GeminiFunctionResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeminiFunctionCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GeminiFunctionResponse {
    pub name: String,
    pub response: Value,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
}

#[derive(Debug, Serialize, Clone)]
struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize, Clone)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiApiErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiApiErrorDetail {
    code: Option<i32>,
    message: Option<String>,
}

pub fn gemini_generate_content_url() -> &'static str {
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent"
}

pub fn sanitize_provider_error(message: &str, api_key: &str) -> String {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return message.to_string();
    }
    message.replace(trimmed_key, "[REDACTED_API_KEY]")
}

pub fn request_to_json(request: &ModelRequest) -> anyhow::Result<Value> {
    let request = GeminiRequest::from(request);
    Ok(serde_json::to_value(request)?)
}

pub fn response_from_json(value: Value) -> anyhow::Result<ModelResponse> {
    let response: GeminiResponse = serde_json::from_value(value)?;
    model_response_from_gemini(response, "")
}

fn model_response_from_gemini(
    response: GeminiResponse,
    api_key: &str,
) -> anyhow::Result<ModelResponse> {
    if let Some(err) = response.error {
        return Err(anyhow::anyhow!(
            "Gemini API Error ({}): {}",
            err.code.unwrap_or(0),
            sanitize_provider_error(&err.message.unwrap_or_default(), api_key)
        ));
    }

    let candidate = response
        .candidates
        .as_ref()
        .and_then(|candidates| candidates.first())
        .ok_or_else(|| anyhow::anyhow!("Gemini response contains no candidates."))?;

    let content = candidate
        .content
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Gemini response candidate contains no content."))?;

    Ok(ModelResponse {
        message: content.clone().into(),
    })
}

impl From<&ModelRequest> for GeminiRequest {
    fn from(request: &ModelRequest) -> Self {
        let tools = if request.tools.is_empty() {
            None
        } else {
            Some(vec![GeminiTool {
                function_declarations: request
                    .tools
                    .iter()
                    .map(GeminiFunctionDeclaration::from)
                    .collect(),
            }])
        };

        Self {
            contents: request.messages.iter().map(GeminiContent::from).collect(),
            tools,
        }
    }
}

impl From<&ModelToolDeclaration> for GeminiFunctionDeclaration {
    fn from(declaration: &ModelToolDeclaration) -> Self {
        Self {
            name: declaration.name.clone(),
            description: declaration.description.clone(),
            parameters: declaration.parameters.clone(),
        }
    }
}

impl From<&ModelMessage> for GeminiContent {
    fn from(message: &ModelMessage) -> Self {
        Self {
            role: message.role.as_provider_role().to_string(),
            parts: message.parts.iter().map(GeminiPart::from).collect(),
        }
    }
}

impl From<&ModelPart> for GeminiPart {
    fn from(part: &ModelPart) -> Self {
        match part {
            ModelPart::Text(text) => Self {
                text: Some(text.clone()),
                function_call: None,
                function_response: None,
            },
            ModelPart::ToolCall(call) => Self {
                text: None,
                function_call: Some(GeminiFunctionCall {
                    name: call.name.clone(),
                    args: call.arguments.clone(),
                }),
                function_response: None,
            },
            ModelPart::ToolResult(result) => Self {
                text: None,
                function_call: None,
                function_response: Some(GeminiFunctionResponse {
                    name: result.name.clone(),
                    response: result.response.clone(),
                }),
            },
        }
    }
}

impl From<GeminiContent> for ModelMessage {
    fn from(content: GeminiContent) -> Self {
        Self {
            role: ModelRole::from_provider_role(&content.role),
            parts: content.parts.into_iter().map(ModelPart::from).collect(),
        }
    }
}

impl From<GeminiPart> for ModelPart {
    fn from(part: GeminiPart) -> Self {
        if let Some(text) = part.text {
            return Self::Text(text);
        }
        if let Some(call) = part.function_call {
            return Self::ToolCall(ModelToolCall {
                tool_call_id: crate::runtime::ids::new_tool_call_id(),
                name: call.name,
                arguments: call.args,
            });
        }
        if let Some(response) = part.function_response {
            return Self::ToolResult(super::types::ModelToolResult {
                tool_call_id: crate::runtime::ids::new_tool_call_id(),
                name: response.name,
                response: response.response,
            });
        }
        Self::Text(String::new())
    }
}

#[derive(Debug, Clone)]
pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
    url: &'static str,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
            url: gemini_generate_content_url(),
        }
    }
}

impl ModelProvider for GeminiProvider {
    fn generate<'a>(&'a self, request: ModelRequest) -> ModelProviderFuture<'a> {
        Box::pin(async move {
            let api_key = self.api_key.trim();
            let request_payload = GeminiRequest::from(&request);
            let response = self
                .client
                .post(self.url)
                .header("x-goog-api-key", api_key)
                .json(&request_payload)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Gemini API connection error: {}", e))?;

            let status = response.status();
            let response_text = response
                .text()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

            let api_response: GeminiResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to parse Gemini API JSON response: {}\nRaw Response: {}",
                        e,
                        sanitize_provider_error(&response_text, api_key)
                    )
                })?;

            if !status.is_success() && api_response.error.is_none() {
                return Err(anyhow::anyhow!(
                    "Gemini API returned unsuccessful status {}: {}",
                    status,
                    sanitize_provider_error(&response_text, api_key)
                ));
            }

            model_response_from_gemini(api_response, api_key)
        })
    }
}
