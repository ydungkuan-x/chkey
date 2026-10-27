use fluent::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn new(language: &str) -> Self {
        let lang_id: LanguageIdentifier = language.parse().expect("Ngôn ngữ không hợp lệ");

        let mut bundle = FluentBundle::new(vec![lang_id]);

        let content = match language {
            "en" => include_str!("../locales/en/main.ftl"),
            "ede" => include_str!("../locales/ede/main.ftl"),
            _ => include_str!("../locales/vi/main.ftl"),
        };

        let resource =
            FluentResource::try_new(content.to_string()).expect("Không thể đọc file locale");

        bundle
            .add_resource(resource)
            .expect("Không thể thêm locale");

        Self { bundle }
    }

    pub fn get(&self, key: &str) -> String {
        let Some(message) = self.bundle.get_message(key) else {
            return key.to_string();
        };

        let Some(pattern) = message.value() else {
            return key.to_string();
        };

        let mut errors = Vec::new();

        self.bundle
            .format_pattern(pattern, None, &mut errors)
            .to_string()
    }

    pub fn to_ui_texts(&self) -> chkey_ui::I18nTexts {
        chkey_ui::I18nTexts {
           
            app_name: self.get("app-name").into(),

            // Bảng điều khiển
            dashboard: self.get("dashboard").into(),
            encoding: self.get("encoding").into(),
            typing_mode: self.get("typing-mode").into(),

            // Cài đặt
            settings: self.get("settings").into(),
            auto_update: self.get("auto-update").into(),
            run_on_startup: self.get("run-on-startup").into(),
            show_dialog_on_startup: self.get("show-dialog-on-startup").into(),

            // Hành động
            close_window: self.get("close-window").into(),
            quit_app: self.get("quit-app").into(),
            help: self.get("help").into(),
            restore_defaults: self.get("restore-defaults").into(),
            check_for_updates: self.get("check-for-updates").into(),

            // Ngôn ngữ giao diện
            language: self.get("language").into(),
            language_vi: self.get("language-vi").into(),
            language_en: self.get("language-en").into(),

            // Thông tin
            about: self.get("about").into(),
            about_description: self.get("about-description").into(),
            about_guide: self.get("about-guide").into(),
            about_website: self.get("about-website").into(),
            about_email: self.get("about-email").into(),
            about_version: self.get("about-version").into(),
            about_check_for_updates: self.get("about-check-for-updates").into(),
            latest_version: self.get("latest-version").into(),

            about_copyright: format!("{} 2026 - Y Dũng Kuan", self.get("about-copyright")).into(),

            // Cập nhật phiên bản
            update_title: self.get("update-title").into(),
            update_check_title: self.get("update-check-title").into(),
            update_checking: self.get("update-checking").into(),
            update_available: self.get("update-available").into(),
            update_current_version: self.get("update-current-version").into(),
            update_latest: self.get("update-latest").into(),
            update_btn: self.get("update-btn").into(),
            update_downloading: self.get("update-downloading").into(),
            install_now: self.get("install-now").into(),
            update_download_starting: self.get("update-download-starting").into(),
            update_downloading_progress: self.get("update-downloading-progress").into(),
            update_download_success: self.get("update-download-success").into(),
            update_download_failed: self.get("update-download-failed").into(),
            update_check_error: self.get("update-check-error").into(),
        }
    }
}

pub fn load_ui_texts(lang: &str) -> chkey_ui::I18nTexts {
    I18n::new(lang).to_ui_texts()
}
