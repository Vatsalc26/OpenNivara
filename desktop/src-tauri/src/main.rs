// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::var("OPENNIVARA_DISABLE_GPU").ok().as_deref() == Some("1")
        && std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").is_err()
    {
        std::env::set_var(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--disable-gpu --disable-gpu-compositing",
        );
    }

    opennivara::load_env();
    desktop_lib::run()
}
