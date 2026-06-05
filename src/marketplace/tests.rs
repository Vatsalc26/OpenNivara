use crate::marketplace::builtin::*;
use crate::marketplace::merge::*;
use crate::marketplace::modes::*;
use crate::marketplace::packs::*;
use crate::marketplace::repair::*;
use crate::marketplace::themes::*;
use crate::marketplace::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    old: Option<String>,
    _config_lock: Option<MutexGuard<'static, ()>>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let config_lock = lock_config_env_if_needed(key);
        let old = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key,
            old,
            _config_lock: config_lock,
        }
    }

    fn remove(key: &'static str) -> Self {
        let config_lock = lock_config_env_if_needed(key);
        let old = std::env::var(key).ok();
        std::env::remove_var(key);
        Self {
            key,
            old,
            _config_lock: config_lock,
        }
    }
}

fn lock_config_env_if_needed(key: &str) -> Option<MutexGuard<'static, ()>> {
    if key != "OPENNIVARA_TEST_CONFIG_DIR" {
        return None;
    }

    Some(
        crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    )
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

fn setup_test_env(test_name: &str) -> (PathBuf, EnvGuard) {
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(format!(
        "opennivara_test_{}_{}",
        test_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let guard = EnvGuard::set("OPENNIVARA_TEST_CONFIG_DIR", &temp_dir);
    (temp_dir, guard)
}

#[derive(Clone, Copy)]
struct MockPackContents {
    preferences: bool,
    contexts: bool,
    style_presets: bool,
    theme: bool,
}

fn mock_contents(
    preferences: bool,
    contexts: bool,
    style_presets: bool,
    theme: bool,
) -> MockPackContents {
    MockPackContents {
        preferences,
        contexts,
        style_presets,
        theme,
    }
}

fn create_mock_pack(
    dir: &std::path::Path,
    id: &str,
    name: &str,
    version: &str,
    contents: MockPackContents,
) -> PathBuf {
    let has_pref = contents.preferences;
    let has_ctx = contents.contexts;
    let has_style = contents.style_presets;
    let has_theme = contents.theme;
    let pack_dir = dir.join(id);
    fs::create_dir_all(&pack_dir).unwrap();

    let pack_toml = format!(
        r##"schema_version = 1
id = "{id}"
name = "{name}"
version = "{version}"
author = "Test Author"
category = "Testing"
description = "A mock pack for testing"
[compatibility]
opennivara_min_version = "0.1.0"
[contents]
preferences = {has_pref}
contexts = {has_ctx}
style_presets = {has_style}
profile_templates = false
tool_presets = false
workspace_map_rules = false
prompt_behaviors = false
command_snippets = false
theme = {has_theme}
[safety]
contains_executable_code = false
modifies_tool_permissions = false
requires_network = false
risk_level = "low"
"##
    );
    fs::write(pack_dir.join("pack.toml"), pack_toml).unwrap();

    if has_pref {
        let pref_toml = r#"schema_version = 1
[[sections]]
id = "test_pref_sec"
enabled = true
send_policy = "always"
min_score = 0
description = "Mock preferences"
likes = [
  { item = "rust", strength = 5 }
]
dislikes = []
notes = []
"#;
        fs::write(pack_dir.join("preferences.toml"), pref_toml).unwrap();
    }

    if has_ctx {
        let ctx_toml = r#"schema_version = 1
[[contexts]]
id = "test_ctx_sec"
enabled = true
kind = "goal"
send_policy = "always"
title = "Test Context Title"
summary = "Mock context summary"
min_score = 0
facts = ["Mock Context Data"]
rules = []
"#;
        fs::write(pack_dir.join("contexts.toml"), ctx_toml).unwrap();
    }

    if has_style {
        let style_toml = r#"schema_version = 2
[communication]
tone = "Formal"
detail_level = "High"
use_examples = true
use_step_by_step = true
avoid_unexplained_jargon = true
ask_fewer_questions = true
prefer_actionable_answers = true

[coding]
show_simple_solution_first = true
explain_after_code = true
prefer_mvp_architecture = true
avoid_overengineering = true
use_beginner_comments = true

[formatting]
use_markdown = true
use_short_sections = true
include_next_step = false
avoid_long_walls_of_text = true

[behavior]
be_honest_about_uncertainty = true
do_not_pretend_to_have_done_things = true
do_not_reveal_private_context_unless_relevant = true
"#;
        fs::write(pack_dir.join("style.toml"), style_toml).unwrap();
    }

    if has_theme {
        let theme_toml = format!(
            r##"schema_version = 1
id = "theme_{id}"
name = "Theme for {name}"
description = "Mock theme"
[colors]
background = "#1a1a1a"
panel = "#2d2d2d"
card = "#3d3d3d"
primary = "#00ffff"
accent = "#ff00ff"
success = "#00ff00"
warning = "#ffff00"
danger = "#ff0000"
foreground = "#ffffff"
muted = "#888888"
[effects]
background_gradient = false
glow = "cyan"
density = "compact"
"##
        );
        fs::write(pack_dir.join("theme.toml"), theme_toml).unwrap();
    }

    pack_dir
}

fn skill_manifest_toml(skill_id: &str, pack_id: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{skill_id}"
pack_id = "{pack_id}"
name = "Pack Skill"
description = "A test skill from a pack."
enabled = true
category = "testing"
route_policy = "auto"
aliases = ["pack skill"]
triggers = ["pack skill"]
required_any = []
negative_triggers = []
examples = ["use the pack skill"]
min_score = 10

[prompt]
role = "Test helper"
instructions = "Help with a test skill."
constraints = []

[tools]
allow = ["read_file"]
deny = []

[safety]
risk_level = "low"
requires_confirmation = false
allows_file_write = false
allows_shell = false
allows_network = false
requires_fresh_info = false

[metadata]
audience = ["aspirant"]
country = "IN"
exam = "UPSC CSE"
exam_stage = "preparation"
language_style = ["english"]
last_reviewed_at = "2026-06-03"
freshness_sensitive = false
official_source_labels = []

[store_preview]
best_for = ["Testing Store skill preview rows"]
not_for = ["Runtime execution"]
sample_prompts = ["use the pack skill"]
what_it_does = ["Provides inspectable Store metadata"]
what_it_will_not_do = ["Enable behavior at install time"]
"#
    )
}

fn create_mock_skill_pack(dir: &std::path::Path, id: &str, include_skills_dir: bool) -> PathBuf {
    let pack_dir = dir.join(id);
    fs::create_dir_all(&pack_dir).unwrap();

    let pack_toml = format!(
        r##"schema_version = 1
id = "{id}"
name = "Skill Pack"
version = "1.0.0"
author = "Test Author"
category = "Testing"
description = "A mock skill pack for testing"
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

    if include_skills_dir {
        let skills_dir = pack_dir.join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        fs::write(
            skills_dir.join("pack_skill.toml"),
            skill_manifest_toml("pack_skill", id),
        )
        .unwrap();
    }

    pack_dir
}

#[test]
fn test_default_mode_blocks_adding_packs() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("default_mode_blocks");
    init_marketplace().unwrap();

    let res = add_pack_to_mode("default", "coding_basics");
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Default Mode is protected"));
}

#[test]
fn test_disabled_packs_skipped_by_bundle() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("disabled_packs_bundle");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "mock_pack",
        "Mock Pack",
        "1.0.0",
        mock_contents(true, true, false, false),
    );
    install_pack_from_path(src_dir).unwrap();
    super::addon_settings::toggle_pack_enabled("coding_basics", false).unwrap();
    super::addon_settings::toggle_pack_enabled("study_coach", false).unwrap();
    super::addon_settings::toggle_pack_enabled("mock_pack", true).unwrap();

    let mut mode = get_active_mode().unwrap();
    mode.id = "custom_mode".to_string();
    mode.name = "Custom Mode".to_string();
    mode.enabled_pack_ids = vec!["mock_pack".to_string()];
    create_mode(mode).unwrap();
    set_active_mode("custom_mode").unwrap();

    let bundle_enabled = get_active_pack_bundle().unwrap();
    assert!(!bundle_enabled.preferences.is_empty());
    assert!(!bundle_enabled.contexts.is_empty());

    enable_pack("mock_pack", false).unwrap();

    let bundle_disabled = get_active_pack_bundle().unwrap();
    assert!(bundle_disabled.preferences.is_empty());
    assert!(bundle_disabled.contexts.is_empty());
}

#[test]
fn test_disabled_packs_skipped_by_themes() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("disabled_packs_themes");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "mock_theme_pack",
        "Mock Theme Pack",
        "1.0.0",
        mock_contents(false, false, false, true),
    );
    install_pack_from_path(src_dir).unwrap();

    let themes_enabled = list_themes().unwrap();
    assert!(themes_enabled
        .iter()
        .any(|t| t.id == "theme_mock_theme_pack"));

    enable_pack("mock_theme_pack", false).unwrap();

    let themes_disabled = list_themes().unwrap();
    assert!(!themes_disabled
        .iter()
        .any(|t| t.id == "theme_mock_theme_pack"));
}

#[test]
fn test_missing_mode_fallback_to_default() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("missing_mode_fallback");
    init_marketplace().unwrap();

    let modes_path = get_modes_path().unwrap();
    let mut modes_file = read_modes().unwrap();
    modes_file.active_mode = "missing_mode".to_string();
    crate::config_store::save_toml_file(&modes_path, &modes_file).unwrap();

    let report = marketplace_repair(false).unwrap();
    assert!(report.repaired);
    assert!(report
        .actions
        .iter()
        .any(|a| a.contains("reset to 'default' mode")));

    let active = get_active_mode().unwrap();
    assert_eq!(active.id, "default");
}

#[test]
fn test_installed_pack_preview_uses_copied_path() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("installed_pack_preview");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "preview_pack",
        "Preview Pack",
        "1.0.0",
        mock_contents(false, false, false, false),
    );
    install_pack_from_path(src_dir).unwrap();

    let preview = preview_installed_pack("preview_pack").unwrap();

    let packs_dir = get_packs_dir().unwrap();
    let expected_installed_dir = packs_dir.join("preview_pack");
    assert_eq!(PathBuf::from(preview.source_path), expected_installed_dir);
}

#[test]
fn test_preview_pack_with_skills_counts_skill_manifests() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("preview_pack_skills_count");

    let src_dir = create_mock_skill_pack(&temp, "skill_pack", true);
    let preview = preview_pack_from_path(src_dir).unwrap();

    assert_eq!(preview.additions.skills_count, 1);
    assert_eq!(preview.skill_previews.len(), 1);
    assert_eq!(preview.skill_previews[0].id, "pack_skill");
    assert_eq!(
        preview.skill_previews[0].pack_id.as_deref(),
        Some("skill_pack")
    );
    assert!(preview.warnings.is_empty());
}

#[test]
fn test_preview_pack_with_missing_skills_dir_warns() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("preview_pack_missing_skills");

    let src_dir = create_mock_skill_pack(&temp, "skill_pack", false);
    let preview = preview_pack_from_path(src_dir).unwrap();

    assert_eq!(preview.additions.skills_count, 0);
    assert!(preview
        .warnings
        .iter()
        .any(|warning| warning.contains("skills/ is missing")));
}

#[test]
fn test_install_pack_copies_skill_manifests() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("install_pack_copies_skills");
    init_marketplace().unwrap();

    let src_dir = create_mock_skill_pack(&temp, "skill_pack", true);
    install_pack_from_path(src_dir).unwrap();

    let installed_skill_path = get_packs_dir()
        .unwrap()
        .join("skill_pack")
        .join("skills")
        .join("pack_skill.toml");
    assert!(installed_skill_path.exists());
    assert!(fs::read_to_string(installed_skill_path)
        .unwrap()
        .contains("pack_skill"));
}

#[test]
fn test_install_pack_rejects_duplicate_skill_ids_before_copying() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("install_pack_rejects_duplicate_skills");
    init_marketplace().unwrap();

    let first_pack = create_mock_skill_pack(&temp, "first_skill_pack", true);
    install_pack_from_path(first_pack).unwrap();

    let second_pack = create_mock_skill_pack(&temp, "second_skill_pack", true);
    let err = install_pack_from_path(second_pack).unwrap_err().to_string();

    assert!(err.contains("skill ID \"pack_skill\" already exists in pack \"first_skill_pack\""));
    assert!(!get_packs_dir().unwrap().join("second_skill_pack").exists());
    assert!(crate::skills::registry::load_available_skills().is_ok());
}

#[test]
fn test_builtin_packs_discovery() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("builtin_discovery");

    let custom_builtin = temp.join("custom_builtin");
    fs::create_dir_all(&custom_builtin).unwrap();
    create_mock_pack(
        &custom_builtin,
        "custom_builtin_pack",
        "Custom Builtin Pack",
        "1.0.0",
        mock_contents(false, false, false, false),
    );

    let _builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &custom_builtin);

    let builtin_dir = get_builtin_packs_dir().unwrap();
    assert_eq!(builtin_dir, custom_builtin);

    let list = list_builtin_packs().unwrap();
    assert!(list.iter().any(|p| p.id == "custom_builtin_pack"));
}

#[test]
fn test_style_loaded_solely_from_style_pack() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("style_from_pack");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "style_pack",
        "Style Pack",
        "1.0.0",
        mock_contents(false, false, true, false),
    );
    install_pack_from_path(src_dir).unwrap();
    super::addon_settings::toggle_pack_enabled("coding_basics", false).unwrap();
    super::addon_settings::toggle_pack_enabled("study_coach", false).unwrap();
    super::addon_settings::toggle_pack_enabled("style_pack", true).unwrap();

    let mut mode = get_active_mode().unwrap();
    mode.id = "style_mode".to_string();
    mode.name = "Style Mode".to_string();
    mode.style_pack_id = Some("style_pack".to_string());
    mode.enabled_pack_ids = vec!["style_pack".to_string()];
    create_mode(mode).unwrap();
    set_active_mode("style_mode").unwrap();

    let bundle = get_active_pack_bundle().unwrap();
    assert_eq!(bundle.style_pack_id, Some("style_pack".to_string()));

    let packs_dir = get_packs_dir().unwrap();
    let style_path = packs_dir.join("style_pack").join("style.toml");
    assert!(style_path.exists());

    let content = fs::read_to_string(&style_path).unwrap();
    let style: crate::style::OpenNivaraStyle = toml::from_str(&content).unwrap();
    assert_eq!(style.communication.tone, "Formal");
    assert_eq!(style.communication.detail_level, "High");
}

#[test]
fn test_repair_dry_run_does_not_write() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("repair_dry_run");

    let mkt_dir = get_marketplace_dir().unwrap();
    assert!(!mkt_dir.exists());

    let report = marketplace_repair(true).unwrap();
    assert!(report.repaired);
    assert!(!report.actions.is_empty());

    assert!(!mkt_dir.exists());
}

#[test]
fn test_fresh_init_without_builtins_has_no_missing_refs() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("fresh_init_no_builtins");
    let missing_builtin_dir = temp.join("does_not_exist");
    let _builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &missing_builtin_dir);

    init_marketplace().unwrap();

    let status = marketplace_status().unwrap();
    assert!(status.missing_pack_ids.is_empty());
    assert_eq!(status.installed_count, 0);

    let modes = read_modes().unwrap();
    assert!(modes.modes.iter().any(|m| m.id == "default"));
    assert!(modes
        .modes
        .iter()
        .all(|m| m.enabled_pack_ids.iter().all(|pack_id| {
            status.builtin_packs_available.contains(pack_id)
                || status
                    .missing_pack_ids
                    .iter()
                    .all(|missing| missing != pack_id)
        })));
}

#[test]
fn test_fresh_init_with_builtins_installs_modes_cleanly() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("fresh_init_with_builtins");
    let custom_builtin = temp.join("custom_builtin");
    fs::create_dir_all(&custom_builtin).unwrap();
    create_mock_pack(
        &custom_builtin,
        "coding_basics",
        "Coding Basics",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    create_mock_pack(
        &custom_builtin,
        "study_coach",
        "Study Coach",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    let _builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &custom_builtin);

    init_marketplace().unwrap();

    let installed = list_installed_packs().unwrap();
    assert!(installed.installed.is_empty());

    let builtins = super::builtin::list_builtin_packs().unwrap();
    assert!(builtins.iter().any(|p| p.id == "coding_basics"));
    assert!(builtins.iter().any(|p| p.id == "study_coach"));

    let modes = read_modes().unwrap();
    assert!(modes.modes.iter().any(|m| m.id == "default"));
    assert!(!modes.modes.iter().any(|m| m.id == "coding"));
    assert!(!modes.modes.iter().any(|m| m.id == "study"));

    let status = marketplace_status().unwrap();
    assert!(status.missing_pack_ids.is_empty());
}

#[test]
fn test_remove_missing_reference_clears_status_warning() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("remove_missing_ref");
    let _builtin_guard = EnvGuard::remove("OPENNIVARA_BUILTIN_PACKS_DIR");
    init_marketplace().unwrap();

    create_mode(OpenNivaraMode {
        id: "broken".to_string(),
        name: "Broken Mode".to_string(),
        description: "References a missing pack.".to_string(),
        enabled_pack_ids: vec!["missing_pack".to_string()],
        theme_id: None,
        style_pack_id: None,
    })
    .unwrap();

    let before = marketplace_status().unwrap();
    assert!(before
        .missing_pack_ids
        .contains(&"missing_pack".to_string()));

    remove_pack_from_mode("broken", "missing_pack").unwrap();

    let after = marketplace_status().unwrap();
    assert!(!after.missing_pack_ids.contains(&"missing_pack".to_string()));
}

#[test]
fn test_add_pack_to_mode_with_activation() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("add_pack_activation");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "theme_style_pack",
        "Theme & Style Pack",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    install_pack_from_path(src_dir).unwrap();

    create_mode(OpenNivaraMode {
        id: "custom".to_string(),
        name: "Custom Mode".to_string(),
        description: "".to_string(),
        enabled_pack_ids: vec![],
        theme_id: None,
        style_pack_id: None,
    })
    .unwrap();

    let res = add_pack_to_mode_with_activation("custom", "theme_style_pack", true, true).unwrap();
    assert!(res.added_pack);
    assert_eq!(
        res.applied_theme_id,
        Some("theme_theme_style_pack".to_string())
    );
    assert_eq!(
        res.applied_style_pack_id,
        Some("theme_style_pack".to_string())
    );

    let mode = read_modes()
        .unwrap()
        .modes
        .into_iter()
        .find(|m| m.id == "custom")
        .unwrap();
    assert_eq!(mode.theme_id, Some("theme_theme_style_pack".to_string()));
    assert_eq!(mode.style_pack_id, Some("theme_style_pack".to_string()));
}

#[test]
fn test_create_mode_from_pack() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("create_from_pack");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "themed_pack",
        "Themed Pack",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    install_pack_from_path(src_dir).unwrap();

    let mode = create_mode_from_pack(
        "themed_pack",
        "new_custom_mode",
        "New Custom",
        true,
        true,
        true,
    )
    .unwrap();
    assert_eq!(mode.id, "new_custom_mode");
    assert_eq!(mode.theme_id, Some("theme_themed_pack".to_string()));
    assert_eq!(mode.style_pack_id, Some("themed_pack".to_string()));

    let file = read_modes().unwrap();
    assert_eq!(file.active_mode, "new_custom_mode");
}

#[test]
fn test_update_mode_theme_and_style_pack_validation() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("update_theme_style_val");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "pack_a",
        "Pack A",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    install_pack_from_path(src_dir).unwrap();

    let src_dir_b = create_mock_pack(
        &temp,
        "pack_b",
        "Pack B",
        "1.0.0",
        mock_contents(false, false, false, false),
    );
    install_pack_from_path(src_dir_b).unwrap();

    create_mode(OpenNivaraMode {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "".to_string(),
        enabled_pack_ids: vec!["pack_a".to_string()],
        theme_id: None,
        style_pack_id: None,
    })
    .unwrap();

    // theme exists in enabled pack -> ok
    assert!(update_mode_theme("custom", Some("theme_pack_a".to_string())).is_ok());

    // theme does NOT exist in enabled pack -> error
    assert!(update_mode_theme("custom", Some("theme_pack_b".to_string())).is_err());

    // style exists in enabled pack -> ok
    assert!(update_mode_style_pack("custom", Some("pack_a".to_string())).is_ok());

    // style pack does NOT have style.toml -> error
    create_mode(OpenNivaraMode {
        id: "custom_b".to_string(),
        name: "Custom B".to_string(),
        description: "".to_string(),
        enabled_pack_ids: vec!["pack_b".to_string()],
        theme_id: None,
        style_pack_id: None,
    })
    .unwrap();
    assert!(update_mode_style_pack("custom_b", Some("pack_b".to_string())).is_err());
}

#[test]
fn test_marketplace_status_readonly_does_not_create_files() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("validate_readonly");

    // Before init, check directories
    let mkt_dir = get_marketplace_dir().unwrap();
    assert!(!mkt_dir.exists());

    let status = marketplace_status_readonly().unwrap();
    assert_eq!(status.installed_count, 0);
    assert_eq!(status.active_mode_id, "default");

    // It should not create the marketplace directory
    assert!(!mkt_dir.exists());
}

#[test]
fn test_repair_missing_modes_toml_restores_builtins() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("repair_modes_restore");

    // Setup mock builtins in discovery path
    let custom_builtin = temp.join("custom_builtin");
    fs::create_dir_all(&custom_builtin).unwrap();
    create_mock_pack(
        &custom_builtin,
        "coding_basics",
        "Coding Basics",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    create_mock_pack(
        &custom_builtin,
        "study_coach",
        "Study Coach",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    let _builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &custom_builtin);

    init_marketplace().unwrap();

    // Verify builtins are discoverable but not auto-installed as modes.
    assert!(get_modes_path().unwrap().exists());
    let modes = read_modes().unwrap();
    assert!(modes.modes.iter().any(|m| m.id == "default"));
    assert!(!modes.modes.iter().any(|m| m.id == "coding"));

    // Delete modes.toml
    fs::remove_file(get_modes_path().unwrap()).unwrap();
    assert!(!get_modes_path().unwrap().exists());

    // Repair should restore modes.toml without installing built-in packs.
    let report = marketplace_repair(false).unwrap();
    assert!(report.repaired);
    assert!(get_modes_path().unwrap().exists());

    let restored_modes = read_modes().unwrap();
    assert!(restored_modes.modes.iter().any(|m| m.id == "default"));
    assert!(!restored_modes.modes.iter().any(|m| m.id == "coding"));
}

#[test]
fn test_default_mode_cannot_update_theme_style() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("default_mode_protect");
    init_marketplace().unwrap();

    assert!(update_mode_theme("default", Some("theme_coding_basics".to_string())).is_err());
    assert!(update_mode_style_pack("default", Some("coding_basics".to_string())).is_err());
}

#[test]
fn test_disabled_pack_cannot_be_theme_style_source() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("disabled_source_val");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "disabled_pack",
        "Disabled Pack",
        "1.0.0",
        mock_contents(false, false, true, true),
    );
    install_pack_from_path(src_dir).unwrap();

    create_mode(OpenNivaraMode {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "".to_string(),
        enabled_pack_ids: vec!["disabled_pack".to_string()],
        theme_id: None,
        style_pack_id: None,
    })
    .unwrap();

    // disable the pack
    enable_pack("disabled_pack", false).unwrap();

    // theme update fails because pack is disabled
    assert!(update_mode_theme("custom", Some("theme_disabled_pack".to_string())).is_err());

    // style update fails because pack is disabled
    assert!(update_mode_style_pack("custom", Some("disabled_pack".to_string())).is_err());
}

#[test]
fn test_addon_settings_defaults_and_toggle() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("addon_settings_defaults");
    init_marketplace().unwrap();

    let settings = super::addon_settings::read_addon_settings().unwrap();
    assert_eq!(settings.schema_version, 1);
    assert_eq!(settings.active_theme_id, None);
    assert_eq!(settings.active_theme_source_pack_id, None);
    assert!(settings.enabled_packs.is_empty());
    assert!(settings.disabled_contributions.is_empty());

    // Test toggles
    super::addon_settings::toggle_pack_enabled("mock_basics", true).unwrap();
    let settings = super::addon_settings::read_addon_settings().unwrap();
    assert!(settings.enabled_packs.contains(&"mock_basics".to_string()));

    super::addon_settings::toggle_contribution_enabled(
        "mock_basics",
        "preference",
        "mvp_architecture",
        false,
    )
    .unwrap();
    let settings = super::addon_settings::read_addon_settings().unwrap();
    assert!(settings
        .disabled_contributions
        .contains(&"mock_basics:preference:mvp_architecture".to_string()));

    super::addon_settings::toggle_contribution_enabled(
        "mock_basics",
        "preference",
        "mvp_architecture",
        true,
    )
    .unwrap();
    let settings = super::addon_settings::read_addon_settings().unwrap();
    assert!(!settings
        .disabled_contributions
        .contains(&"mock_basics:preference:mvp_architecture".to_string()));
}

#[test]
fn test_active_addon_theme_resolves() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("active_addon_theme");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "theme_pack",
        "Theme Pack",
        "1.0.0",
        mock_contents(false, false, false, true),
    );
    install_pack_from_path(src_dir).unwrap();

    // Set active theme in addon settings
    super::addon_settings::set_active_theme(
        Some("theme_theme_pack".to_string()),
        Some("theme_pack".to_string()),
    )
    .unwrap();

    let active_theme = super::themes::get_active_addon_theme().unwrap();
    assert!(active_theme.is_some());
    let theme = active_theme.unwrap();
    assert_eq!(theme.id, "theme_theme_pack");
    assert_eq!(theme.name, "Theme for Theme Pack");
    assert_eq!(theme.colors.background, "#1a1a1a");
}

#[test]
fn test_theme_store_lists_builtin_themes_without_behavior_content() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("theme_store_builtin_only");
    init_marketplace().unwrap();

    let custom_builtin = temp.join("custom_builtin");
    create_mock_pack(
        &custom_builtin,
        "coding_basics",
        "Coding Basics",
        "1.0.0",
        mock_contents(true, true, true, true),
    );
    create_mock_pack(
        &custom_builtin,
        "study_coach",
        "Study Coach",
        "1.0.0",
        mock_contents(true, true, true, true),
    );
    let _builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &custom_builtin);

    let items = super::themes::list_theme_store_items().unwrap();

    assert!(items.iter().any(|item| item.id == "theme_coding_basics"));
    assert!(items.iter().any(|item| item.id == "theme_study_coach"));
    assert!(items.iter().all(|item| item.safety.data_only));
    assert!(items
        .iter()
        .all(|item| !item.safety.contains_executable_code));
    assert!(items.iter().all(|item| !item.safety.modifies_tool_security));
}

#[test]
fn test_installing_theme_does_not_enable_pack_behavior_for_prompt() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("theme_no_prompt_behavior");
    init_marketplace().unwrap();

    let src_dir = create_mock_pack(
        &temp,
        "behavior_theme_pack",
        "Behavior Theme Pack",
        "1.0.0",
        mock_contents(true, true, true, true),
    );
    super::themes::install_theme_from_path(src_dir).unwrap();
    super::themes::apply_theme("theme_behavior_theme_pack").unwrap();

    let preview = crate::engine::OpenNivaraEngine::new()
        .preview_context_for_message("mvp architecture", None)
        .unwrap();

    assert!(preview.active_packs.is_empty());
    assert!(preview.active_mode.is_empty());
    assert_eq!(preview.style_source_pack, None);
    assert!(!preview
        .final_context_text
        .contains("Prefer lean MVP architecture"));
    assert!(!preview.final_context_text.contains("Mock Context"));
}

#[test]
fn test_preview_builtin_pack_returns_valid_spec() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (temp, _config_guard) = setup_test_env("preview_builtin");

    let custom_builtin = temp.join("custom_builtin");
    fs::create_dir_all(&custom_builtin).unwrap();
    create_mock_pack(
        &custom_builtin,
        "study_coach",
        "Study Coach",
        "1.0.0",
        mock_contents(true, true, false, true),
    );

    let _builtin_guard = EnvGuard::set("OPENNIVARA_BUILTIN_PACKS_DIR", &custom_builtin);

    let preview = super::builtin::preview_builtin_pack("study_coach").unwrap();
    assert_eq!(preview.manifest.id, "study_coach");
    assert_eq!(preview.manifest.name, "Study Coach");
    assert_eq!(preview.additions.preferences_count, 1);
    assert_eq!(preview.additions.contexts_count, 1);
    assert_eq!(preview.additions.themes_count, 1);
}

#[test]
fn test_migrate_modes_to_addons() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (_temp, _config_guard) = setup_test_env("migrate_modes");
    init_marketplace().unwrap();

    let modes_path = get_modes_path().unwrap();
    let legacy_modes = r#"
schema_version = 1
active_mode = "coding"

[[modes]]
id = "coding"
name = "Coding Mode"
description = "Study coding"
enabled_pack_ids = ["coding_basics"]
theme_id = "coding_cyan"
style_pack_id = "coding_basics"
"#;
    fs::write(&modes_path, legacy_modes).unwrap();

    assert!(super::addon_settings::has_legacy_modes_file());
    super::addon_settings::migrate_modes_to_addons().unwrap();

    let settings = super::addon_settings::read_addon_settings().unwrap();
    assert_eq!(settings.enabled_packs, vec!["coding_basics".to_string()]);
    assert_eq!(settings.active_theme_id, Some("coding_cyan".to_string()));

    // legacy modes should be renamed
    assert!(!modes_path.exists());
    assert!(modes_path.with_extension("toml.old").exists());
}
