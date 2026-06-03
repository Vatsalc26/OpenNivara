use super::manifest::*;
use super::prompt::format_active_skills_prompt;
use super::registry::{
    get_enabled_skills_path, get_user_skills_path, load_available_skills, load_routable_skills,
    set_skill_enabled,
};
use super::selector::{select_skill_route, SkillRouteRequest};
use super::tool_policy::allowed_tools_for_selected_skills;
use crate::marketplace::packs::{enable_pack, install_pack_from_path, uninstall_pack};
use crate::tools::{GeneralConfig, PathsConfig, ToolRegistry, ToolSettings, ToolsConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _config_lock: Option<MutexGuard<'static, ()>>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let config_lock = if key == "OPENNIVARA_TEST_CONFIG_DIR" {
            Some(
                crate::config_paths::TEST_CONFIG_ENV_MUTEX
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            )
        } else {
            None
        };
        let old = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key,
            old,
            _config_lock: config_lock,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(ref old) = self.old {
            std::env::set_var(self.key, old);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn setup_registry_test_env(test_name: &str) -> (PathBuf, EnvGuard, EnvGuard) {
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(format!(
        "opennivara_skills_test_{}_{}",
        test_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let config_guard = EnvGuard::set("OPENNIVARA_TEST_CONFIG_DIR", &temp_dir);
    let missing_builtins = temp_dir.join("missing_builtins");
    let builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &missing_builtins);
    (temp_dir, config_guard, builtin_guard)
}

fn valid_skill(id: &str) -> SkillManifest {
    SkillManifest {
        schema_version: 1,
        id: id.to_string(),
        pack_id: Some("upsc_exam".to_string()),
        name: "UPSC Exam Preparation".to_string(),
        description: "Helps plan UPSC study work.".to_string(),
        enabled: true,
        category: "education".to_string(),
        route_policy: SkillRoutePolicy::Auto,
        aliases: vec!["upsc prep".to_string()],
        triggers: vec!["upsc".to_string(), "prelims".to_string()],
        required_any: vec!["upsc".to_string()],
        negative_triggers: vec!["movie".to_string()],
        examples: vec!["make a UPSC prelims study plan".to_string()],
        min_score: 10,
        prompt: SkillPrompt {
            role: "Exam coach".to_string(),
            instructions: "Give structured exam guidance.".to_string(),
            constraints: vec!["Do not invent official deadlines.".to_string()],
        },
        tools: SkillToolPolicy {
            allow: vec!["read_file".to_string()],
            deny: vec![],
        },
        safety: SkillSafety {
            risk_level: "low".to_string(),
            requires_confirmation: false,
            allows_file_write: false,
            allows_shell: false,
            allows_network: false,
            requires_fresh_info: true,
        },
        metadata: SkillMetadata {
            audience: vec!["aspirant".to_string()],
            country: Some("IN".to_string()),
            exam: Some("UPSC CSE".to_string()),
            exam_stage: Some("preparation".to_string()),
            language_style: vec!["english".to_string()],
            last_reviewed_at: Some("2026-06-03".to_string()),
            freshness_sensitive: true,
            official_source_labels: vec!["UPSC".to_string()],
        },
        store_preview: SkillStorePreview {
            best_for: vec!["UPSC aspirants building a study plan".to_string()],
            not_for: vec!["Official notice replacement".to_string()],
            sample_prompts: vec!["make a UPSC prelims study plan".to_string()],
            what_it_will_do: vec!["Turns prep goals into a practical schedule".to_string()],
            what_it_will_not_do: vec!["Invent official deadlines".to_string()],
        },
    }
}

fn valid_user_skill(id: &str) -> SkillManifest {
    let mut skill = valid_skill(id);
    skill.pack_id = None;
    skill
}

fn create_skill_pack_source(dir: &Path, pack_id: &str, skill_id: &str) -> PathBuf {
    let pack_dir = dir.join(pack_id);
    fs::create_dir_all(pack_dir.join("skills")).unwrap();
    let pack_toml = format!(
        r##"schema_version = 1
id = "{pack_id}"
name = "{pack_id}"
version = "1.0.0"
author = "Test Author"
category = "Testing"
description = "A test skill pack"
[compatibility]
opennivara_min_version = "0.1.0"
[contents]
preferences = false
contexts = false
style_presets = false
profile_templates = false
tool_presets = false
workspace_map_rules = false
prompt_behaviors = false
command_snippets = false
theme = false
skills = true
[safety]
contains_executable_code = false
modifies_tool_permissions = false
requires_network = false
risk_level = "low"
"##
    );
    fs::write(pack_dir.join("pack.toml"), pack_toml).unwrap();

    let mut skill = valid_skill(skill_id);
    skill.pack_id = Some(pack_id.to_string());
    let skill_toml = toml::to_string_pretty(&skill).unwrap();
    fs::write(
        pack_dir.join("skills").join(format!("{skill_id}.toml")),
        skill_toml,
    )
    .unwrap();
    pack_dir
}

fn tools_config() -> ToolsConfig {
    let mut tools = HashMap::new();
    for name in ["get_current_dir", "list_dir", "file_exists", "read_file"] {
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

#[test]
fn duplicate_skill_ids_fail_validation() {
    let skills = SkillsFile {
        schema_version: 1,
        skills: vec![valid_skill("upsc_exam"), valid_skill("upsc_exam")],
    };

    assert!(validate_skills_file(&skills, &ToolRegistry::new(false)).is_err());
}

#[test]
fn unknown_tool_in_allowlist_fails_validation() {
    let mut skill = valid_skill("upsc_exam");
    skill.tools.allow = vec!["missing_tool".to_string()];

    assert!(validate_skill_manifest(&skill, &ToolRegistry::new(false)).is_err());
}

#[test]
fn disabled_skill_is_never_selected() {
    let mut skill = valid_skill("upsc_exam");
    skill.enabled = false;

    let decision = select_skill_route(
        &[skill],
        SkillRouteRequest {
            message: "make a UPSC prelims plan".to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );

    assert!(decision.primary_skill.is_none());
}

#[test]
fn explicit_skill_command_selects_exact_skill() {
    let skill = valid_skill("upsc_exam_preparation");

    let decision = select_skill_route(
        &[skill],
        SkillRouteRequest {
            message: "/skill upsc_exam_preparation make a plan".to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );

    assert_eq!(
        decision
            .primary_skill
            .as_ref()
            .map(|skill| skill.id.as_str()),
        Some("upsc_exam_preparation")
    );
}

#[test]
fn ambiguous_close_scores_return_no_primary() {
    let mut a = valid_skill("upsc_exam_preparation");
    let mut b = valid_skill("upsc_syllabus_breakdown");
    a.aliases = vec!["upsc".to_string()];
    b.aliases = vec!["upsc".to_string()];

    let decision = select_skill_route(
        &[a, b],
        SkillRouteRequest {
            message: "help with upsc".to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );

    assert!(decision.primary_skill.is_none());
    assert!(decision.reason.contains("ambiguous"));
}

#[test]
fn selected_skill_injects_prompt_block() {
    let skill = valid_skill("upsc_exam_preparation");
    let decision = select_skill_route(
        &[skill],
        SkillRouteRequest {
            message: "make a UPSC prelims study plan".to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );

    let prompt = format_active_skills_prompt(&decision, &[valid_skill("upsc_exam_preparation")]);

    assert!(prompt.contains("Active Skills:"));
    assert!(prompt.contains("Primary Skill: UPSC Exam Preparation"));
    assert!(prompt.contains("Give structured exam guidance."));
}

#[test]
fn selected_skill_tool_allowlist_filters_declarations() {
    let skill = valid_skill("upsc_exam_preparation");
    let decision = select_skill_route(
        &[skill],
        SkillRouteRequest {
            message: "/skill upsc_exam_preparation read my notes".to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );

    let allowed = allowed_tools_for_selected_skills(
        &decision.selected_skills(),
        &ToolRegistry::new(false),
        &tools_config(),
    );

    assert_eq!(
        allowed.allowed_tool_names,
        ["read_file"].into_iter().map(String::from).collect()
    );
}

#[test]
fn duplicate_skill_ids_across_installed_packs_fail() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) = setup_registry_test_env("duplicate_pack_skill_ids");
    crate::marketplace::init_marketplace().unwrap();

    let pack_a = create_skill_pack_source(&temp, "skill_pack_a", "shared_skill");
    let pack_b = create_skill_pack_source(&temp, "skill_pack_b", "shared_skill");
    install_pack_from_path(pack_a).unwrap();
    install_pack_from_path(pack_b).unwrap();

    let err = load_available_skills().unwrap_err().to_string();
    assert!(err.contains("Duplicate skill IDs are not allowed"));
    assert!(err.contains("shared_skill"));
    assert!(err.contains("skill_pack_a"));
    assert!(err.contains("skill_pack_b"));
}

#[test]
fn duplicate_user_and_pack_skill_ids_fail() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) =
        setup_registry_test_env("duplicate_user_pack_skill_ids");
    crate::marketplace::init_marketplace().unwrap();

    fs::create_dir_all(get_user_skills_path().unwrap().parent().unwrap()).unwrap();
    crate::config_store::save_toml_file(
        &get_user_skills_path().unwrap(),
        &SkillsFile {
            schema_version: 1,
            skills: vec![valid_user_skill("shared_skill")],
        },
    )
    .unwrap();

    let pack = create_skill_pack_source(&temp, "skill_pack", "shared_skill");
    install_pack_from_path(pack).unwrap();

    let err = load_available_skills().unwrap_err().to_string();
    assert!(err.contains("Duplicate skill IDs are not allowed"));
    assert!(err.contains("shared_skill"));
    assert!(err.contains("user skills"));
    assert!(err.contains("skill_pack"));
}

#[test]
fn set_skill_enabled_fails_when_duplicate_ids_are_loaded() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) =
        setup_registry_test_env("set_enabled_duplicate_skill_ids");
    crate::marketplace::init_marketplace().unwrap();

    let pack_a = create_skill_pack_source(&temp, "skill_pack_a", "shared_skill");
    let pack_b = create_skill_pack_source(&temp, "skill_pack_b", "shared_skill");
    install_pack_from_path(pack_a).unwrap();
    install_pack_from_path(pack_b).unwrap();

    let err = set_skill_enabled("shared_skill", true)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Duplicate skill IDs are not allowed"));
    assert!(!get_enabled_skills_path().unwrap().exists());
}

#[test]
fn installed_pack_skills_are_available_but_disabled_by_default() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) =
        setup_registry_test_env("pack_skills_disabled_default");
    crate::marketplace::init_marketplace().unwrap();

    let pack = create_skill_pack_source(&temp, "skill_pack", "pack_skill");
    install_pack_from_path(pack).unwrap();

    let skills = load_available_skills().unwrap();
    let pack_skill = skills
        .iter()
        .find(|skill| skill.id == "pack_skill")
        .expect("installed pack skill should be available");
    assert_eq!(pack_skill.pack_id.as_deref(), Some("skill_pack"));
    assert!(!pack_skill.enabled);
    assert!(load_routable_skills().unwrap().is_empty());
}

#[test]
fn enabling_pack_skill_writes_enabled_skills_state_only() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) =
        setup_registry_test_env("pack_skill_enabled_state_only");
    crate::marketplace::init_marketplace().unwrap();

    let pack = create_skill_pack_source(&temp, "skill_pack", "pack_skill");
    install_pack_from_path(pack).unwrap();
    super::registry::init_skills().unwrap();
    let user_skills_before = fs::read_to_string(get_user_skills_path().unwrap()).unwrap();

    set_skill_enabled("pack_skill", true).unwrap();

    let enabled_state = fs::read_to_string(get_enabled_skills_path().unwrap()).unwrap();
    assert!(enabled_state.contains("pack_skill"));
    assert!(enabled_state.contains("skill_pack"));
    assert_eq!(
        fs::read_to_string(get_user_skills_path().unwrap()).unwrap(),
        user_skills_before
    );
    assert!(load_routable_skills()
        .unwrap()
        .iter()
        .any(|skill| skill.id == "pack_skill"));
}

#[test]
fn disabling_parent_pack_makes_child_skills_non_routable() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) =
        setup_registry_test_env("disabled_parent_pack_skill");
    crate::marketplace::init_marketplace().unwrap();

    let pack = create_skill_pack_source(&temp, "skill_pack", "pack_skill");
    install_pack_from_path(pack).unwrap();
    set_skill_enabled("pack_skill", true).unwrap();
    assert!(load_routable_skills()
        .unwrap()
        .iter()
        .any(|skill| skill.id == "pack_skill"));

    enable_pack("skill_pack", false).unwrap();

    let available = load_available_skills().unwrap();
    let pack_skill = available
        .iter()
        .find(|skill| skill.id == "pack_skill")
        .expect("disabled parent pack should not remove availability");
    assert!(!pack_skill.enabled);
    assert!(!load_routable_skills()
        .unwrap()
        .iter()
        .any(|skill| skill.id == "pack_skill"));
}

#[test]
fn uninstalling_pack_removes_child_skill_availability() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard, _builtin_guard) =
        setup_registry_test_env("uninstall_pack_removes_skill");
    crate::marketplace::init_marketplace().unwrap();

    let pack = create_skill_pack_source(&temp, "skill_pack", "pack_skill");
    install_pack_from_path(pack).unwrap();
    assert!(load_available_skills()
        .unwrap()
        .iter()
        .any(|skill| skill.id == "pack_skill"));

    uninstall_pack("skill_pack").unwrap();

    assert!(!load_available_skills()
        .unwrap()
        .iter()
        .any(|skill| skill.id == "pack_skill"));
}

fn load_builtin_pack_skills_for_routing(pack_id: &str) -> Vec<SkillManifest> {
    let skills_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("packs")
        .join("builtin")
        .join(pack_id)
        .join("skills");
    fs::read_dir(&skills_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("toml"))
        .map(|entry| {
            let mut skill: SkillManifest =
                crate::config_store::read_toml_file(&entry.path()).unwrap();
            skill.enabled = true;
            skill
        })
        .collect()
}

fn assert_routes_to(skills: &[SkillManifest], message: &str, expected_skill_id: &str) {
    let decision = select_skill_route(
        skills,
        SkillRouteRequest {
            message: message.to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );
    assert_eq!(
        decision
            .primary_skill
            .as_ref()
            .map(|skill| skill.id.as_str()),
        Some(expected_skill_id),
        "message: {message}; reason: {}; candidates: {:?}",
        decision.reason,
        decision.candidates
    );
}

fn assert_routes_to_none(skills: &[SkillManifest], message: &str) {
    let decision = select_skill_route(
        skills,
        SkillRouteRequest {
            message: message.to_string(),
            explicit_skill_id: None,
            pack_hint: None,
            ui_selected_skill_id: None,
            session_pinned_skill_ids: vec![],
        },
    );
    assert!(
        decision.primary_skill.is_none(),
        "message: {message}; selected: {:?}; candidates: {:?}",
        decision.primary_skill,
        decision.candidates
    );
}

#[test]
fn india_builtin_engineering_routes_expected_exam_skills() {
    let skills = load_builtin_pack_skills_for_routing("india_engineering_exams");
    assert_routes_to(
        &skills,
        "make me a 90 day plan for jee mains",
        "jee_main_study_planner",
    );
    assert_routes_to(
        &skills,
        "analyze my jee main mock p 50 c 70 m 40",
        "jee_main_mock_test_analyzer",
    );
    assert_routes_to(
        &skills,
        "am i ready for jee advanced after mains",
        "jee_advanced_readiness_evaluator",
    );
    assert_routes_to(
        &skills,
        "build error notebook for jee advanced pyqs",
        "jee_advanced_error_notebook_builder",
    );
    assert_routes_to(&skills, "gate cse pyq analysis", "gate_pyq_analyzer");
    assert_routes_to(
        &skills,
        "gate two paper combination da cse",
        "gate_two_paper_combination_advisor",
    );
    assert_routes_to_none(&skills, "jee advanced paper 1 mock analysis");
}

#[test]
fn india_builtin_medical_routes_expected_exam_skills() {
    let skills = load_builtin_pack_skills_for_routing("india_medical_exams");
    assert_routes_to(
        &skills,
        "neet ug biology ncert revision plan",
        "neet_ug_ncert_biology_mastery_coach",
    );
    assert_routes_to(
        &skills,
        "neet mock score 480 what should i improve",
        "neet_ug_mock_test_analyzer",
    );
    assert_routes_to(
        &skills,
        "neet pg grand test analysis medicine weak",
        "neet_pg_grand_test_analyzer",
    );
    assert_routes_to(
        &skills,
        "i am in internship how to study for neet pg",
        "neet_pg_internship_time_study_planner",
    );
}

#[test]
fn india_builtin_upsc_routes_expected_exam_skills() {
    let skills = load_builtin_pack_skills_for_routing("india_upsc_cse");
    assert_routes_to(
        &skills,
        "upsc prelims strategy for 6 months",
        "upsc_prelims_strategy_coach",
    );
    assert_routes_to(
        &skills,
        "help me with upsc mains answer writing",
        "upsc_mains_answer_writing_coach",
    );
    assert_routes_to_none(&skills, "react form validation");
}

#[test]
fn india_builtin_management_law_routes_expected_exam_skills() {
    let skills = load_builtin_pack_skills_for_routing("india_management_law_exams");
    assert_routes_to(
        &skills,
        "cat dilr set selection strategy",
        "cat_dilr_set_selection_coach",
    );
    assert_routes_to(
        &skills,
        "cat mock varc low accuracy",
        "cat_mock_test_analyzer",
    );
    assert_routes_to(
        &skills,
        "clat legal reasoning practice plan",
        "clat_legal_reasoning_coach",
    );
    assert_routes_to(
        &skills,
        "clat gk current affairs tracker",
        "clat_gk_current_affairs_tracker",
    );
}

#[test]
fn india_builtin_ambiguity_routes_to_broad_skills_only_when_appropriate() {
    let mut skills = load_builtin_pack_skills_for_routing("india_student_essentials");
    skills.extend(load_builtin_pack_skills_for_routing(
        "india_engineering_exams",
    ));
    skills.extend(load_builtin_pack_skills_for_routing("india_medical_exams"));
    skills.extend(load_builtin_pack_skills_for_routing("india_upsc_cse"));
    skills.extend(load_builtin_pack_skills_for_routing(
        "india_management_law_exams",
    ));
    assert_routes_to_none(&skills, "mock test analysis");
    assert_routes_to_none(&skills, "form filling help");
    assert_routes_to(&skills, "study plan", "india_study_plan_builder");
}
