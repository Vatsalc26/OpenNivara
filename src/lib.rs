#![allow(dead_code)]

pub mod bindings;
pub mod config_paths;
pub mod config_store;
pub mod context;
pub mod context_selector;
pub mod daemon;
pub mod engine;
pub mod error;
pub mod first_run;
pub mod llm;
pub mod marketplace;
pub mod memory;
pub mod output;
pub mod preferences;
pub mod profile;
pub mod prompt;
pub mod remote_policy;
pub mod runtime;
pub mod secrets;
pub mod service;
pub mod sessions;
pub mod skills;
pub mod state;
pub mod style;
pub mod telegram;
pub mod tools;
pub mod workspace_map;

pub fn load_env() {
    // 1. Try standard dotenv lookup from current working directory or ancestors
    let _ = dotenvy::dotenv();

    // 2. Try current working directory specifically
    if std::env::var("GEMINI_API_KEY").is_err() {
        if let Ok(cwd) = std::env::current_dir() {
            let path = cwd.join(".env");
            let _ = dotenvy::from_path(path);
        }
    }

    // 3. Try current executable directory and parent of current executable directory
    if std::env::var("GEMINI_API_KEY").is_err() {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let path = exe_dir.join(".env");
                let _ = dotenvy::from_path(&path);

                if std::env::var("GEMINI_API_KEY").is_err() {
                    if let Some(parent_dir) = exe_dir.parent() {
                        let path = parent_dir.join(".env");
                        let _ = dotenvy::from_path(path);
                    }
                }
            }
        }
    }

    // 4. Try root crate directory via CARGO_MANIFEST_DIR (dev/test environment fallback)
    if std::env::var("GEMINI_API_KEY").is_err() {
        let root_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        let _ = dotenvy::from_path(root_env);
    }
}

#[cfg(test)]
mod first_run_contract_tests {
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

    #[test]
    #[serial]
    fn clean_first_run_initializes_required_state_without_demo_data_or_packs() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().expect("temp dir");
        let _guard = EnvGuard::new(temp.path());

        let before = crate::first_run::first_run_status().expect("status before");
        assert!(before.is_first_run);
        assert!(!before.required_state_ready);

        let after = crate::first_run::initialize_clean_first_run(crate::first_run::FirstRunInput {
            accepted_alpha_notice: true,
            gemini_api_key: None,
        })
        .expect("initialize clean first run");

        assert!(!after.is_first_run);
        assert!(after.required_state_ready);

        let contexts = crate::context::read_contexts().expect("contexts");
        assert!(contexts.contexts.is_empty());
        let profile = crate::profile::read_profile().expect("profile");
        assert_eq!(profile.identity.display_name, "User");
        assert!(!profile.privacy.send_technical);
        assert_eq!(
            crate::skills::registry::read_enabled_skills()
                .unwrap()
                .skills
                .len(),
            0
        );
        assert_eq!(
            crate::marketplace::packs::list_installed_packs()
                .unwrap()
                .installed
                .len(),
            0
        );
    }

    #[test]
    #[serial]
    fn saved_local_gemini_key_is_detected_without_env_file() {
        let _lock = crate::config_paths::TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = tempfile::tempdir().expect("temp dir");
        let _guard = EnvGuard::new(temp.path());

        assert!(!crate::secrets::gemini_key_status().unwrap().available);

        crate::secrets::save_gemini_key("local-test-key").expect("save key");
        let status = crate::secrets::gemini_key_status().expect("key status");

        assert!(status.available);
        assert_eq!(status.source.as_deref(), Some("local_config"));
        assert_eq!(
            crate::secrets::get_gemini_api_key().unwrap(),
            "local-test-key"
        );
    }
}
