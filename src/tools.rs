use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tool_operation_policy::{OperationDecision, OperationKind};

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ToolPreviewEnvelope {
    pub schema_version: u32,
    pub tool_name: String,
    pub preview_kind: String,
    pub operation_target: Option<String>,
    pub summary: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[specta(rename_all = "snake_case")]
pub enum ToolExecutionStatus {
    Succeeded,
    Failed,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ToolOutputTruncation {
    pub original_bytes: u32,
    pub returned_bytes: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub tool_call_id: String,
    pub status: ToolExecutionStatus,
    pub result_json: Option<serde_json::Value>,
    pub result_summary: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub truncation: Option<ToolOutputTruncation>,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ModelVisibleToolError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct ModelVisibleToolResult {
    pub ok: bool,
    pub tool_name: String,
    pub tool_call_id: String,
    pub summary: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<ModelVisibleToolError>,
    pub metadata: Option<serde_json::Value>,
}

/// High-level struct representing the tools configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolsConfig {
    pub general: GeneralConfig,
    pub paths: PathsConfig,
    pub tools: std::collections::HashMap<String, ToolSettings>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub enabled: bool,
    pub max_tool_rounds: u32,
    pub show_tool_activity: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathsConfig {
    pub allowed_roots: Vec<String>,
    pub blocked_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolSettings {
    pub enabled: bool,
    pub requires_confirmation: bool,
    #[serde(default)]
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ToolRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub risk_level: ToolRisk,
    pub operation_kind: OperationKind,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ToolSource {
    Cli,
    Telegram,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolContext {
    pub source: ToolSource,
}

pub trait ToolHandler {
    fn definition(&self) -> ToolDefinition;
    fn execute(&self, args: &serde_json::Value, config: &ToolsConfig) -> serde_json::Value;
}

#[derive(Debug, Clone)]
pub struct ToolRegistry {
    has_map_db: bool,
}

impl ToolRegistry {
    pub fn new(has_map_db: bool) -> Self {
        Self { has_map_db }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut definitions = vec![
            ToolDefinition {
                name: "get_current_dir".to_string(),
                description: "Get the current working directory where OpenNivara is running."
                    .to_string(),
                parameters: serde_json::json!({
                    "type": "OBJECT",
                    "properties": {}
                }),
                risk_level: ToolRisk::Low,
                operation_kind: OperationKind::ReadOnly,
            },
            ToolDefinition {
                name: "list_dir".to_string(),
                description:
                    "List files and folders in a safe allowed local directory. Non-recursive."
                        .to_string(),
                parameters: serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "path": {
                            "type": "STRING",
                            "description": "Relative path to list, such as '.' or 'src'"
                        }
                    },
                    "required": ["path"]
                }),
                risk_level: ToolRisk::Low,
                operation_kind: OperationKind::ReadOnly,
            },
            ToolDefinition {
                name: "file_exists".to_string(),
                description: "Check whether a safe allowed local path exists.".to_string(),
                parameters: serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "path": {
                            "type": "STRING",
                            "description": "Relative path such as 'Cargo.toml' or 'src/main.rs'"
                        }
                    },
                    "required": ["path"]
                }),
                risk_level: ToolRisk::Low,
                operation_kind: OperationKind::ReadOnly,
            },
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read a safe allowed UTF-8 text file from the local project."
                    .to_string(),
                parameters: serde_json::json!({
                    "type": "OBJECT",
                    "properties": {
                        "path": {
                            "type": "STRING",
                            "description": "Relative path such as 'Cargo.toml' or 'src/main.rs'"
                        }
                    },
                    "required": ["path"]
                }),
                risk_level: ToolRisk::Medium,
                operation_kind: OperationKind::ReadOnly,
            },
        ];

        if self.has_map_db {
            definitions.extend([
                ToolDefinition {
                    name: "map_summary".to_string(),
                    description: "Get a high-level summary of the workspace map database, including category breakdown counts.".to_string(),
                    parameters: serde_json::json!({
                        "type": "OBJECT",
                        "properties": {}
                    }),
                    risk_level: ToolRisk::Low,
                    operation_kind: OperationKind::ReadOnly,
                },
                ToolDefinition {
                    name: "map_tree".to_string(),
                    description: "Retrieve a visual tree diagram representing the workspace directory structure, filtered optionally by maximum depth.".to_string(),
                    parameters: serde_json::json!({
                        "type": "OBJECT",
                        "properties": {
                            "depth": {
                                "type": "INTEGER",
                                "description": "Optional depth limit (e.g. 2 to see top levels)"
                            }
                        }
                    }),
                    risk_level: ToolRisk::Low,
                    operation_kind: OperationKind::ReadOnly,
                },
                ToolDefinition {
                    name: "map_search".to_string(),
                    description: "Search the workspace map database for files matching a query on name, path, extension, or category.".to_string(),
                    parameters: serde_json::json!({
                        "type": "OBJECT",
                        "properties": {
                            "query": {
                                "type": "STRING",
                                "description": "The search term (e.g. 'main', 'Cargo', 'rust')"
                            }
                        },
                        "required": ["query"]
                    }),
                    risk_level: ToolRisk::Low,
                    operation_kind: OperationKind::ReadOnly,
                },
                ToolDefinition {
                    name: "map_get_node".to_string(),
                    description: "Retrieve detailed metadata details for a single target path in the workspace map database.".to_string(),
                    parameters: serde_json::json!({
                        "type": "OBJECT",
                        "properties": {
                            "path": {
                                "type": "STRING",
                                "description": "The relative path (e.g. 'Cargo.toml' or 'src/main.rs')"
                            }
                        },
                        "required": ["path"]
                    }),
                    risk_level: ToolRisk::Low,
                    operation_kind: OperationKind::ReadOnly,
                },
            ]);
        }

        definitions
    }

    pub fn definition(&self, name: &str) -> Option<ToolDefinition> {
        self.definitions()
            .into_iter()
            .find(|definition| definition.name == name)
    }

    pub fn declared_definitions(
        &self,
        config: &ToolsConfig,
        allowed_tool_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<ToolDefinition> {
        self.definitions()
            .into_iter()
            .filter(|definition| {
                allowed_tool_names
                    .map(|allowed| allowed.contains(&definition.name))
                    .unwrap_or(true)
            })
            .filter(|definition| self.is_enabled_for_declaration(&definition.name, config))
            .collect()
    }

    pub fn declared_tool_names(
        &self,
        config: &ToolsConfig,
        allowed_tool_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<String> {
        self.declared_definitions(config, allowed_tool_names)
            .into_iter()
            .map(|definition| definition.name)
            .collect()
    }

    pub fn execute(
        &self,
        name: &str,
        args: &serde_json::Value,
        config: &ToolsConfig,
    ) -> serde_json::Value {
        if self.definition(name).is_none() {
            return serde_json::json!({
                "error": format!("Tool '{}' is not recognized by OpenNivara.", name)
            });
        }

        let settings = self.settings_for(name, config);
        if !settings.enabled {
            return serde_json::json!({
                "error": format!("Tool '{}' is currently disabled in tools.toml.", name)
            });
        }

        match name {
            "get_current_dir" => match std::env::current_dir() {
                Ok(path) => {
                    serde_json::json!({ "current_dir": path.to_string_lossy().to_string() })
                }
                Err(e) => {
                    serde_json::json!({ "error": format!("Failed to get current directory: {}", e) })
                }
            },
            "list_dir" => execute_list_dir(args, config),
            "file_exists" => execute_file_exists(args, config),
            "read_file" => execute_read_file(args, config, &settings),
            "map_summary" => {
                if let Ok(map_config) = crate::workspace_map::config::read_map() {
                    crate::workspace_map::tool_map_summary(&map_config.database.filename)
                } else {
                    serde_json::json!({ "error": "Map configuration is not loaded." })
                }
            }
            "map_tree" => {
                let depth = args.get("depth").and_then(|v| v.as_u64()).map(|d| d as u32);
                if let Ok(map_config) = crate::workspace_map::config::read_map() {
                    crate::workspace_map::tool_map_tree(&map_config.database.filename, depth)
                } else {
                    serde_json::json!({ "error": "Map configuration is not loaded." })
                }
            }
            "map_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                if let Ok(map_config) = crate::workspace_map::config::read_map() {
                    crate::workspace_map::tool_map_search(&map_config.database.filename, query)
                } else {
                    serde_json::json!({ "error": "Map configuration is not loaded." })
                }
            }
            "map_get_node" => {
                let path_param = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if let Ok(map_config) = crate::workspace_map::config::read_map() {
                    crate::workspace_map::tool_map_get_node(
                        &map_config.database.filename,
                        path_param,
                    )
                } else {
                    serde_json::json!({ "error": "Map configuration is not loaded." })
                }
            }
            _ => serde_json::json!({
                "error": format!("Tool '{}' is currently recognized in config but not yet executable.", name)
            }),
        }
    }

    pub fn classify_tool_call(&self, name: &str, args: &serde_json::Value) -> OperationDecision {
        self.definition(name).map_or_else(
            || crate::tool_operation_policy::classify_unknown_tool(name),
            |definition| {
                crate::tool_operation_policy::classify_tool_operation(
                    &definition.name,
                    definition.operation_kind,
                    args,
                )
            },
        )
    }

    fn settings_for(&self, name: &str, config: &ToolsConfig) -> ToolSettings {
        config
            .tools
            .get(name)
            .cloned()
            .unwrap_or_else(|| ToolSettings {
                enabled: Self::is_map_tool(name),
                requires_confirmation: false,
                max_bytes: None,
            })
    }

    fn is_enabled_for_declaration(&self, name: &str, config: &ToolsConfig) -> bool {
        self.settings_for(name, config).enabled
    }

    pub fn is_map_tool(name: &str) -> bool {
        matches!(
            name,
            "map_summary" | "map_tree" | "map_search" | "map_get_node"
        )
    }
}

/// Helper function to clean and normalize a path by resolving ".." and "." components.
/// This prevents path traversal vulnerabilities (e.g. "../secret") without accessing the filesystem.
pub fn clean_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(c) => {
                out.push(c);
            }
            Component::CurDir => {}
            Component::RootDir => {
                out.push("/");
            }
            Component::Prefix(_) => {
                out.push(comp.as_os_str());
            }
        }
    }
    out
}

/// Resolves a path, checks it against allowed roots and blocked keywords,
/// and returns the fully normalized canonical absolute path if safe.
pub fn validate_and_resolve_path(
    user_path: &str,
    allowed_roots: &[String],
    blocked_patterns: &[String],
) -> anyhow::Result<PathBuf> {
    let raw_path = Path::new(user_path);

    // 1. Resolve relative path against CWD
    let absolute_path = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(raw_path)
    };

    // 2. Clean parent directory traversals
    let cleaned_path = clean_path(&absolute_path);

    // 3. Prevent reading blocked patterns (case-insensitive checks)
    let path_str = cleaned_path.to_string_lossy().to_lowercase();
    for pattern in blocked_patterns {
        if path_str.contains(&pattern.to_lowercase()) {
            return Err(anyhow::anyhow!(
                "Path access blocked by safety rule: path contains blocked pattern '{}'",
                pattern
            ));
        }
    }

    // 4. Verify that the path resides within at least one allowed root.
    // Empty allowed roots are intentionally unrestricted for local-first read-only tools.
    let mut is_allowed = allowed_roots.is_empty();
    for root in allowed_roots {
        let root_path = if Path::new(root).is_absolute() {
            Path::new(root).to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        let canonical_root = clean_path(&root_path);

        if cleaned_path.starts_with(&canonical_root) {
            is_allowed = true;
            break;
        }
    }

    if !is_allowed {
        return Err(anyhow::anyhow!(
            "Path access blocked by safety rule: path is outside allowed directories."
        ));
    }

    Ok(cleaned_path)
}

pub const DEFAULT_TOOLS_TOML: &str = r#"[general]
enabled = true
max_tool_rounds = 3
show_tool_activity = true

[paths]
allowed_roots = []
blocked_patterns = []

[tools.get_current_dir]
enabled = true
requires_confirmation = false

[tools.list_dir]
enabled = true
requires_confirmation = false

[tools.file_exists]
enabled = true
requires_confirmation = false

[tools.read_file]
enabled = true
requires_confirmation = false
max_bytes = 20000
"#;

/// Finds the OS-specific path where the tools.toml should reside.
pub fn get_tools_path() -> anyhow::Result<PathBuf> {
    Ok(crate::config_paths::config_dir()?.join("tools.toml"))
}

/// Initializes tools.toml in the standard config path.
/// Does not overwrite if the file already exists.
pub fn init_tools() -> anyhow::Result<String> {
    let path = get_tools_path()?;

    if path.exists() {
        return Ok(format!(
            "Tools configuration file already exists at:\n  {}\n\nYou can edit it directly with any text editor.",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create tools parent directories: {}", e))?;
    }

    fs::write(&path, DEFAULT_TOOLS_TOML)
        .map_err(|e| anyhow::anyhow!("Failed to write default tools.toml: {}", e))?;

    Ok(format!(
        "Successfully initialized your OpenNivara tools configuration at:\n  {}",
        path.display()
    ))
}

/// Reads and parses the tools configuration file.
pub fn read_tools() -> anyhow::Result<ToolsConfig> {
    let path = get_tools_path()?;

    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Tools configuration file not found at:\n  {}\n\nPlease run: opennivara init-tools",
            path.display()
        ));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read tools.toml: {}", e))?;

    let config: ToolsConfig = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!(
            "Failed to parse tools TOML. Please ensure your tools file is formatted correctly.\nError: {}",
            e
        ))?;

    Ok(config)
}

fn execute_list_dir(args: &serde_json::Value, config: &ToolsConfig) -> serde_json::Value {
    let path_arg = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    match validate_and_resolve_path(
        path_arg,
        &config.paths.allowed_roots,
        &config.paths.blocked_patterns,
    ) {
        Ok(resolved_path) => match fs::read_dir(&resolved_path) {
            Ok(entries) => {
                let mut items = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    let mut item_name = entry.file_name().to_string_lossy().to_string();
                    if path.is_dir() {
                        item_name.push('/');
                    }
                    items.push(item_name);
                }
                items.sort();
                serde_json::json!({ "files": items })
            }
            Err(e) => serde_json::json!({ "error": format!("Failed to list directory: {}", e) }),
        },
        Err(e) => {
            serde_json::json!({ "error": format!("Tool call blocked by OpenNivara safety policy: {}", e) })
        }
    }
}

fn execute_file_exists(args: &serde_json::Value, config: &ToolsConfig) -> serde_json::Value {
    let path_arg = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return serde_json::json!({ "error": "Missing 'path' parameter." }),
    };

    match validate_and_resolve_path(
        path_arg,
        &config.paths.allowed_roots,
        &config.paths.blocked_patterns,
    ) {
        Ok(resolved_path) => {
            let exists = resolved_path.exists();
            let is_file = resolved_path.is_file();
            let is_dir = resolved_path.is_dir();
            serde_json::json!({
                "exists": exists,
                "is_file": is_file,
                "is_dir": is_dir
            })
        }
        Err(e) => {
            serde_json::json!({ "error": format!("Tool call blocked by OpenNivara safety policy: {}", e) })
        }
    }
}

fn execute_read_file(
    args: &serde_json::Value,
    config: &ToolsConfig,
    settings: &ToolSettings,
) -> serde_json::Value {
    let path_arg = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return serde_json::json!({ "error": "Missing 'path' parameter." }),
    };

    match validate_and_resolve_path(
        path_arg,
        &config.paths.allowed_roots,
        &config.paths.blocked_patterns,
    ) {
        Ok(resolved_path) => {
            if !resolved_path.exists() {
                return serde_json::json!({ "error": "File does not exist." });
            }
            if !resolved_path.is_file() {
                return serde_json::json!({ "error": "Path exists but is not a file." });
            }

            match fs::read(&resolved_path) {
                Ok(bytes) => {
                    let max_bytes = settings.max_bytes.unwrap_or(20000);
                    let truncated = bytes.len() > max_bytes;
                    let slice = if truncated {
                        &bytes[..max_bytes]
                    } else {
                        &bytes
                    };

                    match String::from_utf8(slice.to_vec()) {
                        Ok(mut text) => {
                            if truncated {
                                text.push_str("\n\n[WARNING: File truncated because it exceeded max_bytes limit.]");
                            }
                            serde_json::json!({
                                "content": text,
                                "truncated": truncated,
                                "bytes_read": slice.len()
                            })
                        }
                        Err(_) => {
                            serde_json::json!({ "error": "File contains non-UTF-8 bytes. OpenNivara only reads text files." })
                        }
                    }
                }
                Err(e) => serde_json::json!({ "error": format!("Failed to read file: {}", e) }),
            }
        }
        Err(e) => {
            serde_json::json!({ "error": format!("Tool call blocked by OpenNivara safety policy: {}", e) })
        }
    }
}

/// Executes a tool locally inside a Rust safety shell and returns the result as JSON.
pub fn execute_tool(
    name: &str,
    args: &serde_json::Value,
    config: &ToolsConfig,
) -> serde_json::Value {
    ToolRegistry::new(true).execute(name, args, config)
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    use std::collections::HashMap;

    fn config_with_tool_enabled(name: &str, enabled: bool) -> ToolsConfig {
        let mut tools = HashMap::new();
        for tool_name in [
            "get_current_dir",
            "list_dir",
            "file_exists",
            "read_file",
            "map_summary",
            "map_tree",
            "map_search",
            "map_get_node",
        ] {
            tools.insert(
                tool_name.to_string(),
                ToolSettings {
                    enabled: true,
                    requires_confirmation: false,
                    max_bytes: None,
                },
            );
        }
        tools.insert(
            name.to_string(),
            ToolSettings {
                enabled,
                requires_confirmation: false,
                max_bytes: None,
            },
        );
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
    fn registry_declarations_match_expected_tool_names_without_map() {
        let registry = ToolRegistry::new(false);
        let names =
            registry.declared_tool_names(&config_with_tool_enabled("read_file", true), None);

        assert!(names.contains(&"get_current_dir".to_string()));
        assert!(names.contains(&"list_dir".to_string()));
        assert!(names.contains(&"file_exists".to_string()));
        assert!(names.contains(&"read_file".to_string()));
        assert!(!names.contains(&"map_summary".to_string()));
    }

    #[test]
    fn registry_declares_map_tools_only_when_map_database_exists() {
        let config = config_with_tool_enabled("map_summary", true);

        let without_map = ToolRegistry::new(false).declared_tool_names(&config, None);
        let with_map = ToolRegistry::new(true).declared_tool_names(&config, None);

        assert!(!without_map.contains(&"map_summary".to_string()));
        assert!(with_map.contains(&"map_summary".to_string()));
    }

    #[test]
    fn disabled_tools_are_not_executable() {
        let config = config_with_tool_enabled("get_current_dir", false);
        let result =
            ToolRegistry::new(false).execute("get_current_dir", &serde_json::json!({}), &config);

        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("currently disabled"));
    }

    #[test]
    fn unknown_tools_return_safe_errors() {
        let config = config_with_tool_enabled("get_current_dir", true);
        let result =
            ToolRegistry::new(false).execute("missing_tool", &serde_json::json!({}), &config);

        assert!(result["error"].as_str().unwrap().contains("not recognized"));
    }
}
