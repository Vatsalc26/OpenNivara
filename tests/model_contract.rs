use opennivara::model::gemini;
use opennivara::model::mock::{MockProvider, MockStep};
use opennivara::model::provider::ModelProvider;
use opennivara::model::types::{
    ModelMessage, ModelPart, ModelRequest, ModelRole, ModelToolCall, ModelToolDeclaration,
    ModelToolResult,
};
use serde_json::json;

#[test]
fn native_model_messages_round_trip_through_json() {
    let message = ModelMessage {
        role: ModelRole::Model,
        parts: vec![
            ModelPart::Text("checking the file".to_string()),
            ModelPart::ToolCall(ModelToolCall {
                tool_call_id: "toolcall_existing".to_string(),
                name: "read_file".to_string(),
                arguments: json!({ "path": "README.md" }),
            }),
            ModelPart::ToolResult(ModelToolResult {
                tool_call_id: "toolcall_existing".to_string(),
                name: "read_file".to_string(),
                response: json!({ "content": "hello" }),
            }),
        ],
    };

    let encoded = serde_json::to_string(&message).expect("serialize native message");
    let decoded: ModelMessage = serde_json::from_str(&encoded).expect("deserialize native message");

    assert_eq!(decoded, message);
}

#[test]
fn gemini_request_conversion_preserves_text_history_and_tools() {
    let request = ModelRequest {
        messages: vec![ModelMessage::text(ModelRole::User, "hello")],
        tools: vec![ModelToolDeclaration {
            name: "read_file".to_string(),
            description: "Read a file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }),
        }],
    };

    let payload = gemini::request_to_json(&request).expect("convert request");

    assert_eq!(payload["contents"][0]["role"], "user");
    assert_eq!(payload["contents"][0]["parts"][0]["text"], "hello");
    assert_eq!(
        payload["tools"][0]["functionDeclarations"][0]["name"],
        "read_file"
    );
}

#[test]
fn gemini_response_generates_tool_call_id_when_provider_omits_one() {
    let response = gemini::response_from_json(json!({
        "candidates": [{
            "content": {
                "role": "model",
                "parts": [{
                    "functionCall": {
                        "name": "read_file",
                        "args": { "path": "README.md" }
                    }
                }]
            }
        }]
    }))
    .expect("parse gemini response");

    let call = response.message.first_tool_call().expect("tool call");

    assert!(call.tool_call_id.starts_with("toolcall_"));
    assert_eq!(call.name, "read_file");
    assert_eq!(call.arguments, json!({ "path": "README.md" }));
}

#[tokio::test]
async fn mock_provider_scripts_text_tool_calls_continuations_and_failures() {
    let provider = MockProvider::new(vec![
        MockStep::tool_call("read_file", json!({ "path": "README.md" })),
        MockStep::text("The file says hello."),
        MockStep::failure("provider unavailable"),
    ]);

    let first = provider
        .generate(ModelRequest::from_user_text("please read README"))
        .await
        .expect("tool call response");
    let call = first.message.first_tool_call().expect("tool call").clone();
    assert!(call.tool_call_id.starts_with("toolcall_"));

    let second = provider
        .generate(ModelRequest {
            messages: vec![
                ModelMessage::text(ModelRole::User, "please read README"),
                first.message,
                ModelMessage::tool_result(
                    call.tool_call_id.clone(),
                    call.name.clone(),
                    json!({ "content": "hello" }),
                ),
            ],
            tools: vec![],
        })
        .await
        .expect("continuation response");
    assert_eq!(second.message.first_text(), Some("The file says hello."));

    let failure = provider
        .generate(ModelRequest::from_user_text("try again"))
        .await
        .expect_err("scripted provider failure");
    assert!(failure.to_string().contains("provider unavailable"));
    assert_eq!(provider.requests().len(), 3);
}
