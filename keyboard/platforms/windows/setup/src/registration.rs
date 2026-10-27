//! Quản lý đăng ký và hủy đăng ký COM Server và Windows TSF Text Service Profile.
//!
//! Tái sử dụng đồng bộ với các GUID và tiêu chuẩn TSF của dự án:
//! - CLSID: {A3F2B801-1234-4567-89AB-CDEF01234567}
//! - Profile GUID: {A3F2B802-1234-4567-89AB-CDEF01234567}
//! - Hỗ trợ đăng ký User-scope (HKCU) và Machine-scope (HKCR khi elevated)

use std::path::Path;
use windows::core::{GUID, PCWSTR, Result};
use windows::Win32::Foundation::{BOOL, LPARAM, S_OK, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD,
    ITfCategoryMgr, ITfInputProcessorProfiles,
};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, HWND_BROADCAST, WM_SETTINGCHANGE};

pub const CHKEY_TSF_CLSID_GUID: GUID = GUID::from_u128(0xa3f2b801_1234_4567_89ab_cdef01234567);
pub const CHKEY_TSF_PROFILE_GUID: GUID = GUID::from_u128(0xa3f2b802_1234_4567_89ab_cdef01234567);

pub const LANGID_ENGLISH_US: u16 = 0x0409;
pub const LANGID_VIETNAMESE: u16 = 0x042A;

type LayoutOrTipFn = unsafe extern "system" fn(PCWSTR, u32) -> BOOL;

/// RAII Guard bảo đảm gọi CoInitialize / CoUninitialize đúng quy tắc COM
pub struct ComGuard(bool);

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

/// Định dạng GUID thành chuỗi chữ HOA: "A3F2B801-1234-4567-89AB-CDEF01234567".
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

/// Kiểm tra tiến trình có đang chạy dưới quyền Administrator (Elevated) hay không.
pub fn is_elevated() -> bool {
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_ok() {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut ret_len = 0u32;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut ret_len,
            );
            let _ = windows::Win32::Foundation::CloseHandle(token);
            if ok.is_ok() {
                return elevation.TokenIsElevated != 0;
            }
        }
        false
    }
}

/// Đọc giá trị chuỗi (REG_SZ) từ Windows Registry.
fn get_registry_sz(hkey: HKEY, subkey: &str, value_name: &str) -> Result<String> {
    unsafe {
        let mut subkey_handle = HKEY::default();
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
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
            val_w = format!("{}\0", value_name).encode_utf16().collect();
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
            let u16_slice = std::slice::from_raw_parts(buf.as_ptr() as *const u16, (len as usize) / 2);
            let s = String::from_utf16_lossy(u16_slice);
            Ok(s.trim_matches('\0').to_string())
        } else {
            Err(windows::core::Error::from_hresult(res.to_hresult()))
        }
    }
}

/// Ghi giá trị chuỗi (REG_SZ) vào Windows Registry.
fn set_registry_sz(hkey: HKEY, subkey: &str, value_name: &str, data: &str) -> Result<()> {
    unsafe {
        let mut subkey_handle = HKEY::default();
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
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

        let data_w: Vec<u16> = format!("{}\0", data).encode_utf16().collect();
        let data_bytes = std::slice::from_raw_parts(data_w.as_ptr() as *const u8, data_w.len() * 2);

        let val_w: Vec<u16>;
        let val_ptr = if value_name.is_empty() {
            PCWSTR::null()
        } else {
            val_w = format!("{}\0", value_name).encode_utf16().collect();
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

unsafe fn call_input_dll_func(func_name: &str, langid: u16) {
    unsafe {
        if let Ok(lib) = LoadLibraryA(windows::core::s!("input.dll")) {
            let name_c = std::ffi::CString::new(func_name).unwrap();
            if let Some(proc) = GetProcAddress(lib, windows::core::PCSTR(name_c.as_ptr() as *const u8)) {
                let func: LayoutOrTipFn = std::mem::transmute(proc);
                let tip_str = format!(
                    "{:04x}:{{{}}}{{{}}}\0",
                    langid,
                    guid_to_string(&CHKEY_TSF_CLSID_GUID),
                    guid_to_string(&CHKEY_TSF_PROFILE_GUID)
                );
                let tip_w: Vec<u16> = tip_str.encode_utf16().collect();
                let _ = func(PCWSTR(tip_w.as_ptr()), 0);
            }
        }
    }
}

/// Lấy đường dẫn file DLL hiện tại đang được đăng ký trong COM InProcServer32 (HKCU hoặc HKCR).
pub fn get_registered_dll_path() -> Option<String> {
    let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

    // 1. Kiểm tra HKCU trước (User Scope)
    let hkcu_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32", clsid_str);
    if let Ok(path) = get_registry_sz(HKEY_CURRENT_USER, &hkcu_key, "") {
        if !path.trim().is_empty() {
            return Some(path);
        }
    }

    // 2. Kiểm tra HKCR (Machine Scope)
    let hkcr_key = format!("CLSID\\{{{}}}\\InProcServer32", clsid_str);
    if let Ok(path) = get_registry_sz(HKEY_CLASSES_ROOT, &hkcr_key, "") {
        if !path.trim().is_empty() {
            return Some(path);
        }
    }

    None
}

/// Kiểm tra xem Profile CLSID của CHKey đã được đăng ký trong TSF Profiles hay chưa.
pub fn is_tsf_profile_registered() -> Result<bool> {
    unsafe {
        let _com_guard = ComGuard::new();
        let profiles: ITfInputProcessorProfiles = match CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER) {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };

        if let Ok(enum_profiles) = profiles.EnumInputProcessorInfo() {
            let mut profile = [Default::default(); 1];
            let mut fetched;
            loop {
                fetched = 0;
                let hr = enum_profiles.Next(&mut profile, Some(&mut fetched));
                if hr != S_OK || fetched == 0 {
                    break;
                }
                if profile[0] == CHKEY_TSF_CLSID_GUID {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

/// Kiểm tra toàn diện xem CHKey TSF đã được đăng ký VÀ đường dẫn COM InProc trỏ đúng file DLL hay chưa.
pub fn is_registered_with_dll(expected_dll: &Path) -> bool {
    let expected_str = expected_dll.to_string_lossy();
    if is_tsf_profile_registered().unwrap_or(false) {
        if let Some(registered_path) = get_registered_dll_path() {
            return registered_path.trim().eq_ignore_ascii_case(expected_str.trim());
        }
    }
    false
}

/// Gửi thông báo đến toàn bộ các cửa sổ Windows để làm mới cấu hình Text Service.
pub fn notify_windows_tsf_changed() {
    unsafe {
        let intl_w: Vec<u16> = "intl\0".encode_utf16().collect();
        let _ = PostMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), LPARAM(intl_w.as_ptr() as isize));
    }
}

/// Đăng ký COM InProcServer32 và TSF Text Service Profile cho file DLL chỉ định.
pub fn register_tsf(dll_path: &Path) -> std::result::Result<(), String> {
    let dll_str = dll_path.to_string_lossy().to_string();
    let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

    unsafe {
        let _com_guard = ComGuard::new();

        // 1. Ghi COM InProcServer32 vào HKCU (User Scope)
        let hkcu_clsid_key = format!("Software\\Classes\\CLSID\\{{{}}}", clsid_str);
        let hkcu_inproc_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32", clsid_str);
        let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_clsid_key, "", "CHKey Êđê Text Service");
        let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_inproc_key, "", &dll_str);
        let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_inproc_key, "ThreadingModel", "Apartment");

        // 2. Nếu có quyền Admin, ghi thêm vào HKCR để toàn bộ hệ thống nhận diện
        if is_elevated() {
            let clsid_key = format!("CLSID\\{{{}}}", clsid_str);
            let inproc_key = format!("CLSID\\{{{}}}\\InProcServer32", clsid_str);
            let _ = set_registry_sz(HKEY_CLASSES_ROOT, &clsid_key, "", "CHKey Êđê Text Service");
            let _ = set_registry_sz(HKEY_CLASSES_ROOT, &inproc_key, "", &dll_str);
            let _ = set_registry_sz(HKEY_CLASSES_ROOT, &inproc_key, "ThreadingModel", "Apartment");
        }

        // 3. Đăng ký TSF Input Processor Profile
        let profiles: ITfInputProcessorProfiles = CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("Không thể tạo ITfInputProcessorProfiles: {e}"))?;

        let _ = profiles.Register(&CHKEY_TSF_CLSID_GUID);

        let desc: Vec<u16> = "Native (Rade)\0".encode_utf16().collect();
        // Icon được đóng gói trực tiếp trong file DLL (index 0)
        let icon_file: Vec<u16> = format!("{}\0", dll_str).encode_utf16().collect();

        // Gỡ bỏ cấu hình tiếng Việt (0x042A) cũ nếu có để tránh trùng lặp ngôn ngữ
        let _ = profiles.RemoveLanguageProfile(&CHKEY_TSF_CLSID_GUID, LANGID_VIETNAMESE, &CHKEY_TSF_PROFILE_GUID);
        call_input_dll_func("UninstallLayoutOrTip", LANGID_VIETNAMESE);

        // Gỡ bỏ cấu hình English US (0x0409) cũ nếu có để cập nhật thông tin mới nhất
        let _ = profiles.RemoveLanguageProfile(&CHKEY_TSF_CLSID_GUID, LANGID_ENGLISH_US, &CHKEY_TSF_PROFILE_GUID);

        // Đăng ký profile duy nhất cho English US (0x0409) với rad.ico
        profiles.AddLanguageProfile(
            &CHKEY_TSF_CLSID_GUID,
            LANGID_ENGLISH_US,
            &CHKEY_TSF_PROFILE_GUID,
            &desc,
            &icon_file,
            0,
        ).map_err(|e| format!("Lỗi AddLanguageProfile: {e}"))?;

        profiles.EnableLanguageProfile(
            &CHKEY_TSF_CLSID_GUID,
            LANGID_ENGLISH_US,
            &CHKEY_TSF_PROFILE_GUID,
            true,
        ).map_err(|e| format!("Lỗi EnableLanguageProfile: {e}"))?;

        call_input_dll_func("InstallLayoutOrTip", LANGID_ENGLISH_US);

        // 4. Đăng ký Category bàn phím trong ITfCategoryMgr
        let category_mgr: ITfCategoryMgr = CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("Không thể tạo ITfCategoryMgr: {e}"))?;

        let _ = category_mgr.RegisterCategory(&CHKEY_TSF_CLSID_GUID, &GUID_TFCAT_TIP_KEYBOARD, &CHKEY_TSF_CLSID_GUID);

        // 5. Phát thông điệp cập nhật
        notify_windows_tsf_changed();
    }

    Ok(())
}

/// Hủy đăng ký TSF Text Service và xóa COM registry.
pub fn unregister_tsf() -> std::result::Result<(), String> {
    let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

    unsafe {
        // 1. Gỡ cài đặt LayoutOrTip
        for &langid in &[LANGID_ENGLISH_US, LANGID_VIETNAMESE] {
            call_input_dll_func("UninstallLayoutOrTip", langid);
        }

        let _com_guard = ComGuard::new();

        // 2. Hủy Category
        if let Ok(category_mgr) = CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER) {
            let _ = category_mgr.UnregisterCategory(&CHKEY_TSF_CLSID_GUID, &GUID_TFCAT_TIP_KEYBOARD, &CHKEY_TSF_CLSID_GUID);
        }

        // 3. Hủy Profile
        if let Ok(profiles) = CoCreateInstance::<_, ITfInputProcessorProfiles>(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER) {
            let _ = profiles.Unregister(&CHKEY_TSF_CLSID_GUID);
        }

        // 4. Xóa Registry HKCU
        let hkcu_inproc_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32\0", clsid_str);
        let hkcu_clsid_key = format!("Software\\Classes\\CLSID\\{{{}}}\0", clsid_str);
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(hkcu_inproc_key.encode_utf16().collect::<Vec<_>>().as_ptr()));
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(hkcu_clsid_key.encode_utf16().collect::<Vec<_>>().as_ptr()));

        // 5. Nếu elevated, xóa Registry HKCR
        if is_elevated() {
            let inproc_key = format!("CLSID\\{{{}}}\\InProcServer32\0", clsid_str);
            let clsid_key = format!("CLSID\\{{{}}}\0", clsid_str);
            let _ = RegDeleteKeyW(HKEY_CLASSES_ROOT, PCWSTR(inproc_key.encode_utf16().collect::<Vec<_>>().as_ptr()));
            let _ = RegDeleteKeyW(HKEY_CLASSES_ROOT, PCWSTR(clsid_key.encode_utf16().collect::<Vec<_>>().as_ptr()));
        }

        // 6. Phát thông điệp cập nhật
        notify_windows_tsf_changed();
    }

    Ok(())
}
