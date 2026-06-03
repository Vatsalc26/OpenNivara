use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinPackSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub description: String,
    pub risk_level: String,
}

fn builtin_candidates_for_exe_dir(exe_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![
        // Observed Tauri release layout for ../../packs/builtin/**/* resources.
        exe_dir
            .join("_up_")
            .join("_up_")
            .join("packs")
            .join("builtin"),
        exe_dir
            .join("resources")
            .join("_up_")
            .join("_up_")
            .join("packs")
            .join("builtin"),
        exe_dir.join("resources").join("packs").join("builtin"),
        exe_dir.join("packs").join("builtin"),
    ];
    if let Some(parent) = exe_dir.parent() {
        candidates.push(parent.join("packs").join("builtin"));
    }
    candidates
}

/// Dynamically locates the built-in packs directory in development or production.
pub fn get_builtin_packs_dir() -> anyhow::Result<PathBuf> {
    // 1. Try OPENNIVARA_BUILTIN_PACKS_DIR environment variable (Option A)
    if let Ok(env_path_str) = std::env::var("OPENNIVARA_BUILTIN_PACKS_DIR") {
        let env_path = PathBuf::from(env_path_str);
        if env_path.exists() {
            return Ok(env_path);
        }
        return Err(anyhow::anyhow!(
            "Built-in packs were not found at OPENNIVARA_BUILTIN_PACKS_DIR: {}",
            env_path.display()
        ));
    }

    // 2. Try relative to current executable (release mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            for candidate in builtin_candidates_for_exe_dir(exe_dir) {
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
    }

    // 3. Try manifest directory (dev mode)
    let dev_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("packs")
        .join("builtin");
    if dev_dir.exists() {
        return Ok(dev_dir);
    }

    Err(anyhow::anyhow!(
        "Built-in packs were not found. Use Import Local Pack."
    ))
}

/// Lists all built-in packs that are discoverable.
pub fn list_builtin_packs() -> anyhow::Result<Vec<BuiltinPackSummary>> {
    let builtin_dir = get_builtin_packs_dir()?;
    let mut summaries = Vec::new();

    if builtin_dir.exists() {
        for entry in fs::read_dir(builtin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("pack.toml");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = toml::from_str::<super::packs::PackManifest>(&content)
                        {
                            summaries.push(BuiltinPackSummary {
                                id: manifest.id,
                                name: manifest.name,
                                version: manifest.version,
                                author: manifest.author,
                                category: manifest.category,
                                description: manifest.description,
                                risk_level: manifest.safety.risk_level,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(summaries)
}

/// Installs a specific built-in pack by ID.
pub fn install_builtin_pack(pack_id: &str) -> anyhow::Result<super::packs::InstalledPack> {
    let builtin_dir = get_builtin_packs_dir()?;
    let pack_folder = builtin_dir.join(pack_id);
    if !pack_folder.exists() {
        return Err(anyhow::anyhow!(
            "Built-in pack '{}' not found at expected path: {:?}",
            pack_id,
            pack_folder
        ));
    }
    super::packs::install_pack_from_path(pack_folder)
}

/// Previews a specific built-in pack by ID.
pub fn preview_builtin_pack(pack_id: &str) -> anyhow::Result<super::packs::PackPreview> {
    let builtin_dir = get_builtin_packs_dir()?;
    let pack_folder = builtin_dir.join(pack_id);
    if !pack_folder.exists() {
        return Err(anyhow::anyhow!(
            "Built-in pack '{}' not found at expected path: {:?}",
            pack_id,
            pack_folder
        ));
    }
    super::packs::preview_pack_from_path(pack_folder)
}

#[cfg(test)]
mod tests {
    use super::builtin_candidates_for_exe_dir;
    use std::path::Path;

    #[test]
    fn release_candidates_include_observed_tauri_up_layout() {
        let exe_dir = Path::new(r"D:\app\target\release");
        let expected = exe_dir
            .join("_up_")
            .join("_up_")
            .join("packs")
            .join("builtin");
        assert!(builtin_candidates_for_exe_dir(exe_dir).contains(&expected));
    }
}
