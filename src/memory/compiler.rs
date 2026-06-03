use super::audit;
use super::graph;
use super::privacy;
use super::retrieval;
use super::token_budget;
use super::types::{
    CompilerActiveTheme, CompilerSelectedSkill, CompilerSkillCandidate, ContextCompilerInput,
    ContextCompilerOutput, IntentClassification, MemorySearchQuery,
};
use rusqlite::Connection;

pub fn classify_intent(message: &str) -> IntentClassification {
    let lower = message.to_lowercase();
    let mut labels = Vec::new();
    let mut reasons = Vec::new();

    if contains_any(
        &lower,
        &[
            "remember",
            "last time",
            "did i",
            "when was",
            "who is",
            "what did i buy",
            "birthday",
            "gift",
            "friend",
            "where did we go",
            "previously",
        ],
    ) {
        labels.push("memory_lookup".to_string());
        reasons.push("history/person/time keyword".to_string());
    }
    if contains_any(
        &lower,
        &["no ", "wrong", "i meant", "not ", "actually", "correction"],
    ) {
        labels.push("correction".to_string());
        reasons.push("correction keyword".to_string());
    }
    if contains_any(
        &lower,
        &[
            "todo", "task", "need to", "plan", "schedule", "today", "tomorrow", "due", "remind",
            "buy",
        ],
    ) {
        labels.push("task_planning".to_string());
        reasons.push("planning/task keyword".to_string());
    }
    if contains_any(
        &lower,
        &[
            "code",
            "file",
            "repo",
            "rust",
            "tauri",
            "cargo",
            "workspace",
            "build",
            "test",
        ],
    ) {
        labels.push("workspace_query".to_string());
        reasons.push("workspace keyword".to_string());
    }
    if contains_any(
        &lower,
        &[
            "route",
            "traffic",
            "leave",
            "destination",
            "grocery",
            "store",
        ],
    ) {
        labels.push("route_query".to_string());
        reasons.push("route/place keyword".to_string());
    }
    if contains_any(
        &lower,
        &[
            "where am i",
            "near me",
            "nearby",
            "location",
            "weather here",
            "city am i",
        ],
    ) {
        labels.push("location_query".to_string());
        reasons.push("location keyword".to_string());
    }
    if contains_any(
        &lower,
        &["yesterday", "this week", "next week", "last month"],
    ) {
        labels.push("relative_time_query".to_string());
        reasons.push("relative time keyword".to_string());
    }
    if contains_any(&lower, &["email", "send", "open app", "tool", "workflow"]) {
        labels.push("tool_workflow_query".to_string());
        reasons.push("tool/workflow keyword".to_string());
    }
    if labels.is_empty() {
        labels.push("normal_chat".to_string());
        reasons.push("no deterministic retrieval trigger".to_string());
    }

    let confidence = if labels == ["normal_chat"] { 0.7 } else { 0.9 };
    IntentClassification {
        labels,
        confidence,
        reason: reasons.join("; "),
    }
}

pub fn compile_context(
    conn: &Connection,
    input: ContextCompilerInput,
) -> anyhow::Result<ContextCompilerOutput> {
    let intent = classify_intent(&input.user_message);
    let memory_allowed = privacy::memory_inclusion_allowed(&input.privacy_mode);
    let runtime_relevant = runtime_relevant(&input.user_message, &intent);
    let location_relevant = location_relevant(&input.user_message, &intent);

    let mut notes = Vec::new();
    let mut included_memory_ids = Vec::new();
    let mut included_graph_edge_ids = Vec::new();
    let mut memory_brief = String::new();

    if !memory_allowed {
        notes.push("privacy_off: memory lookup skipped".to_string());
    } else if intent.labels.iter().any(|label| {
        matches!(
            label.as_str(),
            "memory_lookup" | "task_planning" | "correction" | "route_query"
        )
    }) {
        let results = retrieval::search_memory(
            conn,
            &MemorySearchQuery {
                query: Some(input.user_message.clone()),
                memory_type: None,
                status: None,
                domain: None,
                facet_type: None,
                label: None,
                limit: 5,
            },
        )?;
        let mut lines = Vec::new();
        for result in results {
            included_memory_ids.push(result.item.id.clone());
            let qualifier = match result.answerability.as_str() {
                "planned_only" => "planned, not confirmed",
                "confirmed" => "confirmed",
                "ambiguous" => "ambiguous",
                "contradicted" => "contradicted/retracted",
                _ => "inferred",
            };
            lines.push(format!(
                "- {}: {} ({qualifier}; reason: {})",
                result.item.title, result.item.summary, result.reason
            ));
        }
        memory_brief = lines.join("\n");
        if memory_brief.is_empty() {
            notes.push("memory_lookup: no matching memory found".to_string());
        }
    }

    for memory_id in &included_memory_ids {
        if let Ok(context) = graph::graph_memory_context(conn, memory_id, 1) {
            included_graph_edge_ids.extend(context.edges.into_iter().map(|edge| edge.id));
        }
    }
    included_graph_edge_ids.sort();
    included_graph_edge_ids.dedup();

    let mut workspace_brief = if intent.labels.contains(&"workspace_query".to_string()) {
        input.current_workspace_context.clone().unwrap_or_default()
    } else {
        String::new()
    };
    let mut route_brief = if intent.labels.contains(&"route_query".to_string()) {
        input.current_route_context.clone().unwrap_or_default()
    } else {
        String::new()
    };
    let (mut runtime_brief, runtime_decision) = if runtime_relevant {
        (
            runtime_brief(&input.runtime_context),
            "included:relevant_intent".to_string(),
        )
    } else {
        (String::new(), "skipped:not_relevant".to_string())
    };
    let (mut location_brief, location_decision) = if !location_relevant {
        (String::new(), "skipped:not_relevant".to_string())
    } else if input.runtime_context.location.permission_state != "granted"
        || input.runtime_context.location.privacy_level == "disabled"
    {
        notes.push("location_context: permission denied or disabled".to_string());
        (String::new(), "skipped:permission_denied".to_string())
    } else if !location_is_prompt_safe(&input.runtime_context.location) {
        notes.push("location_context: stale or unknown".to_string());
        (String::new(), "skipped:not_fresh".to_string())
    } else {
        (
            location_brief(&input.runtime_context.location),
            "included:relevant_and_fresh".to_string(),
        )
    };
    let mut task_reminder_brief = if intent.labels.contains(&"task_planning".to_string()) {
        memory_brief
            .lines()
            .filter(|line| line.to_lowercase().contains("planned"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    let system_policy = system_policy();
    let active_theme = crate::marketplace::themes::get_active_theme_ui_only()
        .ok()
        .flatten()
        .map(|theme| CompilerActiveTheme {
            id: theme.id,
            name: theme.name,
            ui_only: true,
        });

    let (mut profile_brief, profile_sent) = match crate::profile::read_profile() {
        Ok(profile) => (
            profile.to_compact_context_string(),
            profile_sent_fields(&profile),
        ),
        Err(err) => {
            notes.push(format!("profile_context: unavailable ({err})"));
            (String::new(), Vec::new())
        }
    };
    let (mut style_brief, style_sent) = match crate::style::read_style() {
        Ok(style) => (style.to_compact_context_string(), style_sent_fields(&style)),
        Err(err) => {
            notes.push(format!("style_context: unavailable ({err})"));
            (String::new(), Vec::new())
        }
    };
    let (mut preference_brief, preferences_sent) = match crate::preferences::read_preferences() {
        Ok(prefs) => {
            let selected = crate::context_selector::select_relevant_preference_sections(
                &prefs,
                &input.user_message,
            );
            let sent = selected.iter().map(|section| section.id.clone()).collect();
            (
                crate::preferences::format_relevant_preferences(&selected),
                sent,
            )
        }
        Err(err) => {
            notes.push(format!("preference_context: unavailable ({err})"));
            (String::new(), Vec::new())
        }
    };

    let (mut settings_context_brief, contexts_sent, contexts_pinned, contexts_not_sent) =
        match crate::context::read_contexts() {
            Ok(contexts_file) => {
                let selected = crate::context_selector::select_relevant_contexts(
                    &contexts_file,
                    &input.user_message,
                    &input.pinned_context_ids,
                );
                let mut sent = Vec::new();
                let mut pinned = Vec::new();
                for entry in &selected {
                    if input.pinned_context_ids.contains(&entry.id) {
                        pinned.push(entry.id.clone());
                    } else {
                        sent.push(entry.id.clone());
                    }
                }
                let not_sent = contexts_file
                    .contexts
                    .iter()
                    .filter(|entry| {
                        entry.enabled && !selected.iter().any(|selected| selected.id == entry.id)
                    })
                    .map(|entry| entry.id.clone())
                    .collect();
                (format_context_entries(&selected), sent, pinned, not_sent)
            }
            Err(err) => {
                notes.push(format!("settings_context: unavailable ({err})"));
                (String::new(), Vec::new(), Vec::new(), Vec::new())
            }
        };

    let available_skills = crate::skills::registry::load_available_skills().unwrap_or_default();
    let skill_decision = crate::skills::selector::select_skill_route(
        &available_skills,
        crate::skills::selector::SkillRouteRequest {
            message: input.user_message.clone(),
            explicit_skill_id: input.explicit_skill_id.clone(),
            pack_hint: input.pack_hint.clone(),
            ui_selected_skill_id: input.ui_selected_skill_id.clone(),
            session_pinned_skill_ids: input.session_pinned_skill_ids.clone(),
        },
    );
    let mut skill_brief =
        crate::skills::prompt::format_active_skills_prompt(&skill_decision, &available_skills);
    let selected_skills: Vec<CompilerSelectedSkill> = skill_decision
        .selected_skills()
        .into_iter()
        .map(|skill| CompilerSelectedSkill {
            id: skill.id,
            pack_id: skill.pack_id,
            name: skill.name,
            score: skill.score,
            reason: skill.reason,
            allowed_tools: skill.allowed_tools,
            denied_tools: skill.denied_tools,
        })
        .collect();
    let skill_candidates = skill_decision
        .candidates
        .iter()
        .map(|candidate| CompilerSkillCandidate {
            id: candidate.id.clone(),
            name: candidate.name.clone(),
            score: candidate.score,
            accepted: candidate.accepted,
            reason: candidate.reason.clone(),
        })
        .collect();
    let skill_warnings = skill_decision.warnings.clone();
    notes.extend(skill_warnings.clone());
    let mut warnings = skill_warnings.clone();
    if std::env::var("GEMINI_API_KEY").is_err() {
        warnings.push("GEMINI_API_KEY is not defined in the environment.".to_string());
    }

    let budgeted_prompt = token_budget::build_budgeted_prompt(
        &input.runtime_context.model,
        input.model_context_limit,
        input.reserved_output_tokens,
        vec![
            prompt_section("System", &system_policy, 1, true),
            prompt_section("Style", &style_brief, 2, false),
            prompt_section("Profile", &profile_brief, 2, false),
            prompt_section("Preferences", &preference_brief, 2, false),
            prompt_section("Settings Contexts", &settings_context_brief, 3, false),
            prompt_section("Skills", &skill_brief, 3, false),
            prompt_section("Runtime", &runtime_brief, 4, false),
            prompt_section("Location", &location_brief, 6, false),
            prompt_section("Route Brief", &route_brief, 6, false),
            prompt_section("Task Reminder Brief", &task_reminder_brief, 4, false),
            prompt_section("Memory Brief", &memory_brief, 2, false),
            prompt_section("Graph Facts", &included_graph_edge_ids.join("\n"), 9, false),
            prompt_section("Workspace Brief", &workspace_brief, 10, false),
            prompt_section("Current User Message", &input.user_message, 1, true),
        ],
        notes,
    );
    let raw_prompt = budgeted_prompt.raw_prompt;
    let token_budget_report = budgeted_prompt.report;
    style_brief = included_section_body(&budgeted_prompt.included_sections, "Style");
    profile_brief = included_section_body(&budgeted_prompt.included_sections, "Profile");
    preference_brief = included_section_body(&budgeted_prompt.included_sections, "Preferences");
    settings_context_brief =
        included_section_body(&budgeted_prompt.included_sections, "Settings Contexts");
    skill_brief = included_section_body(&budgeted_prompt.included_sections, "Skills");
    runtime_brief = included_section_body(&budgeted_prompt.included_sections, "Runtime");
    location_brief = included_section_body(&budgeted_prompt.included_sections, "Location");
    route_brief = included_section_body(&budgeted_prompt.included_sections, "Route Brief");
    task_reminder_brief =
        included_section_body(&budgeted_prompt.included_sections, "Task Reminder Brief");
    memory_brief = included_section_body(&budgeted_prompt.included_sections, "Memory Brief");
    workspace_brief = included_section_body(&budgeted_prompt.included_sections, "Workspace Brief");
    let included_memory_ids_json = serde_json::to_string(&included_memory_ids)?;
    let included_workspace_refs = if workspace_brief.is_empty() {
        Vec::<String>::new()
    } else {
        vec!["current_workspace_context".to_string()]
    };
    let compiled_context_json = serde_json::json!({
        "intent": intent,
        "profile_sent": &profile_sent,
        "preferences_sent": &preferences_sent,
        "contexts_sent": &contexts_sent,
        "contexts_pinned": &contexts_pinned,
        "selected_skills": &selected_skills,
        "skill_candidates": &skill_candidates,
        "memory_brief": memory_brief,
        "settings_context_brief": settings_context_brief,
        "skill_brief": skill_brief,
        "included_graph_edge_ids": &included_graph_edge_ids,
        "runtime_decision": &runtime_decision,
        "location_decision": &location_decision,
        "runtime_brief": &runtime_brief,
        "location_brief": &location_brief,
        "workspace_brief": workspace_brief,
        "route_brief": route_brief,
    })
    .to_string();
    let audit = audit::create_audit(
        conn,
        audit::CreatePromptAudit {
            session_id: input.session_id.clone(),
            message_id: input.message_id.clone(),
            user_message: input.user_message.clone(),
            compiled_context_json,
            included_memory_ids_json,
            included_task_ids_json: "[]".to_string(),
            included_workspace_refs_json: serde_json::to_string(&included_workspace_refs)?,
            token_budget_json: serde_json::to_string(&token_budget_report)?,
        },
    )?;

    Ok(ContextCompilerOutput {
        system_policy,
        current_user_message: input.user_message,
        recent_conversation_window: String::new(),
        session_summary: String::new(),
        profile_brief,
        style_brief,
        preference_brief,
        memory_brief,
        task_reminder_brief,
        workspace_brief,
        route_brief,
        raw_prompt,
        token_budget_report,
        audit,
        intent,
        included_memory_ids,
        included_graph_edge_ids,
        runtime_decision,
        location_decision,
        profile_sent,
        style_sent,
        preferences_sent,
        contexts_sent,
        contexts_pinned,
        contexts_not_sent,
        selected_skills,
        skill_candidates,
        skill_warnings,
        warnings,
        active_mode: String::new(),
        active_packs: vec![],
        active_theme,
        style_source_pack: None,
    })
}

fn system_policy() -> String {
    let mut policy =
        "You are OpenNivara, a helpful, intelligent, and friendly personal CLI assistant. \
        Use profile context to understand the user. \
        Use style context to control tone and formatting. \
        Use triggered preferences only if relevant. \
        You may request local tools only when needed. \
        Never ask for unrelated private files. \
        Do not try to read secrets, API keys, SSH keys, .env files, passwords, or credentials. \
        If a tool result is blocked by policy, explain that safely and continue. \
        Do not claim you read a file unless a tool result confirms it. \
        Store themes are UI-only and must not alter assistant behavior. \
        Focus directly on answering the user's question concisely. \
        \n\n\
        Formatting rules:\n\
        Use simple Markdown only.\n\
        Use short headings, bullet lists, numbered lists, bold text, inline code, and fenced code blocks.\n\
        Avoid raw HTML.\n\
        Avoid complex tables unless necessary.\n\
        Avoid deeply nested lists."
            .to_string();

    let has_map = crate::workspace_map::get_db_path()
        .map(|path| path.exists())
        .unwrap_or(false);
    if has_map {
        policy.push_str(
            " A workspace map is available through map tools. Use map_search/map_tree/map_summary before reading files when locating project context.",
        );
    }
    policy
}

fn profile_sent_fields(profile: &crate::profile::Profile) -> Vec<String> {
    let mut sent = Vec::new();
    if profile.privacy.send_identity {
        sent.push(format!(
            "identity.display_name: {}",
            profile.identity.display_name
        ));
    }
    if profile.privacy.send_technical {
        sent.push(format!(
            "technical.coding_level: {}",
            profile.technical.coding_level
        ));
        sent.push(format!(
            "technical.preferred_coding_languages: {:?}",
            profile.technical.preferred_coding_languages
        ));
    }
    if profile.privacy.send_location {
        sent.push(format!("location.country: {}", profile.location.country));
    }
    sent
}

fn style_sent_fields(style: &crate::style::OpenNivaraStyle) -> Vec<String> {
    vec![
        format!("communication.tone: {}", style.communication.tone),
        format!(
            "communication.detail_level: {}",
            style.communication.detail_level
        ),
        format!(
            "coding.prefer_mvp_architecture: {}",
            style.coding.prefer_mvp_architecture
        ),
    ]
}

fn format_context_entries(entries: &[crate::context::ContextEntry]) -> String {
    let mut text = String::new();
    for entry in entries {
        text.push_str(&format!(
            "\n[Context: {} (Kind: {})]\n",
            entry.title, entry.kind
        ));
        text.push_str(&format!("Summary: {}\n", entry.summary));
        if !entry.facts.is_empty() {
            text.push_str("Known Facts:\n");
            for fact in &entry.facts {
                text.push_str(&format!("- {}\n", fact));
            }
        }
        if !entry.rules.is_empty() {
            text.push_str("Handling Rules:\n");
            for rule in &entry.rules {
                text.push_str(&format!("- {}\n", rule));
            }
        }
    }
    text
}

fn contains_any(message: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| message.contains(needle))
}

fn prompt_section(
    label: &str,
    body: &str,
    priority: u32,
    required: bool,
) -> token_budget::PromptSectionBudget {
    token_budget::PromptSectionBudget {
        label: label.into(),
        body: body.into(),
        priority,
        required,
    }
}

fn included_section_body(sections: &[token_budget::PromptSectionBudget], label: &str) -> String {
    sections
        .iter()
        .find(|section| section.label == label)
        .map(|section| section.body.clone())
        .unwrap_or_default()
}

fn runtime_relevant(message: &str, intent: &IntentClassification) -> bool {
    intent.labels.iter().any(|label| {
        matches!(
            label.as_str(),
            "memory_lookup"
                | "task_planning"
                | "route_query"
                | "location_query"
                | "relative_time_query"
                | "tool_workflow_query"
        )
    }) || contains_any(
        &message.to_lowercase(),
        &[
            "today",
            "tomorrow",
            "yesterday",
            "this week",
            "next week",
            "tonight",
        ],
    )
}

fn location_relevant(message: &str, intent: &IntentClassification) -> bool {
    intent
        .labels
        .iter()
        .any(|label| matches!(label.as_str(), "route_query" | "location_query"))
        || contains_any(
            &message.to_lowercase(),
            &[
                "leave",
                "near me",
                "nearby",
                "grocery",
                "weather here",
                "where am i",
            ],
        )
}

fn runtime_brief(runtime: &crate::runtime::context::RuntimeContext) -> String {
    format!(
        "Now: {} local ({}). Date: {} {}. Today: {}..{}. Tomorrow: {}..{}.",
        runtime.now_local,
        runtime.timezone,
        runtime.day_of_week,
        runtime.date_local,
        runtime.relative_date_context.today_start,
        runtime.relative_date_context.today_end,
        runtime.relative_date_context.tomorrow_start,
        runtime.relative_date_context.tomorrow_end
    )
}

fn location_is_prompt_safe(location: &crate::runtime::location::LocationContext) -> bool {
    matches!(
        location.status.as_str(),
        "exact_current" | "approximate_current" | "saved_place"
    ) && location.captured_at.is_some()
}

fn location_brief(location: &crate::runtime::location::LocationContext) -> String {
    let place = [
        location.label.as_deref(),
        location.city.as_deref(),
        location.region.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ");
    let freshness = location
        .freshness_seconds
        .map(|seconds| format!("{seconds}s old"))
        .unwrap_or_else(|| "saved place".into());
    format!(
        "Status: {}. Place: {}. Source: {}. Privacy: {}. Freshness: {}.",
        location.status,
        if place.is_empty() { "unknown" } else { &place },
        location.source,
        location.privacy_level,
        freshness
    )
}
