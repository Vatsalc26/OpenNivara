#![allow(dead_code)]

pub mod bindings;
pub mod config_paths;
pub mod config_store;
pub mod context;
pub mod context_selector;
pub mod daemon;
pub mod engine;
pub mod llm;
pub mod marketplace;
pub mod memory;
pub mod output;
pub mod preferences;
pub mod profile;
pub mod prompt;
pub mod remote_policy;
pub mod runtime;
pub mod service;
pub mod sessions;
pub mod skills;
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
