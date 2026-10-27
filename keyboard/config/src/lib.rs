pub mod defaults;
pub mod model;
pub mod path;
pub mod storage;

pub use model::AppConfig;
pub use path::{get_config_dir, get_config_path, PathError};
pub use storage::{
    load_config, load_config_from_path, reset_config, reset_config_at_path, save_config,
    save_config_to_path, ConfigError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.config_version, 1);
        assert_eq!(config.language, "vi");
        assert_eq!(config.encoding, "unicode");
        assert_eq!(config.input_method, "ede");
        assert!(config.keyboard_enabled);
        assert!(config.run_on_startup);
        assert!(config.show_dialog_on_startup);
        assert!(config.auto_update);
        assert!(config.dark_mode);
    }

    #[test]
    fn test_toml_serialization_and_deserialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).expect("Serialize toml failed");
        let deserialized: AppConfig = toml::from_str(&toml_str).expect("Deserialize toml failed");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_missing_fields_defaulting() {
        let partial_toml = r#"
            language = "en"
            input_method = "eng"
        "#;
        let config: AppConfig = toml::from_str(partial_toml).expect("Deserialization failed");
        assert_eq!(config.language, "en");
        assert_eq!(config.input_method, "eng");
        // Remaining fields should have default values
        assert_eq!(config.config_version, 1);
        assert_eq!(config.encoding, "unicode");
        assert!(config.keyboard_enabled);
        assert!(config.run_on_startup);
        assert!(config.show_dialog_on_startup);
        assert!(config.auto_update);
        assert!(config.dark_mode);
    }

    #[test]
    fn test_first_launch_creates_default_config() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let config_file = temp_dir.path().join("CHKey").join("config.toml");

        assert!(!config_file.exists());
        let loaded = load_config_from_path(&config_file).expect("Failed to load config");
        assert_eq!(loaded, AppConfig::default());
        assert!(config_file.exists());
    }

    #[test]
    fn test_save_and_reload() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let config_file = temp_dir.path().join("config.toml");

        let config = AppConfig {
            language: "ede".to_string(),
            dark_mode: false,
            run_on_startup: true,
            ..Default::default()
        };

        save_config_to_path(&config_file, &config).expect("Failed to save config");

        let reloaded = load_config_from_path(&config_file).expect("Failed to reload config");
        assert_eq!(config, reloaded);
    }

    #[test]
    fn test_invalid_toml_preserves_file_and_errors() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let config_file = temp_dir.path().join("config.toml");

        let invalid_toml = "language = [corrupt-toml syntax";
        fs::write(&config_file, invalid_toml).expect("Failed to write invalid config");

        let result = load_config_from_path(&config_file);
        assert!(result.is_err());

        // Verify the original file content was preserved
        let content_after = fs::read_to_string(&config_file).expect("Failed to read file");
        assert_eq!(content_after, invalid_toml);
    }

    #[test]
    fn test_reset_config() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let config_file = temp_dir.path().join("config.toml");

        let custom_config = AppConfig {
            language: "en".to_string(),
            ..Default::default()
        };
        save_config_to_path(&config_file, &custom_config).expect("Failed to save custom config");

        let reset = reset_config_at_path(&config_file).expect("Failed to reset config");
        assert_eq!(reset, AppConfig::default());

        let reloaded = load_config_from_path(&config_file).expect("Failed to reload config");
        assert_eq!(reloaded, AppConfig::default());
    }

    #[test]
    fn test_unsupported_future_version() {
        let temp_dir = tempdir().expect("Failed to create tempdir");
        let config_file = temp_dir.path().join("config.toml");

        let future_toml = r#"
            config_version = 999
            language = "vi"
        "#;
        fs::write(&config_file, future_toml).expect("Failed to write future version config");

        let result = load_config_from_path(&config_file);
        match result {
            Err(ConfigError::UnsupportedVersion {
                version,
                max_supported,
            }) => {
                assert_eq!(version, 999);
                assert_eq!(max_supported, 1);
            }
            _ => panic!("Expected UnsupportedVersion error"),
        }
    }

    #[test]
    fn test_path_resolution() {
        let config_path_res = get_config_path();
        assert!(config_path_res.is_ok());
        let path = config_path_res.unwrap();
        assert!(path.ends_with("config.toml"));
    }
}
