use std::path::PathBuf;
use directories::ProjectDirs;

pub const QUALIFIER: &str = "";
pub const ORGANIZATION: &str = "";
pub const APPLICATION_NAME: &str = "CHKey";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug)]
pub enum PathError {
    DirectoryNotFound,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::DirectoryNotFound => {
                write!(f, "Could not determine standard configuration directory")
            }
        }
    }
}

impl std::error::Error for PathError {}

/// Resolves standard config directory for the application:
/// - Windows: %LOCALAPPDATA%\CHKey (with automatic migration from %APPDATA%\CHKey if present)
/// - macOS: ~/Library/Application Support/CHKey
/// - Linux: $XDG_CONFIG_HOME/chkey or ~/.config/chkey
pub fn get_config_dir() -> Result<PathBuf, PathError> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let local_dir = PathBuf::from(&local_app_data).join("CHKey");
            let local_cfg = local_dir.join(CONFIG_FILE_NAME);

            // Tự động di chuyển (copy) cấu hình cũ từ %APPDATA%\CHKey nếu người dùng đã có từ trước
            if !local_cfg.exists() {
                if let Ok(roaming_app_data) = std::env::var("APPDATA") {
                    let roaming_cfg = PathBuf::from(&roaming_app_data).join("CHKey").join(CONFIG_FILE_NAME);
                    if roaming_cfg.exists() {
                        let _ = std::fs::create_dir_all(&local_dir);
                        let _ = std::fs::copy(&roaming_cfg, &local_cfg);
                    }
                }
            }

            return Ok(local_dir);
        }
    }

    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION_NAME) {
        Ok(proj_dirs.config_dir().to_path_buf())
    } else {
        Err(PathError::DirectoryNotFound)
    }
}

/// Resolves the absolute path to the configuration file `config.toml`.
pub fn get_config_path() -> Result<PathBuf, PathError> {
    let dir = get_config_dir()?;
    Ok(dir.join(CONFIG_FILE_NAME))
}
