//! Quản lý khởi động cùng Windows cho CHKey.
//!
//! Hỗ trợ:
//! - Đăng ký khởi động cùng Windows qua Registry (HKCU\...\CurrentVersion\Run)
//!   kết hợp Windows Task Scheduler (cho phép chạy với quyền Admin không hỏi lại UAC khi đăng nhập).
//! - Hủy đăng ký khởi động cùng Windows.
//! - Kiểm tra trạng thái hiện tại trên hệ thống Windows.

use std::path::PathBuf;
use windows::core::{w, PCWSTR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, REG_OPTION_NON_VOLATILE, REG_SZ,
};

const RUN_SUBKEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("CHKey");
const TASK_NAME: &str = "CHKey";

/// Lấy đường dẫn thực thi của CHKey để đăng ký khởi động.
/// Ưu tiên file cài đặt trong %LOCALAPPDATA%\Programs\CHKey\CHKey.exe nếu có.
pub fn get_startup_exe_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let installed = PathBuf::from(local_app_data)
            .join("Programs")
            .join("CHKey")
            .join("CHKey.exe");
        if installed.exists() {
            return installed;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        return exe;
    }
    PathBuf::from("CHKey.exe")
}

/// Kiểm tra xem CHKey hiện tại có đang được cấu hình khởi động cùng Windows hay không.
pub fn is_startup_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();

        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, 0, KEY_READ, &mut hkey).is_ok() {
            let mut data = [0u16; 260];
            let mut len = (data.len() * 2) as u32;
            let res = RegQueryValueExW(
                hkey,
                APP_NAME,
                None,
                None,
                Some(data.as_mut_ptr() as *mut u8),
                Some(&mut len),
            );
            let _ = RegCloseKey(hkey);
            if res.is_ok() && len > 0 {
                return true;
            }
        }
    }

    false
}

/// Bật hoặc tắt khởi động cùng Windows.
pub fn set_startup_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe_path = get_startup_exe_path();
        let cmd_line = format!("\"{}\" --startup", exe_path.display());

        // 1. Ghi vào Registry HKCU\Software\Microsoft\Windows\CurrentVersion\Run
        unsafe {
            let mut hkey = HKEY::default();

            let status = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                RUN_SUBKEY,
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                None,
                &mut hkey,
                None,
            );
            if status.is_err() {
                return Err(format!("Lỗi mở registry Run key: {:?}", status));
            }

            let data_w: Vec<u16> = cmd_line.encode_utf16().chain(Some(0)).collect();
            let data_bytes = std::slice::from_raw_parts(
                data_w.as_ptr() as *const u8,
                data_w.len() * 2,
            );

            let res = RegSetValueExW(
                hkey,
                APP_NAME,
                0,
                REG_SZ,
                Some(data_bytes),
            );
            let _ = RegCloseKey(hkey);

            if res.is_err() {
                return Err(format!("Lỗi ghi registry startup: {:?}", res));
            }
        }

        // 2. Tạo Windows Task Scheduler để chạy với quyền cao nhất (Admin không hỏi UAC khi login)
        let _ = std::process::Command::new("schtasks")
            .args([
                "/create",
                "/tn",
                TASK_NAME,
                "/tr",
                &cmd_line,
                "/sc",
                "onlogon",
                "/rl",
                "highest",
                "/f",
            ])
            .output();

        println!("[Startup] Đã bật khởi động cùng Windows: {}", cmd_line);
        Ok(())
    } else {
        // 1. Xóa khỏi Registry HKCU
        unsafe {
            let mut hkey = HKEY::default();

            if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_SUBKEY, 0, KEY_ALL_ACCESS, &mut hkey).is_ok() {
                let _ = RegDeleteValueW(hkey, APP_NAME);
                let _ = RegCloseKey(hkey);
            }
        }

        // 2. Xóa Task Scheduler nếu có
        let _ = std::process::Command::new("schtasks")
            .args(["/delete", "/tn", TASK_NAME, "/f"])
            .output();

        println!("[Startup] Đã hủy khởi động cùng Windows.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_toggle_roundtrip() {
        let initial = is_startup_enabled();
        println!("Initial startup enabled: {}", initial);

        assert!(set_startup_enabled(true).is_ok());
        assert!(is_startup_enabled());

        assert!(set_startup_enabled(false).is_ok());
        assert!(!is_startup_enabled());

        // Restore initial
        let _ = set_startup_enabled(initial);
    }
}
