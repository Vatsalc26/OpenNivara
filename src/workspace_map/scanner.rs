use crate::workspace_map::classifier::classify_entry;
use crate::workspace_map::config::MapConfig;
use crate::workspace_map::db::MapEntry;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Computes a secure, fast, and content-independent SHA-256 metadata hash
/// using path name, file size, and modification timestamp.
fn compute_metadata_hash(path: &str, size: u64, modified: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    hasher.update(size.to_be_bytes());
    hasher.update(modified.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Recursively scans allowed roots using standard configurations.
/// Extracts metadata for SQLite indexing and strictly avoids reading file contents.
pub fn scan_allowed_roots(config: &MapConfig) -> anyhow::Result<Vec<MapEntry>> {
    // 1. Build blocked patterns globset
    let mut glob_builder = GlobSetBuilder::new();
    for pattern in &config.ignore.blocked_globs {
        let glob = Glob::new(pattern)
            .map_err(|e| anyhow::anyhow!("Invalid blocked glob pattern '{}': {}", pattern, e))?;
        glob_builder.add(glob);
    }
    let blocked_set = glob_builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build blocked globset: {}", e))?;

    let mut scanned_entries = Vec::new();
    let max_files = config.general.max_files as usize;
    let max_depth = config.general.max_depth as usize;
    let max_size = config.general.max_file_size_bytes;

    for root in &config.roots.allowed_roots {
        // Resolve absolute path and clean traverse components
        let root_path = if Path::new(root).is_absolute() {
            Path::new(root).to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        let canonical_root = crate::tools::clean_path(&root_path);

        // 2. Setup the walk parameters using WalkBuilder
        let mut walker_builder = WalkBuilder::new(&canonical_root);
        walker_builder
            .max_depth(Some(max_depth))
            .follow_links(config.general.follow_symlinks)
            .git_ignore(config.general.respect_gitignore)
            .hidden(config.general.skip_hidden);

        let walker = walker_builder.build();

        for walk_res in walker {
            let entry = match walk_res {
                Ok(e) => e,
                Err(_) => continue, // Skip unreadable nodes gracefully
            };

            let entry_path = entry.path();
            let relative_path = match entry_path.strip_prefix(&canonical_root) {
                Ok(p) => {
                    let s = p.to_string_lossy().to_string().replace('\\', "/");
                    if s.is_empty() {
                        ".".to_string()
                    } else if s.starts_with('.') {
                        s
                    } else {
                        format!("./{}", s)
                    }
                }
                Err(_) => continue,
            };

            // Enforce skip rules on paths matching blocked patterns
            let is_blocked =
                blocked_set.is_match(&relative_path) || blocked_set.is_match(entry_path);

            let metadata = entry.metadata().ok();
            let size_bytes = metadata.as_ref().map(|m| m.len());
            let modified_at = metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339());

            let is_symlink = metadata.as_ref().map(|m| m.is_symlink()).unwrap_or(false);
            let is_hidden = entry.file_name().to_string_lossy().starts_with('.');
            let is_dir = entry_path.is_dir();

            // 3. Categorize file using the Classifier
            let (mut category, mime_type) = if is_blocked {
                (
                    "blocked".to_string(),
                    Some("application/restricted".to_string()),
                )
            } else {
                classify_entry(entry_path, is_dir)
            };

            // Skip hashing/indexing if size exceeds bounds
            let mut is_ignored = false;
            if let Some(sz) = size_bytes {
                if !is_dir && sz > max_size {
                    is_ignored = true;
                    category = "ignored".to_string();
                }
            }

            // Exclude files not listed under allowed extensions
            if !is_dir && !is_blocked && !is_ignored {
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if !config.include.extensions.contains(&ext_lower) {
                        is_ignored = true;
                        category = "ignored".to_string();
                    }
                } else {
                    is_ignored = true;
                    category = "ignored".to_string();
                }
            }

            // Calculate parent path for hierarchical rendering
            let parent_path = if relative_path == "." {
                None
            } else {
                let p = Path::new(&relative_path);
                p.parent().map(|parent| {
                    let s = parent.to_string_lossy().to_string().replace('\\', "/");
                    if s.is_empty() {
                        ".".to_string()
                    } else {
                        s
                    }
                })
            };

            // Compute metadata hash
            let hash_val = if !is_dir && !is_blocked && !is_ignored {
                let size_val = size_bytes.unwrap_or(0);
                let mod_val = modified_at.clone().unwrap_or_default();
                Some(compute_metadata_hash(&relative_path, size_val, &mod_val))
            } else {
                None
            };

            let name = entry.file_name().to_string_lossy().to_string();

            let map_entry = MapEntry {
                path: relative_path,
                name,
                parent_path,
                kind: if is_dir {
                    "dir".to_string()
                } else {
                    "file".to_string()
                },
                category,
                extension: entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase()),
                mime_type,
                size_bytes,
                modified_at,
                depth: entry.depth() as u32,
                is_hidden,
                is_blocked,
                is_ignored,
                is_symlink,
                hash: hash_val,
            };

            scanned_entries.push(map_entry);

            // Halt if max limit is reached
            if scanned_entries.len() >= max_files {
                break;
            }
        }
    }

    Ok(scanned_entries)
}
