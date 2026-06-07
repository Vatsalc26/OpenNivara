use crate::engine::{EngineRequest, RequestSource};
use crate::model::provider::ModelProvider;
use crate::model::types::{
    ModelMessage, ModelRequest, ModelRole, ModelToolCall, ModelToolDeclaration,
};
use crate::state::types::{MessageRole, StoreMessageInput};
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq)]
pub struct PromptAssembly {
    pub audit_id: Option<String>,
    pub messages: Vec<ModelMessage>,
    pub selected_skill_ids: Vec<String>,
    pub pinned_context_ids: Vec<String>,
    pub pinned_skill_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromptAssemblyInput {
    pub audit_id: Option<String>,
    pub prior_messages: Vec<crate::state::types::DbMessage>,
    pub compiled_current_prompt: String,
    pub max_previous_messages: usize,
    pub selected_skill_ids: Vec<String>,
    pub pinned_context_ids: Vec<String>,
    pub pinned_skill_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelTurnOutput {
    pub answer: String,
    pub messages_so_far: Vec<ModelMessage>,
    pub rounds: u32,
}

pub fn assemble_prompt(input: PromptAssemblyInput) -> PromptAssembly {
    let mut conversational_messages: Vec<ModelMessage> = input
        .prior_messages
        .into_iter()
        .filter(|message| matches!(message.role.as_str(), "user" | "assistant" | "model"))
        .map(|message| {
            ModelMessage::text(
                ModelRole::from_provider_role(&message.role),
                message.content,
            )
        })
        .collect();

    let start_idx = conversational_messages
        .len()
        .saturating_sub(input.max_previous_messages);
    let mut messages = conversational_messages.split_off(start_idx);
    messages.push(ModelMessage::text(
        ModelRole::User,
        input.compiled_current_prompt,
    ));

    PromptAssembly {
        audit_id: input.audit_id,
        messages,
        selected_skill_ids: input.selected_skill_ids,
        pinned_context_ids: input.pinned_context_ids,
        pinned_skill_ids: input.pinned_skill_ids,
        warnings: input.warnings,
    }
}

pub fn state_surface_from_request_source(source: &RequestSource) -> crate::state::types::Surface {
    match source {
        RequestSource::Cli => crate::state::types::Surface::Cli,
        RequestSource::Desktop => crate::state::types::Surface::Desktop,
        RequestSource::Telegram { .. } => crate::state::types::Surface::Telegram,
    }
}

pub fn resolve_state_session_for_request(
    conn: &Connection,
    request: &EngineRequest,
) -> anyhow::Result<crate::state::types::Session> {
    let surface = state_surface_from_request_source(&request.source);
    let actor_id = request.source.actor_id();

    if let Some(session_id) = request.session_id.as_deref() {
        let session = crate::state::sessions::get_session(conn, session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {session_id}"))?;
        crate::state::active_sessions::set_active_session(conn, &actor_id, surface, session_id)?;
        return Ok(session);
    }

    if let Some(session_id) =
        crate::state::active_sessions::get_active_session(conn, &actor_id, surface)?
    {
        if let Some(session) = crate::state::sessions::get_session(conn, &session_id)? {
            return Ok(session);
        }
    }

    let title = match request.source {
        RequestSource::Cli => "CLI",
        RequestSource::Desktop => "Desktop",
        RequestSource::Telegram { .. } => "Telegram",
    };
    let session = crate::state::sessions::create_session(
        conn,
        crate::state::types::CreateSessionInput {
            title: Some(title.to_string()),
            surface_created: surface,
            actor_id_created: Some(actor_id.clone()),
        },
    )?;
    crate::state::active_sessions::set_active_session(conn, &actor_id, surface, &session.id)?;
    Ok(session)
}

pub fn store_state_user_message_for_request(
    conn: &Connection,
    session_id: &str,
    request: &EngineRequest,
) -> anyhow::Result<crate::state::types::DbMessage> {
    crate::state::messages::store_message(
        conn,
        StoreMessageInput {
            session_id: session_id.to_string(),
            role: MessageRole::User,
            surface: state_surface_from_request_source(&request.source),
            actor_id: Some(request.source.actor_id()),
            content: request.message.clone(),
            metadata_json: request.client_message_id.as_ref().map(|client_message_id| {
                serde_json::json!({ "client_message_id": client_message_id }).to_string()
            }),
        },
    )
}

pub async fn run_model_turn_with_provider<P, F>(
    provider: &P,
    mut messages: Vec<ModelMessage>,
    tools: Vec<ModelToolDeclaration>,
    max_rounds: u32,
    mut execute_tool: F,
) -> anyhow::Result<ModelTurnOutput>
where
    P: ModelProvider,
    F: FnMut(&ModelToolCall) -> serde_json::Value,
{
    let mut current_round = 0;

    let answer = loop {
        let model_response = provider
            .generate(ModelRequest {
                messages: messages.clone(),
                tools: tools.clone(),
            })
            .await?;
        let content = model_response.message;
        messages.push(content.clone());

        if let Some(call) = content.first_tool_call().cloned() {
            let result = if current_round >= max_rounds {
                serde_json::json!({
                    "error": format!(
                        "Tool execution halted: maximum rounds limit ({max_rounds}) was reached."
                    )
                })
            } else {
                execute_tool(&call)
            };
            messages.push(ModelMessage::tool_result(
                call.tool_call_id,
                call.name,
                result,
            ));
            current_round += 1;
            continue;
        }

        match content.first_text() {
            Some(text) => break text.trim().to_string(),
            None => {
                return Err(anyhow::anyhow!(
                    "Model returned a response, but no text block was found."
                ));
            }
        }
    };

    Ok(ModelTurnOutput {
        answer,
        messages_so_far: messages,
        rounds: current_round,
    })
}
