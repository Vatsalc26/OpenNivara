use super::*;
use crate::config_paths::TEST_CONFIG_ENV_MUTEX;
use pretty_assertions::assert_eq;
use serial_test::serial;

struct EnvGuard {
    previous: Option<String>,
}

impl EnvGuard {
    fn set_test_config(path: &std::path::Path) -> Self {
        let previous = std::env::var("OPENNIVARA_TEST_CONFIG_DIR").ok();
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", path);
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", previous);
        } else {
            std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
        }
    }
}

fn with_temp_memory_db<T>(f: impl FnOnce() -> T) -> T {
    let _lock = TEST_CONFIG_ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let temp = tempfile::tempdir().expect("temp dir");
    let _guard = EnvGuard::set_test_config(temp.path());
    f()
}

fn test_runtime_context() -> crate::runtime::context::RuntimeContext {
    crate::runtime::clock::runtime_context_at(
        chrono::DateTime::parse_from_rfc3339("2026-06-02T10:00:00Z")
            .expect("time")
            .with_timezone(&chrono::Utc),
        Some("Asia/Kolkata"),
        crate::runtime::location::LocationContext::default(),
    )
}

fn test_runtime_context_with_location(
    location: crate::runtime::location::LocationContext,
) -> crate::runtime::context::RuntimeContext {
    crate::runtime::clock::runtime_context_at(
        chrono::DateTime::parse_from_rfc3339("2026-06-02T10:00:00Z")
            .expect("time")
            .with_timezone(&chrono::Utc),
        Some("Asia/Kolkata"),
        location,
    )
}

fn save_compiler_fixture_config() {
    crate::profile::save_profile(&crate::profile::Profile {
        schema_version: 2,
        identity: crate::profile::Identity {
            display_name: "Asha".into(),
            full_name: String::new(),
            gender: String::new(),
            pronouns: String::new(),
            date_of_birth: String::new(),
            timezone: "Asia/Kolkata".into(),
        },
        location: crate::profile::Location {
            country: "India".into(),
            state_or_region: String::new(),
            city: String::new(),
            living_situation: String::new(),
        },
        languages: crate::profile::Languages {
            preferred_human_language: "English".into(),
            other_human_languages: vec![],
        },
        technical: crate::profile::Technical {
            coding_level: "beginner".into(),
            preferred_coding_languages: vec!["Rust".into()],
            current_os: "Windows".into(),
            main_editor: "VS Code".into(),
            secondary_editor: String::new(),
            terminal: "PowerShell".into(),
        },
        personal: crate::profile::Personal {
            occupation_or_role: String::new(),
            education_level: String::new(),
            interests: vec![],
        },
        privacy: crate::profile::Privacy {
            send_identity: true,
            send_location: true,
            send_gender: false,
            send_technical: true,
            send_personal: false,
        },
    })
    .expect("save profile");

    crate::style::save_style(&crate::style::OpenNivaraStyle {
        schema_version: 2,
        communication: crate::style::CommunicationStyle {
            tone: "precise".into(),
            detail_level: "medium".into(),
            use_examples: true,
            use_step_by_step: true,
            avoid_unexplained_jargon: true,
            ask_fewer_questions: true,
            prefer_actionable_answers: true,
        },
        coding: crate::style::CodingStyle {
            show_simple_solution_first: true,
            explain_after_code: true,
            prefer_mvp_architecture: true,
            avoid_overengineering: true,
            use_beginner_comments: false,
        },
        formatting: crate::style::FormattingStyle {
            use_markdown: true,
            use_short_sections: true,
            include_next_step: false,
            avoid_long_walls_of_text: true,
        },
        behavior: crate::style::BehaviorStyle {
            be_honest_about_uncertainty: true,
            do_not_pretend_to_have_done_things: true,
            do_not_reveal_private_context_unless_relevant: true,
        },
    })
    .expect("save style");

    crate::preferences::save_preferences(&crate::preferences::PreferencesFile {
        schema_version: 2,
        sections: vec![crate::preferences::PreferenceSection {
            id: "rust_debugging".into(),
            enabled: true,
            send_policy: "triggered_strict".into(),
            description: Some("Rust debugging preferences".into()),
            triggers: vec!["rust".into(), "compiler".into()],
            required_any: vec!["error".into()],
            negative_triggers: vec![],
            min_score: 2,
            likes: vec![crate::preferences::PreferenceItem {
                item: "minimal repro first".into(),
                strength: 5,
            }],
            dislikes: vec![],
            notes: vec!["Explain borrow checker issues plainly.".into()],
        }],
    })
    .expect("save preferences");

    crate::context::save_contexts(&crate::context::ContextsFile {
        schema_version: 1,
        contexts: vec![
            crate::context::ContextEntry {
                id: "pinned_project".into(),
                enabled: true,
                kind: "project".into(),
                send_policy: "session_pinned".into(),
                title: "Pinned Project".into(),
                summary: "Pinned project summary.".into(),
                triggers: vec![],
                required_any: vec![],
                negative_triggers: vec![],
                min_score: 0,
                facts: vec!["Pinned project fact.".into()],
                rules: vec!["Pinned project rule.".into()],
            },
            crate::context::ContextEntry {
                id: "rust_context".into(),
                enabled: true,
                kind: "learning".into(),
                send_policy: "triggered_strict".into(),
                title: "Rust Context".into(),
                summary: "Rust context summary.".into(),
                triggers: vec!["rust".into()],
                required_any: vec!["compiler".into()],
                negative_triggers: vec![],
                min_score: 2,
                facts: vec!["Rust fact.".into()],
                rules: vec![],
            },
        ],
    })
    .expect("save contexts");

    crate::skills::registry::init_skills().expect("init skills");
    crate::config_store::save_toml_file(
        &crate::skills::registry::get_user_skills_path().expect("user skills path"),
        &crate::skills::manifest::SkillsFile {
            schema_version: 1,
            skills: vec![crate::skills::manifest::SkillManifest {
                schema_version: 1,
                id: "rust_error_coach".into(),
                pack_id: None,
                name: "Rust Error Coach".into(),
                description: "Helps explain Rust compiler errors.".into(),
                enabled: true,
                category: "coding".into(),
                route_policy: crate::skills::manifest::SkillRoutePolicy::Auto,
                aliases: vec!["rust error coach".into()],
                triggers: vec!["rust".into(), "compiler error".into()],
                required_any: vec!["error".into()],
                negative_triggers: vec![],
                examples: vec!["help with this Rust compiler error".into()],
                min_score: 10,
                prompt: crate::skills::manifest::SkillPrompt {
                    role: "Rust debugging coach".into(),
                    instructions: "Explain Rust compiler errors with small examples.".into(),
                    constraints: vec!["Do not invent compiler output.".into()],
                },
                tools: crate::skills::manifest::SkillToolPolicy {
                    allow: vec![],
                    deny: vec![],
                },
                safety: crate::skills::manifest::SkillSafety {
                    risk_level: "low".into(),
                    requires_confirmation: false,
                    allows_file_write: false,
                    allows_shell: false,
                    allows_network: false,
                    requires_fresh_info: false,
                },
                metadata: crate::skills::manifest::SkillMetadata::default(),
                store_preview: crate::skills::manifest::SkillStorePreview::default(),
            }],
        },
    )
    .expect("save user skill");
}

#[test]
#[serial]
fn memory_db_opens_under_isolated_test_config_and_migrates_idempotently() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        migrations::run_migrations(&conn).expect("second migration pass");

        let table_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name IN (
                    'memory_sources',
                    'memory_items',
                    'entities',
                    'entity_aliases',
                    'entity_relationships',
                    'memory_entities',
                    'memory_corrections',
                    'tasks',
                    'session_summaries',
                    'rollups',
                    'prompt_audits',
                    'memory_facets',
                    'memory_graph_nodes',
                    'memory_graph_edges',
                    'memory_graph_edge_index',
                    'saved_places',
                    'location_observations',
                    'memory_fts'
                )",
                [],
                |row| row.get(0),
            )
            .expect("table count");

        assert_eq!(table_count, 18);
        assert!(db::memory_db_path()
            .expect("memory db path")
            .starts_with(crate::config_paths::config_dir().expect("config dir")));
    });
}

#[test]
#[serial]
fn memory_crud_and_fts_search_distinguishes_planned_from_completed() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "I need to buy bread and fruits today.".into(),
                source_quote: Some("buy bread and fruits".into()),
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T10:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");

        let item = db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "task".into(),
                title: "Buy bread".into(),
                summary: "User planned to buy bread.".into(),
                details_json: serde_json::json!({"item":"bread"}).to_string(),
                status: "planned".into(),
                confidence: 0.92,
                user_verified: false,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id.clone(),
                observed_at: "2026-06-02T10:00:00Z".into(),
                happened_at: None,
                due_at: Some("2026-06-02T18:00:00Z".into()),
                timezone: "Asia/Kolkata".into(),
                time_precision: "date_only".into(),
                natural_time_phrase: Some("today".into()),
                tags: vec!["grocery".into(), "bread".into()],
            },
        )
        .expect("memory item");

        let results = retrieval::search_memory(
            &conn,
            &types::MemorySearchQuery {
                query: Some("bread".into()),
                memory_type: Some("task".into()),
                status: None,
                domain: None,
                facet_type: None,
                label: None,
                limit: 10,
            },
        )
        .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.id, item.id);
        assert_eq!(results[0].answerability, "planned_only");
        assert!(results[0].reason.contains("FTS"));
    });
}

#[test]
#[serial]
fn context_compiler_excludes_memory_for_hello_and_records_audit() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "hello".into(),
                session_id: Some("session-1".into()),
                message_id: Some("message-1".into()),
                runtime_context: test_runtime_context(),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: None,
                enabled_sources: vec!["manual".into(), "chat".into()],
                current_workspace_context: None,
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert_eq!(output.intent.labels, vec!["normal_chat"]);
        assert!(output.memory_brief.is_empty());
        assert!(output.included_memory_ids.is_empty());
        assert!(output.raw_prompt.contains("hello"));

        let audit = audit::get_last_audit(&conn)
            .expect("audit")
            .expect("present");
        assert_eq!(audit.included_memory_ids_json, "[]");
    });
}

#[test]
#[serial]
fn context_compiler_is_superset_of_old_preview_builder() {
    with_temp_memory_db(|| {
        save_compiler_fixture_config();
        let conn = db::open_memory_db().expect("open memory db");

        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "help me with this rust compiler error".into(),
                session_id: Some("session-preview".into()),
                message_id: Some("message-preview".into()),
                runtime_context: test_runtime_context(),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: None,
                enabled_sources: vec!["manual".into(), "chat".into()],
                current_workspace_context: Some("workspace map summary".into()),
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec!["pinned_project".into()],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert!(output
            .profile_sent
            .contains(&"identity.display_name: Asha".into()));
        assert!(output
            .style_sent
            .contains(&"communication.tone: precise".into()));
        assert_eq!(output.preferences_sent, vec!["rust_debugging"]);
        assert_eq!(output.contexts_pinned, vec!["pinned_project"]);
        assert_eq!(output.contexts_sent, vec!["rust_context"]);
        assert!(output
            .contexts_not_sent
            .iter()
            .all(|id| id != "pinned_project" && id != "rust_context"));
        assert_eq!(output.selected_skills[0].id, "rust_error_coach");
        assert!(output.raw_prompt.contains("Pinned Project"));
        assert!(output.raw_prompt.contains("Rust context summary"));
        assert!(output.raw_prompt.contains("Section: rust_debugging"));
        assert!(output.raw_prompt.contains("Rust Error Coach"));
        assert!(output.raw_prompt.contains("workspace map summary"));
        assert!(output
            .audit
            .compiled_context_json
            .contains("selected_skills"));
    });
}

#[test]
#[serial]
fn context_compiler_includes_relevant_memory_only_for_memory_lookup() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "I planned to buy bread on Tuesday.".into(),
                source_quote: Some("planned to buy bread".into()),
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T09:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "task".into(),
                title: "Buy bread".into(),
                summary: "User planned to buy bread Tuesday; no completion is recorded.".into(),
                details_json: "{}".into(),
                status: "planned".into(),
                confidence: 0.9,
                user_verified: true,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id,
                observed_at: "2026-06-02T09:00:00Z".into(),
                happened_at: None,
                due_at: Some("2026-06-02T18:00:00Z".into()),
                timezone: "Asia/Kolkata".into(),
                time_precision: "date_only".into(),
                natural_time_phrase: Some("Tuesday".into()),
                tags: vec!["bread".into(), "grocery".into()],
            },
        )
        .expect("memory item");

        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "did I buy bread Tuesday?".into(),
                session_id: Some("session-2".into()),
                message_id: Some("message-2".into()),
                runtime_context: test_runtime_context(),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: None,
                enabled_sources: vec!["manual".into()],
                current_workspace_context: Some("Rust Tauri workspace".into()),
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert!(output.intent.labels.contains(&"memory_lookup".into()));
        assert_eq!(output.included_memory_ids.len(), 1);
        assert!(output.memory_brief.contains("planned"));
        assert!(output.memory_brief.contains("not confirmed"));
        assert!(output.workspace_brief.is_empty());
        assert!(output.audit.included_memory_ids_json.contains("mem_"));
    });
}

#[test]
#[serial]
fn privacy_off_prevents_memory_context_inclusion() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "what do you remember about bread?".into(),
                session_id: None,
                message_id: None,
                runtime_context: test_runtime_context(),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::Off,
                effective_privacy_policy: None,
                enabled_sources: vec!["manual".into()],
                current_workspace_context: None,
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert_eq!(output.memory_brief, "");
        assert!(output.audit.token_budget_json.contains("privacy_off"));
    });
}

#[test]
#[serial]
fn memory_facets_support_multiple_domains_without_templates() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "Exam next Friday and invoice tomorrow.".into(),
                source_quote: None,
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T10:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        let item = db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "task".into(),
                title: "Mixed responsibilities".into(),
                summary: "Education and business obligations can coexist.".into(),
                details_json: "{}".into(),
                status: "planned".into(),
                confidence: 0.9,
                user_verified: true,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id,
                observed_at: "2026-06-02T10:00:00Z".into(),
                happened_at: None,
                due_at: None,
                timezone: "Asia/Kolkata".into(),
                time_precision: "unknown".into(),
                natural_time_phrase: None,
                tags: vec![],
            },
        )
        .expect("item");

        facets::create_facet(
            &conn,
            &types::CreateMemoryFacet {
                memory_id: item.id.clone(),
                domain: "education".into(),
                facet_type: "deadline".into(),
                label: "exam".into(),
                details_json: "{}".into(),
                sensitivity: "normal".into(),
                confidence: 0.9,
                source: "test".into(),
            },
        )
        .expect("education facet");
        facets::create_facet(
            &conn,
            &types::CreateMemoryFacet {
                memory_id: item.id.clone(),
                domain: "business".into(),
                facet_type: "document".into(),
                label: "invoice".into(),
                details_json: "{}".into(),
                sensitivity: "normal".into(),
                confidence: 0.8,
                source: "test".into(),
            },
        )
        .expect("business facet");

        let education = facets::list_facets(
            &conn,
            facets::FacetFilter {
                memory_id: Some(item.id.clone()),
                domain: Some("education".into()),
                facet_type: None,
                label: None,
            },
        )
        .expect("list education");
        let business = facets::list_facets(
            &conn,
            facets::FacetFilter {
                memory_id: Some(item.id),
                domain: Some("business".into()),
                facet_type: None,
                label: None,
            },
        )
        .expect("list business");

        assert_eq!(education.len(), 1);
        assert_eq!(business.len(), 1);
        assert_eq!(education[0].label, "exam");
        assert_eq!(business[0].label, "invoice");
    });
}

#[test]
#[serial]
fn search_filters_by_facet_domain_and_label() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "Buy bread and fruits.".into(),
                source_quote: None,
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T10:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        let item = db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "task".into(),
                title: "Buy bread and fruits".into(),
                summary: "Shopping task.".into(),
                details_json: "{}".into(),
                status: "planned".into(),
                confidence: 0.9,
                user_verified: true,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id,
                observed_at: "2026-06-02T10:00:00Z".into(),
                happened_at: None,
                due_at: None,
                timezone: "Asia/Kolkata".into(),
                time_precision: "unknown".into(),
                natural_time_phrase: None,
                tags: vec!["bread".into()],
            },
        )
        .expect("item");
        facets::create_facet(
            &conn,
            &types::CreateMemoryFacet {
                memory_id: item.id.clone(),
                domain: "household".into(),
                facet_type: "shopping_item".into(),
                label: "bread".into(),
                details_json: "{}".into(),
                sensitivity: "normal".into(),
                confidence: 0.9,
                source: "test".into(),
            },
        )
        .expect("facet");

        let results = retrieval::search_memory(
            &conn,
            &types::MemorySearchQuery {
                query: None,
                memory_type: None,
                status: None,
                domain: Some("household".into()),
                facet_type: Some("shopping_item".into()),
                label: Some("bread".into()),
                limit: 10,
            },
        )
        .expect("facet search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].item.id, item.id);
    });
}

#[test]
#[serial]
fn graph_rebuild_indexes_facets_and_related_memories() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "Gift for Alex.".into(),
                source_quote: None,
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T10:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        let gift = db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "event".into(),
                title: "Bought book for Alex".into(),
                summary: "Book gift for Alex.".into(),
                details_json: "{}".into(),
                status: "completed".into(),
                confidence: 0.9,
                user_verified: true,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id,
                observed_at: "2026-06-02T10:00:00Z".into(),
                happened_at: None,
                due_at: None,
                timezone: "Asia/Kolkata".into(),
                time_precision: "unknown".into(),
                natural_time_phrase: None,
                tags: vec!["alex".into(), "gift".into()],
            },
        )
        .expect("gift");
        let facet = facets::create_facet(
            &conn,
            &types::CreateMemoryFacet {
                memory_id: gift.id.clone(),
                domain: "social".into(),
                facet_type: "gift".into(),
                label: "alex".into(),
                details_json: "{}".into(),
                sensitivity: "normal".into(),
                confidence: 0.9,
                source: "test".into(),
            },
        )
        .expect("facet");

        graph::graph_rebuild_from_sqlite(&conn).expect("rebuild graph");
        let validation = graph::graph_validate_consistency(&conn).expect("validate graph");
        assert!(validation.is_empty());
        let related = graph::graph_find_related_memories(&conn, &gift.id, 10).expect("related");
        let memory_context = graph::graph_memory_context(&conn, &gift.id, 2).expect("context");
        assert!(related.iter().any(|node| node.source_id == facet.id));
        assert!(memory_context
            .edges
            .iter()
            .any(|edge| edge.edge_type == "has_facet"));
    });
}

#[test]
#[serial]
fn runtime_clock_resolves_relative_dates() {
    let runtime = test_runtime_context();
    assert_eq!(runtime.timezone, "Asia/Kolkata");
    assert_eq!(runtime.day_of_week, "Tue");
    assert!(
        crate::runtime::clock::resolve_relative_time_phrase("tomorrow", &runtime)
            .expect("tomorrow")
            .contains("2026-06-03")
    );
    assert!(
        crate::runtime::clock::resolve_relative_time_phrase("this week", &runtime)
            .expect("this week")
            .contains("2026-06-01")
    );
}

#[test]
#[serial]
fn location_context_saved_place_and_disabled_privacy() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let place = crate::runtime::location::save_place(
            &conn,
            &crate::runtime::location::CreateSavedPlace {
                label: "Home".into(),
                place_type: "home".into(),
                latitude: Some(12.9),
                longitude: Some(77.6),
                address: None,
                city: Some("Bengaluru".into()),
                region: Some("Karnataka".into()),
                country: Some("India".into()),
                timezone: Some("Asia/Kolkata".into()),
                details_json: "{}".into(),
            },
        )
        .expect("save place");
        let places = crate::runtime::location::list_saved_places(&conn).expect("places");
        assert_eq!(places[0].id, place.id);

        let disabled = crate::runtime::location::get_location_context(&conn, false)
            .expect("location disabled");
        assert_eq!(disabled.permission_state, "denied");
        assert_eq!(disabled.privacy_level, "disabled");
    });
}

#[test]
#[serial]
fn compiler_includes_runtime_and_fresh_location_only_when_relevant() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let location = crate::runtime::location::LocationContext {
            status: "exact_current".into(),
            latitude: Some(12.9),
            longitude: Some(77.6),
            accuracy_meters: Some(50.0),
            source: "manual".into(),
            captured_at: Some("2026-06-02T09:59:00Z".into()),
            freshness_seconds: Some(60),
            timezone_hint: Some("Asia/Kolkata".into()),
            city: Some("Bengaluru".into()),
            region: Some("Karnataka".into()),
            country: Some("India".into()),
            label: Some("Home".into()),
            permission_state: "granted".into(),
            privacy_level: "exact".into(),
        };
        let hello = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "hello".into(),
                session_id: None,
                message_id: None,
                runtime_context: test_runtime_context_with_location(location.clone()),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: None,
                enabled_sources: vec![],
                current_workspace_context: Some("workspace".into()),
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile hello");
        assert!(hello.memory_brief.is_empty());
        assert!(hello.workspace_brief.is_empty());
        assert_eq!(hello.location_decision, "skipped:not_relevant");
        assert!(!hello.raw_prompt.contains("Bengaluru"));

        let route = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "when should I leave for grocery?".into(),
                session_id: None,
                message_id: None,
                runtime_context: test_runtime_context_with_location(location),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: None,
                enabled_sources: vec![],
                current_workspace_context: Some("workspace".into()),
                current_route_context: Some("destination: grocery".into()),
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile route");
        assert!(route.runtime_decision.contains("included"));
        assert!(route.location_decision.contains("included"));
        assert!(route.raw_prompt.contains("Bengaluru"));
        assert!(route.workspace_brief.is_empty());
    });
}

#[test]
#[serial]
fn sensitive_memory_is_blocked_when_approval_is_required() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "Health condition: migraine treatment plan.".into(),
                source_quote: None,
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T10:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "health".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        let item = db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "fact".into(),
                title: "Migraine treatment".into(),
                summary: "User has a migraine treatment plan.".into(),
                details_json: "{}".into(),
                status: "active".into(),
                confidence: 0.95,
                user_verified: true,
                sensitivity: "health".into(),
                visibility: "private".into(),
                source_id: source.id,
                observed_at: "2026-06-02T10:00:00Z".into(),
                happened_at: None,
                due_at: None,
                timezone: "Asia/Kolkata".into(),
                time_precision: "unknown".into(),
                natural_time_phrase: None,
                tags: vec!["migraine".into()],
            },
        )
        .expect("memory item");

        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "what do you remember about my migraine?".into(),
                session_id: None,
                message_id: None,
                runtime_context: test_runtime_context(),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: Some(types::EffectivePrivacyPolicy {
                    memory_enabled: true,
                    private_chat: false,
                    pause_memory: false,
                    allow_sensitive_memory_transmission: false,
                    allow_location_context: false,
                }),
                enabled_sources: vec!["manual".into()],
                current_workspace_context: None,
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert!(!output.memory_brief.contains("Migraine treatment"));
        assert!(!output.included_memory_ids.contains(&item.id));
        assert!(output
            .warnings
            .iter()
            .any(|warning| warning.contains("Sensitive memory is blocked")));
    });
}

#[test]
#[serial]
fn private_chat_and_pause_memory_exclude_memory_context() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "I planned to buy saffron.".into(),
                source_quote: None,
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T10:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "task".into(),
                title: "Buy saffron".into(),
                summary: "User planned to buy saffron.".into(),
                details_json: "{}".into(),
                status: "planned".into(),
                confidence: 0.9,
                user_verified: true,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id,
                observed_at: "2026-06-02T10:00:00Z".into(),
                happened_at: None,
                due_at: None,
                timezone: "Asia/Kolkata".into(),
                time_precision: "unknown".into(),
                natural_time_phrase: None,
                tags: vec!["saffron".into()],
            },
        )
        .expect("memory item");

        for (policy, expected_warning) in [
            (
                types::EffectivePrivacyPolicy {
                    memory_enabled: true,
                    private_chat: true,
                    pause_memory: false,
                    allow_sensitive_memory_transmission: true,
                    allow_location_context: false,
                },
                "Private chat is enabled. Stored memory is excluded.",
            ),
            (
                types::EffectivePrivacyPolicy {
                    memory_enabled: true,
                    private_chat: false,
                    pause_memory: true,
                    allow_sensitive_memory_transmission: true,
                    allow_location_context: false,
                },
                "Memory is paused. No memory will be read or saved.",
            ),
        ] {
            let output = compiler::compile_context(
                &conn,
                types::ContextCompilerInput {
                    user_message: "did I buy saffron?".into(),
                    session_id: None,
                    message_id: None,
                    runtime_context: test_runtime_context(),
                    model_context_limit: 8_000,
                    reserved_output_tokens: 1_000,
                    privacy_mode: types::MemoryMode::AskBeforeSaving,
                    effective_privacy_policy: Some(policy),
                    enabled_sources: vec!["manual".into()],
                    current_workspace_context: None,
                    current_route_context: None,
                    manual_context_overrides: vec![],
                    pinned_context_ids: vec![],
                    explicit_skill_id: None,
                    pack_hint: None,
                    ui_selected_skill_id: None,
                    session_pinned_skill_ids: vec![],
                },
            )
            .expect("compile");

            assert!(output.memory_brief.is_empty());
            assert!(output.included_memory_ids.is_empty());
            assert!(output
                .warnings
                .iter()
                .any(|warning| warning == expected_warning));
        }
    });
}

#[test]
#[serial]
fn policy_blocks_location_context_even_when_runtime_location_is_available() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let location = crate::runtime::location::LocationContext {
            status: "exact_current".into(),
            latitude: Some(12.9),
            longitude: Some(77.6),
            accuracy_meters: Some(50.0),
            source: "manual".into(),
            captured_at: Some("2026-06-02T09:59:00Z".into()),
            freshness_seconds: Some(60),
            timezone_hint: Some("Asia/Kolkata".into()),
            city: Some("Bengaluru".into()),
            region: Some("Karnataka".into()),
            country: Some("India".into()),
            label: Some("Home".into()),
            permission_state: "granted".into(),
            privacy_level: "exact".into(),
        };

        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "what is nearby?".into(),
                session_id: None,
                message_id: None,
                runtime_context: test_runtime_context_with_location(location),
                model_context_limit: 8_000,
                reserved_output_tokens: 1_000,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: Some(types::EffectivePrivacyPolicy {
                    memory_enabled: true,
                    private_chat: false,
                    pause_memory: false,
                    allow_sensitive_memory_transmission: true,
                    allow_location_context: false,
                }),
                enabled_sources: vec![],
                current_workspace_context: None,
                current_route_context: None,
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert_eq!(output.location_decision, "skipped:privacy_policy");
        assert!(!output.raw_prompt.contains("Bengaluru"));
    });
}

#[test]
#[serial]
fn model_registry_returns_context_info_and_estimates_tokens() {
    let default_model = crate::runtime::model_registry::get_current_model_context_info();
    assert_eq!(default_model.provider, "gemini");
    assert!(default_model.context_window_tokens >= 32_000);
    assert!(default_model.default_reserved_output_tokens > 0);
    assert_eq!(default_model.tokenizer_strategy, "local_estimate");

    let looked_up =
        crate::runtime::model_registry::get_model_context_info("gemini", "gemini-2.5-flash");
    assert_eq!(looked_up.model_name, "gemini-2.5-flash");
    assert_eq!(looked_up.provider, "gemini");

    let short = crate::runtime::model_registry::estimate_tokens("short text", &looked_up);
    let long = crate::runtime::model_registry::estimate_tokens(
        "short text plus enough extra words to be larger",
        &looked_up,
    );
    assert!(long > short);
}

#[test]
#[serial]
fn compiler_uses_model_budget_and_trims_lower_priority_sections_first() {
    with_temp_memory_db(|| {
        let conn = db::open_memory_db().expect("open memory db");
        let source = db::create_source(
            &conn,
            &types::CreateMemorySource {
                source_type: "manual".into(),
                source_ref: None,
                source_text: "I planned to buy bread today.".into(),
                source_quote: Some("buy bread".into()),
                session_id: None,
                message_id: None,
                observed_at: "2026-06-02T09:00:00Z".into(),
                timezone: "Asia/Kolkata".into(),
                sensitivity: "normal".into(),
                privacy_scope: "local".into(),
            },
        )
        .expect("source");
        db::create_memory_item(
            &conn,
            &types::CreateMemoryItem {
                memory_type: "task".into(),
                title: "Buy bread".into(),
                summary: "User planned to buy bread today.".into(),
                details_json: "{}".into(),
                status: "planned".into(),
                confidence: 0.9,
                user_verified: true,
                sensitivity: "normal".into(),
                visibility: "default".into(),
                source_id: source.id,
                observed_at: "2026-06-02T09:00:00Z".into(),
                happened_at: None,
                due_at: Some("2026-06-02T18:00:00Z".into()),
                timezone: "Asia/Kolkata".into(),
                time_precision: "date_only".into(),
                natural_time_phrase: Some("today".into()),
                tags: vec!["bread".into()],
            },
        )
        .expect("memory item");

        let output = compiler::compile_context(
            &conn,
            types::ContextCompilerInput {
                user_message: "did I buy bread today in this Rust workspace?".into(),
                session_id: None,
                message_id: None,
                runtime_context: test_runtime_context(),
                model_context_limit: 480,
                reserved_output_tokens: 80,
                privacy_mode: types::MemoryMode::AskBeforeSaving,
                effective_privacy_policy: None,
                enabled_sources: vec!["manual".into()],
                current_workspace_context: Some("workspace ".repeat(200)),
                current_route_context: Some("route ".repeat(200)),
                manual_context_overrides: vec![],
                pinned_context_ids: vec![],
                explicit_skill_id: None,
                pack_hint: None,
                ui_selected_skill_id: None,
                session_pinned_skill_ids: vec![],
            },
        )
        .expect("compile");

        assert!(output.token_budget_report.input_budget_tokens <= 400);
        assert!(output.token_budget_report.estimated_prompt_tokens <= 400);
        assert!(output
            .raw_prompt
            .contains("did I buy bread today in this Rust workspace?"));
        assert!(output.memory_brief.contains("Buy bread"));
        assert!(output.workspace_brief.is_empty());
        assert!(output
            .token_budget_report
            .trimmed_sections
            .iter()
            .any(|section| section == "Workspace Brief"));
    });
}
