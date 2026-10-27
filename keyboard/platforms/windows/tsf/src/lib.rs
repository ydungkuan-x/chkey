//! chkey_tsf: Windows Text Services Framework (TSF) Implementation cho CHKey.
//!
//! Thư viện COM In-Process Server (DLL) triển khai chuẩn bàn phím TSF hiện đại của Microsoft Windows,
//! hỗ trợ gõ tiếng Êđê & Tây Nguyên trực tiếp vào mọi ứng dụng Windows.

pub mod composition;
pub mod layout_switcher;
pub mod profile;
pub mod registration;
pub mod send_input;
pub mod service;
pub mod status;
pub mod text_service;
pub mod utils;

pub use composition::CompositionManager;
pub use layout_switcher::{get_windows_input_mode, set_windows_input_mode, watch_windows_input_mode};
pub use profile::{
    CHKEY_TSF_CLSID_GUID, CHKEY_TSF_PROFILE_GUID, LANGID_ENGLISH_US, LANGID_VIETNAMESE,
};
pub use registration::{
    is_elevated, is_tsf_registered_with_dll, register_tsf, unregister_tsf, TsfRegistration,
};
pub use service::TsfClassFactory;
pub use status::{
    check_tsf_status, get_installed_dll_path, repair_tsf, request_elevated_repair, uninstall_tsf,
    TsfStatus,
};
pub use text_service::TsfTextService;

use std::{ffi::c_void, path::PathBuf};
use windows::{
    core::{Interface, GUID, HRESULT},
    Win32::{
        Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_FAIL, S_OK},
        System::Com::IClassFactory,
    },
};

/// Thông báo cho Windows cập nhật trạng thái Text Service sau khi đăng ký thành công.
pub fn notify_windows_tsf_changed() {
    // 1. Broadcast WM_SETTINGCHANGE ("intl") để thông báo các ứng dụng đang chạy
    unsafe {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, HWND_BROADCAST, WM_SETTINGCHANGE};
        let intl_w = utils::to_utf16_null("intl");
        let _ = PostMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(intl_w.as_ptr() as isize),
        );
    }

    // 2. Làm mới TIP trong danh sách bàn phím đang hoạt động của người dùng
    registration::TsfRegistration::refresh_active_tips();
}

/// COM entry point: Windows gọi hàm này để lấy COM class factory.
///
/// # Safety
/// Caller must pass valid COM GUID pointers and an out-pointer for the interface.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_FAIL;
    }

    unsafe {
        *ppv = std::ptr::null_mut();

        if *rclsid != profile::CHKEY_TSF_CLSID_GUID {
            return CLASS_E_CLASSNOTAVAILABLE;
        }

        let factory: IClassFactory = service::TsfClassFactory.into();
        factory.query(&*riid, ppv)
    }
}

/// COM entry point: Windows gọi hàm này để xác định DLL có thể dỡ khỏi bộ nhớ hay không.
///
/// # Safety
/// Always safe to call from Windows COM subsystem.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_OK
}

/// Lấy đường dẫn DLL hiện tại thông qua Win32 module handle.
fn current_dll_path() -> Result<PathBuf, std::io::Error> {
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::{
        GetModuleFileNameW, GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    };
    use windows::core::PCWSTR;

    unsafe {
        let mut hmodule = HMODULE::default();
        let flag = GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
        let addr = DllRegisterServer as *const c_void;
        if GetModuleHandleExW(flag, PCWSTR(addr as *const u16), &mut hmodule).is_ok() {
            let mut buf = vec![0u16; 1024];
            let len = GetModuleFileNameW(hmodule, &mut buf);
            if len > 0 {
                let os_str = std::ffi::OsString::from_wide(&buf[..len as usize]);
                return Ok(PathBuf::from(os_str));
            }
        }
    }

    // Fallback: nếu gọi ngoài ngữ cảnh DLL
    std::env::current_exe()
        .and_then(|_| std::env::current_dir())
        .map(|path| path.join("chkey_tsf.dll"))
}

fn log_debug(msg: &str) {
    utils::log_tsf_msg("chkey_register.log", msg);
}

/// COM entry point: đăng ký TSF profile.
///
/// # Safety
/// Safe to call during regsvr32 or COM installation.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    log_debug("DllRegisterServer called");

    let dll_path = match current_dll_path() {
        Ok(path) => path,
        Err(err) => {
            log_debug(&format!("Cannot determine DLL path: {err}"));
            return E_FAIL;
        }
    };

    match registration::TsfRegistration::register_profile(&dll_path.to_string_lossy()) {
        Ok(()) => {
            log_debug("TSF profile registered successfully");
            S_OK
        }
        Err(err) => {
            log_debug(&format!("Registration failed: {err:?}"));
            err.code()
        }
    }
}

/// COM entry point: hủy đăng ký TSF profile.
///
/// # Safety
/// Safe to call during regsvr32 /u or COM uninstallation.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    log_debug("DllUnregisterServer called");

    match registration::TsfRegistration::unregister_profile() {
        Ok(()) => {
            log_debug("TSF profile unregistered successfully");
            S_OK
        }
        Err(err) => {
            log_debug(&format!("Unregistration failed: {err:?}"));
            err.code()
        }
    }
}
