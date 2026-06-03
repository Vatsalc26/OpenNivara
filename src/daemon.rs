use crate::remote_policy;

/// Runs the always-on OpenNivara daemon, initializing structured tracing and launching the Telegram bot.
pub async fn run_daemon() -> anyhow::Result<()> {
    // 1. Set up structured logging for the background runner
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 2. Read configuration rules
    let telegram_config = remote_policy::read_telegram()?;

    if !telegram_config.general.enabled {
        println!("OpenNivara daemon is currently disabled in telegram.toml.");
        return Ok(());
    }

    // 3. Inspect Workspace Map presence
    let has_map = if let Ok(db_path) = crate::workspace_map::get_db_path() {
        db_path.exists()
    } else {
        false
    };

    let map_status = if has_map { "available" } else { "unavailable" };

    // 4. Print beautiful start banner
    println!("OpenNivara daemon started.");
    println!("Telegram: enabled");
    println!(
        "Authorized chats: {}",
        telegram_config.auth.allowed_chat_ids.len()
    );
    println!("Workspace map: {}", map_status);
    println!("Press Ctrl+C to stop.");

    // 5. Start Teloxide Bot Polling
    crate::telegram::start_bot().await?;

    Ok(())
}
