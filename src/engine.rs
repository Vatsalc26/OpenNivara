use crate::remote_policy;
use crate::sessions::{self, DbMessage};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;

use crate::model::provider::ModelProvider;
use crate::model::types::{ModelMessage, ModelRequest, ModelRole, ModelToolDeclaration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestSource {
    Cli,
    Desktop,
    Telegram {
        chat_id: i64,
        username: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[specta(rename_all = "snake_case")]
pub enum Surface {
    Cli,
    Desktop,
    Telegram,
}

impl RequestSource {
    pub fn surface(&self) -> Surface {
        match self {
            RequestSource::Cli => Surface::Cli,
            RequestSource::Desktop => Surface::Desktop,
            RequestSource::Telegram { .. } => Surface::Telegram,
        }
    }

    pub fn actor_id(&self) -> String {
        match self {
            RequestSource::Cli => "cli_owner".to_string(),
            RequestSource::Desktop => "desktop_owner".to_string(),
            RequestSource::Telegram { chat_id, .. } => format!("telegram_{chat_id}"),
        }
    }
}

pub struct EngineRequest {
    pub request_id: String,
    pub source: RequestSource,
    pub session_id: Option<String>,
    pub message: String,
    pub ui_selected_skill_id: Option<String>,
    pub pin_selected_skill: bool,
    pub client_message_id: Option<String>,
    pub created_at: String,
}

impl EngineRequest {
    pub fn new(
        source: RequestSource,
        session_id: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            request_id: crate::runtime::ids::new_request_id(),
            source,
            session_id,
            message: message.into(),
            ui_selected_skill_id: None,
            pin_selected_skill: false,
            client_message_id: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn with_skill_selection(
        mut self,
        ui_selected_skill_id: Option<String>,
        pin_selected_skill: bool,
    ) -> Self {
        self.ui_selected_skill_id = ui_selected_skill_id;
        self.pin_selected_skill = pin_selected_skill;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[specta(rename_all = "snake_case")]
pub enum EngineResponseKind {
    Answer,
    ApprovalRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EngineResponse {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub kind: EngineResponseKind,
    pub answer: String,
    pub approval: Option<crate::state::views::ApprovalView>,
}

impl EngineResponse {
    pub fn answer(request_id: String, turn_id: String, session_id: String, answer: String) -> Self {
        Self {
            request_id,
            turn_id,
            session_id,
            kind: EngineResponseKind::Answer,
            answer,
            approval: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ApprovalActionResponse {
    pub request_id: String,
    pub approval_id: String,
    pub session_id: String,
    pub status: crate::state::types::ApprovalStatus,
    pub message: String,
    pub engine_response: Option<EngineResponse>,
    pub approval: Option<crate::state::views::ApprovalView>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnEnvelope {
    pub request_id: String,
    pub turn_id: String,
    pub session_id: String,
    pub surface: Surface,
    pub actor_id: String,
    pub user_message_id: String,
    pub created_at: String,
}

impl TurnEnvelope {
    pub fn new(request: &EngineRequest, session_id: String, user_message_id: String) -> Self {
        Self {
            request_id: request.request_id.clone(),
            turn_id: crate::runtime::ids::new_turn_id(),
            session_id,
            surface: request.source.surface(),
            actor_id: request.source.actor_id(),
            user_message_id,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveThemePreview {
    pub id: String,
    pub name: String,
    pub ui_only: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextPreview {
    pub profile_sent: Vec<String>,
    pub style_sent: Vec<String>,
    pub preferences_sent: Vec<String>,
    pub contexts_sent: Vec<String>,
    pub contexts_pinned: Vec<String>,
    pub contexts_not_sent: Vec<String>,
    #[serde(default)]
    pub selected_skills: Vec<SelectedSkillPreview>,
    #[serde(default)]
    pub skill_candidates: Vec<SkillCandidatePreview>,
    #[serde(default)]
    pub skill_warnings: Vec<String>,
    pub warnings: Vec<String>,
    pub final_context_text: String,
    #[serde(default)]
    pub active_mode: String,
    #[serde(default)]
    pub active_packs: Vec<String>,
    #[serde(default)]
    pub active_theme: Option<ActiveThemePreview>,
    #[serde(default)]
    pub style_source_pack: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectedSkillPreview {
    pub id: String,
    pub pack_id: Option<String>,
    pub name: String,
    pub score: u32,
    pub reason: String,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillCandidatePreview {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub accepted: bool,
    pub reason: String,
}

pub struct OpenNivaraEngine;

fn selected_skill_tool_allowlist(
    selected_skills: &[SelectedSkillPreview],
    registry: &crate::tools::ToolRegistry,
    tools_config: &crate::tools::ToolsConfig,
) -> Option<HashSet<String>> {
    if selected_skills.is_empty() {
        return None;
    }

    let selected: Vec<crate::skills::selector::SelectedSkill> = selected_skills
        .iter()
        .map(|skill| crate::skills::selector::SelectedSkill {
            id: skill.id.clone(),
            pack_id: skill.pack_id.clone(),
            name: skill.name.clone(),
            score: skill.score,
            reason: skill.reason.clone(),
            allowed_tools: skill.allowed_tools.clone(),
            denied_tools: skill.denied_tools.clone(),
        })
        .collect();

    Some(
        crate::skills::tool_policy::allowed_tools_for_selected_skills(
            &selected,
            registry,
            tools_config,
        )
        .allowed_tool_names,
    )
}

fn tool_execution_policy_error(
    tool_name: &str,
    selected_allowlist: Option<&HashSet<String>>,
    telegram_config: Option<&crate::remote_policy::TelegramConfig>,
    source: &RequestSource,
    tools_config: Option<&crate::tools::ToolsConfig>,
) -> Option<serde_json::Value> {
    if let Some(allowed) = selected_allowlist {
        if !allowed.contains(tool_name) {
            return Some(serde_json::json!({
                "error": format!(
                    "Tool '{}' blocked because it is not allowed by the selected skill policy.",
                    tool_name
                )
            }));
        }
    }

    if let Some(t_config) = telegram_config {
        if !remote_policy::is_tool_allowed(tool_name, t_config) {
            return Some(serde_json::json!({
                "error": "Tool call blocked by remote permissions policy."
            }));
        }
    }

    if matches!(source, RequestSource::Desktop) {
        let registry = crate::tools::ToolRegistry::new(true);
        let requires_confirmation = tools_config
            .and_then(|config| config.tools.get(tool_name))
            .map(|settings| settings.requires_confirmation)
            .unwrap_or(false);
        let is_risky = registry
            .definition(tool_name)
            .map(|definition| definition.risk_level != crate::tools::ToolRisk::Low)
            .unwrap_or(true);
        if requires_confirmation || is_risky {
            return Some(serde_json::json!({
                "error": format!(
                    "Tool '{}' requires desktop approval and was not executed.",
                    tool_name
                )
            }));
        }
    }

    None
}

fn context_preview_from_compiler(
    compiled: crate::memory::types::ContextCompilerOutput,
) -> ContextPreview {
    ContextPreview {
        profile_sent: compiled.profile_sent,
        style_sent: compiled.style_sent,
        preferences_sent: compiled.preferences_sent,
        contexts_sent: compiled.contexts_sent,
        contexts_pinned: compiled.contexts_pinned,
        contexts_not_sent: compiled.contexts_not_sent,
        selected_skills: compiled
            .selected_skills
            .into_iter()
            .map(|skill| SelectedSkillPreview {
                id: skill.id,
                pack_id: skill.pack_id,
                name: skill.name,
                score: skill.score,
                reason: skill.reason,
                allowed_tools: skill.allowed_tools,
                denied_tools: skill.denied_tools,
            })
            .collect(),
        skill_candidates: compiled
            .skill_candidates
            .into_iter()
            .map(|candidate| SkillCandidatePreview {
                id: candidate.id,
                name: candidate.name,
                score: candidate.score,
                accepted: candidate.accepted,
                reason: candidate.reason,
            })
            .collect(),
        skill_warnings: compiled.skill_warnings,
        warnings: compiled.warnings,
        final_context_text: compiled.raw_prompt,
        active_mode: compiled.active_mode,
        active_packs: compiled.active_packs,
        active_theme: compiled.active_theme.map(|theme| ActiveThemePreview {
            id: theme.id,
            name: theme.name,
            ui_only: theme.ui_only,
        }),
        style_source_pack: compiled.style_source_pack,
    }
}

fn current_workspace_context() -> Option<String> {
    crate::workspace_map::get_db_path()
        .ok()
        .filter(|path| path.exists())
        .map(|path| format!("Workspace map database available at {}.", path.display()))
}

fn content_from_text(role: &str, text: String) -> ModelMessage {
    ModelMessage::text(ModelRole::from_provider_role(role), text)
}

fn build_history_with_compiled_current(
    conversational_msgs: &[&DbMessage],
    current_user_message: &str,
    compiled_current_prompt: String,
    max_previous_messages: usize,
) -> Vec<ModelMessage> {
    let prior_end = conversational_msgs
        .last()
        .filter(|msg| msg.role == "user" && msg.content == current_user_message)
        .map(|_| conversational_msgs.len().saturating_sub(1))
        .unwrap_or(conversational_msgs.len());
    let prior_msgs = &conversational_msgs[..prior_end];
    let start_idx = prior_msgs.len().saturating_sub(max_previous_messages);

    let mut history: Vec<ModelMessage> = prior_msgs[start_idx..]
        .iter()
        .map(|msg| content_from_text(&msg.role, msg.content.clone()))
        .collect();
    history.push(content_from_text("user", compiled_current_prompt));
    history
}

impl Default for OpenNivaraEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenNivaraEngine {
    pub fn new() -> Self {
        Self
    }

    /// Assembles the complete LLM prompt context and reviews privacy selections, pins, and strict keyword scores.
    pub fn preview_context_for_message(
        &self,
        message: &str,
        session_id: Option<&str>,
    ) -> anyhow::Result<ContextPreview> {
        self.preview_context_for_message_with_skill(message, session_id, None)
    }

    pub fn preview_context_for_message_with_skill(
        &self,
        message: &str,
        session_id: Option<&str>,
        ui_selected_skill_id: Option<&str>,
    ) -> anyhow::Result<ContextPreview> {
        let session_conn = crate::sessions::init_db()?;
        let memory_conn = crate::memory::db::open_memory_db()?;
        let pinned_context_ids = session_id
            .and_then(|id| crate::sessions::list_pinned_contexts(&session_conn, id).ok())
            .unwrap_or_default();
        let session_pinned_skill_ids = session_id
            .and_then(|id| crate::sessions::list_pinned_skills(&session_conn, id).ok())
            .unwrap_or_default();
        let settings = crate::memory::db::get_settings(&memory_conn).unwrap_or_default();
        let timezone = crate::profile::read_profile()
            .ok()
            .map(|profile| profile.identity.timezone)
            .filter(|timezone| !timezone.trim().is_empty());
        let location = crate::runtime::location::get_location_context(
            &memory_conn,
            settings.allow_location_memories,
        )
        .unwrap_or_default();
        let runtime_context = crate::runtime::clock::runtime_context_at(
            chrono::Utc::now(),
            timezone.as_deref(),
            location,
        );
        let compiled = crate::memory::compiler::compile_context(
            &memory_conn,
            crate::memory::types::ContextCompilerInput {
                user_message: message.to_string(),
                session_id: session_id.map(str::to_string),
                message_id: None,
                runtime_context,
                model_context_limit:
                    crate::runtime::model_registry::get_current_model_context_info()
                        .context_window_tokens,
                reserved_output_tokens:
                    crate::runtime::model_registry::get_current_model_context_info()
                        .default_reserved_output_tokens,
                privacy_mode: settings.mode.clone(),
                effective_privacy_policy: Some(
                    crate::memory::types::EffectivePrivacyPolicy::from_settings(&settings),
                ),
                enabled_sources: vec!["chat".to_string(), "manual".to_string()],
                current_workspace_context: current_workspace_context(),
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids,
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: ui_selected_skill_id.map(str::to_string),
                session_pinned_skill_ids,
            },
        )?;

        Ok(context_preview_from_compiler(compiled))
    }

    /// Handles a new message from CLI or Telegram, resolving sessions, managing database state,
    /// constructing multi-turn context, invoking Gemini 2.5 Flash, executing tools safely, and returning the result.
    pub async fn handle_message(&self, request: EngineRequest) -> anyhow::Result<EngineResponse> {
        // 1. Initialize and connect to the sessions database
        let conn = sessions::init_db()?;

        // 2. Resolve active user key and session ID
        let user_key = match &request.source {
            RequestSource::Cli => "cli".to_string(),
            RequestSource::Desktop => "desktop".to_string(),
            RequestSource::Telegram { chat_id, .. } => format!("telegram_{}", chat_id),
        };

        let source_str = match &request.source {
            RequestSource::Cli => "CLI".to_string(),
            RequestSource::Desktop => "Desktop".to_string(),
            RequestSource::Telegram { chat_id, .. } => format!("Telegram ({})", chat_id),
        };

        let session_id = match request.session_id.clone() {
            Some(id) => {
                if sessions::get_session(&conn, &id)?.is_some() {
                    // Update user's active session marker to this specific ID
                    sessions::set_active_session(&conn, &user_key, &id)?;
                    id
                } else {
                    return Err(anyhow::anyhow!("Session not found: {}", id));
                }
            }
            None => {
                // Look up current active session for this user
                match sessions::get_active_session(&conn, &user_key)? {
                    Some(id) => id,
                    None => {
                        // Create a new session
                        let id = sessions::create_session(&conn, &source_str, None)?;
                        sessions::set_active_session(&conn, &user_key, &id)?;
                        id
                    }
                }
            }
        };

        if request.pin_selected_skill {
            if let Some(skill_id) = request.ui_selected_skill_id.as_deref() {
                sessions::pin_skill(&conn, &session_id, skill_id)?;
            }
        }

        // 3. Store the user's message in the session history database
        let user_message = sessions::store_message(
            &conn,
            &session_id,
            "user",
            &source_str,
            &request.message,
            None,
        )?;
        let turn = TurnEnvelope::new(&request, session_id.clone(), user_message.id.clone());

        // 4. Load configuration files (Profile, Style, Preferences, Tools)
        let tools_path = crate::tools::get_tools_path()?;
        let tools_config = if tools_path.exists() {
            Some(crate::tools::read_tools()?)
        } else {
            None
        };

        // Load telegram config if using Telegram
        let telegram_config = if let RequestSource::Telegram { .. } = &request.source {
            Some(crate::remote_policy::read_telegram()?)
        } else {
            None
        };

        // 5. Build system prompt instructions and context blocks using preview selector
        let preview = self.preview_context_for_message_with_skill(
            &request.message,
            Some(&session_id),
            request.ui_selected_skill_id.as_deref(),
        )?;
        let context_block_full = preview.final_context_text;

        // 6. Build the multi-turn conversational history
        let all_msgs = sessions::get_session_messages(&conn, &session_id)?;

        // Filter history to "user" and "model" roles to avoid API conflicts
        let conversational_msgs: Vec<&DbMessage> = all_msgs
            .iter()
            .filter(|m| m.role == "user" || m.role == "model")
            .collect();

        // Limit to last 20 messages including the compiled current user turn.
        const MAX_SESSION_CONTEXT_MESSAGES: usize = 20;
        let mut history = build_history_with_compiled_current(
            &conversational_msgs,
            &request.message,
            context_block_full,
            MAX_SESSION_CONTEXT_MESSAGES.saturating_sub(1),
        );

        // 7. Resolve allowed tool declarations
        let tools_enabled = tools_config
            .as_ref()
            .map(|c| c.general.enabled)
            .unwrap_or(false);
        let mut tools_declaration = Vec::new();

        if tools_enabled {
            let has_map = if let Ok(db_path) = crate::workspace_map::get_db_path() {
                db_path.exists()
            } else {
                false
            };
            let registry = crate::tools::ToolRegistry::new(has_map);
            let selected_skill_tool_allowlist = selected_skill_tool_allowlist(
                &preview.selected_skills,
                &registry,
                tools_config
                    .as_ref()
                    .expect("tools_config is present when tools_enabled is true"),
            );
            let functions: Vec<_> = registry
                .declared_definitions(
                    tools_config
                        .as_ref()
                        .expect("tools_config is present when tools_enabled is true"),
                    selected_skill_tool_allowlist.as_ref(),
                )
                .into_iter()
                .collect();

            // Filter declarations by source policy before exposing tools to Gemini.
            let filtered_functions: Vec<ModelToolDeclaration> = functions
                .into_iter()
                .filter(|f| {
                    if let Some(ref t_config) = telegram_config {
                        return remote_policy::is_tool_allowed(&f.name, t_config);
                    }
                    if matches!(&request.source, RequestSource::Desktop) {
                        let requires_confirmation = tools_config
                            .as_ref()
                            .and_then(|config| config.tools.get(&f.name))
                            .map(|settings| settings.requires_confirmation)
                            .unwrap_or(false);
                        return !requires_confirmation
                            && f.risk_level == crate::tools::ToolRisk::Low;
                    }
                    true
                })
                .map(|definition| ModelToolDeclaration {
                    name: definition.name,
                    description: definition.description,
                    parameters: definition.parameters,
                })
                .collect();

            if !filtered_functions.is_empty() {
                tools_declaration = filtered_functions;
            }
        }
        let selected_skill_tool_allowlist = if let Some(ref config) = tools_config {
            let has_map = if let Ok(db_path) = crate::workspace_map::get_db_path() {
                db_path.exists()
            } else {
                false
            };
            selected_skill_tool_allowlist(
                &preview.selected_skills,
                &crate::tools::ToolRegistry::new(has_map),
                config,
            )
        } else if preview.selected_skills.is_empty() {
            None
        } else {
            Some(HashSet::new())
        };

        // 8. Configure model provider
        let api_key = crate::secrets::get_gemini_api_key()?;
        let provider = crate::model::gemini::GeminiProvider::new(api_key);

        let max_rounds = tools_config
            .as_ref()
            .map(|c| c.general.max_tool_rounds)
            .unwrap_or(3);
        let show_activity = tools_config
            .as_ref()
            .map(|c| c.general.show_tool_activity)
            .unwrap_or(true);
        let mut current_round = 0;

        // 9. Tool Calling Dialogue loop
        let answer = loop {
            let model_response = provider
                .generate(ModelRequest {
                    messages: history.clone(),
                    tools: tools_declaration.clone(),
                })
                .await?;
            let content = model_response.message;

            // Push model dialogue turn to temporary thread history
            history.push(content.clone());

            // Inspect if a tool function call was requested
            let requested_call = content.first_tool_call().cloned();

            if let Some(call) = requested_call {
                if current_round >= max_rounds {
                    if show_activity {
                        match &request.source {
                            RequestSource::Cli => println!("\n\x1b[1;33m[OpenNivara Limit]\x1b[0m Max tool rounds limit ({}) reached.", max_rounds),
                            RequestSource::Desktop => tracing::warn!("Max tool rounds limit ({}) reached.", max_rounds),
                            RequestSource::Telegram { .. } => tracing::warn!("Max tool rounds limit ({}) reached.", max_rounds),
                        }
                    }

                    history.push(ModelMessage::tool_result(
                        call.tool_call_id,
                        call.name,
                        serde_json::json!({
                            "error": format!("Tool execution halted: maximum rounds limit ({}) was reached.", max_rounds)
                        }),
                    ));
                    current_round += 1;
                    continue;
                }

                if let Some(policy_error) = tool_execution_policy_error(
                    &call.name,
                    selected_skill_tool_allowlist.as_ref(),
                    telegram_config.as_ref(),
                    &request.source,
                    tools_config.as_ref(),
                ) {
                    history.push(ModelMessage::tool_result(
                        call.tool_call_id,
                        call.name,
                        policy_error,
                    ));
                    current_round += 1;
                    continue;
                }

                // Log or print activity
                let args_str = serde_json::to_string(&call.arguments).unwrap_or_default();
                match &request.source {
                    RequestSource::Cli => {
                        if show_activity {
                            println!(
                                "\x1b[1;34mOpenNivara tool:\x1b[0m {} {}",
                                call.name, args_str
                            );
                        }
                    }
                    RequestSource::Desktop => {
                        tracing::info!(
                            "Desktop OpenNivara tool requested: {} {}",
                            call.name,
                            args_str
                        );
                    }
                    RequestSource::Telegram { .. } => {
                        tracing::info!(
                            "Telegram OpenNivara tool requested: {} {}",
                            call.name,
                            args_str
                        );
                    }
                }

                // Execute safe local or workspace map tools through the registry.
                let has_map = if let Ok(db_path) = crate::workspace_map::get_db_path() {
                    db_path.exists()
                } else {
                    false
                };
                let mut tool_result = if let Some(ref t_config) = tools_config {
                    crate::tools::ToolRegistry::new(has_map).execute(
                        &call.name,
                        &call.arguments,
                        t_config,
                    )
                } else {
                    serde_json::json!({ "error": "Local tools are not initialized." })
                };

                // Truncate file reading content if Telegram limits are configured
                if call.name == "read_file" {
                    if let Some(ref t_config) = telegram_config {
                        if let Some(content) = tool_result.get_mut("content") {
                            if let Some(text) = content.as_str() {
                                let max_chars = t_config.limits.max_file_preview_chars;
                                if text.len() > max_chars {
                                    let mut truncated = text[..max_chars].to_string();
                                    truncated.push_str("\n\n[WARNING: File preview truncated for Telegram remote security constraints.]");
                                    *content = serde_json::json!(truncated);
                                    if let Some(trunc_field) = tool_result.get_mut("truncated") {
                                        *trunc_field = serde_json::json!(true);
                                    }
                                }
                            }
                        }
                    }
                }

                // Feed response back to conversation loop history
                history.push(ModelMessage::tool_result(
                    call.tool_call_id,
                    call.name,
                    tool_result,
                ));

                current_round += 1;
                continue;
            }

            // Expose conversational text reply
            match content.first_text() {
                Some(text) => break text.trim().to_string(),
                None => {
                    return Err(anyhow::anyhow!(
                        "Gemini returned response candidates, but no text block was found."
                    ))
                }
            }
        };

        // 10. Persist model reply in session history database
        sessions::store_message(
            &conn,
            &session_id,
            "model",
            "OpenNivara Engine",
            &answer,
            None,
        )?;

        Ok(EngineResponse::answer(
            turn.request_id,
            turn.turn_id,
            session_id,
            answer,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote_policy::{
        AuthConfig, ConfirmationsConfig, GeneralConfig as TelegramGeneralConfig, LimitsConfig,
        PermissionsConfig, TelegramConfig,
    };
    use crate::tools::{GeneralConfig, PathsConfig, ToolSettings, ToolsConfig};
    use std::collections::{HashMap, HashSet};

    fn tools_config() -> ToolsConfig {
        let mut tools = HashMap::new();
        for name in [
            "get_current_dir",
            "list_dir",
            "file_exists",
            "read_file",
            "map_search",
        ] {
            tools.insert(
                name.to_string(),
                ToolSettings {
                    enabled: true,
                    requires_confirmation: false,
                    max_bytes: None,
                },
            );
        }
        ToolsConfig {
            general: GeneralConfig {
                enabled: true,
                max_tool_rounds: 3,
                show_tool_activity: false,
            },
            paths: PathsConfig {
                allowed_roots: vec![".".to_string()],
                blocked_patterns: vec![],
            },
            tools,
        }
    }

    fn selected_skill(allow: Vec<&str>, deny: Vec<&str>) -> SelectedSkillPreview {
        SelectedSkillPreview {
            id: "file_reader".to_string(),
            pack_id: Some("test_pack".to_string()),
            name: "File Reader".to_string(),
            score: 100,
            reason: "test".to_string(),
            allowed_tools: allow.into_iter().map(String::from).collect(),
            denied_tools: deny.into_iter().map(String::from).collect(),
        }
    }

    fn telegram_config_allowing_read_file(allow_read_file: bool) -> TelegramConfig {
        TelegramConfig {
            general: TelegramGeneralConfig {
                enabled: true,
                mode: "polling".to_string(),
                bot_name: "OpenNivara".to_string(),
            },
            auth: AuthConfig {
                allowed_chat_ids: vec![],
            },
            permissions: PermissionsConfig {
                allow_ask: true,
                allow_chat: true,
                allow_status: true,
                allow_sessions: true,
                allow_map_summary: true,
                allow_map_search: true,
                allow_map_tree: true,
                allow_map_get_node: true,
                allow_read_file,
                allow_open_app: false,
                allow_open_url: false,
                allow_write_file: false,
                allow_run_command: false,
                allow_profile_write: false,
                allow_style_write: false,
                allow_preferences_write: false,
                allow_contexts_write: false,
            },
            confirmations: ConfirmationsConfig {
                require_confirmation_for_read_file: true,
                require_confirmation_for_open_app: true,
                require_confirmation_for_open_url: true,
                require_confirmation_for_any_local_tool: true,
            },
            limits: LimitsConfig {
                max_response_chars: 3500,
                max_file_preview_chars: 2000,
                max_messages_per_minute: 20,
            },
        }
    }

    #[test]
    fn selected_skill_blocks_tool_execution_outside_allowlist() {
        let allowlist = HashSet::from(["read_file".to_string()]);

        let error = tool_execution_policy_error(
            "map_search",
            Some(&allowlist),
            None,
            &RequestSource::Cli,
            None,
        )
        .unwrap();

        assert_eq!(
            error["error"],
            "Tool 'map_search' blocked because it is not allowed by the selected skill policy."
        );
    }

    #[test]
    fn no_selected_skill_preserves_default_tool_execution_policy() {
        assert!(
            tool_execution_policy_error("map_search", None, None, &RequestSource::Cli, None)
                .is_none()
        );
    }

    #[test]
    fn telegram_policy_blocks_even_when_skill_allows_tool() {
        let allowlist = HashSet::from(["read_file".to_string()]);
        let telegram = telegram_config_allowing_read_file(false);

        let error = tool_execution_policy_error(
            "read_file",
            Some(&allowlist),
            Some(&telegram),
            &RequestSource::Cli,
            None,
        )
        .unwrap();

        assert_eq!(
            error["error"],
            "Tool call blocked by remote permissions policy."
        );
    }

    #[test]
    fn selected_skill_allowlist_uses_tool_policy_deny_wins() {
        let selected = vec![selected_skill(
            vec!["read_file", "map_search"],
            vec!["map_search"],
        )];
        let allowlist = selected_skill_tool_allowlist(
            &selected,
            &crate::tools::ToolRegistry::new(true),
            &tools_config(),
        )
        .unwrap();

        assert!(allowlist.contains("read_file"));
        assert!(!allowlist.contains("map_search"));
    }

    fn db_message(id: &str, role: &str, content: &str) -> DbMessage {
        DbMessage {
            id: id.to_string(),
            session_id: "session".to_string(),
            role: role.to_string(),
            source: "test".to_string(),
            content: content.to_string(),
            created_at: "2026-06-03T00:00:00Z".to_string(),
            metadata_json: None,
        }
    }

    #[test]
    fn compiled_prompt_is_sent_as_current_user_turn_without_duplication() {
        let prior_user = db_message("1", "user", "earlier question");
        let prior_model = db_message("2", "model", "earlier answer");
        let current = db_message("3", "user", "current question");
        let messages = vec![&prior_user, &prior_model, &current];

        let history = build_history_with_compiled_current(
            &messages,
            "current question",
            "compiled prompt with Current User Message: current question".to_string(),
            19,
        );

        assert_eq!(history.len(), 3);
        assert_eq!(history[0].role, ModelRole::User);
        assert_eq!(history[1].role, ModelRole::Model);
        assert_eq!(history[2].role, ModelRole::User);
        assert_eq!(
            history[2].first_text(),
            Some("compiled prompt with Current User Message: current question")
        );
        assert!(!history[0].first_text().unwrap().contains("compiled prompt"));
    }

    #[test]
    fn gemini_endpoint_does_not_put_api_key_in_url() {
        let url = crate::model::gemini::gemini_generate_content_url();

        assert!(!url.contains("?key="));
        assert!(!url.contains("raw-secret"));
    }

    #[test]
    fn desktop_source_blocks_confirmation_required_tool_execution() {
        let mut config = tools_config();
        config.tools.insert(
            "read_file".to_string(),
            ToolSettings {
                enabled: true,
                requires_confirmation: true,
                max_bytes: Some(20_000),
            },
        );

        let error = tool_execution_policy_error(
            "read_file",
            Some(&HashSet::from(["read_file".to_string()])),
            None,
            &RequestSource::Desktop,
            Some(&config),
        )
        .unwrap();

        assert_eq!(
            error["error"],
            "Tool 'read_file' requires desktop approval and was not executed."
        );
    }

    #[test]
    fn cli_source_keeps_declared_tool_policy_separate_from_desktop() {
        let mut config = tools_config();
        config.tools.insert(
            "read_file".to_string(),
            ToolSettings {
                enabled: true,
                requires_confirmation: true,
                max_bytes: Some(20_000),
            },
        );

        assert!(tool_execution_policy_error(
            "read_file",
            Some(&HashSet::from(["read_file".to_string()])),
            None,
            &RequestSource::Cli,
            Some(&config),
        )
        .is_none());
    }

    #[test]
    fn request_source_maps_to_surface_and_actor_id() {
        assert_eq!(RequestSource::Cli.surface(), Surface::Cli);
        assert_eq!(RequestSource::Cli.actor_id(), "cli_owner");

        assert_eq!(RequestSource::Desktop.surface(), Surface::Desktop);
        assert_eq!(RequestSource::Desktop.actor_id(), "desktop_owner");

        let telegram = RequestSource::Telegram {
            chat_id: 42,
            username: Some("vatsal".to_string()),
        };
        assert_eq!(telegram.surface(), Surface::Telegram);
        assert_eq!(telegram.actor_id(), "telegram_42");
    }

    #[test]
    fn engine_request_new_fills_runtime_fields() {
        let request =
            EngineRequest::new(RequestSource::Cli, Some("sess_existing".to_string()), "hi");

        assert!(request.request_id.starts_with("req_"));
        assert_eq!(request.source, RequestSource::Cli);
        assert_eq!(request.session_id.as_deref(), Some("sess_existing"));
        assert_eq!(request.message, "hi");
        assert_eq!(request.client_message_id, None);
        assert!(!request.created_at.is_empty());
        assert_eq!(request.ui_selected_skill_id, None);
        assert!(!request.pin_selected_skill);
    }

    #[test]
    fn turn_envelope_uses_request_source_metadata() {
        let request = EngineRequest::new(RequestSource::Desktop, None, "hello");
        let envelope = TurnEnvelope::new(&request, "sess_123".to_string(), "msg_456".to_string());

        assert_eq!(envelope.request_id, request.request_id);
        assert!(envelope.turn_id.starts_with("turn_"));
        assert_eq!(envelope.session_id, "sess_123");
        assert_eq!(envelope.surface, Surface::Desktop);
        assert_eq!(envelope.actor_id, "desktop_owner");
        assert_eq!(envelope.user_message_id, "msg_456");
        assert!(!envelope.created_at.is_empty());
    }

    #[test]
    fn engine_response_answer_carries_request_and_turn_ids() {
        let response = EngineResponse::answer(
            "req_123".to_string(),
            "turn_456".to_string(),
            "sess_789".to_string(),
            "done".to_string(),
        );

        assert_eq!(response.request_id, "req_123");
        assert_eq!(response.turn_id, "turn_456");
        assert_eq!(response.session_id, "sess_789");
        assert_eq!(response.kind, EngineResponseKind::Answer);
        assert_eq!(response.answer, "done");
    }
}
