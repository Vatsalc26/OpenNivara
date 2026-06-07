use opennivara::engine::{EngineRequest, RequestSource};
use opennivara::engine_foundation::{
    assemble_prompt, resolve_state_session_for_request, run_model_turn_with_provider,
    state_surface_from_request_source, store_state_user_message_for_request, PromptAssemblyInput,
};
use opennivara::model::mock::{MockProvider, MockStep};
use opennivara::model::types::{ModelMessage, ModelPart, ModelRole, ModelToolDeclaration};
use opennivara::state::db::open_state_db_at;
use opennivara::state::messages;
use opennivara::state::types::DbMessage;
use serde_json::json;
use serial_test::serial;

struct EnvGuard {
    previous_config: Option<String>,
    previous_key: Option<String>,
}

impl EnvGuard {
    fn new(config_dir: &std::path::Path) -> Self {
        let previous_config = std::env::var("OPENNIVARA_TEST_CONFIG_DIR").ok();
        let previous_key = std::env::var("GEMINI_API_KEY").ok();
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", config_dir);
        std::env::remove_var("GEMINI_API_KEY");
        Self {
            previous_config,
            previous_key,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous_config {
            std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", previous);
        } else {
            std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
        }
        if let Some(previous) = &self.previous_key {
            std::env::set_var("GEMINI_API_KEY", previous);
        } else {
            std::env::remove_var("GEMINI_API_KEY");
        }
    }
}

fn db_message(role: &str, content: &str) -> DbMessage {
    DbMessage {
        id: format!("msg_{role}"),
        session_id: "sess_test".into(),
        role: role.into(),
        surface: "cli".into(),
        actor_id: Some("cli_owner".into()),
        content: content.into(),
        created_at: "2026-06-07T00:00:00Z".into(),
        metadata_json: None,
    }
}

#[test]
fn prompt_assembly_uses_user_and_assistant_history_only() {
    let assembly = assemble_prompt(PromptAssemblyInput {
        audit_id: Some("audit_1".into()),
        prior_messages: vec![
            db_message("system", "system policy"),
            db_message("user", "hello"),
            db_message("assistant", "hi"),
            db_message("tool", r#"{"path":"D:/"}"#),
            db_message("event", r#"{"type":"approval"}"#),
            db_message("model", "legacy assistant"),
        ],
        compiled_current_prompt: "compiled current turn".into(),
        max_previous_messages: 10,
        selected_skill_ids: vec!["skill_a".into()],
        pinned_context_ids: vec!["ctx_1".into()],
        pinned_skill_ids: vec!["skill_pin".into()],
        warnings: vec!["low confidence".into()],
    });

    assert_eq!(assembly.audit_id.as_deref(), Some("audit_1"));
    assert_eq!(assembly.selected_skill_ids, vec!["skill_a"]);
    assert_eq!(assembly.pinned_context_ids, vec!["ctx_1"]);
    assert_eq!(assembly.pinned_skill_ids, vec!["skill_pin"]);
    assert_eq!(assembly.warnings, vec!["low confidence"]);

    let roles: Vec<ModelRole> = assembly
        .messages
        .iter()
        .map(|message| message.role.clone())
        .collect();
    assert_eq!(
        roles,
        vec![
            ModelRole::User,
            ModelRole::Model,
            ModelRole::Model,
            ModelRole::User
        ]
    );

    let texts: Vec<&str> = assembly
        .messages
        .iter()
        .map(|message| message.first_text().expect("text part"))
        .collect();
    assert_eq!(
        texts,
        vec!["hello", "hi", "legacy assistant", "compiled current turn"]
    );
}

#[test]
fn prompt_assembly_limits_previous_messages_before_current_prompt() {
    let assembly = assemble_prompt(PromptAssemblyInput {
        audit_id: None,
        prior_messages: vec![
            db_message("user", "one"),
            db_message("assistant", "two"),
            db_message("user", "three"),
        ],
        compiled_current_prompt: "current".into(),
        max_previous_messages: 2,
        selected_skill_ids: vec![],
        pinned_context_ids: vec![],
        pinned_skill_ids: vec![],
        warnings: vec![],
    });

    let texts: Vec<&str> = assembly
        .messages
        .iter()
        .map(|message| message.first_text().expect("text part"))
        .collect();
    assert_eq!(texts, vec!["two", "three", "current"]);
}

#[test]
fn state_session_helpers_map_request_source_to_surface_actor_and_user_message() {
    let temp = tempfile::tempdir().expect("temp dir");
    let conn = open_state_db_at(temp.path().join("state.sqlite")).expect("state db");
    let request = EngineRequest::new(
        RequestSource::Telegram {
            chat_id: 42,
            username: Some("nivara".into()),
        },
        None,
        "hello from telegram",
    );

    assert_eq!(
        state_surface_from_request_source(&request.source),
        opennivara::state::types::Surface::Telegram
    );

    let session = resolve_state_session_for_request(&conn, &request).expect("session");
    assert_eq!(session.surface_created, "telegram");
    assert_eq!(session.actor_id_created.as_deref(), Some("telegram_42"));

    let message =
        store_state_user_message_for_request(&conn, &session.id, &request).expect("message");
    assert_eq!(message.role, "user");
    assert_eq!(message.surface, "telegram");
    assert_eq!(message.actor_id.as_deref(), Some("telegram_42"));
    assert_eq!(message.content, "hello from telegram");

    let loaded = messages::get_session_messages(&conn, &session.id).expect("messages");
    assert_eq!(loaded, vec![message]);
}

#[tokio::test]
async fn provider_injected_turn_returns_plain_answer() {
    let provider = MockProvider::new(vec![MockStep::text("plain answer")]);
    let output = run_model_turn_with_provider(
        &provider,
        vec![ModelMessage::text(ModelRole::User, "compiled prompt")],
        vec![],
        3,
        |_| json!({"unused": true}),
    )
    .await
    .expect("model turn");

    assert_eq!(output.answer, "plain answer");
    assert_eq!(provider.requests().len(), 1);
    assert_eq!(output.messages_so_far.len(), 2);
}

#[tokio::test]
async fn provider_injected_turn_runs_automatic_tool_loop() {
    let provider = MockProvider::new(vec![
        MockStep::tool_call("get_current_dir", json!({})),
        MockStep::text("tool complete"),
    ]);
    let mut executed_tools = Vec::new();

    let output = run_model_turn_with_provider(
        &provider,
        vec![ModelMessage::text(ModelRole::User, "where am I?")],
        vec![ModelToolDeclaration {
            name: "get_current_dir".into(),
            description: "Returns current directory".into(),
            parameters: json!({"type":"object","properties":{}}),
        }],
        3,
        |call| {
            executed_tools.push(call.name.clone());
            json!({"path":"D:/Prototypes/Project_2"})
        },
    )
    .await
    .expect("model turn");

    assert_eq!(output.answer, "tool complete");
    assert_eq!(executed_tools, vec!["get_current_dir"]);
    assert_eq!(provider.requests().len(), 2);
    assert!(output.messages_so_far.iter().any(|message| matches!(
        message.parts.first(),
        Some(ModelPart::ToolResult(result)) if result.name == "get_current_dir"
    )));
}

#[tokio::test]
#[serial]
async fn engine_handler_accepts_injected_provider_for_plain_answer() {
    let temp = tempfile::tempdir().expect("temp dir");
    let _guard = EnvGuard::new(temp.path());
    opennivara::first_run::initialize_clean_first_run(opennivara::first_run::FirstRunInput {
        accepted_alpha_notice: true,
        gemini_api_key: None,
    })
    .expect("first run");

    let provider = MockProvider::new(vec![MockStep::text("answer from injected provider")]);
    let engine = opennivara::engine::OpenNivaraEngine::new();
    let response = engine
        .handle_message_with_provider(
            EngineRequest::new(RequestSource::Cli, None, "hello through engine"),
            &provider,
        )
        .await
        .expect("engine response");

    assert_eq!(response.answer, "answer from injected provider");
    assert_eq!(provider.requests().len(), 1);
}
