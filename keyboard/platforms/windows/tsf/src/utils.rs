use windows::core::{GUID, PCWSTR, Result};
use windows::Win32::Foundation::BOOL;
use windows::Win32::System::Com::{CoInitialize, CoUninitialize};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::Registry::{
    HKEY, KEY_ALL_ACCESS, KEY_READ, REG_OPTION_NON_VOLATILE, REG_SZ,
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};

/// RAII Guard bảo vệ COM Apartment.
pub struct ComGuard(bool);

impl Default for ComGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ComGuard {
    pub fn new() -> Self {
        unsafe {
            let hr = CoInitialize(None);
            ComGuard(hr.is_ok())
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

/// Chuyển đổi chuỗi &str thành Vec<u16> UTF-16 có ký tự kết thúc null '\0'.
#[inline]
pub fn to_utf16_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Định dạng GUID thành chuỗi viết hoa: "A3F2B801-1234-4567-89AB-CDEF01234567".
pub fn guid_to_string(guid: &GUID) -> String {
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid.data1,
        guid.data2,
        guid.data3,
        guid.data4[0],
        guid.data4[1],
        guid.data4[2],
        guid.data4[3],
        guid.data4[4],
        guid.data4[5],
        guid.data4[6],
        guid.data4[7],
    )
}

/// Ghi log hoạt động vào thư mục %TEMP%\CHKey.
pub fn log_tsf_msg(log_name: &str, msg: &str) {
    use std::io::Write;
    let log_dir = std::env::temp_dir().join("CHKey");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join(log_name);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(
            f,
            "[{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            msg
        );
    }
}

/// Ghi chuỗi REG_SZ vào Registry.
pub fn set_registry_sz(hkey: HKEY, subkey: &str, value_name: &str, data: &str) -> Result<()> {
    unsafe {
        let mut subkey_handle = HKEY::default();
        let subkey_w = to_utf16_null(subkey);
        let status = RegCreateKeyExW(
            hkey,
            PCWSTR(subkey_w.as_ptr()),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            None,
            &mut subkey_handle,
            None,
        );
        if status.is_err() {
            return Err(windows::core::Error::from_hresult(status.to_hresult()));
        }

        let data_w = to_utf16_null(data);
        let data_bytes = std::slice::from_raw_parts(data_w.as_ptr() as *const u8, data_w.len() * 2);

        let val_w: Vec<u16>;
        let val_ptr = if value_name.is_empty() {
            PCWSTR::null()
        } else {
            val_w = to_utf16_null(value_name);
            PCWSTR(val_w.as_ptr())
        };

        let res = RegSetValueExW(subkey_handle, val_ptr, 0, REG_SZ, Some(data_bytes));
        let _ = RegCloseKey(subkey_handle);

        if res.is_err() {
            return Err(windows::core::Error::from_hresult(res.to_hresult()));
        }
        Ok(())
    }
}

/// Đọc chuỗi REG_SZ từ Registry.
pub fn get_registry_sz(hkey: HKEY, subkey: &str, value_name: &str) -> Result<String> {
    unsafe {
        let mut subkey_handle = HKEY::default();
        let subkey_w = to_utf16_null(subkey);
        let status = RegOpenKeyExW(
            hkey,
            PCWSTR(subkey_w.as_ptr()),
            0,
            KEY_READ,
            &mut subkey_handle,
        );
        if status.is_err() {
            return Err(windows::core::Error::from_hresult(status.to_hresult()));
        }

        let val_w: Vec<u16>;
        let val_ptr = if value_name.is_empty() {
            PCWSTR::null()
        } else {
            val_w = to_utf16_null(value_name);
            PCWSTR(val_w.as_ptr())
        };

        let mut buf = [0u8; 1024];
        let mut len = buf.len() as u32;
        let res = RegQueryValueExW(
            subkey_handle,
            val_ptr,
            None,
            None,
            Some(buf.as_mut_ptr()),
            Some(&mut len),
        );
        let _ = RegCloseKey(subkey_handle);

        if res.is_ok() && len >= 2 {
            let u16_slice =
                std::slice::from_raw_parts(buf.as_ptr() as *const u16, (len as usize) / 2);
            let s = String::from_utf16_lossy(u16_slice);
            Ok(s.trim_matches('\0').to_string())
        } else {
            Err(windows::core::Error::from_hresult(res.to_hresult()))
        }
    }
}

/// Xóa một khóa Registry (an toàn, bỏ qua nếu khóa không tồn tại).
pub fn delete_registry_key(hkey: HKEY, subkey: &str) {
    unsafe {
        let subkey_w = to_utf16_null(subkey);
        let _ = RegDeleteKeyW(hkey, PCWSTR(subkey_w.as_ptr()));
    }
}

type LayoutOrTipFn = unsafe extern "system" fn(PCWSTR, u32) -> BOOL;

/// Gọi InstallLayoutOrTip / UninstallLayoutOrTip từ input.dll của Windows.
pub fn call_layout_or_tip(func_name: &str, langid: u16, clsid: &GUID, profile: &GUID) {
    unsafe {
        if let Ok(lib) = LoadLibraryA(windows::core::s!("input.dll")) {
            let name_c = match std::ffi::CString::new(func_name) {
                Ok(c) => c,
                Err(_) => return,
            };
            if let Some(proc) = GetProcAddress(lib, windows::core::PCSTR(name_c.as_ptr() as *const u8)) {
                let func: LayoutOrTipFn = std::mem::transmute(proc);
                let tip_str = format!(
                    "{:04x}:{{{}}}{{{}}}",
                    langid,
                    guid_to_string(clsid),
                    guid_to_string(profile)
                );
                let tip_w = to_utf16_null(&tip_str);
                let _ = func(PCWSTR(tip_w.as_ptr()), 0);
            }
        }
    }
}
