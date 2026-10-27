//! Trạng thái và dịch vụ kiểm tra TSF của CHKey.
//!
//! Đảm nhiệm:
//! - Xác định đường dẫn DLL cài đặt thực tế (%LOCALAPPDATA%\CHKey\runtime\chkey_tsf.dll hoặc cùng thư mục CHKey.exe).
//! - Kiểm tra trạng thái đăng ký TSF và COM InProcServer32.
//! - Cung cấp hàm sửa chữa (repair) khi TSF chưa đăng ký hoặc đăng ký sai đường dẫn.
//! - Cung cấp hàm gỡ đăng ký TSF.
//! - Hỗ trợ nâng quyền Administrator khi cần sửa chữa đăng ký TSF vào HKLM.

use std::path::PathBuf;
use super::registration::{self, TsfRegistration};

/// Trạng thái của dịch vụ TSF Text Service trên Windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TsfStatus {
    /// TSF đã được đăng ký hợp lệ và file DLL tồn tại tại đường dẫn cài đặt.
    Valid(PathBuf),
    /// Không tìm thấy file chkey_tsf.dll tại các đường dẫn cài đặt chuẩn.
    MissingDll(PathBuf),
    /// File DLL tồn tại nhưng TSF chưa được đăng ký trong Windows hoặc COM InProcServer32 chưa trỏ đúng file.
    NotRegistered {
        dll_path: PathBuf,
        registered_path: Option<String>,
    },
}

/// Lấy đường dẫn cài đặt chuẩn của chkey_tsf.dll.
///
/// Ưu tiên:
/// 1. %LOCALAPPDATA%\CHKey\runtime\chkey_tsf.dll
/// 2. %LOCALAPPDATA%\Programs\CHKey\chkey_tsf.dll
/// 3. Thư mục cùng cấp với file CHKey.exe đang chạy
pub fn get_installed_dll_path() -> PathBuf {
    // 0. Ưu tiên thư mục runtime chuẩn: %LOCALAPPDATA%\CHKey\runtime\
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let runtime_tsf = PathBuf::from(&local_app_data)
            .join("CHKey")
            .join("runtime")
            .join("chkey_tsf.dll");
        if runtime_tsf.exists() {
            return runtime_tsf;
        }
        let runtime_ime = PathBuf::from(&local_app_data)
            .join("CHKey")
            .join("runtime")
            .join("chkey_ime.dll");
        if runtime_ime.exists() {
            return runtime_ime;
        }
    }

    // 1. Thư mục cài đặt theo Programs (tương thích ngược với CHKey-Setup)
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let programs_tsf = PathBuf::from(&local_app_data)
            .join("Programs")
            .join("CHKey")
            .join("chkey_tsf.dll");
        if programs_tsf.exists() {
            return programs_tsf;
        }
        let programs_ime = PathBuf::from(&local_app_data)
            .join("Programs")
            .join("CHKey")
            .join("chkey_ime.dll");
        if programs_ime.exists() {
            return programs_ime;
        }
    }

    // 2. Thư mục cùng cấp với EXE đang chạy
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let local_tsf = exe_dir.join("chkey_tsf.dll");
        if local_tsf.exists() {
            return local_tsf;
        }
        let local_ime = exe_dir.join("chkey_ime.dll");
        if local_ime.exists() {
            return local_ime;
        }
    }

    // Fallback mặc định về %LOCALAPPDATA%\CHKey\runtime\chkey_tsf.dll
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data)
            .join("CHKey")
            .join("runtime")
            .join("chkey_tsf.dll")
    } else {
        PathBuf::from("chkey_tsf.dll")
    }
}

/// Kiểm tra trạng thái tích hợp TSF hiện tại của CHKey.
pub fn check_tsf_status() -> TsfStatus {
    let dll_path = get_installed_dll_path();

    if !dll_path.exists() {
        return TsfStatus::MissingDll(dll_path);
    }

    let is_registered = registration::is_tsf_registered_with_dll(&dll_path);
    if is_registered {
        TsfStatus::Valid(dll_path)
    } else {
        let registered_path = TsfRegistration::get_registered_dll_path();
        TsfStatus::NotRegistered {
            dll_path,
            registered_path,
        }
    }
}

/// Sửa chữa hoặc đăng ký mới TSF Text Service với file DLL đã cài đặt.
pub fn repair_tsf() -> Result<(), String> {
    let dll_path = get_installed_dll_path();
    if !dll_path.exists() {
        return Err(format!(
            "Không tìm thấy file DLL TSF tại: {}. Vui lòng chạy bộ cài đặt CHKey-Setup.exe.",
            dll_path.display()
        ));
    }

    registration::register_tsf(&dll_path)?;
    super::notify_windows_tsf_changed();
    Ok(())
}

/// Hủy đăng ký TSF Text Service khỏi Windows.
pub fn uninstall_tsf() -> Result<(), String> {
    registration::unregister_tsf()?;
    super::notify_windows_tsf_changed();
    Ok(())
}

/// Chạy tiến trình CHKey.exe với quyền Administrator (UAC prompt) để thực hiện sửa chữa TSF.
pub fn request_elevated_repair() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW;

    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;

    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let file: Vec<u16> = exe_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let params: Vec<u16> = "--repair-tsf\0".encode_utf16().collect();

    let res = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            SW_SHOW,
        )
    };

    if res as usize > 32 {
        Ok(())
    } else {
        Err(format!("Không thể kích hoạt quyền Administrator (Mã lỗi: {res:?})"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsf_unregister_and_register_roundtrip() {
        if !registration::is_elevated() {
            println!("Skipping HKLM unregister roundtrip when running without Administrator privilege");
            return;
        }
        // Test uninstall
        let uninst_res = uninstall_tsf();
        println!("uninstall_tsf: {:?}", uninst_res);
        assert!(uninst_res.is_ok());

        // Test repair / register back
        let repair_res = repair_tsf();
        println!("repair_tsf: {:?}", repair_res);
        assert!(repair_res.is_ok());
    }
}
