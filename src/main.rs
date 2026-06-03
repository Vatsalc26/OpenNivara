#![allow(dead_code)]

mod config_paths;
mod config_store;
mod context;
mod context_selector;
mod daemon;
mod engine;
mod llm;
mod marketplace;
mod memory;
mod output;
mod preferences;
mod profile;
mod remote_policy;
mod runtime;
mod service;
mod sessions;
mod skills;
mod style;
mod telegram;
mod tools;
mod workspace_map;

use clap::{Parser, Subcommand};

/// Command-Line interface definition using clap's derive features.
/// This will automatically generate help menus and parse arguments.
#[derive(Parser)]
#[command(name = "opennivara")]
#[command(author = "Vatsal Chavda")]
#[command(version = "0.1.0")]
#[command(about = "OpenNivara: a local-first personal AI agent powered by Gemini", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// The set of supported subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new OpenNivara profile containing your details and preferences
    InitProfile,

    /// Show the exact file path where your configuration profile is located
    ProfilePath,

    /// Initialize a new topic-based private preferences file
    InitPreferences,

    /// Show the exact file path where your private preferences file is located
    PreferencesPath,

    /// Initialize a new OpenNivara style configuration file
    InitStyle,

    /// Show the exact file path where your style preferences file is located
    StylePath,

    /// Initialize a new local tools configuration file
    InitTools,

    /// Show the exact file path where your local tools config is located
    ToolsPath,

    /// Display the list of recognized local tools and their enabled/disabled status
    ToolsList,

    /// Test a specific local tool directly in the shell without calling Gemini
    ToolTest {
        #[command(subcommand)]
        command: ToolTestCommand,
    },

    /// Initialize a new Workspace Map configuration file
    InitMap,

    /// Show the exact file path where your map configuration map.toml is located
    MapPath,

    /// Show the exact file path where your workspace map SQLite file is located
    MapDbPath,

    /// deterministic recursively scan allowed workspace roots and store metadata to SQLite
    MapScan,

    /// Print a useful high-level summary of workspace file counts and categories
    MapSummary,

    /// Render a compact directory tree of the mapped workspace with emoji icons
    MapTree {
        /// Optional maximum depth limit (e.g. 2 to see top-level folders)
        #[arg(long)]
        depth: Option<u32>,
    },

    /// Search for files in the database matching a search term in path/name/category
    MapSearch {
        /// The query search keyword
        query: String,
    },

    /// Inspect and show full metadata fields for a single target path
    MapInfo {
        /// Relative path of file/directory (e.g. Cargo.toml)
        path: String,
    },

    /// Run experimental live directory watching to update maps
    #[cfg(feature = "map-watch")]
    MapWatch,

    /// Extract syntax code symbols (functions, structs, enums) from a Rust file
    #[cfg(feature = "code-symbols")]
    MapSymbols {
        /// Relative path to target Rust file
        path: String,
    },

    /// View what context OpenNivara would send to Gemini for a given question
    DebugContext {
        /// The query/question to test trigger matching on
        question: String,
    },

    /// Ask OpenNivara a question. It will respond tailoring answers to your profile
    Ask {
        /// The query/question you'd like to ask
        question: String,
    },

    /// Initialize a new OpenNivara Telegram configuration
    InitTelegram,

    /// Show the exact file path where your telegram.toml is located
    TelegramPath,

    /// Show the exact file path where your state SQLite database is located
    StateDbPath,

    /// Initialize a new OpenNivara contexts configuration file
    InitContexts,

    /// Show the exact file path where your contexts.toml is located
    ContextsPath,

    /// Pin a context entry to a target chat session so it is always included
    ContextPin {
        /// The target session ID
        session_id: String,
        /// The context ID to pin
        context_id: String,
    },

    /// Unpin a context entry from a target chat session
    ContextUnpin {
        /// The target session ID
        session_id: String,
        /// The context ID to unpin
        context_id: String,
    },

    /// List all currently pinned contexts for a session
    ContextList {
        /// The target session ID
        session_id: String,
    },

    /// Display the current active profile identity and privacy details
    ShowProfile,

    /// Display the current response style settings
    ShowStyle,

    /// Display the list of topic preferences and dynamic likes
    ShowPreferences,

    /// Display all configured active goals and project contexts
    ShowContexts,

    /// Start a local interactive chat session
    Chat {
        /// Start a completely new session instead of resuming latest
        #[arg(long)]
        new: bool,

        /// Resume the latest active session ('latest')
        #[arg(long)]
        resume: Option<String>,

        /// Resume a specific session by ID
        #[arg(long)]
        session: Option<String>,
    },

    /// Manage conversation sessions
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },

    /// Run the always-on OpenNivara daemon in polling mode
    Daemon,

    /// Manage OpenNivara system daemon OS service
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },

    /// Manage the marketplace of OpenNivara Packs
    Marketplace {
        #[command(subcommand)]
        command: MarketplaceCommand,
    },

    /// Manage OpenNivara data-only Skills
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },

    /// Manage active modes and enabled packs
    Modes {
        #[command(subcommand)]
        command: ModesCommand,
    },
}

#[derive(Subcommand)]
enum MarketplaceCommand {
    /// Initialize the marketplace folders
    Init,
    /// List all installed packs
    List,
    /// Preview a local pack folder
    Preview {
        /// The folder path to the local pack
        path: String,
    },
    /// Install a local pack folder
    Install {
        /// The folder path to the local pack
        path: String,
    },
    /// Uninstall a pack by ID
    Uninstall {
        /// The ID of the pack to uninstall
        pack_id: String,
    },
    /// Preview an installed pack from config storage
    PreviewInstalled {
        /// The ID of the installed pack
        pack_id: String,
    },
    /// View capabilities of an installed pack
    Capabilities {
        /// The ID of the installed pack
        pack_id: String,
    },
    /// View diagnostics status of the marketplace
    Status,
    /// Run diagnostics without repairing
    Validate,
    /// Run diagnostics and repair broken settings/packs
    Repair {
        /// Run preview only without committing any file changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Enable an installed pack by ID
    Enable {
        /// The ID of the pack to enable
        pack_id: String,
    },
    /// Disable an installed pack by ID
    Disable {
        /// The ID of the pack to disable
        pack_id: String,
    },
    /// Safe reset of the marketplace configurations (deletes only config/marketplace/)
    Reset {
        /// Bypass interactive safety confirmation
        #[arg(long)]
        yes: bool,
    },
    /// Convert legacy modes configurations to the new Addon settings
    MigrateAddons,
}

#[derive(Subcommand)]
enum ModesCommand {
    /// List all available modes
    List,
    /// Show the active mode
    Active,
    /// Set the active mode
    Set {
        /// The ID of the mode to activate
        mode_id: String,
    },
    /// Create a new custom mode
    Create {
        /// The ID of the mode
        mode_id: String,
        /// The friendly name of the mode
        #[arg(long)]
        name: String,
    },
    /// Create a new mode directly from an installed pack
    CreateFromPack {
        /// The ID of the pack
        pack_id: String,
        /// The friendly name of the mode
        #[arg(long)]
        name: String,
        /// The ID of the mode to create
        #[arg(long)]
        id: String,
        /// Activate the mode immediately
        #[arg(long)]
        activate: bool,
        /// Apply theme if available in the pack
        #[arg(long)]
        apply_theme: bool,
        /// Apply style if available in the pack
        #[arg(long)]
        apply_style: bool,
    },
    /// Add an installed pack to a mode
    AddPack {
        /// The ID of the mode
        mode_id: String,
        /// The ID of the pack to add
        pack_id: String,
        /// Apply theme if available in the pack
        #[arg(long)]
        apply_theme: bool,
        /// Apply style if available in the pack
        #[arg(long)]
        apply_style: bool,
    },
    /// Remove an installed pack from a mode
    RemovePack {
        /// The ID of the mode
        mode_id: String,
        /// The ID of the pack to remove
        pack_id: String,
    },
}

#[derive(Subcommand)]
enum SkillsCommand {
    /// Initialize the Skills configuration files
    Init,
    /// List installed skills and enablement state
    List,
    /// Show a skill manifest
    Show {
        /// Skill ID
        skill_id: String,
    },
    /// Enable a skill in Settings-controlled state
    Enable {
        /// Skill ID
        skill_id: String,
    },
    /// Disable a skill in Settings-controlled state
    Disable {
        /// Skill ID
        skill_id: String,
    },
    /// Test deterministic skill routing for a message
    RouteTest {
        /// Message to route
        message: String,
    },
}

/// Session subcommands to list, show, or close conversation sessions.
#[derive(Subcommand)]
enum SessionsCommand {
    /// List all conversation sessions
    List,
    /// Show message history for a specific session ID
    Show {
        /// The target session ID
        session_id: String,
    },
    /// Rename a session's title
    Rename {
        /// The target session ID
        session_id: String,
        /// The new title
        title: String,
    },
    /// Close a session
    Close {
        /// The target session ID
        session_id: String,
    },
    /// Show all currently mapped active sessions
    Active,
}

/// System service subcommands.
#[derive(Subcommand)]
enum ServiceCommand {
    /// Install OpenNivara daemon as a background OS service
    Install,
    /// Start the OpenNivara background service
    Start,
    /// Stop the OpenNivara background service
    Stop,
    /// Uninstall the OpenNivara service
    Uninstall,
    /// Query the OpenNivara service status
    Status,
}

/// Standalone test subcommands to verify local read-only tools.
#[derive(Subcommand)]
enum ToolTestCommand {
    /// Get the current working directory path
    GetCurrentDir,

    /// List files and folders in a safe allowed local directory (non-recursive)
    ListDir {
        /// Path to list (e.g. . or src)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Check whether a safe allowed local path exists
    FileExists {
        /// Path to verify
        path: String,
    },

    /// Read a safe allowed UTF-8 text file
    ReadFile {
        /// Path to read
        path: String,
    },
}

#[tokio::main]
async fn main() {
    // 1. Load the .env file if it exists.
    opennivara::load_env();

    // 2. Parse command line arguments.
    let cli = Cli::parse();

    // 3. Execute the commands and handle any potential errors nicely.
    if let Err(err) = run_app(cli).await {
        // \x1b[1;31m makes the word "Error:" bold red. \x1b[0m resets formatting.
        eprintln!("\n\x1b[1;31mError:\x1b[0m {}", err);
        std::process::exit(1);
    }
}

/// Helper function to execute the chosen subcommand.
/// We return an anyhow::Result so that any `?` inside can propagate errors back to main.
async fn run_app(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::InitProfile => {
            println!("Initializing OpenNivara user profile...");
            let message = profile::init_profile()?;
            println!("{}", message);
        }
        Commands::ProfilePath => {
            let path = profile::get_profile_path()?;
            println!("{}", path.display());
        }
        Commands::InitPreferences => {
            println!("Initializing OpenNivara private preferences...");
            let message = preferences::init_preferences()?;
            println!("{}", message);
        }
        Commands::PreferencesPath => {
            let path = preferences::get_preferences_path()?;
            println!("{}", path.display());
        }
        Commands::InitStyle => {
            println!("Initializing OpenNivara style guidelines...");
            let message = style::init_style()?;
            println!("{}", message);
        }
        Commands::StylePath => {
            let path = style::get_style_path()?;
            println!("{}", path.display());
        }
        Commands::InitTools => {
            println!("Initializing OpenNivara tools configuration...");
            let message = tools::init_tools()?;
            println!("{}", message);
        }
        Commands::ToolsPath => {
            let path = tools::get_tools_path()?;
            println!("{}", path.display());
        }
        Commands::ToolsList => {
            let path = tools::get_tools_path()?;
            if !path.exists() {
                println!(
                    "\x1b[1;33mTools configuration is not initialized. Please run: opennivara init-tools\x1b[0m"
                );
                return Ok(());
            }

            let config = tools::read_tools()?;
            println!("\x1b[1;36m=== OpenNivara Tool System ===\x1b[0m");
            println!("Configuration Path: {}", path.display());
            println!("Globally Enabled:   {}", config.general.enabled);
            println!("Max Tool Rounds:    {}", config.general.max_tool_rounds);
            println!("Show Tool Activity: {}", config.general.show_tool_activity);
            println!("\nAllowed Roots:    {:?}", config.paths.allowed_roots);
            println!("Blocked Patterns: {:?}", config.paths.blocked_patterns);

            println!("\nTool Status:");
            let mut sorted_keys: Vec<&String> = config.tools.keys().collect();
            sorted_keys.sort();
            for key in sorted_keys {
                let settings = config.tools.get(key).unwrap();
                let status_color = if settings.enabled {
                    "\x1b[1;32m[ENABLED]\x1b[0m"
                } else {
                    "\x1b[1;31m[DISABLED]\x1b[0m"
                };
                println!(
                    "  {} {} (Requires Confirmation: {})",
                    status_color, key, settings.requires_confirmation
                );
            }
            println!("\x1b[1;36m==========================\x1b[0m");
        }
        Commands::ToolTest { command } => {
            let config = tools::read_tools()?;

            match command {
                ToolTestCommand::GetCurrentDir => {
                    let result =
                        tools::execute_tool("get_current_dir", &serde_json::json!({}), &config);
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                ToolTestCommand::ListDir { path } => {
                    let result = tools::execute_tool(
                        "list_dir",
                        &serde_json::json!({ "path": path }),
                        &config,
                    );
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                ToolTestCommand::FileExists { path } => {
                    let result = tools::execute_tool(
                        "file_exists",
                        &serde_json::json!({ "path": path }),
                        &config,
                    );
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                ToolTestCommand::ReadFile { path } => {
                    let result = tools::execute_tool(
                        "read_file",
                        &serde_json::json!({ "path": path }),
                        &config,
                    );
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            }
        }
        Commands::InitMap => {
            println!("Initializing OpenNivara workspace map configuration...");
            let message = workspace_map::config::init_map()?;
            println!("{}", message);
        }
        Commands::MapPath => {
            let path = workspace_map::config::get_map_path()?;
            println!("{}", path.display());
        }
        Commands::MapDbPath => {
            let path = workspace_map::get_db_path()?;
            println!("{}", path.display());
        }
        Commands::MapScan => {
            println!("Scanning allowed workspace roots...");
            let message = workspace_map::scan_workspace()?;
            println!("{}", message);
        }
        Commands::MapSummary => {
            let summary = workspace_map::render_summary()?;
            println!("{}", summary);
        }
        Commands::MapTree { depth } => {
            let tree = workspace_map::render_tree(depth)?;
            println!("{}", tree);
        }
        Commands::MapSearch { query } => {
            let matches = workspace_map::search_entries(&query)?;
            println!("{}", matches);
        }
        Commands::MapInfo { path } => {
            let info = workspace_map::get_entry_info(&path)?;
            println!("{}", info);
        }
        #[cfg(feature = "map-watch")]
        Commands::MapWatch => {
            println!("Map watch is experimental. Use map-scan for stable behavior.");
        }
        #[cfg(feature = "code-symbols")]
        Commands::MapSymbols { path } => {
            extract_rust_symbols(&path)?;
        }
        Commands::DebugContext { question } => {
            // 1. User Profile Context
            let profile = profile::read_profile()?;
            let profile_context = profile.to_compact_context_string();

            println!("\x1b[1;36m=== [DEBUG CONTEXT] ===\x1b[0m");
            println!("\x1b[1m1. User Profile Context (always sent):\x1b[0m");
            println!("{}", profile_context);
            println!("\n-----------------------");

            // 2. Style Context
            let style_path = style::get_style_path()?;
            if style_path.exists() {
                let jstyle = style::read_style()?;
                println!("\x1b[1m2. Style Context (always sent when present):\x1b[0m");
                println!("{}", jstyle.to_compact_context_string());
            } else {
                println!("\x1b[1;33m2. No style.toml file found. Style context omitted.\x1b[0m");
            }
            println!("\n-----------------------");

            // 3. Triggered Private Preferences
            let preferences_path = preferences::get_preferences_path()?;
            if preferences_path.exists() {
                let prefs = preferences::read_preferences()?;
                let matched_sections =
                    crate::context_selector::select_relevant_preference_sections(&prefs, &question);
                println!("\x1b[1m3. Triggered Private Preferences:\x1b[0m");
                if matched_sections.is_empty() {
                    println!(
                        "No private preference sections triggered by the question: '{}'",
                        question
                    );
                } else {
                    let formatted = preferences::format_relevant_preferences(&matched_sections);
                    println!("{}", formatted);
                }
            } else {
                println!(
                    "\x1b[1;33m3. No preferences.toml file found. Private preferences context omitted.\x1b[0m"
                );
            }
            println!("\n-----------------------");

            // 4. Tool system status and enabled tools
            let tools_path = tools::get_tools_path()?;
            println!("\x1b[1m4. Tool System Status:\x1b[0m");
            if tools_path.exists() {
                let tools_config = tools::read_tools()?;
                println!("- System Enabled:       {}", tools_config.general.enabled);
                println!(
                    "- Max Tool Rounds:      {}",
                    tools_config.general.max_tool_rounds
                );
                println!(
                    "- Show Tool Activity:   {}",
                    tools_config.general.show_tool_activity
                );

                println!("\nRecognized Tools status:");
                let mut sorted_keys: Vec<&String> = tools_config.tools.keys().collect();
                sorted_keys.sort();
                for key in sorted_keys {
                    let settings = tools_config.tools.get(key).unwrap();
                    let status_txt = if settings.enabled {
                        "\x1b[1;32m[ENABLED]\x1b[0m"
                    } else {
                        "\x1b[1;31m[DISABLED]\x1b[0m"
                    };
                    println!(
                        "  - {}: status={}, requires_confirmation={}",
                        key, status_txt, settings.requires_confirmation
                    );
                }
            } else {
                println!("\x1b[1;33mNo tools.toml file found. Tool execution disabled.\x1b[0m");
            }
            println!("\x1b[1;36m=======================\x1b[0m");
        }
        Commands::Ask { question } => {
            // Direct request utilizing the unified OpenNivaraEngine so it is logged in the database
            let engine = engine::OpenNivaraEngine::new();
            let request = engine::EngineRequest {
                source: engine::RequestSource::Cli,
                session_id: None, // Resolves or resumes default active CLI session
                message: question,
            };

            println!("Consulting with OpenNivara...");
            let response = engine.handle_message(request).await?;
            println!("\n\x1b[1;32mOpenNivara:\x1b[0m");
            output::cli::print_markdown(&response.answer);
        }
        Commands::InitTelegram => {
            println!("Initializing OpenNivara Telegram configuration...");
            let message = remote_policy::init_telegram()?;
            println!("{}", message);
        }
        Commands::TelegramPath => {
            let path = remote_policy::get_telegram_path()?;
            println!("{}", path.display());
        }
        Commands::StateDbPath => {
            let path = sessions::get_state_db_path()?;
            println!("{}", path.display());
        }
        Commands::InitContexts => {
            println!("Initializing OpenNivara contexts configuration...");
            let message = context::init_contexts()?;
            println!("{}", message);
        }
        Commands::ContextsPath => {
            let path = context::get_contexts_path()?;
            println!("{}", path.display());
        }
        Commands::ContextPin {
            session_id,
            context_id,
        } => {
            let conn = sessions::init_db()?;
            sessions::pin_context(&conn, &session_id, &context_id)?;
            println!(
                "Context '{}' successfully pinned to session '{}'.",
                context_id, session_id
            );
        }
        Commands::ContextUnpin {
            session_id,
            context_id,
        } => {
            let conn = sessions::init_db()?;
            sessions::unpin_context(&conn, &session_id, &context_id)?;
            println!(
                "Context '{}' successfully unpinned from session '{}'.",
                context_id, session_id
            );
        }
        Commands::ContextList { session_id } => {
            let conn = sessions::init_db()?;
            let list = sessions::list_pinned_contexts(&conn, &session_id)?;
            if list.is_empty() {
                println!("No contexts currently pinned to session '{}'.", session_id);
            } else {
                println!(
                    "\x1b[1;36m=== Pinned Contexts for Session '{}' ===\x1b[0m",
                    session_id
                );
                for id in list {
                    println!("  - {}", id);
                }
            }
        }
        Commands::ShowProfile => {
            let profile = profile::read_profile()?;
            println!("\x1b[1;36m=== Active User Profile (profile.toml) ===\x1b[0m");
            println!("{}", toml::to_string(&profile)?);
        }
        Commands::ShowStyle => {
            let style = style::read_style()?;
            println!("\x1b[1;36m=== Active Style Settings (style.toml) ===\x1b[0m");
            println!("{}", toml::to_string(&style)?);
        }
        Commands::ShowPreferences => {
            let prefs = preferences::read_preferences()?;
            println!("\x1b[1;36m=== Topic Preferences (preferences.toml) ===\x1b[0m");
            println!("{}", toml::to_string(&prefs)?);
        }
        Commands::ShowContexts => {
            let contexts = context::read_contexts()?;
            println!("\x1b[1;36m=== Project Goals & Contexts (contexts.toml) ===\x1b[0m");
            println!("{}", toml::to_string(&contexts)?);
        }
        Commands::Chat {
            new,
            resume,
            session,
        } => {
            run_chat_loop(new, resume, session).await?;
        }
        Commands::Sessions { command } => {
            let conn = sessions::init_db()?;
            match command {
                SessionsCommand::List => {
                    let list = sessions::list_sessions(&conn)?;
                    if list.is_empty() {
                        println!("No sessions found.");
                    } else {
                        println!("\x1b[1;36m=== OpenNivara Sessions ===\x1b[0m");
                        for s in list {
                            let active_str = if s.active {
                                "\x1b[1;32m[ACTIVE]\x1b[0m"
                            } else {
                                "\x1b[1;31m[CLOSED]\x1b[0m"
                            };
                            println!(
                                "{} ID: {} | Title: \"{}\" | Source: {} | Updated: {}",
                                active_str, s.id, s.title, s.source_created, s.updated_at
                            );
                        }
                    }
                }
                SessionsCommand::Show { session_id } => {
                    let messages = sessions::get_session_messages(&conn, &session_id)?;
                    let session_info = sessions::get_session(&conn, &session_id)?;
                    if let Some(s) = session_info {
                        println!("\x1b[1;36m=== Session: {} ({}) ===\x1b[0m", s.title, s.id);
                        println!("Created: {} | Source: {}", s.created_at, s.source_created);
                        println!("------------------------------------------------");
                        for m in messages {
                            let role_color = if m.role == "user" {
                                "\x1b[1;32mYou:\x1b[0m"
                            } else {
                                "\x1b[1;36mOpenNivara:\x1b[0m"
                            };
                            println!("{} {}", role_color, m.content);
                        }
                    } else {
                        println!("Session ID not found: {}", session_id);
                    }
                }
                SessionsCommand::Rename { session_id, title } => {
                    sessions::rename_session(&conn, &session_id, &title)?;
                    println!(
                        "Successfully renamed session {} to \"{}\"",
                        session_id, title
                    );
                }
                SessionsCommand::Close { session_id } => {
                    sessions::close_session(&conn, &session_id)?;
                    println!("Successfully closed session {}", session_id);
                }
                SessionsCommand::Active => {
                    let active = sessions::get_active_sessions_list(&conn)?;
                    println!("\x1b[1;36m=== Active Session Mappings ===\x1b[0m");
                    for (user_key, sess_id, updated) in active {
                        println!(
                            "User Key: {:<20} | Session ID: {} | Updated: {}",
                            user_key, sess_id, updated
                        );
                    }
                }
            }
        }
        Commands::Daemon => {
            daemon::run_daemon().await?;
        }
        Commands::Service { command } => match command {
            ServiceCommand::Install => service::service_install()?,
            ServiceCommand::Start => service::service_start()?,
            ServiceCommand::Stop => service::service_stop()?,
            ServiceCommand::Uninstall => service::service_uninstall()?,
            ServiceCommand::Status => service::service_status()?,
        },
        Commands::Skills { command } => match command {
            SkillsCommand::Init => {
                let message = skills::registry::init_skills()?;
                println!("{}", message);
            }
            SkillsCommand::List => {
                let summaries = skills::registry::list_skill_summaries()?;
                if summaries.is_empty() {
                    println!("No skills installed. Install a skill pack from Store first.");
                } else {
                    println!("\x1b[1;36m=== OpenNivara Skills ===\x1b[0m");
                    for skill in summaries {
                        let status = if skill.enabled {
                            "\x1b[1;32m[ENABLED]\x1b[0m"
                        } else {
                            "\x1b[1;31m[DISABLED]\x1b[0m"
                        };
                        let pack = skill.pack_id.unwrap_or_else(|| "user".to_string());
                        println!(
                            "  {} {} ({}) | pack: {} | policy: {:?} | tools: {:?}",
                            status,
                            skill.id,
                            skill.name,
                            pack,
                            skill.route_policy,
                            skill.allowed_tools
                        );
                    }
                }
            }
            SkillsCommand::Show { skill_id } => {
                let skill = skills::registry::get_skill(&skill_id)?;
                println!("{}", toml::to_string_pretty(&skill)?);
            }
            SkillsCommand::Enable { skill_id } => {
                skills::registry::set_skill_enabled(&skill_id, true)?;
                println!("\x1b[1;32mSkill '{}' enabled.\x1b[0m", skill_id);
            }
            SkillsCommand::Disable { skill_id } => {
                skills::registry::set_skill_enabled(&skill_id, false)?;
                println!("\x1b[1;32mSkill '{}' disabled.\x1b[0m", skill_id);
            }
            SkillsCommand::RouteTest { message } => {
                let decision = skills::registry::test_route(message)?;
                println!("\x1b[1;36m=== Skill Route Test ===\x1b[0m");
                if let Some(primary) = &decision.primary_skill {
                    println!("Primary skill: {} ({})", primary.name, primary.id);
                    println!("Score: {}", primary.score);
                    println!("Reason: {}", primary.reason);
                    println!("Allowed tools: {:?}", primary.allowed_tools);
                } else {
                    println!("Primary skill: none");
                    println!("Reason: {}", decision.reason);
                }
                if !decision.supporting_skills.is_empty() {
                    println!("\nSupporting skills:");
                    for skill in &decision.supporting_skills {
                        println!("  - {} ({}) score {}", skill.name, skill.id, skill.score);
                    }
                }
                if !decision.candidates.is_empty() {
                    println!("\nCandidates:");
                    for candidate in &decision.candidates {
                        println!(
                            "  - {} score {} accepted={} reason={}",
                            candidate.id, candidate.score, candidate.accepted, candidate.reason
                        );
                    }
                }
                if !decision.warnings.is_empty() {
                    println!("\nWarnings:");
                    for warning in &decision.warnings {
                        println!("  - {}", warning);
                    }
                }
            }
        },
        Commands::Marketplace { command } => match command {
            MarketplaceCommand::Init => {
                println!("Initializing OpenNivara Marketplace...");
                let msg = marketplace::init_marketplace()?;
                println!("{}", msg);
            }
            MarketplaceCommand::List => {
                let file = marketplace::packs::list_installed_packs()?;
                println!("\x1b[1;36m=== Installed OpenNivara Packs ===\x1b[0m");
                if file.installed.is_empty() {
                    println!("No packs installed yet. Use 'opennivara marketplace install <path>' to import local packs.");
                } else {
                    for pack in file.installed {
                        let status = if pack.enabled {
                            "\x1b[1;32m[ENABLED]\x1b[0m"
                        } else {
                            "\x1b[1;31m[DISABLED]\x1b[0m"
                        };
                        println!(
                            "{} {} v{} | Source: {} | Installed: {}",
                            status, pack.name, pack.version, pack.source, pack.installed_at
                        );
                    }
                }
            }
            MarketplaceCommand::Preview { path } => {
                let path_buf = std::path::PathBuf::from(&path);
                println!("Previewing pack from folder: {}...", path_buf.display());
                let preview = marketplace::packs::preview_pack_from_path(path_buf)?;
                println!(
                    "\n\x1b[1;36m=== Pack Preview: {} (v{}) ===\x1b[0m",
                    preview.manifest.name, preview.manifest.version
                );
                println!("ID:          {}", preview.manifest.id);
                println!("Author:      {}", preview.manifest.author);
                println!("Category:    {}", preview.manifest.category);
                println!("Description: {}", preview.manifest.description);
                println!(
                    "Risk Level:  \x1b[1m{}\x1b[0m",
                    preview.safety_summary.risk_level
                );
                println!("\n\x1b[1mDeclared Content:\x1b[0m");
                println!(
                    "  Preferences:    {} ({} sections)",
                    preview.manifest.contents.preferences, preview.additions.preferences_count
                );
                println!(
                    "  Contexts:       {} ({} items)",
                    preview.manifest.contents.contexts, preview.additions.contexts_count
                );
                println!(
                    "  Theme Accent:   {} ({} themes)",
                    preview.manifest.contents.theme, preview.additions.themes_count
                );
                println!(
                    "  Commands:       {} ({} snippets)",
                    preview.manifest.contents.command_snippets,
                    preview.additions.command_snippets_count
                );
                println!(
                    "  Style Presets:  {}",
                    preview.manifest.contents.style_presets
                );

                if !preview.errors.is_empty() {
                    println!("\n\x1b[1;31mSafety Constraints / Errors:\x1b[0m");
                    for err in &preview.errors {
                        println!("  [BLOCKING] {}", err);
                    }
                }
                if !preview.warnings.is_empty() {
                    println!("\n\x1b[1;33mSafety Warnings:\x1b[0m");
                    for warn in &preview.warnings {
                        println!("  [WARN] {}", warn);
                    }
                }
            }
            MarketplaceCommand::Install { path } => {
                let path_buf = std::path::PathBuf::from(&path);
                println!("Installing pack from folder: {}...", path_buf.display());
                let pack = marketplace::packs::install_pack_from_path(path_buf)?;
                println!(
                    "\x1b[1;32mSuccessfully installed pack '{}' (v{})!\x1b[0m",
                    pack.name, pack.version
                );
            }
            MarketplaceCommand::Uninstall { pack_id } => {
                println!("Uninstalling pack '{}'...", pack_id);
                marketplace::packs::uninstall_pack(&pack_id)?;
                println!(
                    "\x1b[1;32mPack '{}' uninstalled successfully.\x1b[0m",
                    pack_id
                );
            }
            MarketplaceCommand::PreviewInstalled { pack_id } => {
                println!("Previewing installed pack '{}'...", pack_id);
                let preview = marketplace::packs::preview_installed_pack(&pack_id)?;
                println!(
                    "\x1b[1;36m=== Pack Preview: {} ({}) ===\x1b[0m",
                    preview.manifest.name, preview.manifest.id
                );
                println!("Version:     {}", preview.manifest.version);
                println!("Author:      {}", preview.manifest.author);
                println!("Description: {}", preview.manifest.description);
                println!("Safety Risk: {}", preview.manifest.safety.risk_level);
                println!("Additions Summary:");
                println!("  Preferences: {}", preview.additions.preferences_count);
                println!("  Contexts:    {}", preview.additions.contexts_count);
                println!("  Themes:      {}", preview.additions.themes_count);
                println!(
                    "  Commands:    {}",
                    preview.additions.command_snippets_count
                );
            }
            MarketplaceCommand::Capabilities { pack_id } => {
                println!("Reading capabilities for pack '{}'...", pack_id);
                let cap = marketplace::packs::get_pack_activation_capabilities(&pack_id)?;
                println!("\n\x1b[1;36m=== Pack Activation Capabilities ===\x1b[0m");
                println!("Pack ID:            {}", cap.pack_id);
                println!(
                    "Has Theme:          {} (Theme ID: {:?}, Theme Name: {:?})",
                    cap.has_theme, cap.theme_id, cap.theme_name
                );
                println!("Has Style:          {}", cap.has_style);
                println!("Has Preferences:    {}", cap.has_preferences);
                println!("Has Contexts:       {}", cap.has_contexts);
                println!("Has Commands:       {}", cap.has_command_snippets);
                println!("Has Workspace Rules:{}", cap.has_workspace_rules);
            }
            MarketplaceCommand::Status => {
                println!("Auditing OpenNivara Marketplace Status...");
                let status = marketplace::repair::marketplace_status()?;
                println!("\n\x1b[1;36m=== OpenNivara Marketplace Status ===\x1b[0m");
                println!("Marketplace Dir:     {}", status.marketplace_dir);
                println!("Installed Packs:     {}", status.installed_count);
                println!("Enabled Packs:       {}", status.enabled_count);
                println!("Disabled Packs:      {}", status.disabled_count);
                println!("Available Modes:     {}", status.modes_count);
                println!(
                    "Active Mode:         {} ({})",
                    status.active_mode_name, status.active_mode_id
                );
                if let Some(ref t_id) = status.active_theme_id {
                    println!(
                        "Active Theme:        {} ({})",
                        status.active_theme_name.as_deref().unwrap_or("Default"),
                        t_id
                    );
                } else {
                    println!("Active Theme:        None (Default)");
                }
                println!("Builtin Available:   {:?}", status.builtin_packs_available);
                println!(
                    "Builtin Path:        {}",
                    status.builtin_resource_path_checked
                );
                println!(
                    "Builtin Path Exists: {}",
                    status.builtin_resource_path_exists
                );

                if !status.missing_pack_ids.is_empty() {
                    println!("\n\x1b[1;31mWarning: The following packs are referenced but missing:\x1b[0m");
                    for id in &status.missing_pack_ids {
                        println!("  - {}", id);
                    }
                }
                if !status.disabled_packs_in_active_mode.is_empty() {
                    println!("\n\x1b[1;33mWarning: The active mode references the following disabled pack(s):\x1b[0m");
                    for id in &status.disabled_packs_in_active_mode {
                        println!("  - {}", id);
                    }
                }
            }
            MarketplaceCommand::Validate => {
                println!("Validating OpenNivara Marketplace...");
                let status = marketplace::repair::marketplace_status_readonly()?;
                let has_invalid_active_mode = status.active_mode_name == "Unknown";
                let has_warnings = has_invalid_active_mode
                    || !status.missing_pack_ids.is_empty()
                    || !status.disabled_packs_in_active_mode.is_empty()
                    || !status.builtin_resource_path_exists;

                if !has_warnings {
                    println!("\x1b[1;32mOK:\x1b[0m Marketplace diagnostics are clean.");
                } else {
                    println!("\x1b[1;33mWarnings:\x1b[0m Marketplace needs attention.");
                    if has_invalid_active_mode {
                        println!("  - Active mode '{}' is invalid.", status.active_mode_id);
                    }
                    if !status.missing_pack_ids.is_empty() {
                        println!("  - Missing pack references: {:?}", status.missing_pack_ids);
                    }
                    if !status.disabled_packs_in_active_mode.is_empty() {
                        println!(
                            "  - Disabled packs in active mode: {:?}",
                            status.disabled_packs_in_active_mode
                        );
                    }
                    if !status.builtin_resource_path_exists {
                        println!(
                            "  - Built-in pack resource path is unavailable: {}",
                            status.builtin_resource_path_checked
                        );
                    }
                }
            }
            MarketplaceCommand::Repair { dry_run } => {
                if dry_run {
                    println!("Simulating OpenNivara Marketplace diagnostics and repair suite (DRY-RUN)...");
                } else {
                    println!("Running OpenNivara Marketplace diagnostics and repair suite...");
                }
                let report = marketplace::repair::marketplace_repair(dry_run)?;
                println!("\n\x1b[1;36m=== Marketplace Repair Audit ===\x1b[0m");
                if dry_run {
                    println!("Synchronized State:  \x1b[1;33mPREVIEW ONLY\x1b[0m");
                } else {
                    println!(
                        "Synchronized State:  {}",
                        if report.repaired {
                            "\x1b[1;33mREPAIRED\x1b[0m"
                        } else {
                            "\x1b[1;32mOPTIMAL\x1b[0m"
                        }
                    );
                }

                if !report.actions.is_empty() {
                    if dry_run {
                        println!("\n\x1b[1mPlanned Actions (Not Committed):\x1b[0m");
                    } else {
                        println!("\n\x1b[1mRepair Actions Taken:\x1b[0m");
                    }
                    for action in &report.actions {
                        println!("  - {}", action);
                    }
                }
                if !report.warnings.is_empty() {
                    println!("\n\x1b[1;33mDiagnostics Warnings:\x1b[0m");
                    for warn in &report.warnings {
                        println!("  - [WARN] {}", warn);
                    }
                }
                if !report.errors.is_empty() {
                    println!("\n\x1b[1;31mRepair Failures/Errors:\x1b[0m");
                    for err in &report.errors {
                        println!("  - [ERROR] {}", err);
                    }
                }
            }
            MarketplaceCommand::Enable { pack_id } => {
                println!("Enabling pack '{}'...", pack_id);
                marketplace::packs::enable_pack(&pack_id, true)?;
                println!("\x1b[1;32mPack '{}' successfully enabled.\x1b[0m", pack_id);
            }
            MarketplaceCommand::Disable { pack_id } => {
                println!("Disabling pack '{}'...", pack_id);
                marketplace::packs::enable_pack(&pack_id, false)?;
                println!("\x1b[1;32mPack '{}' successfully disabled.\x1b[0m", pack_id);
            }
            MarketplaceCommand::Reset { yes } => {
                let proceed = if yes {
                    true
                } else {
                    println!("\x1b[1;31;5mWARNING: Resetting the marketplace will delete all installed packs and mode pack assignments.\x1b[0m");
                    println!("This will NOT delete your profile identity, local tools, private preferences, or sessions database.");
                    print!("Are you absolutely sure you want to proceed? (yes/no): ");
                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    input.trim().to_lowercase() == "yes"
                };

                if proceed {
                    println!("Resetting OpenNivara Marketplace configuration...");
                    marketplace::marketplace_reset(true).map_err(|e| anyhow::anyhow!(e))?;
                    println!("\x1b[1;32mOpenNivara Marketplace configuration was successfully reset to fresh defaults.\x1b[0m");
                } else {
                    println!("Reset aborted by user.");
                }
            }
            MarketplaceCommand::MigrateAddons => {
                println!("Migrating legacy modes to addon settings...");
                marketplace::addon_settings::migrate_modes_to_addons()?;
                println!("\x1b[1;32mMigration completed successfully. Active pack/theme setups converted to addon configurations.\x1b[0m");
            }
        },
        Commands::Modes { command } => match command {
            ModesCommand::List => {
                let file = marketplace::modes::read_modes()?;
                println!("\x1b[1;36m=== OpenNivara Active Modes ===\x1b[0m");
                for mode in file.modes {
                    let active_indicator = if mode.id == file.active_mode {
                        "\x1b[1;32m* [ACTIVE]\x1b[0m"
                    } else {
                        "  [INACTIVE]"
                    };
                    let theme = mode.theme_id.unwrap_or_else(|| "none".to_string());
                    println!(
                        "{} ID: {} | Name: \"{}\" | Description: \"{}\"",
                        active_indicator, mode.id, mode.name, mode.description
                    );
                    println!("             Packs in Mode: {:?}", mode.enabled_pack_ids);
                    println!("             Theme: {}", theme);
                }
            }
            ModesCommand::Active => {
                let mode = marketplace::modes::get_active_mode()?;
                println!("\x1b[1;36m=== Active Mode ===\x1b[0m");
                println!("ID:          {}", mode.id);
                println!("Name:        {}", mode.name);
                println!("Description: {}", mode.description);
                println!("Packs in Mode: {:?}", mode.enabled_pack_ids);
            }
            ModesCommand::Set { mode_id } => {
                println!("Switching active mode to '{}'...", mode_id);
                marketplace::modes::set_active_mode(&mode_id)?;
                println!(
                    "\x1b[1;32mActive mode successfully updated to '{}'.\x1b[0m",
                    mode_id
                );
            }
            ModesCommand::Create { mode_id, name } => {
                println!("Creating new mode '{}'...", mode_id);
                let new_mode = marketplace::modes::OpenNivaraMode {
                    id: mode_id.clone(),
                    name,
                    description: "User created custom mode.".to_string(),
                    enabled_pack_ids: vec![],
                    theme_id: None,
                    style_pack_id: None,
                };
                marketplace::modes::create_mode(new_mode)?;
                println!(
                    "\x1b[1;32mSuccessfully created custom mode '{}'.\x1b[0m",
                    mode_id
                );
            }
            ModesCommand::CreateFromPack {
                pack_id,
                name,
                id,
                activate,
                apply_theme,
                apply_style,
            } => {
                println!("Creating new mode '{}' from pack '{}'...", id, pack_id);
                let mode = marketplace::modes::create_mode_from_pack(
                    &pack_id,
                    &id,
                    &name,
                    activate,
                    apply_theme,
                    apply_style,
                )?;
                println!(
                    "\x1b[1;32mSuccessfully created custom mode '{}' (Name: \"{}\")!\x1b[0m",
                    mode.id, mode.name
                );
                if activate {
                    println!("Mode '{}' is now active.", mode.id);
                }
            }
            ModesCommand::AddPack {
                mode_id,
                pack_id,
                apply_theme,
                apply_style,
            } => {
                println!("Adding pack '{}' to mode '{}'...", pack_id, mode_id);
                let res = marketplace::modes::add_pack_to_mode_with_activation(
                    &mode_id,
                    &pack_id,
                    apply_theme,
                    apply_style,
                )?;
                println!(
                    "\x1b[1;32mSuccessfully added pack '{}' to mode '{}'.\x1b[0m",
                    pack_id, mode_id
                );
                if let Some(ref t_id) = res.applied_theme_id {
                    println!("Applied theme: {}", t_id);
                }
                if let Some(ref s_id) = res.applied_style_pack_id {
                    println!("Applied style source pack: {}", s_id);
                }
                for warning in res.warnings {
                    println!("\x1b[1;33mWarning:\x1b[0m {}", warning);
                }
            }
            ModesCommand::RemovePack { mode_id, pack_id } => {
                println!("Removing pack '{}' from mode '{}'...", pack_id, mode_id);
                marketplace::modes::remove_pack_from_mode(&mode_id, &pack_id)?;
                println!(
                    "\x1b[1;32mSuccessfully removed pack '{}' from mode '{}'.\x1b[0m",
                    pack_id, mode_id
                );
            }
        },
    }
    Ok(())
}

/// Interactive Multi-Turn CLI Chat Session loop.
async fn run_chat_loop(
    new: bool,
    resume: Option<String>,
    session: Option<String>,
) -> anyhow::Result<()> {
    let conn = sessions::init_db()?;
    let user_key = "cli";

    // 1. Resolve Session ID
    let session_id = if new {
        let id = sessions::create_session(&conn, "CLI", None)?;
        sessions::set_active_session(&conn, user_key, &id)?;
        id
    } else if let Some(ref s_id) = session {
        if sessions::get_session(&conn, s_id)?.is_none() {
            return Err(anyhow::anyhow!("Session ID not found: {}", s_id));
        }
        sessions::set_active_session(&conn, user_key, s_id)?;
        s_id.clone()
    } else if let Some(ref resume_val) = resume {
        if resume_val == "latest" {
            match sessions::get_latest_active_session(&conn)? {
                Some(id) => {
                    sessions::set_active_session(&conn, user_key, &id)?;
                    id
                }
                None => {
                    // Create a new session if none is available
                    let id = sessions::create_session(&conn, "CLI", None)?;
                    sessions::set_active_session(&conn, user_key, &id)?;
                    id
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "Invalid --resume value. Use '--resume latest'."
            ));
        }
    } else {
        // Default behavior: Check active CLI session first, fall back to latest active session, then create new
        match sessions::get_active_session(&conn, user_key)? {
            Some(id) => id,
            None => match sessions::get_latest_active_session(&conn)? {
                Some(id) => {
                    sessions::set_active_session(&conn, user_key, &id)?;
                    id
                }
                None => {
                    let id = sessions::create_session(&conn, "CLI", None)?;
                    sessions::set_active_session(&conn, user_key, &id)?;
                    id
                }
            },
        }
    };

    // 2. Fetch session details to present details
    let session_info = sessions::get_session(&conn, &session_id)?
        .ok_or_else(|| anyhow::anyhow!("Session details not found for ID: {}", session_id))?;

    println!("\x1b[1;36mResuming session:\x1b[0m");
    println!("ID:           {}", session_info.id);
    println!("Title:        {}", session_info.title);
    println!("Started from: {}", session_info.source_created);
    println!("Last updated: {}", session_info.updated_at);
    println!("\nType your message below. Type 'exit' or 'quit' to end the chat.\n");

    // 3. Dialogue loop
    let engine = engine::OpenNivaraEngine::new();
    use std::io::Write;

    loop {
        print!("\x1b[1;32mYou:\x1b[0m ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        println!("Consulting with OpenNivara...");

        let request = engine::EngineRequest {
            source: engine::RequestSource::Cli,
            session_id: Some(session_id.clone()),
            message: trimmed.to_string(),
        };

        match engine.handle_message(request).await {
            Ok(response) => {
                println!("\n\x1b[1;32mOpenNivara:\x1b[0m");
                output::cli::print_markdown(&response.answer);
                println!();

                // Proactively rename the session if it's still using the default name
                let messages = sessions::get_session_messages(&conn, &session_id)?;
                let current_sess = sessions::get_session(&conn, &session_id)?;
                if let Some(s) = current_sess {
                    if s.title == "New Conversation" && messages.len() >= 2 {
                        let first_user_msg = messages
                            .iter()
                            .find(|m| m.role == "user")
                            .map(|m| m.content.clone())
                            .unwrap_or_else(|| "New Conversation".to_string());

                        let new_title = if first_user_msg.len() > 30 {
                            format!("{}...", &first_user_msg[..27])
                        } else {
                            first_user_msg
                        };
                        let _ = sessions::rename_session(&conn, &session_id, &new_title);
                    }
                }
            }
            Err(e) => {
                println!("\n\x1b[1;31mError:\x1b[0m {}", e);
                println!();
            }
        }
    }

    Ok(())
}

/// Helper function to extract code symbols from Rust files, gated by the code-symbols feature.
#[cfg(feature = "code-symbols")]
fn extract_rust_symbols(path_str: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(path_str);
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path_str));
    }
    let content = std::fs::read_to_string(path)?;
    println!("\x1b[1;36m=== Rust Symbols for {} ===\x1b[0m", path_str);
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
            println!("  Line {:>3}: Function      -> {}", line_num + 1, trimmed);
        } else if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
            println!("  Line {:>3}: Struct        -> {}", line_num + 1, trimmed);
        } else if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
            println!("  Line {:>3}: Enum          -> {}", line_num + 1, trimmed);
        } else if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
            println!("  Line {:>3}: Module        -> {}", line_num + 1, trimmed);
        } else if trimmed.starts_with("impl ") {
            println!("  Line {:>3}: Impl Block    -> {}", line_num + 1, trimmed);
        }
    }
    println!("\x1b[1;36m========================================\x1b[0m");
    Ok(())
}
