#[tauri::command]
async fn ask_opennivara(
    message: String,
    session_id: Option<String>,
    ui_selected_skill_id: Option<String>,
    pin_selected_skill: Option<bool>,
) -> Result<opennivara::engine::EngineResponse, String> {
    if !opennivara::secrets::gemini_key_status()
        .map(|status| status.available)
        .unwrap_or(false)
    {
        return Err(
            "Missing Gemini API key. Add it in desktop onboarding/settings or set GEMINI_API_KEY."
                .to_string(),
        );
    }

    // Instantiate a new engine handler
    let engine = opennivara::engine::OpenNivaraEngine::new();

    // Call unified message handler
    engine
        .handle_message(
            opennivara::engine::EngineRequest::new(
                opennivara::engine::RequestSource::Desktop,
                session_id,
                message,
            )
            .with_skill_selection(ui_selected_skill_id, pin_selected_skill.unwrap_or(false)),
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_sessions() -> Result<Vec<opennivara::sessions::Session>, String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    let list = opennivara::sessions::list_sessions(&conn).map_err(|e| e.to_string())?;
    Ok(list)
}

#[tauri::command]
async fn get_session_messages(
    session_id: String,
) -> Result<Vec<opennivara::sessions::DbMessage>, String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    let list = opennivara::sessions::get_session_messages(&conn, &session_id)
        .map_err(|e| e.to_string())?;
    Ok(list)
}

#[tauri::command]
async fn list_tools() -> Result<opennivara::tools::ToolsConfig, String> {
    opennivara::tools::read_tools().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tools_path() -> Result<String, String> {
    opennivara::tools::get_tools_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_profile_path() -> Result<String, String> {
    opennivara::profile::get_profile_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_preferences_path() -> Result<String, String> {
    opennivara::preferences::get_preferences_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_style_path() -> Result<String, String> {
    opennivara::style::get_style_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_contexts_path() -> Result<String, String> {
    opennivara::context::get_contexts_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_telegram_path() -> Result<String, String> {
    opennivara::remote_policy::get_telegram_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn map_summary() -> Result<String, String> {
    opennivara::workspace_map::render_summary().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_profile() -> Result<opennivara::profile::Profile, String> {
    opennivara::profile::read_profile().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_profile(profile: opennivara::profile::Profile) -> Result<(), String> {
    opennivara::profile::save_profile(&profile).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_style() -> Result<opennivara::style::OpenNivaraStyle, String> {
    opennivara::style::read_style().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_style(style: opennivara::style::OpenNivaraStyle) -> Result<(), String> {
    opennivara::style::save_style(&style).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_preferences() -> Result<opennivara::preferences::PreferencesFile, String> {
    opennivara::preferences::read_preferences().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_preferences(
    preferences: opennivara::preferences::PreferencesFile,
) -> Result<(), String> {
    opennivara::preferences::save_preferences(&preferences).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_contexts() -> Result<opennivara::context::ContextsFile, String> {
    opennivara::context::read_contexts().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_contexts(contexts: opennivara::context::ContextsFile) -> Result<(), String> {
    opennivara::context::save_contexts(&contexts).map_err(|e| e.to_string())
}

#[tauri::command]
async fn preview_context_for_message(
    message: String,
    session_id: Option<String>,
    ui_selected_skill_id: Option<String>,
) -> Result<opennivara::engine::ContextPreview, String> {
    let engine = opennivara::engine::OpenNivaraEngine::new();
    engine
        .preview_context_for_message_with_skill(
            &message,
            session_id.as_deref(),
            ui_selected_skill_id.as_deref(),
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn skills_list() -> Result<Vec<opennivara::skills::registry::SkillSummary>, String> {
    opennivara::skills::registry::list_skill_summaries().map_err(|e| e.to_string())
}

#[tauri::command]
fn skills_get(skill_id: String) -> Result<opennivara::skills::manifest::SkillManifest, String> {
    opennivara::skills::registry::get_skill(&skill_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn skills_set_enabled(skill_id: String, enabled: bool) -> Result<(), String> {
    opennivara::skills::registry::set_skill_enabled(&skill_id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn skills_test_route(
    message: String,
) -> Result<opennivara::skills::selector::RouteDecision, String> {
    opennivara::skills::registry::test_route(message).map_err(|e| e.to_string())
}

#[tauri::command]
async fn pin_context(session_id: String, context_id: String) -> Result<(), String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    opennivara::sessions::pin_context(&conn, &session_id, &context_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn unpin_context(session_id: String, context_id: String) -> Result<(), String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    opennivara::sessions::unpin_context(&conn, &session_id, &context_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn pin_skill(session_id: String, skill_id: String) -> Result<(), String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    opennivara::sessions::pin_skill(&conn, &session_id, &skill_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn unpin_skill(session_id: String, skill_id: String) -> Result<(), String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    opennivara::sessions::unpin_skill(&conn, &session_id, &skill_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_pinned_skills(session_id: String) -> Result<Vec<String>, String> {
    let conn = opennivara::sessions::init_db().map_err(|e| e.to_string())?;
    opennivara::sessions::list_pinned_skills(&conn, &session_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn check_api_key() -> bool {
    opennivara::secrets::gemini_key_status()
        .map(|status| status.available)
        .unwrap_or(false)
}

#[tauri::command]
fn check_gemini_key() -> Result<opennivara::secrets::ApiKeyStatus, String> {
    opennivara::secrets::gemini_key_status().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_gemini_key(secret: String) -> Result<(), String> {
    opennivara::secrets::save_gemini_key(&secret).map_err(|e| e.to_string())
}

#[tauri::command]
fn first_run_status() -> Result<opennivara::first_run::FirstRunStatus, String> {
    opennivara::first_run::first_run_status().map_err(|e| e.to_string())
}

#[tauri::command]
fn initialize_clean_first_run(
    input: opennivara::first_run::FirstRunInput,
) -> Result<opennivara::first_run::FirstRunStatus, String> {
    opennivara::first_run::initialize_clean_first_run(input).map_err(|e| e.to_string())
}

fn memory_conn() -> Result<rusqlite::Connection, String> {
    opennivara::memory::db::open_memory_db().map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_init() -> Result<opennivara::memory::types::MemoryStatus, String> {
    opennivara::memory::db::status().map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_status() -> Result<opennivara::memory::types::MemoryStatus, String> {
    opennivara::memory::db::status().map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_validate() -> Result<opennivara::memory::types::MemoryStatus, String> {
    opennivara::memory::db::validate().map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_repair() -> Result<opennivara::memory::types::MemoryStatus, String> {
    opennivara::memory::db::repair().map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_get_settings() -> Result<opennivara::memory::types::MemorySettings, String> {
    let conn = memory_conn()?;
    opennivara::memory::db::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_save_settings(settings: opennivara::memory::types::MemorySettings) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::db::save_settings(&conn, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_create_source(
    input: opennivara::memory::types::CreateMemorySource,
) -> Result<opennivara::memory::types::MemorySource, String> {
    let conn = memory_conn()?;
    opennivara::memory::db::create_source(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_create_item(
    input: opennivara::memory::types::CreateMemoryItem,
) -> Result<opennivara::memory::types::MemoryItem, String> {
    let conn = memory_conn()?;
    opennivara::memory::db::create_memory_item(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_get_item(
    memory_id: String,
) -> Result<Option<opennivara::memory::types::MemoryItem>, String> {
    let conn = memory_conn()?;
    opennivara::memory::db::get_memory_item(&conn, &memory_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_list_items(
    limit: Option<u32>,
) -> Result<Vec<opennivara::memory::types::MemoryItem>, String> {
    let conn = memory_conn()?;
    opennivara::memory::db::list_memory_items(&conn, limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_update_item(
    item: opennivara::memory::types::MemoryItem,
) -> Result<opennivara::memory::types::MemoryItem, String> {
    let conn = memory_conn()?;
    opennivara::memory::db::update_memory_item(&conn, &item).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_delete_item(memory_id: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::db::delete_memory_item(&conn, &memory_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_retract_item(memory_id: String, reason: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::db::retract_memory_item(&conn, &memory_id, &reason)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_search(
    query: opennivara::memory::types::MemorySearchQuery,
) -> Result<Vec<opennivara::memory::types::MemorySearchResult>, String> {
    let conn = memory_conn()?;
    opennivara::memory::retrieval::search_memory(&conn, &query).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_create_facet(
    input: opennivara::memory::types::CreateMemoryFacet,
) -> Result<opennivara::memory::types::MemoryFacet, String> {
    let conn = memory_conn()?;
    opennivara::memory::facets::create_facet(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_add_facet(
    input: opennivara::memory::types::CreateMemoryFacet,
) -> Result<opennivara::memory::types::MemoryFacet, String> {
    memory_create_facet(input)
}

#[tauri::command]
fn memory_list_facets(
    memory_id: Option<String>,
    domain: Option<String>,
    facet_type: Option<String>,
    label: Option<String>,
) -> Result<Vec<opennivara::memory::types::MemoryFacet>, String> {
    let conn = memory_conn()?;
    opennivara::memory::facets::list_facets(
        &conn,
        opennivara::memory::facets::FacetFilter {
            memory_id,
            domain,
            facet_type,
            label,
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_update_facet(
    facet: opennivara::memory::types::MemoryFacet,
) -> Result<opennivara::memory::types::MemoryFacet, String> {
    let conn = memory_conn()?;
    opennivara::memory::facets::update_facet(&conn, &facet).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_delete_facet(facet_id: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::facets::delete_facet(&conn, &facet_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_rebuild() -> Result<opennivara::memory::graph::MemoryGraphStatus, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_rebuild_from_sqlite(&conn).map_err(|e| e.to_string())?;
    opennivara::memory::graph::graph_status(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_status() -> Result<opennivara::memory::graph::MemoryGraphStatus, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_status(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_validate() -> Result<Vec<String>, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_validate_consistency(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_neighbors(
    node_id: String,
    limit: Option<u32>,
) -> Result<Vec<opennivara::memory::graph::MemoryGraphNode>, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_neighbors(&conn, &node_id, limit.unwrap_or(25))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_memory_context(
    memory_id: String,
    max_depth: Option<u32>,
) -> Result<opennivara::memory::graph::MemoryGraphContext, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_memory_context(&conn, &memory_id, max_depth.unwrap_or(2))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_entity_context(
    entity_id: String,
    max_depth: Option<u32>,
) -> Result<opennivara::memory::graph::MemoryGraphContext, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_entity_context(&conn, &entity_id, max_depth.unwrap_or(2))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_related_memories(
    memory_id: String,
    limit: Option<u32>,
) -> Result<Vec<opennivara::memory::graph::MemoryGraphNode>, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_find_related_memories(&conn, &memory_id, limit.unwrap_or(25))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_shortest_path(
    from_node_id: String,
    to_node_id: String,
    max_depth: Option<u32>,
) -> Result<opennivara::memory::graph::MemoryGraphContext, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_shortest_path(
        &conn,
        &from_node_id,
        &to_node_id,
        max_depth.unwrap_or(4),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_find_entity_mentions(
    mention: String,
    limit: Option<u32>,
) -> Result<Vec<opennivara::memory::graph::MemoryGraphNode>, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_find_entity_mentions(&conn, &mention, limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_graph_add_edge(
    input: opennivara::memory::graph::CreateGraphEdge,
) -> Result<opennivara::memory::graph::MemoryGraphEdge, String> {
    let conn = memory_conn()?;
    opennivara::memory::graph::graph_add_edge(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_extract_proposals_for_message(
    message: String,
    session_id: Option<String>,
    mode: Option<opennivara::memory::types::MemoryMode>,
) -> Result<Vec<opennivara::memory::types::MemoryExtractionProposal>, String> {
    let conn = memory_conn()?;
    let settings = opennivara::memory::db::get_settings(&conn).map_err(|e| e.to_string())?;
    let mode = mode.unwrap_or_else(|| settings.mode.clone());
    if !opennivara::memory::privacy::memory_saving_allowed(
        &mode,
        settings.private_chat,
        settings.pause_memory,
    ) {
        return Ok(vec![]);
    }
    opennivara::memory::extraction::extract_proposals_for_message(&conn, &message, session_id, mode)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_list_proposals(
) -> Result<Vec<opennivara::memory::types::MemoryExtractionProposal>, String> {
    let conn = memory_conn()?;
    opennivara::memory::extraction::list_memory_proposals(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_approve_proposal(proposal_id: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::extraction::approve_memory_proposal(&conn, &proposal_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_reject_proposal(proposal_id: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::extraction::reject_memory_proposal(&conn, &proposal_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_compile_context(
    input: opennivara::memory::types::ContextCompilerInput,
) -> Result<opennivara::memory::types::ContextCompilerOutput, String> {
    let conn = memory_conn()?;
    opennivara::memory::compiler::compile_context(&conn, input).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_get_last_audit(
    _session_id: Option<String>,
) -> Result<Option<opennivara::memory::types::PromptAudit>, String> {
    let conn = memory_conn()?;
    opennivara::memory::audit::get_last_audit(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_embedding_status() -> opennivara::memory::embeddings::EmbeddingStatus {
    opennivara::memory::embeddings::status()
}

#[tauri::command]
fn memory_list_tasks(
    status: Option<String>,
) -> Result<Vec<opennivara::memory::tasks::MemoryTask>, String> {
    let conn = memory_conn()?;
    opennivara::memory::tasks::list_tasks(&conn, status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_update_task_status(memory_id: String, status: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::memory::tasks::update_task_status(&conn, &memory_id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_list_due_reminders(
    now_utc: String,
) -> Result<Vec<opennivara::memory::tasks::MemoryTask>, String> {
    let conn = memory_conn()?;
    opennivara::memory::reminders::list_due_reminders(&conn, &now_utc).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_list_corrections(
) -> Result<Vec<opennivara::memory::corrections::MemoryCorrection>, String> {
    let conn = memory_conn()?;
    opennivara::memory::corrections::list_corrections(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_resolve_entity_mention(
    mention: String,
) -> Result<opennivara::memory::entities::EntityResolutionResult, String> {
    let conn = memory_conn()?;
    opennivara::memory::entities::resolve_entity_mention(&conn, &mention).map_err(|e| e.to_string())
}

#[tauri::command]
fn runtime_get_context(
    timezone: Option<String>,
    allow_exact_location: Option<bool>,
) -> Result<opennivara::runtime::context::RuntimeContext, String> {
    let conn = memory_conn()?;
    let location = opennivara::runtime::location::get_location_context(
        &conn,
        allow_exact_location.unwrap_or(false),
    )
    .map_err(|e| e.to_string())?;
    Ok(opennivara::runtime::clock::runtime_context_at(
        chrono::Utc::now(),
        timezone.as_deref(),
        location,
    ))
}

#[tauri::command]
fn runtime_resolve_time_phrase(phrase: String, timezone: Option<String>) -> String {
    let runtime = opennivara::runtime::clock::get_runtime_context(timezone.as_deref());
    opennivara::runtime::clock::resolve_relative_time_phrase(&phrase, &runtime).unwrap_or_default()
}

#[tauri::command]
fn runtime_get_model_context_info(
    provider: Option<String>,
    model: Option<String>,
) -> opennivara::runtime::model_registry::ModelContextInfo {
    match (provider, model) {
        (Some(provider), Some(model)) => {
            opennivara::runtime::model_registry::get_model_context_info(&provider, &model)
        }
        _ => opennivara::runtime::model_registry::get_current_model_context_info(),
    }
}

#[tauri::command]
fn location_get_context(
    allow_exact: Option<bool>,
) -> Result<opennivara::runtime::location::LocationContext, String> {
    let conn = memory_conn()?;
    opennivara::runtime::location::get_location_context(&conn, allow_exact.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn location_save_place(
    input: opennivara::runtime::location::CreateSavedPlace,
) -> Result<opennivara::runtime::location::SavedPlace, String> {
    let conn = memory_conn()?;
    opennivara::runtime::location::save_place(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
fn location_update_manual(
    input: opennivara::runtime::location::LocationObservationInput,
) -> Result<opennivara::runtime::location::LocationContext, String> {
    let conn = memory_conn()?;
    opennivara::runtime::location::update_manual_location(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
fn location_list_saved_places() -> Result<Vec<opennivara::runtime::location::SavedPlace>, String> {
    let conn = memory_conn()?;
    opennivara::runtime::location::list_saved_places(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn location_delete_saved_place(place_id: String) -> Result<(), String> {
    let conn = memory_conn()?;
    opennivara::runtime::location::delete_saved_place(&conn, &place_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn location_record_observation(
    input: opennivara::runtime::location::LocationObservationInput,
) -> Result<opennivara::runtime::location::LocationContext, String> {
    let conn = memory_conn()?;
    opennivara::runtime::location::record_location_observation(&conn, &input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_init() -> Result<String, String> {
    opennivara::marketplace::init_marketplace().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_list_installed_packs(
) -> Result<opennivara::marketplace::packs::InstalledPacksFile, String> {
    opennivara::marketplace::packs::list_installed_packs().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_preview_pack(
    path: String,
) -> Result<opennivara::marketplace::packs::PackPreview, String> {
    opennivara::marketplace::packs::preview_pack_from_path(std::path::PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_install_pack(
    path: String,
) -> Result<opennivara::marketplace::packs::InstalledPack, String> {
    opennivara::marketplace::packs::install_pack_from_path(std::path::PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_uninstall_pack(pack_id: String) -> Result<(), String> {
    opennivara::marketplace::packs::uninstall_pack(&pack_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_list_builtin_packs(
) -> Result<Vec<opennivara::marketplace::builtin::BuiltinPackSummary>, String> {
    opennivara::marketplace::builtin::list_builtin_packs().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_install_builtin_pack(
    pack_id: String,
) -> Result<opennivara::marketplace::packs::InstalledPack, String> {
    opennivara::marketplace::builtin::install_builtin_pack(&pack_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_preview_installed_pack(
    pack_id: String,
) -> Result<opennivara::marketplace::packs::PackPreview, String> {
    opennivara::marketplace::packs::preview_installed_pack(&pack_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_get_modes() -> Result<opennivara::marketplace::modes::ModesFile, String> {
    opennivara::marketplace::modes::read_modes().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_set_active_mode(mode_id: String) -> Result<(), String> {
    opennivara::marketplace::modes::set_active_mode(&mode_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_create_mode(
    mode: opennivara::marketplace::modes::OpenNivaraMode,
) -> Result<(), String> {
    opennivara::marketplace::modes::create_mode(mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_add_pack_to_mode(mode_id: String, pack_id: String) -> Result<(), String> {
    opennivara::marketplace::modes::add_pack_to_mode(&mode_id, &pack_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_remove_pack_from_mode(mode_id: String, pack_id: String) -> Result<(), String> {
    opennivara::marketplace::modes::remove_pack_from_mode(&mode_id, &pack_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_get_active_theme(
) -> Result<Option<opennivara::marketplace::themes::OpenNivaraTheme>, String> {
    opennivara::marketplace::themes::get_active_theme().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_get_active_addon_theme(
) -> Result<Option<opennivara::marketplace::themes::OpenNivaraTheme>, String> {
    opennivara::marketplace::themes::get_active_addon_theme().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_preview_builtin_pack(
    pack_id: String,
) -> Result<opennivara::marketplace::packs::PackPreview, String> {
    opennivara::marketplace::builtin::preview_builtin_pack(&pack_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_list_installed_themes(
) -> Result<Vec<opennivara::marketplace::themes::InstalledThemeSummary>, String> {
    opennivara::marketplace::themes::list_installed_themes().map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_store_list() -> Result<Vec<opennivara::marketplace::themes::ThemeStoreItem>, String> {
    opennivara::marketplace::themes::list_theme_store_items().map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_install_builtin(
    theme_id: String,
) -> Result<opennivara::marketplace::themes::InstalledTheme, String> {
    opennivara::marketplace::themes::install_builtin_theme(&theme_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_install_from_path(
    path: String,
) -> Result<opennivara::marketplace::themes::InstalledTheme, String> {
    opennivara::marketplace::themes::install_theme_from_path(std::path::PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_uninstall(theme_id: String) -> Result<(), String> {
    opennivara::marketplace::themes::uninstall_theme(&theme_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_apply(theme_id: String) -> Result<(), String> {
    opennivara::marketplace::themes::apply_theme(&theme_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_reset() -> Result<(), String> {
    opennivara::marketplace::themes::reset_theme().map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_get_active() -> Result<Option<opennivara::marketplace::themes::OpenNivaraTheme>, String> {
    opennivara::marketplace::themes::get_active_theme_ui_only().map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_get_appearance_settings(
) -> Result<opennivara::marketplace::themes::AppearanceSettings, String> {
    opennivara::marketplace::themes::read_appearance_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_list_installed() -> Result<Vec<opennivara::marketplace::themes::InstalledTheme>, String> {
    opennivara::marketplace::themes::read_installed_themes()
        .map(|file| file.installed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_preview(
    theme_id: String,
) -> Result<opennivara::marketplace::themes::ThemePreview, String> {
    opennivara::marketplace::themes::preview_theme(&theme_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_get_active_command_snippets(
) -> Result<Vec<opennivara::marketplace::snippets::CommandSnippet>, String> {
    opennivara::marketplace::snippets::get_active_command_snippets().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_repair(
    dry_run: bool,
) -> Result<opennivara::marketplace::repair::MarketplaceRepairReport, String> {
    opennivara::marketplace::repair::marketplace_repair(dry_run).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_status() -> Result<opennivara::marketplace::repair::MarketplaceStatus, String> {
    opennivara::marketplace::repair::marketplace_status().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_enable_pack(pack_id: String) -> Result<(), String> {
    opennivara::marketplace::packs::enable_pack(&pack_id, true).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_disable_pack(pack_id: String) -> Result<(), String> {
    opennivara::marketplace::packs::enable_pack(&pack_id, false).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_reset(confirm: bool) -> Result<(), String> {
    opennivara::marketplace::marketplace_reset(confirm).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_get_pack_activation_capabilities(
    pack_id: String,
) -> Result<opennivara::marketplace::packs::PackActivationCapabilities, String> {
    opennivara::marketplace::packs::get_pack_activation_capabilities(&pack_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_add_pack_to_mode_with_activation(
    mode_id: String,
    pack_id: String,
    apply_theme: bool,
    apply_style: bool,
) -> Result<opennivara::marketplace::modes::ModeActivationResult, String> {
    opennivara::marketplace::modes::add_pack_to_mode_with_activation(
        &mode_id,
        &pack_id,
        apply_theme,
        apply_style,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_create_mode_from_pack(
    pack_id: String,
    mode_id: String,
    mode_name: String,
    activate: bool,
    apply_theme: bool,
    apply_style: bool,
) -> Result<opennivara::marketplace::modes::OpenNivaraMode, String> {
    opennivara::marketplace::modes::create_mode_from_pack(
        &pack_id,
        &mode_id,
        &mode_name,
        activate,
        apply_theme,
        apply_style,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_update_mode_theme(mode_id: String, theme_id: Option<String>) -> Result<(), String> {
    opennivara::marketplace::modes::update_mode_theme(&mode_id, theme_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_update_mode_style_pack(
    mode_id: String,
    style_pack_id: Option<String>,
) -> Result<(), String> {
    opennivara::marketplace::modes::update_mode_style_pack(&mode_id, style_pack_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_get_addon_settings(
) -> Result<opennivara::marketplace::addon_settings::AddonSettings, String> {
    opennivara::marketplace::addon_settings::read_addon_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_save_addon_settings(
    settings: opennivara::marketplace::addon_settings::AddonSettings,
) -> Result<(), String> {
    opennivara::marketplace::addon_settings::save_addon_settings(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_toggle_pack_enabled(pack_id: String, enabled: bool) -> Result<(), String> {
    opennivara::marketplace::addon_settings::toggle_pack_enabled(&pack_id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_toggle_contribution_enabled(
    pack_id: String,
    contribution_type: String,
    contribution_id: String,
    enabled: bool,
) -> Result<(), String> {
    opennivara::marketplace::addon_settings::toggle_contribution_enabled(
        &pack_id,
        &contribution_type,
        &contribution_id,
        enabled,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_set_active_theme(
    theme_id: Option<String>,
    source_pack_id: Option<String>,
) -> Result<(), String> {
    opennivara::marketplace::addon_settings::set_active_theme(theme_id, source_pack_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_migrate_addons() -> Result<(), String> {
    opennivara::marketplace::addon_settings::migrate_modes_to_addons().map_err(|e| e.to_string())
}

#[tauri::command]
fn marketplace_has_legacy_modes() -> bool {
    opennivara::marketplace::addon_settings::has_legacy_modes_file()
}

#[tauri::command]
fn marketplace_get_effective_settings_preview(
) -> Result<opennivara::marketplace::merge::EffectiveSettingsPreview, String> {
    opennivara::marketplace::merge::get_effective_settings_preview().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            if let Ok(builtin_path) = opennivara::marketplace::builtin::get_builtin_packs_dir() {
                std::env::set_var(
                    "OPENNIVARA_BUILTIN_PACKS_DIR",
                    builtin_path.to_string_lossy().to_string(),
                );
            } else if let Ok(resource_dir) = app.path().resource_dir() {
                let candidates = [
                    resource_dir.join("packs").join("builtin"),
                    resource_dir
                        .join("_up_")
                        .join("_up_")
                        .join("packs")
                        .join("builtin"),
                ];
                if let Some(builtin_path) = candidates.iter().find(|path| path.exists()) {
                    std::env::set_var(
                        "OPENNIVARA_BUILTIN_PACKS_DIR",
                        builtin_path.to_string_lossy().to_string(),
                    );
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ask_opennivara,
            list_sessions,
            get_session_messages,
            list_tools,
            get_tools_path,
            get_profile_path,
            get_preferences_path,
            get_style_path,
            get_contexts_path,
            get_telegram_path,
            map_summary,
            get_profile,
            save_profile,
            get_style,
            save_style,
            get_preferences,
            save_preferences,
            get_contexts,
            save_contexts,
            preview_context_for_message,
            skills_list,
            skills_get,
            skills_set_enabled,
            skills_test_route,
            pin_context,
            unpin_context,
            pin_skill,
            unpin_skill,
            list_pinned_skills,
            check_api_key,
            check_gemini_key,
            save_gemini_key,
            first_run_status,
            initialize_clean_first_run,
            memory_init,
            memory_status,
            memory_validate,
            memory_repair,
            memory_get_settings,
            memory_save_settings,
            memory_create_source,
            memory_create_item,
            memory_get_item,
            memory_list_items,
            memory_update_item,
            memory_delete_item,
            memory_retract_item,
            memory_search,
            memory_create_facet,
            memory_add_facet,
            memory_list_facets,
            memory_update_facet,
            memory_delete_facet,
            memory_graph_rebuild,
            memory_graph_status,
            memory_graph_validate,
            memory_graph_neighbors,
            memory_graph_memory_context,
            memory_graph_entity_context,
            memory_graph_related_memories,
            memory_graph_shortest_path,
            memory_graph_find_entity_mentions,
            memory_graph_add_edge,
            memory_extract_proposals_for_message,
            memory_list_proposals,
            memory_approve_proposal,
            memory_reject_proposal,
            memory_compile_context,
            memory_get_last_audit,
            memory_embedding_status,
            memory_list_tasks,
            memory_update_task_status,
            memory_list_due_reminders,
            memory_list_corrections,
            memory_resolve_entity_mention,
            runtime_get_context,
            runtime_resolve_time_phrase,
            runtime_get_model_context_info,
            location_get_context,
            location_update_manual,
            location_save_place,
            location_list_saved_places,
            location_delete_saved_place,
            location_record_observation,
            marketplace_init,
            marketplace_list_installed_packs,
            marketplace_preview_pack,
            marketplace_install_pack,
            marketplace_uninstall_pack,
            marketplace_list_builtin_packs,
            marketplace_install_builtin_pack,
            marketplace_preview_installed_pack,
            marketplace_get_modes,
            marketplace_set_active_mode,
            marketplace_create_mode,
            marketplace_add_pack_to_mode,
            marketplace_remove_pack_from_mode,
            marketplace_get_active_theme,
            marketplace_get_active_addon_theme,
            marketplace_preview_builtin_pack,
            marketplace_list_installed_themes,
            theme_store_list,
            theme_install_builtin,
            theme_install_from_path,
            theme_uninstall,
            theme_apply,
            theme_reset,
            theme_get_active,
            theme_get_appearance_settings,
            theme_list_installed,
            theme_preview,
            marketplace_get_active_command_snippets,
            marketplace_repair,
            marketplace_status,
            marketplace_enable_pack,
            marketplace_disable_pack,
            marketplace_reset,
            marketplace_get_pack_activation_capabilities,
            marketplace_add_pack_to_mode_with_activation,
            marketplace_create_mode_from_pack,
            marketplace_update_mode_theme,
            marketplace_update_mode_style_pack,
            marketplace_get_addon_settings,
            marketplace_save_addon_settings,
            marketplace_toggle_pack_enabled,
            marketplace_toggle_contribution_enabled,
            marketplace_set_active_theme,
            marketplace_migrate_addons,
            marketplace_has_legacy_modes,
            marketplace_get_effective_settings_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
