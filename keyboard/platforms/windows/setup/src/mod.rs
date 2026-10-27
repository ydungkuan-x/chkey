//! Setup module coordinator for CHKey packaging.

pub mod extract;
pub mod registration;
pub mod status;

pub use extract::*;
pub use registration::*;
pub use status::*;

use std::path::PathBuf;

/// Đảm bảo môi trường runtime của CHKey đã sẵn sàng:
pub fn ensure_setup() -> Result<PathBuf, String> {
    if let Err(err) = chkey_config::load_config() {
        eprintln!("[Setup] Cảnh báo đọc cấu hình config.toml: {err}. Sẽ sử dụng cấu hình mặc định.");
    }

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

pub fn repair_setup() -> Result<(), String> {
    println!("[Setup] Đang thực hiện sửa chữa / trích xuất lại DLL và TSF...");
    let extracted_path = extract::extract_dll()?;
    registration::register_tsf(&extracted_path)?;
    println!("[Setup] Sửa chữa và đăng ký TSF thành công.");
    Ok(())
}

pub fn uninstall_setup() -> Result<(), String> {
    println!("[Setup] Đang hủy đăng ký TSF Text Service...");
    registration::unregister_tsf()?;
    println!("[Setup] Hủy đăng ký TSF thành công.");
    Ok(())
}
