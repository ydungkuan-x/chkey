//! chkey_setup: Thư viện quản lý trích xuất runtime DLL và đăng ký TSF/COM cho CHKey.
//!
//! Cung cấp quy trình tự động hóa đóng gói 1 file EXE:
//! - Nhúng nhị phân `chkey_ime.dll` tại thời điểm build.
//! - Tự động trích xuất ra `%LOCALAPPDATA%\CHKey\runtime\chkey_ime.dll` khi khởi chạy lần đầu hoặc khi có bản build mới.
//! - Kiểm tra mã băm SHA-256 để tránh trích xuất lại không cần thiết.
//! - Tự động tạo và kiểm tra `%LOCALAPPDATA%\CHKey\config.toml` (văn bản TOML có thể chỉnh sửa).
//! - Tự động đăng ký COM InProcServer32 và TSF Profiles khi chưa có hoặc khi đường dẫn thay đổi.

pub mod extract;
pub mod registration;
pub mod status;

pub use extract::*;
pub use registration::*;
pub use status::*;

use std::path::PathBuf;

/// Đảm bảo môi trường runtime của CHKey đã sẵn sàng:
/// 1. Đảm bảo cấu hình config.toml tồn tại (tạo mặc định nếu lần đầu mở, giữ nguyên cấu hình cũ của người dùng).
/// 2. Kiểm tra trạng thái DLL và TSF.
/// 3. Nếu đã UpToDate, hoàn thành ngay lập tức (không ghi đĩa, không đăng ký lại).
/// 4. Nếu thiếu DLL hoặc DLL cũ: trích xuất an toàn từ dữ liệu nhúng và cập nhật đăng ký TSF.
/// 5. Nếu chưa đăng ký: thực hiện đăng ký TSF/COM.
/// 6. Trả về đường dẫn runtime DLL.
pub fn ensure_setup() -> Result<PathBuf, String> {
    // 1. Đảm bảo cấu hình config.toml tồn tại và hợp lệ
    // chkey_config::load_config() tự tạo cấu hình mặc định nếu file chưa có
    // và giữ nguyên file cấu hình nếu đã tồn tại
    if let Err(err) = chkey_config::load_config() {
        eprintln!("[Setup] Cảnh báo đọc cấu hình config.toml: {err}. Sẽ sử dụng cấu hình mặc định.");
    }

    // 2. Kiểm tra trạng thái hiện tại
    let current_status = status::check_status()?;

    match current_status {
        status::SetupStatus::UpToDate { dll_path, sha256 } => {
            println!(
                "[Setup] Runtime DLL và TSF đã ở trạng thái mới nhất ({}, SHA256: {}).",
                dll_path.display(),
                &sha256[..8]
            );
            Ok(dll_path)
        }
        status::SetupStatus::MissingDll { .. } => {
            println!("[Setup] Chưa tìm thấy runtime DLL. Đang trích xuất từ bản nhúng...");
            let extracted_path = extract::extract_dll()?;
            println!("[Setup] Đã trích xuất: {}. Đang đăng ký TSF...", extracted_path.display());
            registration::register_tsf(&extracted_path)?;
            println!("[Setup] Đăng ký TSF hoàn tất.");
            Ok(extracted_path)
        }
        status::SetupStatus::OutdatedDll { current_sha256, embedded_sha256, .. } => {
            println!(
                "[Setup] Phát hiện bản build mới (Hiện tại: {}, Nhúng: {}). Đang cập nhật DLL an toàn...",
                &current_sha256[..8],
                &embedded_sha256[..8]
            );
            let extracted_path = extract::extract_dll()?;
            println!("[Setup] Đã cập nhật: {}. Đang cập nhật đăng ký TSF...", extracted_path.display());
            registration::register_tsf(&extracted_path)?;
            println!("[Setup] Đăng ký TSF đã cập nhật thành công.");
            Ok(extracted_path)
        }
        status::SetupStatus::NotRegistered { dll_path, registered_path } => {
            println!(
                "[Setup] Runtime DLL đã có sẵn nhưng TSF chưa đăng ký đúng (Hiện tại: {:?}). Đang đăng ký...",
                registered_path
            );
            registration::register_tsf(&dll_path)?;
            println!("[Setup] Đã đăng ký TSF thành công.");
            Ok(dll_path)
        }
    }
}

/// Thực hiện sửa chữa hoặc đăng ký lại runtime DLL và TSF.
pub fn repair_setup() -> Result<(), String> {
    println!("[Setup] Đang thực hiện sửa chữa / trích xuất lại DLL và TSF...");
    let extracted_path = extract::extract_dll()?;
    registration::register_tsf(&extracted_path)?;
    println!("[Setup] Sửa chữa và đăng ký TSF thành công.");
    Ok(())
}

/// Gỡ cài đặt / hủy đăng ký TSF Text Service khỏi hệ thống.
pub fn uninstall_setup() -> Result<(), String> {
    println!("[Setup] Đang hủy đăng ký TSF Text Service...");
    registration::unregister_tsf()?;
    println!("[Setup] Hủy đăng ký TSF thành công.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_dll_properties() {
        assert!(!extract::EMBEDDED_DLL.is_empty(), "Embedded DLL must not be empty");
        // Verify MZ header for valid Windows PE
        assert_eq!(&extract::EMBEDDED_DLL[0..2], b"MZ", "Embedded DLL must have valid PE MZ signature");

        let hash_hex = extract::embedded_dll_sha256_hex();
        assert_eq!(hash_hex.len(), 64, "SHA-256 hex string must be 64 characters long");
        println!("Embedded DLL SHA-256: {}", hash_hex);
    }

    #[test]
    fn test_runtime_paths() {
        let runtime_dir = extract::get_runtime_dir();
        assert!(runtime_dir.is_ok());
        let dir = runtime_dir.unwrap();
        assert!(dir.ends_with("runtime"));

        let dll_path = extract::get_runtime_dll_path();
        assert!(dll_path.is_ok());
        let path = dll_path.unwrap();
        assert!(path.ends_with("chkey_tsf.dll"));
    }

    #[test]
    fn test_extract_and_verify_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("chkey_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let target_file = temp_dir.join("test_ime.dll");

        // Write embedded bytes
        std::fs::write(&target_file, extract::EMBEDDED_DLL).expect("Failed to write test dll");

        // Verify SHA-256 match
        let file_hash = extract::file_sha256_hex(&target_file).expect("Failed to compute file hash");
        let embedded_hash = extract::embedded_dll_sha256_hex();
        assert_eq!(file_hash, embedded_hash);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_extract_to_runtime_and_status() {
        // Test extraction to runtime
        let extracted = extract::extract_dll();
        assert!(extracted.is_ok(), "Extraction should succeed: {:?}", extracted.err());
        let path = extracted.unwrap();
        assert!(path.exists(), "Extracted DLL must exist on disk");

        // Verify SHA-256 matches
        let hash = extract::file_sha256_hex(&path).expect("Failed to get hash");
        let embedded_hash = extract::embedded_dll_sha256_hex();
        assert_eq!(hash, embedded_hash, "Extracted DLL hash must match embedded DLL");

        // Verify is_runtime_dll_up_to_date() reports true
        assert!(extract::is_runtime_dll_up_to_date(), "Runtime DLL should now be up to date");

        // Verify re-extraction is idempotent
        let second_extracted = extract::extract_dll();
        assert_eq!(second_extracted.unwrap(), path);

        // Verify config loading
        let config_res = chkey_config::load_config();
        assert!(config_res.is_ok(), "Config loading/creation should succeed: {:?}", config_res.err());
    }

    #[test]
    fn test_guid_formatting() {
        let clsid_str = registration::guid_to_string(&registration::CHKEY_TSF_CLSID_GUID);
        assert_eq!(clsid_str, "A3F2B801-1234-4567-89AB-CDEF01234567");
        let profile_str = registration::guid_to_string(&registration::CHKEY_TSF_PROFILE_GUID);
        assert_eq!(profile_str, "A3F2B802-1234-4567-89AB-CDEF01234567");
    }
}
