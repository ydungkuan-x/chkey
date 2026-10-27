use serde::{Deserialize, Serialize};

use crate::defaults;

/// Application configuration structure.
///
/// Designed to be shared across platforms (Windows, macOS, Linux) without dependencies
/// on Slint UI or operating system APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    #[serde(default = "defaults::default_config_version")]
    pub config_version: u32,

    #[serde(default = "defaults::default_language")]
    pub language: String,

    #[serde(default = "defaults::default_encoding")]
    pub encoding: String,

    #[serde(default = "defaults::default_input_method")]
    pub input_method: String,

    #[serde(default = "defaults::default_keyboard_enabled")]
    pub keyboard_enabled: bool,

    #[serde(default = "defaults::default_run_on_startup")]
    pub run_on_startup: bool,

    #[serde(default = "defaults::default_show_dialog_on_startup")]
    pub show_dialog_on_startup: bool,

    #[serde(default = "defaults::default_auto_update")]
    pub auto_update: bool,

    #[serde(default = "defaults::default_dark_mode")]
    pub dark_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_version: defaults::default_config_version(),
            language: defaults::default_language(),
            encoding: defaults::default_encoding(),
            input_method: defaults::default_input_method(),
            keyboard_enabled: defaults::default_keyboard_enabled(),
            run_on_startup: defaults::default_run_on_startup(),
            show_dialog_on_startup: defaults::default_show_dialog_on_startup(),
            auto_update: defaults::default_auto_update(),
            dark_mode: defaults::default_dark_mode(),
        }
    }
}
