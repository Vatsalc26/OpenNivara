use std::path::PathBuf;

#[cfg(test)]
pub(crate) static TEST_CONFIG_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn config_dir() -> anyhow::Result<PathBuf> {
    if let Ok(test_path) = std::env::var("OPENNIVARA_TEST_CONFIG_DIR") {
        return Ok(PathBuf::from(test_path));
    }

    let proj_dirs = directories::ProjectDirs::from("io.github", "Vatsalc26", "OpenNivara")
        .ok_or_else(|| anyhow::anyhow!("Failed to locate OS config/home directories"))?;
    Ok(proj_dirs.config_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_prefers_opennivara_test_config_dir() {
        let _lock = TEST_CONFIG_ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        std::env::set_var("OPENNIVARA_TEST_CONFIG_DIR", temp_dir.path());

        let resolved = config_dir().expect("config dir");

        assert_eq!(resolved, temp_dir.path());
        std::env::remove_var("OPENNIVARA_TEST_CONFIG_DIR");
    }
}
