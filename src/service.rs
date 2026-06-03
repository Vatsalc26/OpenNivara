/// Handles installing the OpenNivara daemon as an OS system service.
pub fn service_install() -> anyhow::Result<()> {
    println!("\x1b[1;36m=== OpenNivara OS Service Management ===\x1b[0m");
    println!("Installing OpenNivara Daemon as an OS service...");
    println!("\n> [!NOTE]");
    println!("> Service management is not fully implemented yet. Use `opennivara daemon` for now.");
    Ok(())
}

/// Handles starting the background OpenNivara service.
pub fn service_start() -> anyhow::Result<()> {
    println!("\x1b[1;36m=== OpenNivara OS Service Management ===\x1b[0m");
    println!("Starting OpenNivara Daemon service...");
    println!("\n> [!NOTE]");
    println!("> Service management is not fully implemented yet. Use `opennivara daemon` for now.");
    Ok(())
}

/// Handles stopping the running OpenNivara service.
pub fn service_stop() -> anyhow::Result<()> {
    println!("\x1b[1;36m=== OpenNivara OS Service Management ===\x1b[0m");
    println!("Stopping OpenNivara Daemon service...");
    println!("\n> [!NOTE]");
    println!("> Service management is not fully implemented yet. Use `opennivara daemon` for now.");
    Ok(())
}

/// Handles uninstalling the OpenNivara service.
pub fn service_uninstall() -> anyhow::Result<()> {
    println!("\x1b[1;36m=== OpenNivara OS Service Management ===\x1b[0m");
    println!("Uninstalling OpenNivara Daemon service...");
    println!("\n> [!NOTE]");
    println!("> Service management is not fully implemented yet. Use `opennivara daemon` for now.");
    Ok(())
}

/// Retrieves the status of the background OpenNivara service.
pub fn service_status() -> anyhow::Result<()> {
    println!("\x1b[1;36m=== OpenNivara OS Service Management ===\x1b[0m");
    println!("Service Status: Not installed / inactive.");
    println!("\n> [!NOTE]");
    println!("> Service management is not fully implemented yet. Use `opennivara daemon` for now.");
    Ok(())
}
