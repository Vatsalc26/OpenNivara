use crate::workspace_map::config::MapConfig;
use crate::workspace_map::db::MapEntry;

/// Selects the emoji icon corresponding to the classified category.
pub fn get_icon_for_category(category: &str, config: &MapConfig) -> String {
    match category {
        "directory" => config.legend.directory.clone(),
        "rust" => config.legend.rust.clone(),
        "config" => config.legend.config.clone(),
        "document" => config.legend.document.clone(),
        "image" => config.legend.image.clone(),
        "pdf" => config.legend.pdf.clone(),
        "data" => config.legend.data.clone(),
        "blocked" => config.legend.blocked.clone(),
        "ignored" => config.legend.ignored.clone(),
        _ => config.legend.unknown.clone(),
    }
}

/// Helper function to check if this entry is the final sibling in its directory.
/// Performs a lookahead scan over the sorted remaining nodes.
fn is_last_sibling(entry: &MapEntry, remaining: &[MapEntry]) -> bool {
    let parent = &entry.parent_path;
    for next in remaining {
        if &next.parent_path == parent {
            return false; // Found another node under the same parent folder
        }
    }
    true
}

/// Formats a list of map entries into a compact, beautiful CLI directory tree
/// utilizing standard legend emojis and branching graphics.
pub fn format_tree_string(entries: &[MapEntry], config: &MapConfig) -> String {
    if entries.is_empty() {
        return "Workspace map is empty. Please run `opennivara map-scan` first.".to_string();
    }

    let mut output = String::new();
    output.push_str("Legend:\n");
    output.push_str(&format!("  {} directory\n", config.legend.directory));
    output.push_str(&format!("  {} rust\n", config.legend.rust));
    output.push_str(&format!("  {} config\n", config.legend.config));
    output.push_str(&format!("  {} document\n", config.legend.document));
    output.push_str(&format!("  {} image\n", config.legend.image));
    output.push_str(&format!("  {} pdf\n", config.legend.pdf));
    output.push_str(&format!("  {} data\n", config.legend.data));
    output.push_str(&format!("  {} blocked\n", config.legend.blocked));
    output.push_str(&format!("  {} ignored\n\n", config.legend.ignored));

    for (i, entry) in entries.iter().enumerate() {
        let depth = entry.depth as usize;
        let icon = get_icon_for_category(&entry.category, config);

        if depth == 0 {
            output.push_str(&format!("{} {}\n", icon, entry.name));
            continue;
        }

        // Check lookahead list to see if this is the last sibling
        let is_last = is_last_sibling(entry, &entries[i + 1..]);

        let indent = "    ".repeat(depth - 1);
        let branch = if is_last { "└── " } else { "├── " };

        output.push_str(&format!("{}{}{} {}\n", indent, branch, icon, entry.name));
    }

    output
}
