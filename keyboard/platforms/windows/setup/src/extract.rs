//! Quản lý việc trích xuất DLL từ dữ liệu nhúng và xác minh tính toàn vẹn.
//!
//! Đảm nhiệm:
//! - Nhúng nhị phân `chkey_ime.dll` vào chương trình qua `include_bytes!`.
//! - Xác định thư mục runtime chuẩn: `%LOCALAPPDATA%\CHKey\runtime\`.
//! - Kiểm tra tính toàn vẹn và phiên bản bằng SHA-256.
//! - Trích xuất an toàn với cơ chế file tạm và xử lý khóa file DLL trên Windows (.old rename).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

/// Dữ liệu nhị phân của chkey_tsf.dll được nhúng tại thời điểm biên dịch.
pub const EMBEDDED_DLL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/chkey_tsf.dll"));

/// Dữ liệu nhị phân của chkey.ico được nhúng tại thời điểm biên dịch.
pub const EMBEDDED_ICO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/chkey.ico"));

/// Dữ liệu nhị phân của rad.ico được nhúng tại thời điểm biên dịch.
pub const EMBEDDED_RAD_ICO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rad.ico"));

/// Tính giá trị mã băm SHA-256 của một lát cắt byte.
pub fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Chuyển mảng SHA-256 32-byte thành chuỗi ký tự Hexadecimal (chữ thường).
pub fn sha256_to_hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Trả về mã băm SHA-256 của file DLL nhúng.
pub fn embedded_dll_sha256() -> [u8; 32] {
    compute_sha256(EMBEDDED_DLL)
}

/// Trả về chuỗi Hex SHA-256 của file DLL nhúng.
pub fn embedded_dll_sha256_hex() -> String {
    sha256_to_hex(&embedded_dll_sha256())
}

/// Đọc và tính SHA-256 của file trên ổ đĩa.
pub fn file_sha256(path: &Path) -> std::io::Result<[u8; 32]> {
    let data = fs::read(path)?;
    Ok(compute_sha256(&data))
}

/// Đọc và trả về chuỗi Hex SHA-256 của file trên ổ đĩa.
pub fn file_sha256_hex(path: &Path) -> Result<String, String> {
    file_sha256(path)
        .map(|h| sha256_to_hex(&h))
        .map_err(|e| format!("Không thể tính SHA-256 của '{}': {e}", path.display()))
}

/// Lấy đường dẫn thư mục gốc dữ liệu CHKey: `%LOCALAPPDATA%\CHKey\`.
pub fn get_chkey_data_dir() -> Result<PathBuf, String> {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        Ok(PathBuf::from(local_app_data).join("CHKey"))
    } else {
        std::env::current_dir()
            .map(|p| p.join("CHKey_Data"))
            .map_err(|e| format!("Không thể xác định thư mục làm việc: {e}"))
    }
}

/// Lấy đường dẫn thư mục runtime chuẩn: `%LOCALAPPDATA%\CHKey\runtime\`.
pub fn get_runtime_dir() -> Result<PathBuf, String> {
    Ok(get_chkey_data_dir()?.join("runtime"))
}

/// Lấy đường dẫn file runtime DLL chuẩn: `%LOCALAPPDATA%\CHKey\runtime\chkey_tsf.dll`.
pub fn get_runtime_dll_path() -> Result<PathBuf, String> {
    Ok(get_runtime_dir()?.join("chkey_tsf.dll"))
}

/// Kiểm tra xem file DLL trong thư mục runtime đã tồn tại và đúng phiên bản (trùng SHA-256) hay chưa.
pub fn is_runtime_dll_up_to_date() -> bool {
    if let Ok(dll_path) = get_runtime_dll_path() {
        if dll_path.exists() {
            if let Ok(current_hash) = file_sha256(&dll_path) {
                return current_hash == embedded_dll_sha256();
            }
        }
    }
    false
}

/// Dọn dẹp các file `.old` cũ trong thư mục runtime nếu không còn bị tiến trình nào khóa.
fn cleanup_old_dll_files(runtime_dir: &Path) {
    if let Ok(entries) = fs::read_dir(runtime_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("chkey_tsf.dll.old")
                    || file_name.starts_with("chkey_ime.dll.old")
                    || file_name.starts_with("chkey_ime.dll")
                {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }
}

/// Trích xuất `chkey_tsf.dll` từ dữ liệu nhúng ra `%LOCALAPPDATA%\CHKey\runtime\chkey_tsf.dll`.
///
/// Các bước bảo đảm an toàn:
/// 1. Nếu file đích đã tồn tại và SHA-256 hoàn toàn trùng khớp với bản nhúng, bỏ qua trích xuất.
/// 2. Tạo thư mục đích nếu chưa tồn tại.
/// 3. Ghi dữ liệu ra file tạm `.tmp-<timestamp>-<pid>` trong cùng thư mục.
/// 4. Đồng bộ dữ liệu xuống đĩa (flush/sync) và xác minh SHA-256 của file tạm.
/// 5. Nếu file đích đang tồn tại, đổi tên file cũ sang `.old-<timestamp>` để tránh lỗi khóa file của Windows.
/// 6. Đổi tên file tạm thành `chkey_tsf.dll`.
/// 7. Cố gắng xóa các file `.old` cũ nếu không còn bị khóa.
pub fn extract_dll() -> Result<PathBuf, String> {
    let runtime_dir = get_runtime_dir()?;
    fs::create_dir_all(&runtime_dir)
        .map_err(|e| format!("Không thể tạo thư mục runtime '{}': {e}", runtime_dir.display()))?;

    let target_dll = runtime_dir.join("chkey_tsf.dll");
    let embedded_hash = embedded_dll_sha256();

    // 1. Kiểm tra file hiện có
    if target_dll.exists() {
        if let Ok(current_hash) = file_sha256(&target_dll) {
            if current_hash == embedded_hash {
                // File đã tồn tại và hoàn toàn khớp phiên bản
                return Ok(target_dll);
            }
        }
    }

    // 2. Ghi ra file tạm thời trong cùng thư mục
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_dll = runtime_dir.join(format!("chkey_tsf.dll.tmp-{now}-{}", std::process::id()));

    {
        let mut f = File::create(&tmp_dll)
            .map_err(|e| format!("Không thể tạo file tạm '{}': {e}", tmp_dll.display()))?;
        f.write_all(EMBEDDED_DLL)
            .map_err(|e| format!("Lỗi ghi dữ liệu DLL vào '{}': {e}", tmp_dll.display()))?;
        f.sync_all()
            .map_err(|e| format!("Lỗi flush dữ liệu file tạm '{}': {e}", tmp_dll.display()))?;
    }

    // 3. Xác minh toàn vẹn SHA-256 của file tạm vừa ghi
    let tmp_hash = file_sha256(&tmp_dll).map_err(|e| {
        let _ = fs::remove_file(&tmp_dll);
        format!("Không thể xác minh SHA-256 file tạm '{}': {e}", tmp_dll.display())
    })?;

    if tmp_hash != embedded_hash {
        let _ = fs::remove_file(&tmp_dll);
        return Err(format!(
            "Mã băm SHA-256 của file tạm không khớp với bản nhúng (kỳ vọng: {}, thực tế: {})",
            sha256_to_hex(&embedded_hash),
            sha256_to_hex(&tmp_hash)
        ));
    }

    // 4. Nếu file đích đang tồn tại, xử lý khóa file an toàn bằng cách rename sang .old
    if target_dll.exists() {
        let old_dll = runtime_dir.join(format!("chkey_tsf.dll.old-{now}"));
        if let Err(e) = fs::rename(&target_dll, &old_dll) {
            // Nếu không thể rename (ví dụ bị khóa hoàn toàn độc quyền), thử ghi đè trực tiếp
            eprintln!(
                "[Setup] Không thể đổi tên DLL cũ sang '{}': {e}. Thử ghi đè trực tiếp...",
                old_dll.display()
            );
        }
    }

    // 5. Đưa file tạm vào vị trí đích
    if let Err(e) = fs::rename(&tmp_dll, &target_dll) {
        // Fallback: nếu rename thất bại, thử copy rồi xóa tmp
        if let Err(copy_err) = fs::copy(&tmp_dll, &target_dll) {
            let _ = fs::remove_file(&tmp_dll);
            return Err(format!(
                "Không thể thay thế '{}' bằng file mới: rename lỗi ({e}), copy lỗi ({copy_err})",
                target_dll.display()
            ));
        }
        let _ = fs::remove_file(&tmp_dll);
    }

    // 6. Trích xuất file icon rade.ico/rad.ico và chkey.ico để Windows hiển thị trên Language Flyout
    let target_rad_ico = runtime_dir.join("rad.ico");
    let _ = fs::write(&target_rad_ico, EMBEDDED_RAD_ICO);
    let target_rade_ico = runtime_dir.join("rade.ico");
    let _ = fs::write(&target_rade_ico, EMBEDDED_RAD_ICO);

    let target_ico = runtime_dir.join("chkey.ico");
    if !target_ico.exists() {
        let _ = fs::write(&target_ico, EMBEDDED_ICO);
    }

    // 7. Dọn dẹp các file .old cũ
    cleanup_old_dll_files(&runtime_dir);

    Ok(target_dll)
}
