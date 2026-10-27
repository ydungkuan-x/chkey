use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::defaults::CURRENT_CONFIG_VERSION;
use crate::model::AppConfig;
use crate::path::{get_config_path, PathError};

#[derive(Debug)]
pub enum ConfigError {
    Path(PathError),
    Io(io::Error),
    Serialize(toml::ser::Error),
    Deserialize(toml::de::Error),
    UnsupportedVersion { version: u32, max_supported: u32 },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Path(e) => write!(f, "Configuration path error: {e}"),
            ConfigError::Io(e) => write!(f, "Configuration I/O error: {e}"),
            ConfigError::Serialize(e) => write!(f, "Failed to serialize configuration: {e}"),
            ConfigError::Deserialize(e) => write!(f, "Failed to deserialize configuration: {e}"),
            ConfigError::UnsupportedVersion {
                version,
                max_supported,
            } => write!(
                f,
                "Unsupported configuration version: {version} (maximum supported: {max_supported})"
            ),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Path(e) => Some(e),
            ConfigError::Io(e) => Some(e),
            ConfigError::Serialize(e) => Some(e),
            ConfigError::Deserialize(e) => Some(e),
            ConfigError::UnsupportedVersion { .. } => None,
        }
    }
}

impl From<PathError> for ConfigError {
    fn from(err: PathError) -> Self {
        ConfigError::Path(err)
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::Serialize(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Deserialize(err)
    }
}

/// Helper function to load configuration from an explicit path.
pub fn load_config_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    if !path.exists() {
        let default_cfg = AppConfig::default();
        save_config_to_path(path, &default_cfg)?;
        return Ok(default_cfg);
    }

    let contents = fs::read_to_string(path)?;
    let mut config: AppConfig = toml::from_str(&contents)?;

    if config.config_version > CURRENT_CONFIG_VERSION {
        return Err(ConfigError::UnsupportedVersion {
            version: config.config_version,
            max_supported: CURRENT_CONFIG_VERSION,
        });
    }

    // Future version migration logic can be hooked here if config_version < CURRENT_CONFIG_VERSION
    if config.config_version < CURRENT_CONFIG_VERSION {
        config.config_version = CURRENT_CONFIG_VERSION;
        // Do not fail if rewriting fails; preserve migration output safely
        let _ = save_config_to_path(path, &config);
    }

    Ok(config)
}

/// Helper function to safely save configuration to an explicit path using atomic file replacement.
pub fn save_config_to_path(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    let toml_string = toml::to_string_pretty(config)?;

    // Atomic write via temporary file in the same directory to avoid half-written files
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let temp_file_path: PathBuf = parent.join(format!(
        ".config.tmp-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));

    fs::write(&temp_file_path, toml_string)?;

    if fs::rename(&temp_file_path, path).is_err() {
        // Fallback for filesystems where atomic rename fails or cannot overwrite existing file
        if let Err(fallback_err) = fs::copy(&temp_file_path, path) {
            let _ = fs::remove_file(&temp_file_path);
            return Err(ConfigError::Io(fallback_err));
        }
        let _ = fs::remove_file(&temp_file_path);
        return Ok(());
    }

    Ok(())
}

/// Helper function to reset configuration to defaults at an explicit path.
pub fn reset_config_at_path(path: &Path) -> Result<AppConfig, ConfigError> {
    let default_cfg = AppConfig::default();
    save_config_to_path(path, &default_cfg)?;
    Ok(default_cfg)
}

/// Loads configuration from the standard cross-platform path.
/// If the file does not exist, writes the default configuration and returns it.
pub fn load_config() -> Result<AppConfig, ConfigError> {
    let path = get_config_path()?;
    load_config_from_path(&path)
}

/// Saves configuration to the standard cross-platform path.
pub fn save_config(config: &AppConfig) -> Result<(), ConfigError> {
    let path = get_config_path()?;
    save_config_to_path(&path, config)
}

/// Resets configuration to defaults and saves to standard cross-platform path.
pub fn reset_config() -> Result<AppConfig, ConfigError> {
    let path = get_config_path()?;
    reset_config_at_path(&path)
}
