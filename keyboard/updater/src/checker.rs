use crate::model::{UpdateInfo, UpdateManifest, UpdateStatus};
use crate::platform::current_platform;

const UPDATE_URL: &str = "http://dl.chkey.org/latest.json";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn check_update() -> Result<UpdateStatus, String> {
    let platform = current_platform()?;
    println!("Platform hiện tại: {}", platform);

    let response = crate::http_client()
        .get(UPDATE_URL)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Không thể kết nối server: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Server trả về HTTP {}", response.status()));
    }

    let manifest: UpdateManifest = response
        .json()
        .await
        .map_err(|e| format!("Không đọc được manifest: {e}"))?;

    if manifest.app_id != "chkey" {
        return Err("Manifest không thuộc CHKey".to_string());
    }

    if manifest.channel != "stable" {
        return Err("Kênh cập nhật không hợp lệ".to_string());
    }

    let release = manifest
        .platforms
        .get(platform)
        .ok_or_else(|| format!("Không có bản phát hành cho {platform}"))?;

    let current = semver::Version::parse(CURRENT_VERSION).map_err(|e| e.to_string())?;

    let latest = semver::Version::parse(&release.version).map_err(|e| e.to_string())?;

    if latest <= current {
        return Ok(UpdateStatus::UpToDate);
    }

    let download_url = reqwest::Url::parse(UPDATE_URL)
        .map_err(|e| e.to_string())?
        .join(&release.url)
        .map_err(|e| e.to_string())?
        .to_string();

    Ok(UpdateStatus::UpdateAvailable(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version: release.version.clone(),
        platform: platform.to_string(),
        download_url,
        sha256: release.sha256.clone(),
    }))
}
