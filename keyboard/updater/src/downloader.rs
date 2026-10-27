use crate::model::{DownloadProgress, UpdateInfo};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn download_update<F>(
    info: &UpdateInfo,
    destination: impl AsRef<Path>,
    mut on_progress: F,
) -> Result<PathBuf, String>
where
    F: FnMut(DownloadProgress) + Send,
{
    let destination = destination.as_ref().to_path_buf();

    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Không tạo được thư mục: {e}"))?;
    }

    // Tải vào file tạm để tránh nhầm file chưa tải xong với file hoàn chỉnh.
    let mut temporary = destination.clone();
    temporary.set_extension("download");

    let result = download_to_file(
        info,
        &temporary,
        &destination,
        &mut on_progress,
    )
    .await;

    if result.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
    }

    result
}

async fn download_to_file<F>(
    info: &UpdateInfo,
    temporary: &Path,
    destination: &Path,
    on_progress: &mut F,
) -> Result<PathBuf, String>
where
    F: FnMut(DownloadProgress) + Send,
{
    let response = crate::http_client()
        .get(&info.download_url)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
        .map_err(|e| format!("Không tải được file: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Tải file thất bại: HTTP {}",
            response.status()
        ));
    }

    let total = response.content_length();
    let mut stream = response.bytes_stream();

    let mut file = tokio::fs::File::create(temporary)
        .await
        .map_err(|e| format!("Không tạo được file: {e}"))?;

    let mut downloaded = 0_u64;
    let mut hasher = Sha256::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Lỗi khi tải file: {e}"))?;

        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Không ghi được file: {e}"))?;

        hasher.update(&chunk);
        downloaded += chunk.len() as u64;

        let percent = total
            .filter(|size| *size > 0)
            .map(|size| (downloaded as f64 / size as f64 * 100.0)
                .min(100.0) as f32);

        on_progress(DownloadProgress::Downloading {
            downloaded,
            total,
            percent,
        });
    }

    file.flush()
        .await
        .map_err(|e| format!("Không ghi hoàn chỉnh file: {e}"))?;

    file.sync_all()
        .await
        .map_err(|e| format!("Không đồng bộ được file: {e}"))?;

    drop(file);

    // Kiểm tra SHA-256 trước khi công nhận tải thành công.
    let actual_hash = format!("{:x}", hasher.finalize());
    let expected_hash = info.sha256.trim();

    if !expected_hash.is_empty() && expected_hash != "..." && !actual_hash.eq_ignore_ascii_case(expected_hash) {
        return Err(format!(
            "SHA-256 không khớp. Mong đợi: {}, thực tế: {}",
            info.sha256, actual_hash
        ));
    }

    tokio::fs::rename(temporary, destination)
        .await
        .map_err(|e| format!("Không thể hoàn tất file tải về: {e}"))?;

    on_progress(DownloadProgress::Completed {
        path: destination.to_path_buf(),
    });

    Ok(destination.to_path_buf())
}