use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateManifest {
    pub app_id: String,
    pub channel: String,
    pub platforms: std::collections::HashMap<String, PlatformRelease>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformRelease {
    pub version: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub platform: String,
    pub download_url: String,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable(UpdateInfo),
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    /// Đang tải file.
    Downloading {
        downloaded: u64,
        total: Option<u64>,
        percent: Option<f32>,
    },

    /// Đã tải xong và kiểm tra SHA-256 thành công.
    Completed {
        path: std::path::PathBuf,
    },
}