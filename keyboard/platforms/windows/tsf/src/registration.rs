//! Windows TSF Registration and Profile Manager for CHKey.

use super::profile::{
    CHKEY_TSF_CLSID_GUID, CHKEY_TSF_PROFILE_GUID, LANGID_ENGLISH_US, LANGID_VIETNAMESE,
};
use super::utils::{
    call_layout_or_tip, delete_registry_key, get_registry_sz, guid_to_string, log_tsf_msg,
    set_registry_sz, to_utf16_null, ComGuard,
};
use windows::core::Result;
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Registry::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD, ITfCategoryMgr,
    ITfInputProcessorProfiles,
};

fn log_reg(msg: &str) {
    log_tsf_msg("chkey_register.log", msg);
}

/// Quản lý đăng ký TSF Text Service và COM Server vào hệ thống Windows.
pub struct TsfRegistration;

impl TsfRegistration {
    /// Kiểm tra xem tiến trình hiện tại có đang chạy dưới quyền Administrator không.
    pub fn is_elevated() -> bool {
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
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

    /// Lấy đường dẫn DLL hiện đang đăng ký trong Registry (COM InProcServer32).
    pub fn get_registered_dll_path() -> Option<String> {
        let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

        // 1. Kiểm tra HKCU trước (User Scope)
        let hkcu_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32", clsid_str);
        if let Ok(path) = get_registry_sz(HKEY_CURRENT_USER, &hkcu_key, "")
            && !path.trim().is_empty()
        {
            return Some(path);
        }

        // 2. Kiểm tra HKCR (Machine / Merged Scope)
        let hkcr_key = format!("CLSID\\{{{}}}\\InProcServer32", clsid_str);
        if let Ok(path) = get_registry_sz(HKEY_CLASSES_ROOT, &hkcr_key, "")
            && !path.trim().is_empty()
        {
            return Some(path);
        }

        None
    }

    /// Kiểm tra TSF Text Service đã được đăng ký trong hệ thống hay chưa.
    pub fn is_registered() -> Result<bool> {
        unsafe {
            let _com_guard = ComGuard::new();

            let profiles: ITfInputProcessorProfiles = match CoCreateInstance(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                Ok(p) => p,
                Err(e) => {
                    log_reg(&format!("is_registered: CoCreateInstance error: {:?}", e));
                    return Ok(false);
                }
            };

            let mut is_registered = false;
            if let Ok(enum_profiles) = profiles.EnumInputProcessorInfo() {
                let mut profile = [Default::default(); 1];
                let mut fetched = 0;
                loop {
                    let hr = enum_profiles.Next(&mut profile, Some(&mut fetched));
                    if hr != S_OK || fetched == 0 {
                        break;
                    }
                    if profile[0] == CHKEY_TSF_CLSID_GUID {
                        is_registered = true;
                        break;
                    }
                }
            }

            Ok(is_registered)
        }
    }

    /// Kiểm tra cả đăng ký TSF và COM InProcServer32 có trỏ đúng file DLL mong đợi không.
    pub fn is_registered_with_dll(expected_dll: &str) -> bool {
        match Self::is_registered() {
            Ok(true) => {
                if let Some(registered_path) = Self::get_registered_dll_path() {
                    registered_path.trim().eq_ignore_ascii_case(expected_dll.trim())
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Đăng ký COM Server và TSF Language Profile cho file DLL.
    ///
    /// Icon được đóng gói trực tiếp trong file DLL (uIconIndex = 0), không cần file .ico rời.
    pub fn register_profile(dll_path: &str) -> Result<()> {
        log_reg(&format!("register_profile called with DLL: {dll_path}"));

        unsafe {
            let _com_guard = ComGuard::new();
            let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

            // Step 1: Đăng ký COM Server
            // Ghi vào HKCU (User Scope - không cần quyền Administrator)
            let hkcu_clsid_key = format!("Software\\Classes\\CLSID\\{{{}}}", clsid_str);
            let hkcu_inproc_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32", clsid_str);

            let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_clsid_key, "", "CHKey Êđê Text Service");
            let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_inproc_key, "", dll_path);
            let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_inproc_key, "ThreadingModel", "Apartment");

            // Nếu có quyền Administrator, ghi thêm vào HKCR để toàn bộ hệ thống nhận diện
            let clsid_key = format!("CLSID\\{{{}}}", clsid_str);
            let inproc_key = format!("CLSID\\{{{}}}\\InProcServer32", clsid_str);
            let _ = set_registry_sz(HKEY_CLASSES_ROOT, &clsid_key, "", "CHKey Êđê Text Service");
            let _ = set_registry_sz(HKEY_CLASSES_ROOT, &inproc_key, "", dll_path);
            let _ = set_registry_sz(HKEY_CLASSES_ROOT, &inproc_key, "ThreadingModel", "Apartment");

            log_reg("COM InProcServer32 registry keys written successfully");

            // Step 2: Đăng ký TSF Input Processor Profiles
            let profiles: ITfInputProcessorProfiles = match CoCreateInstance(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                Ok(p) => p,
                Err(e) => {
                    log_reg(&format!("Failed CoCreateInstance ITfInputProcessorProfiles: {:?}", e));
                    return Err(e);
                }
            };

            let already_registered = Self::is_registered().unwrap_or(false);
            if !already_registered && let Err(e) = profiles.Register(&CHKEY_TSF_CLSID_GUID) {
                log_reg(&format!("profiles.Register failed: {:?}", e));
                if !Self::is_elevated() {
                    let msg = "Đăng ký TSF Text Service vào hệ thống (HKLM) yêu cầu quyền Administrator. COM Server đã được đăng ký thành công ở phạm vi người dùng (HKCU). Vui lòng nhấp chuột phải vào CHKey.exe và chọn 'Run as administrator' một lần để hoàn tất đăng ký TSF.";
                    log_reg(msg);
                    return Err(windows::core::Error::new(e.code(), msg));
                }
                return Err(e);
            }
            log_reg("CLSID registered with TSF");

            let description = to_utf16_null("Native (Rade)");

            // Xác định nguồn icon hiển thị:
            // 1. Ưu tiên file rade.ico nếu tồn tại cạnh DLL hoặc tại đường dẫn chuẩn
            // 2. Tự động fallback về chính file DLL (icon resource PE đã nhúng sẵn, uIconIndex = 0)
            let dll_p = std::path::Path::new(dll_path);
            let mut icon_target_str = dll_path.to_string();
            let mut icon_index = 0u32;

            let candidates = [
                dll_p.parent().map(|p| p.join("rade.ico")),
                Some(std::path::PathBuf::from(r"C:\Users\ydung\Desktop\Keyboard\CHKey\platforms\windows\tsf\rade.ico")),
            ];

            for candidate in candidates.into_iter().flatten() {
                if candidate.exists() {
                    icon_target_str = candidate.to_string_lossy().to_string();
                    icon_index = 0;
                    break;
                }
            }

            let icon_file = to_utf16_null(&icon_target_str);

            // Gỡ bỏ profile tiếng Việt (0x042A) cũ nếu tồn tại để tránh trùng lặp
            let _ = profiles.RemoveLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_VIETNAMESE,
                &CHKEY_TSF_PROFILE_GUID,
            );
            call_layout_or_tip("UninstallLayoutOrTip", LANGID_VIETNAMESE, &CHKEY_TSF_CLSID_GUID, &CHKEY_TSF_PROFILE_GUID);

            // Gỡ bỏ profile English US cũ nếu có để làm mới cache icon/mô tả của Windows Shell
            let _ = profiles.RemoveLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_ENGLISH_US,
                &CHKEY_TSF_PROFILE_GUID,
            );

            // Đăng ký profile mới
            let add_res = profiles.AddLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_ENGLISH_US,
                &CHKEY_TSF_PROFILE_GUID,
                &description,
                &icon_file,
                icon_index,
            );
            log_reg(&format!("AddLanguageProfile target='{}' index={} result: {:?}", icon_target_str, icon_index, add_res));

            let _ = profiles.EnableLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_ENGLISH_US,
                &CHKEY_TSF_PROFILE_GUID,
                true,
            );

            call_layout_or_tip("InstallLayoutOrTip", LANGID_ENGLISH_US, &CHKEY_TSF_CLSID_GUID, &CHKEY_TSF_PROFILE_GUID);
            log_reg("Language profile 0x0409 registered and enabled");

            // Step 3: Đăng ký Category Keyboard TIP
            if let Ok(category_mgr) = CoCreateInstance::<_, ITfCategoryMgr>(
                &CLSID_TF_CategoryMgr,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                let _ = category_mgr.RegisterCategory(
                    &CHKEY_TSF_CLSID_GUID,
                    &GUID_TFCAT_TIP_KEYBOARD,
                    &CHKEY_TSF_CLSID_GUID,
                );
            }

            log_reg("register_profile completed successfully");
            Ok(())
        }
    }

    /// Làm mới kích hoạt Layout / TIP trên Windows input subsystem.
    pub fn refresh_active_tips() {
        call_layout_or_tip("InstallLayoutOrTip", LANGID_ENGLISH_US, &CHKEY_TSF_CLSID_GUID, &CHKEY_TSF_PROFILE_GUID);
    }

    /// Hủy đăng ký CHKey TSF Text Service Profile khỏi Windows.
    pub fn unregister_profile() -> Result<()> {
        log_reg("unregister_profile called");

        unsafe {
            let _com_guard = ComGuard::new();

            for &langid in &[LANGID_ENGLISH_US, LANGID_VIETNAMESE] {
                call_layout_or_tip("UninstallLayoutOrTip", langid, &CHKEY_TSF_CLSID_GUID, &CHKEY_TSF_PROFILE_GUID);
            }

            if let Ok(category_mgr) = CoCreateInstance::<_, ITfCategoryMgr>(
                &CLSID_TF_CategoryMgr,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                let _ = category_mgr.UnregisterCategory(
                    &CHKEY_TSF_CLSID_GUID,
                    &GUID_TFCAT_TIP_KEYBOARD,
                    &CHKEY_TSF_CLSID_GUID,
                );
            }

            if let Ok(profiles) = CoCreateInstance::<_, ITfInputProcessorProfiles>(
                &CLSID_TF_InputProcessorProfiles,
                None,
                CLSCTX_INPROC_SERVER,
            ) {
                let _ = profiles.Unregister(&CHKEY_TSF_CLSID_GUID);
            }

            let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

            // Xóa HKCU keys
            delete_registry_key(HKEY_CURRENT_USER, &format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32", clsid_str));
            delete_registry_key(HKEY_CURRENT_USER, &format!("Software\\Classes\\CLSID\\{{{}}}", clsid_str));

            // Xóa HKCR keys
            delete_registry_key(HKEY_CLASSES_ROOT, &format!("CLSID\\{{{}}}\\InProcServer32", clsid_str));
            delete_registry_key(HKEY_CLASSES_ROOT, &format!("CLSID\\{{{}}}", clsid_str));

            log_reg("unregister_profile completed");
            Ok(())
        }
    }
}

/// Đăng ký COM Server và TSF profile cho file DLL.
pub fn register_tsf(dll_path: &std::path::Path) -> std::result::Result<(), String> {
    TsfRegistration::register_profile(&dll_path.to_string_lossy())
        .map_err(|e| format!("Lỗi đăng ký TSF profile: {e}"))
}

/// Gỡ đăng ký TSF profile và COM server.
pub fn unregister_tsf() -> std::result::Result<(), String> {
    TsfRegistration::unregister_profile().map_err(|e| format!("Lỗi gỡ đăng ký TSF profile: {e}"))
}

/// Kiểm tra xem TSF profile đã được đăng ký và đường dẫn COM InProcServer32 có khớp với file DLL hiện tại không.
pub fn is_tsf_registered_with_dll(dll_path: &std::path::Path) -> bool {
    TsfRegistration::is_registered_with_dll(&dll_path.to_string_lossy())
}

/// Kiểm tra xem tiến trình hiện tại có đang chạy dưới quyền Administrator hay không.
pub fn is_elevated() -> bool {
    TsfRegistration::is_elevated()
}
