pub const CURRENT_CONFIG_VERSION: u32 = 1;

pub fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

pub fn default_language() -> String {
    "vi".to_string()
}

pub fn default_encoding() -> String {
    "unicode".to_string()
}

pub fn default_input_method() -> String {
    "ede".to_string()
}

pub fn default_keyboard_enabled() -> bool {
    true
}

pub fn default_run_on_startup() -> bool {
    true
}

pub fn default_show_dialog_on_startup() -> bool {
    true
}

pub fn default_auto_update() -> bool {
    true
}

pub fn default_dark_mode() -> bool {
    true
}
