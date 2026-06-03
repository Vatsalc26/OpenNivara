#![allow(dead_code)]
use crate::remote_policy::TelegramConfig;
use crate::sessions;
use std::sync::{Mutex, OnceLock};
use teloxide::prelude::*;
use teloxide::types::{Message, ParseMode};
use teloxide::utils::command::BotCommands;

pub struct PendingConfirmation {
    pub token: String,
    pub chat_id: i64,
    pub action: String, // "profile", "style"
    pub summary: String,
    pub payload: String,
    pub created_at: std::time::Instant,
}

static PENDING_CONFIRMATIONS: OnceLock<Mutex<Vec<PendingConfirmation>>> = OnceLock::new();

fn get_pending_confirmations() -> &'static Mutex<Vec<PendingConfirmation>> {
    PENDING_CONFIRMATIONS.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported by OpenNivara:"
)]
pub enum Command {
    #[command(description = "Explains OpenNivara and checks authorization status.")]
    Start,
    #[command(description = "Displays the help message.")]
    Help,
    #[command(description = "Displays your Telegram Chat ID so you can authorize it.")]
    Whoami,
    #[command(description = "Shows the active session and configuration status.")]
    Status,
    #[command(description = "Sends a single message to OpenNivara. Usage: /ask <message>")]
    Ask { message: String },
    #[command(description = "Initiates a continuous chat session. Usage: /chat <message>")]
    Chat { message: String },
    #[command(description = "Creates a new conversation session.")]
    New,
    #[command(description = "Lists recent conversation sessions.")]
    Sessions,
    #[command(description = "Resumes a session by ID. Usage: /resume <session_id>")]
    Resume { session_id: String },
    #[command(description = "Resumes the latest active session across CLI and Telegram.")]
    Latest,
    #[command(description = "Gets a high-level summary of the workspace map.")]
    MapSummary,
    #[command(description = "Searches the workspace map. Usage: /map_search <query>")]
    MapSearch { query: String },
    #[command(description = "Displays a visual directory tree structure.")]
    MapTree,
    #[command(description = "Approves a pending execution request. Usage: /approve <request_id>")]
    Approve { request_id: String },
    #[command(description = "Denies a pending execution request. Usage: /deny <request_id>")]
    Deny { request_id: String },
    #[command(
        description = "Displays or modifies profile. Usage: /profile show or /profile set bio <text>"
    )]
    Profile { subcommand: String },
    #[command(
        description = "Displays or modifies style. Usage: /style show or /style set brevity <level>"
    )]
    Style { subcommand: String },
    #[command(description = "Displays topic preference sections.")]
    Prefs,
    #[command(
        description = "Lists or pins contexts. Usage: /contexts show, /contexts pin <id>, /contexts unpin <id>"
    )]
    Contexts { subcommand: String },
    #[command(description = "Confirms a pending write action. Usage: /confirm <token>")]
    Confirm { token: String },
}

/// Splits long responses so they fit within Telegram's character limits (default: 3500).
pub fn split_telegram_message(text: &str, max_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let mut parts = Vec::new();
    let mut current_part = String::new();

    for line in text.lines() {
        if current_part.len() + line.len() + 1 > max_chars {
            if !current_part.is_empty() {
                parts.push(current_part.clone());
                current_part.clear();
            }

            if line.len() > max_chars {
                let mut line_chars = line.chars();
                loop {
                    let chunk: String = line_chars.by_ref().take(max_chars).collect();
                    if chunk.is_empty() {
                        break;
                    }
                    parts.push(chunk);
                }
            } else {
                current_part.push_str(line);
                current_part.push('\n');
            }
        } else {
            current_part.push_str(line);
            current_part.push('\n');
        }
    }

    if !current_part.is_empty() {
        parts.push(current_part);
    }

    parts
}

/// Starts the Teloxide listener loop using the token from environment variables.
pub async fn start_bot() -> anyhow::Result<()> {
    let token = std::env::var("TELEGRAM_BOT_TOKEN").map_err(|_| {
        anyhow::anyhow!(
            "Missing TELEGRAM_BOT_TOKEN environment variable.\n\n\
             Please double-check that you have:\n\
             1. Created a `.env` file in the folder where you run the CLI.\n\
             2. Added `TELEGRAM_BOT_TOKEN=your_botfather_token` to it.\n\
             3. Or set it in your terminal environment directly."
        )
    })?;

    let token = token.trim();
    if token.is_empty() {
        return Err(anyhow::anyhow!(
            "TELEGRAM_BOT_TOKEN environment variable is empty."
        ));
    }

    let bot = Bot::new(token);

    // Make sure telegram configuration can be read
    let _ = crate::remote_policy::read_telegram()?;

    tracing::info!("Initializing Telegram bot dispatcher...");

    let handler = dptree::entry().branch(Update::filter_message().endpoint(handle_message_update));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

/// Entrypoint endpoint for processing all message events.
async fn handle_message_update(bot: Bot, msg: Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id.0;
    let username = msg.chat.username().map(|s| s.to_string());

    // 1. Load Telegram permissions config
    let config = match crate::remote_policy::read_telegram() {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = bot
                .send_message(msg.chat.id, format!("❌ Configuration Error: {}", e))
                .await;
            return Ok(());
        }
    };

    let is_authorized = config.auth.allowed_chat_ids.contains(&chat_id);

    // Extract message text content
    let text = match msg.text() {
        Some(t) => t,
        None => return Ok(()),
    };

    // Parse commands matching /command
    let parsed_cmd = Command::parse(text, "");

    // 2. Gate remote access for unauthorized users
    if !is_authorized {
        match parsed_cmd {
            Ok(Command::Whoami) => {
                let resp = format!(
                    "Your Telegram Chat ID is: `{}`\n\n\
                     To authorize this chat, add this ID to `allowed_chat_ids` under `[auth]` in your `telegram.toml` file, and restart the OpenNivara daemon.",
                    chat_id
                );
                let _ = bot.send_message(msg.chat.id, resp).await;
                return Ok(());
            }
            Ok(Command::Start) => {
                let resp = format!(
                    "Welcome to OpenNivara! 👋\n\n\
                     Status: ❌ UNAUTHORIZED\n\
                     Your Chat ID is: `{}`\n\n\
                     Please add this Chat ID to `allowed_chat_ids` under `[auth]` in your `telegram.toml` and restart the daemon.",
                    chat_id
                );
                let _ = bot.send_message(msg.chat.id, resp).await;
                return Ok(());
            }
            _ => {
                // Ignore completely or return an unauthorized block message
                let resp = format!(
                    "Unauthorized access. Your Telegram Chat ID (`{}`) is not authorized in telegram.toml.\n\n\
                     Please add it under `[auth]` and restart the OpenNivara daemon.\n\
                     Run /whoami to see your Chat ID again.",
                    chat_id
                );
                let _ = bot.send_message(msg.chat.id, resp).await;
                return Ok(());
            }
        }
    }

    // 3. Dispatch authorized command operations
    match parsed_cmd {
        Ok(Command::Start) => {
            let welcome = "Welcome back to OpenNivara! 👋\n\n\
                           Status: ✅ AUTHORIZED\n\n\
                           You can interact with me using commands or simply message me directly to continue your last active conversation!";
            let _ = bot.send_message(msg.chat.id, welcome).await;
        }
        Ok(Command::Help) => {
            let help_text = Command::descriptions().to_string();
            let _ = bot.send_message(msg.chat.id, help_text).await;
        }
        Ok(Command::Whoami) => {
            let resp = format!(
                "Your Telegram Chat ID is: `{}`\nStatus: ✅ Authorized",
                chat_id
            );
            let _ = bot.send_message(msg.chat.id, resp).await;
        }
        Ok(Command::Status) => {
            let has_map = if let Ok(db_path) = crate::workspace_map::get_db_path() {
                db_path.exists()
            } else {
                false
            };

            let map_status = if has_map {
                "✅ Available"
            } else {
                "❌ Unavailable"
            };

            let active_sess = match sessions::init_db() {
                Ok(conn) => {
                    let user_key = format!("telegram_{}", chat_id);
                    match sessions::get_active_session(&conn, &user_key) {
                        Ok(Some(id)) => id,
                        _ => "None (created automatically on next /ask)".to_string(),
                    }
                }
                _ => "Error".to_string(),
            };

            let status_text = format!(
                "=== OpenNivara Bot Status ===\n\n\
                 Bot Name: {}\n\
                 Workspace Map: {}\n\
                 Active Session ID: `{}`\n\n\
                 Remote Permissions:\n\
                 - Ask/Chat: enabled\n\
                 - Workspace Map Summary: {}\n\
                 - Read local files: {}\n\
                 - Unrestricted shell: ❌ Blocked",
                config.general.bot_name,
                map_status,
                active_sess,
                if config.permissions.allow_map_summary {
                    "✅ Allowed"
                } else {
                    "❌ Blocked"
                },
                if config.permissions.allow_read_file {
                    "✅ Allowed"
                } else {
                    "❌ Blocked"
                }
            );
            let _ = bot.send_message(msg.chat.id, status_text).await;
        }
        Ok(Command::Ask { message }) => {
            handle_engine_request(&bot, msg.chat.id, chat_id, username, Some(message), &config)
                .await;
        }
        Ok(Command::Chat { message }) => {
            handle_engine_request(&bot, msg.chat.id, chat_id, username, Some(message), &config)
                .await;
        }
        Ok(Command::New) => match sessions::init_db() {
            Ok(conn) => {
                let user_key = format!("telegram_{}", chat_id);
                let source_str = format!("Telegram ({})", chat_id);
                match sessions::create_session(&conn, &source_str, None) {
                    Ok(new_id) => {
                        let _ = sessions::set_active_session(&conn, &user_key, &new_id);
                        let _ = bot
                            .send_message(
                                msg.chat.id,
                                format!(
                                    "Created a new session and set as active:\nID: `{}`",
                                    new_id
                                ),
                            )
                            .await;
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("Error creating session: {}", e))
                            .await;
                    }
                }
            }
            Err(e) => {
                let _ = bot
                    .send_message(msg.chat.id, format!("Database error: {}", e))
                    .await;
            }
        },
        Ok(Command::Sessions) => match sessions::init_db() {
            Ok(conn) => match sessions::list_sessions(&conn) {
                Ok(list) => {
                    if list.is_empty() {
                        let _ = bot.send_message(msg.chat.id, "No sessions found.").await;
                    } else {
                        let mut resp = "=== Recent Sessions ===\n\n".to_string();
                        for s in list.iter().take(10) {
                            resp.push_str(&format!(
                                "- ID: `{}`\n  Title: {}\n  Source: {}\n  Updated: {}\n\n",
                                s.id, s.title, s.source_created, s.updated_at
                            ));
                        }
                        let _ = bot.send_message(msg.chat.id, resp).await;
                    }
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Database error: {}", e))
                        .await;
                }
            },
            Err(e) => {
                let _ = bot
                    .send_message(msg.chat.id, format!("Database error: {}", e))
                    .await;
            }
        },
        Ok(Command::Resume { session_id }) => match sessions::init_db() {
            Ok(conn) => match sessions::get_session(&conn, &session_id) {
                Ok(Some(s)) => {
                    let user_key = format!("telegram_{}", chat_id);
                    let _ = sessions::set_active_session(&conn, &user_key, &s.id);
                    let _ = bot
                        .send_message(
                            msg.chat.id,
                            format!(
                                "Resumed session successfully:\nID: `{}`\nTitle: {}",
                                s.id, s.title
                            ),
                        )
                        .await;
                }
                Ok(None) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Session ID not found: {}", session_id))
                        .await;
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Database error: {}", e))
                        .await;
                }
            },
            Err(e) => {
                let _ = bot
                    .send_message(msg.chat.id, format!("Database error: {}", e))
                    .await;
            }
        },
        Ok(Command::Latest) => match sessions::init_db() {
            Ok(conn) => match sessions::get_latest_active_session(&conn) {
                Ok(Some(id)) => {
                    let user_key = format!("telegram_{}", chat_id);
                    let _ = sessions::set_active_session(&conn, &user_key, &id);
                    let title = match sessions::get_session(&conn, &id) {
                        Ok(Some(s)) => s.title,
                        _ => "New Conversation".to_string(),
                    };
                    let _ = bot.send_message(msg.chat.id, format!("Resumed latest active session across CLI/Telegram:\nID: `{}`\nTitle: {}", id, title)).await;
                }
                Ok(None) => {
                    let _ = bot
                        .send_message(
                            msg.chat.id,
                            "No active sessions found. Run `/new` to start one.",
                        )
                        .await;
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Database error: {}", e))
                        .await;
                }
            },
            Err(e) => {
                let _ = bot
                    .send_message(msg.chat.id, format!("Database error: {}", e))
                    .await;
            }
        },
        Ok(Command::MapSummary) => {
            if !config.permissions.allow_map_summary {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "That tool is disabled for Telegram remote access.",
                    )
                    .await;
                return Ok(());
            }
            match crate::workspace_map::render_summary() {
                Ok(summary) => {
                    let html = format!("<pre>{}</pre>", html_escape::encode_text(&summary));
                    let _ = bot
                        .send_message(msg.chat.id, html)
                        .parse_mode(ParseMode::Html)
                        .await;
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Failed to read workspace map: {}", e))
                        .await;
                }
            }
        }
        Ok(Command::MapSearch { query }) => {
            if !config.permissions.allow_map_search {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "That tool is disabled for Telegram remote access.",
                    )
                    .await;
                return Ok(());
            }
            if query.trim().is_empty() {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "Please provide a query search term, e.g. `/map_search main`",
                    )
                    .await;
                return Ok(());
            }
            match crate::workspace_map::search_entries(&query) {
                Ok(matches) => {
                    let _ = bot.send_message(msg.chat.id, matches).await;
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Failed to search map: {}", e))
                        .await;
                }
            }
        }
        Ok(Command::MapTree) => {
            if !config.permissions.allow_map_tree {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "That tool is disabled for Telegram remote access.",
                    )
                    .await;
                return Ok(());
            }
            match crate::workspace_map::render_tree(Some(2)) {
                Ok(tree) => {
                    let html = format!("<pre>{}</pre>", html_escape::encode_text(&tree));
                    let _ = bot
                        .send_message(msg.chat.id, html)
                        .parse_mode(ParseMode::Html)
                        .await;
                }
                Err(e) => {
                    let _ = bot
                        .send_message(msg.chat.id, format!("Failed to render map tree: {}", e))
                        .await;
                }
            }
        }
        Ok(Command::Approve { request_id }) => {
            let resp = format!(
                "Request approved: `{}`\n(Approval scaffolding executed successfully)",
                request_id
            );
            let _ = bot.send_message(msg.chat.id, resp).await;
        }
        Ok(Command::Deny { request_id }) => {
            let resp = format!(
                "Request denied: `{}`\n(Denial scaffolding executed successfully)",
                request_id
            );
            let _ = bot.send_message(msg.chat.id, resp).await;
        }
        Ok(Command::Profile { subcommand }) => {
            let sub = subcommand.trim();
            if sub.is_empty() || sub == "show" {
                match crate::profile::read_profile() {
                    Ok(p) => {
                        let resp = format!(
                            "=== User Profile V2 ===\n\n\
                             Display Name: {}\n\
                             Pronouns: {}\n\
                             Occupation/Role: {}\n\
                             Preferred Editor: {}\n\n\
                             Privacy Toggles:\n\
                             - Send Identity: {}\n\
                             - Send Location: {}\n\
                             - Send Technical: {}",
                            p.identity.display_name,
                            p.identity.pronouns,
                            p.personal.occupation_or_role,
                            p.technical.main_editor,
                            p.privacy.send_identity,
                            p.privacy.send_location,
                            p.privacy.send_technical
                        );
                        let _ = bot.send_message(msg.chat.id, resp).await;
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ Error: {}", e))
                            .await;
                    }
                }
            } else if let Some(stripped) = sub.strip_prefix("set ") {
                if !config.permissions.allow_profile_write {
                    let _ = bot
                        .send_message(
                            msg.chat.id,
                            "❌ Error: Remote profile modifications are disabled in telegram.toml.",
                        )
                        .await;
                    return Ok(());
                }

                let parts: Vec<&str> = stripped.splitn(2, ' ').collect();
                if parts.len() < 2 {
                    let _ = bot.send_message(msg.chat.id, "Usage: `/profile set <field> <value>`\nAllowed fields: display_name, pronouns, occupation, editor").await;
                    return Ok(());
                }

                let field = parts[0].trim();
                let value = parts[1].trim().to_string();

                match crate::profile::read_profile() {
                    Ok(mut p) => {
                        let field_changed;
                        match field {
                            "display_name" => {
                                p.identity.display_name = value.clone();
                                field_changed = "identity.display_name";
                            }
                            "pronouns" => {
                                p.identity.pronouns = value.clone();
                                field_changed = "identity.pronouns";
                            }
                            "occupation" => {
                                p.personal.occupation_or_role = value.clone();
                                field_changed = "personal.occupation_or_role";
                            }
                            "editor" => {
                                p.technical.main_editor = value.clone();
                                field_changed = "technical.main_editor";
                            }
                            _ => {
                                let _ = bot.send_message(msg.chat.id, "❌ Invalid field. Allowed fields: display_name, pronouns, occupation, editor").await;
                                return Ok(());
                            }
                        }

                        // Generate confirmation token
                        let token = uuid::Uuid::new_v4().to_string()[0..8].to_string();
                        let summary = format!("`{}` = `\"{}\"`", field_changed, value);

                        let pending = PendingConfirmation {
                            token: token.clone(),
                            chat_id,
                            action: "profile".to_string(),
                            summary: summary.clone(),
                            payload: serde_json::to_string(&p).unwrap_or_default(),
                            created_at: std::time::Instant::now(),
                        };

                        get_pending_confirmations().lock().unwrap().push(pending);

                        let confirm_msg = format!(
                            "⚠️ **Pending Profile Modification Request**\n\n\
                             Proposed Change: {}\n\n\
                             This sensitive remote action requires confirmation. To commit this change to disk, send this command within 60s:\n\
                             `/confirm {}`",
                            summary, token
                        );
                        let _ = bot.send_message(msg.chat.id, confirm_msg).await;
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ Error reading profile: {}", e))
                            .await;
                    }
                }
            } else {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "Usage: `/profile show` or `/profile set <field> <value>`",
                    )
                    .await;
            }
        }
        Ok(Command::Style { subcommand }) => {
            let sub = subcommand.trim();
            if sub.is_empty() || sub == "show" {
                match crate::style::read_style() {
                    Ok(s) => {
                        let resp = format!(
                            "=== Style Guidelines V2 ===\n\n\
                             Communication:\n\
                             - Tone: {}\n\
                             - Detail Level: {}\n\
                             - Use Examples: {}\n\n\
                             Coding Output:\n\
                             - Show Simple First: {}\n\
                             - Explain After Code: {}\n\n\
                             Formatting:\n\
                             - Use Markdown: {}\n\
                             - Use Short Sections: {}",
                            s.communication.tone,
                            s.communication.detail_level,
                            s.communication.use_examples,
                            s.coding.show_simple_solution_first,
                            s.coding.explain_after_code,
                            s.formatting.use_markdown,
                            s.formatting.use_short_sections
                        );
                        let _ = bot.send_message(msg.chat.id, resp).await;
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ Error: {}", e))
                            .await;
                    }
                }
            } else if let Some(stripped) = sub.strip_prefix("set ") {
                if !config.permissions.allow_style_write {
                    let _ = bot
                        .send_message(
                            msg.chat.id,
                            "❌ Error: Remote style modifications are disabled in telegram.toml.",
                        )
                        .await;
                    return Ok(());
                }

                let parts: Vec<&str> = stripped.splitn(2, ' ').collect();
                if parts.len() < 2 {
                    let _ = bot.send_message(msg.chat.id, "Usage: `/style set <tone|detail> <value>` or `/style set examples <true|false>`").await;
                    return Ok(());
                }

                let field = parts[0].trim();
                let value = parts[1].trim().to_string();

                match crate::style::read_style() {
                    Ok(mut s) => {
                        let field_changed;
                        match field {
                            "tone" => {
                                s.communication.tone = value.clone();
                                field_changed = "communication.tone";
                            }
                            "detail" => {
                                s.communication.detail_level = value.clone();
                                field_changed = "communication.detail_level";
                            }
                            "examples" => {
                                let b_val = value.parse().unwrap_or(false);
                                s.communication.use_examples = b_val;
                                field_changed = "communication.use_examples";
                            }
                            _ => {
                                let _ = bot
                                    .send_message(
                                        msg.chat.id,
                                        "❌ Invalid style field. Allowed: tone, detail, examples",
                                    )
                                    .await;
                                return Ok(());
                            }
                        }

                        // Generate confirmation token
                        let token = uuid::Uuid::new_v4().to_string()[0..8].to_string();
                        let summary = format!("`{}` = `\"{}\"`", field_changed, value);

                        let pending = PendingConfirmation {
                            token: token.clone(),
                            chat_id,
                            action: "style".to_string(),
                            summary: summary.clone(),
                            payload: serde_json::to_string(&s).unwrap_or_default(),
                            created_at: std::time::Instant::now(),
                        };

                        get_pending_confirmations().lock().unwrap().push(pending);

                        let confirm_msg = format!(
                            "⚠️ **Pending Style Modification Request**\n\n\
                             Proposed Change: {}\n\n\
                             This sensitive remote action requires confirmation. To commit this change to disk, send this command within 60s:\n\
                             `/confirm {}`",
                            summary, token
                        );
                        let _ = bot.send_message(msg.chat.id, confirm_msg).await;
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ Error reading style: {}", e))
                            .await;
                    }
                }
            } else {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "Usage: `/style show` or `/style set <field> <value>`",
                    )
                    .await;
            }
        }
        Ok(Command::Prefs) => match crate::preferences::read_preferences() {
            Ok(p) => {
                let mut resp = "=== Dynamic Preference Sections ===\n\n".to_string();
                for s in p.sections {
                    let status = if s.enabled { "✅" } else { "❌" };
                    let desc = s.description.as_deref().unwrap_or(&s.id);
                    resp.push_str(&format!(
                            "{} Section: {}\n  Send Policy: {}\n  Triggers: {:?}\n  Likes Count: {}\n\n",
                            status, desc, s.send_policy, s.triggers, s.likes.len()
                        ));
                }
                let _ = bot.send_message(msg.chat.id, resp).await;
            }
            Err(e) => {
                let _ = bot
                    .send_message(msg.chat.id, format!("❌ Error: {}", e))
                    .await;
            }
        },
        Ok(Command::Contexts { subcommand }) => {
            let sub = subcommand.trim();
            if sub.is_empty() || sub == "show" || sub == "list" {
                match crate::context::read_contexts() {
                    Ok(ctxs) => {
                        let mut resp = "=== Goal & Project Contexts ===\n\n".to_string();
                        for c in ctxs.contexts {
                            let status = if c.enabled { "✅" } else { "❌" };
                            resp.push_str(&format!(
                                "{} Context: {}\n  ID: `{}` | Kind: {}\n  Send Policy: {}\n  Triggers: {:?}\n\n",
                                status, c.title, c.id, c.kind, c.send_policy, c.triggers
                            ));
                        }
                        let _ = bot.send_message(msg.chat.id, resp).await;
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ Error: {}", e))
                            .await;
                    }
                }
            } else if let Some(stripped) = sub.strip_prefix("pin ") {
                if !config.permissions.allow_contexts_write {
                    let _ = bot.send_message(msg.chat.id, "❌ Error: Session context pin modifications are disabled in telegram.toml.").await;
                    return Ok(());
                }

                let context_id = stripped.trim().to_string();
                if context_id.is_empty() {
                    let _ = bot
                        .send_message(msg.chat.id, "❌ Usage: `/contexts pin <context_id>`")
                        .await;
                    return Ok(());
                }

                match sessions::init_db() {
                    Ok(conn) => {
                        let user_key = format!("telegram_{}", chat_id);
                        match sessions::get_active_session(&conn, &user_key) {
                            Ok(Some(sess_id)) => {
                                match sessions::pin_context(&conn, &sess_id, &context_id) {
                                    Ok(_) => {
                                        let _ = bot.send_message(msg.chat.id, format!("✅ Context '{}' successfully pinned to active session `{}`!", context_id, sess_id)).await;
                                    }
                                    Err(e) => {
                                        let _ = bot
                                            .send_message(
                                                msg.chat.id,
                                                format!("❌ Pinning error: {}", e),
                                            )
                                            .await;
                                    }
                                }
                            }
                            _ => {
                                let _ = bot.send_message(msg.chat.id, "❌ Error: No active chat session found. Please run `/new` or send a message first.").await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ DB Error: {}", e))
                            .await;
                    }
                }
            } else if let Some(stripped) = sub.strip_prefix("unpin ") {
                if !config.permissions.allow_contexts_write {
                    let _ = bot.send_message(msg.chat.id, "❌ Error: Session context pin modifications are disabled in telegram.toml.").await;
                    return Ok(());
                }

                let context_id = stripped.trim().to_string();
                if context_id.is_empty() {
                    let _ = bot
                        .send_message(msg.chat.id, "❌ Usage: `/contexts unpin <context_id>`")
                        .await;
                    return Ok(());
                }

                match sessions::init_db() {
                    Ok(conn) => {
                        let user_key = format!("telegram_{}", chat_id);
                        match sessions::get_active_session(&conn, &user_key) {
                            Ok(Some(sess_id)) => {
                                match sessions::unpin_context(&conn, &sess_id, &context_id) {
                                    Ok(_) => {
                                        let _ = bot.send_message(msg.chat.id, format!("✅ Context '{}' successfully unpinned from active session `{}`.", context_id, sess_id)).await;
                                    }
                                    Err(e) => {
                                        let _ = bot
                                            .send_message(
                                                msg.chat.id,
                                                format!("❌ Unpinning error: {}", e),
                                            )
                                            .await;
                                    }
                                }
                            }
                            _ => {
                                let _ = bot
                                    .send_message(
                                        msg.chat.id,
                                        "❌ Error: No active chat session found.",
                                    )
                                    .await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = bot
                            .send_message(msg.chat.id, format!("❌ DB Error: {}", e))
                            .await;
                    }
                }
            } else {
                let _ = bot
                    .send_message(
                        msg.chat.id,
                        "Usage: `/contexts show`, `/contexts pin <id>`, or `/contexts unpin <id>`",
                    )
                    .await;
            }
        }
        Ok(Command::Confirm { token }) => {
            let token = token.trim();
            let mut matched_pending = None;

            {
                let mut queue = get_pending_confirmations().lock().unwrap();
                // Clean up expired ones (older than 60s)
                let now = std::time::Instant::now();
                queue.retain(|c| now.duration_since(c.created_at).as_secs() < 60);

                if let Some(pos) = queue
                    .iter()
                    .position(|c| c.token == token && c.chat_id == chat_id)
                {
                    matched_pending = Some(queue.remove(pos));
                }
            }

            match matched_pending {
                Some(pending) => {
                    if pending.action == "profile" {
                        match serde_json::from_str::<crate::profile::Profile>(&pending.payload) {
                            Ok(p) => match crate::profile::save_profile(&p) {
                                Ok(_) => {
                                    let _ = bot.send_message(msg.chat.id, format!("✅ **Success**: Profile modification successfully written to disk!\nApplied: {}", pending.summary)).await;
                                }
                                Err(e) => {
                                    let _ = bot
                                        .send_message(
                                            msg.chat.id,
                                            format!("❌ Error saving profile to file: {}", e),
                                        )
                                        .await;
                                }
                            },
                            Err(e) => {
                                let _ = bot
                                    .send_message(
                                        msg.chat.id,
                                        format!("❌ Error deserializing profile payload: {}", e),
                                    )
                                    .await;
                            }
                        }
                    } else if pending.action == "style" {
                        match serde_json::from_str::<crate::style::OpenNivaraStyle>(
                            &pending.payload,
                        ) {
                            Ok(s) => match crate::style::save_style(&s) {
                                Ok(_) => {
                                    let _ = bot.send_message(msg.chat.id, format!("✅ **Success**: Style guidelines successfully written to disk!\nApplied: {}", pending.summary)).await;
                                }
                                Err(e) => {
                                    let _ = bot
                                        .send_message(
                                            msg.chat.id,
                                            format!("❌ Error saving style to file: {}", e),
                                        )
                                        .await;
                                }
                            },
                            Err(e) => {
                                let _ = bot
                                    .send_message(
                                        msg.chat.id,
                                        format!("❌ Error deserializing style payload: {}", e),
                                    )
                                    .await;
                            }
                        }
                    } else {
                        let _ = bot
                            .send_message(
                                msg.chat.id,
                                "❌ Error: Unrecognized confirmation action.",
                            )
                            .await;
                    }
                }
                None => {
                    let _ = bot.send_message(msg.chat.id, "❌ Error: Token not found or has expired (60s limit). Please check your command and try again.").await;
                }
            }
        }
        Err(_) => {
            // Treat general text messages as natural multi-turn chat input
            handle_engine_request(
                &bot,
                msg.chat.id,
                chat_id,
                username,
                Some(text.to_string()),
                &config,
            )
            .await;
        }
    }

    Ok(())
}

/// Helper function to coordinate engine request loops and split response outputs safely.
async fn handle_engine_request(
    bot: &Bot,
    chat_dest: ChatId,
    chat_id: i64,
    username: Option<String>,
    message: Option<String>,
    config: &TelegramConfig,
) {
    let msg_text = match message {
        Some(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                let _ = bot
                    .send_message(chat_dest, "Please provide some chat details.")
                    .await;
                return;
            }
            trimmed.to_string()
        }
        None => {
            let _ = bot
                .send_message(chat_dest, "Please provide some chat details.")
                .await;
            return;
        }
    };

    // Indicate that the bot is working
    let _ = bot
        .send_chat_action(chat_dest, teloxide::types::ChatAction::Typing)
        .await;

    let engine = crate::engine::OpenNivaraEngine::new();
    let request = crate::engine::EngineRequest {
        source: crate::engine::RequestSource::Telegram { chat_id, username },
        session_id: None,
        message: msg_text,
    };

    match engine.handle_message(request).await {
        Ok(response) => {
            let html = crate::output::telegram::markdown_to_telegram_html(&response.answer);
            let parts = crate::output::telegram::split_telegram_html_message(
                &html,
                config.limits.max_response_chars,
            );

            for part in parts {
                match bot
                    .send_message(chat_dest, &part)
                    .parse_mode(ParseMode::Html)
                    .await
                {
                    Ok(_) => {}
                    Err(_) => {
                        // Fallback path in case Telegram parse mode encounters tag errors
                        let plain = crate::output::telegram::strip_basic_html(&part);
                        let _ = bot.send_message(chat_dest, plain).await;
                    }
                }
            }
        }
        Err(e) => {
            let _ = bot
                .send_message(chat_dest, format!("❌ Error: {}", e))
                .await;
        }
    }
}
