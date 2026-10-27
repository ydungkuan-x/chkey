//! Module chuyển đổi và đồng bộ chế độ gõ CHKey qua Windows Text Services Framework (TSF).
//!
//! Hỗ trợ:
//! - Đổi chế độ gõ sang "EDE" (CHKey Êđê TSF Profile) hoặc "ENG" (US English Keyboard).
//! - Đọc chế độ gõ hiện tại từ Windows TSF thông qua `EnumEnabledLayoutOrTip` (input.dll) và TSF Profile Manager.
//! - Lắng nghe thay đổi chế độ gõ realtime (khi người dùng nhấn Win + Space hoặc chọn từ Windows Language Flyout).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::TextServices::{
    CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD, ITfInputProcessorProfileMgr,
    ITfInputProcessorProfiles,
};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, HWND_BROADCAST, WM_SETTINGCHANGE,
};
use crate::profile::{CHKEY_TSF_CLSID_GUID, LANGID_ENGLISH_US};
use crate::utils::ComGuard;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct LayoutOrTipProfile {
    dw_profile_type: u32,
    langid: u16,
    clsid: windows::core::GUID,
    guid_profile: windows::core::GUID,
    catid: windows::core::GUID,
    dw_substitute_layout: u32,
    dw_flags: u32,
    sz_id: [u16; 260],
}

use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_DWORD, REG_OPEN_CREATE_OPTIONS, REG_SZ,
};

type SetDefaultLayoutOrTipFn = unsafe extern "system" fn(PCWSTR, u32) -> BOOL;
type EnumEnabledLayoutOrTipFn = unsafe extern "system" fn(PCWSTR, PCWSTR, PCWSTR, *mut LayoutOrTipProfile, u32) -> u32;

struct InputDll {
    set_default: Option<SetDefaultLayoutOrTipFn>,
    enum_enabled: Option<EnumEnabledLayoutOrTipFn>,
}

static INPUT_DLL: std::sync::OnceLock<InputDll> = std::sync::OnceLock::new();

fn get_input_dll() -> &'static InputDll {
    INPUT_DLL.get_or_init(|| unsafe {
        if let Ok(lib) = LoadLibraryA(windows::core::s!("input.dll")) {
            let set_default = GetProcAddress(lib, windows::core::s!("SetDefaultLayoutOrTip"))
                .map(|p| std::mem::transmute(p));
            let enum_enabled = GetProcAddress(lib, windows::core::s!("EnumEnabledLayoutOrTip"))
                .map(|p| std::mem::transmute(p));
            InputDll { set_default, enum_enabled }
        } else {
            InputDll { set_default: None, enum_enabled: None }
        }
    })
}

const CTF_ASSEMBLY_SUBKEY: PCWSTR = windows::core::w!("Software\\Microsoft\\CTF\\Assemblies\\0x00000409\\{34745C63-B2F0-4784-8B67-5E12C8701A31}");
const VAL_DEFAULT: PCWSTR = windows::core::w!("Default");
const VAL_PROFILE: PCWSTR = windows::core::w!("Profile");
const VAL_KEYBOARD_LAYOUT: PCWSTR = windows::core::w!("KeyboardLayout");
const TIP_EDE_W: PCWSTR = windows::core::w!("0409:{A3F2B801-1234-4567-89AB-CDEF01234567}{A3F2B802-1234-4567-89AB-CDEF01234567}");
const TIP_ENG_W: PCWSTR = windows::core::w!("0409:00000409");
const CLSID_EDE_W: PCWSTR = windows::core::w!("{A3F2B801-1234-4567-89AB-CDEF01234567}");
const PROFILE_EDE_W: PCWSTR = windows::core::w!("{A3F2B802-1234-4567-89AB-CDEF01234567}");
const GUID_NULL_W: PCWSTR = windows::core::w!("{00000000-0000-0000-0000-000000000000}");
const MSG_INTL_W: PCWSTR = windows::core::w!("intl");
const MSG_INPUT_W: PCWSTR = windows::core::w!("Input");

/// Đồng bộ trạng thái cấu hình TSF Assembly trong Registry người dùng.
/// Windows Language Flyout (menu chọn ngôn ngữ ở taskbar) và CTF đọc trực tiếp khóa này
/// để biết profile nào dưới 0x0409 (US Keyboard hay CHKey Êđê) đang được chọn làm Default.
fn update_ctf_assembly_registry(is_ede: bool) {
    let mut hkey = HKEY::default();
    unsafe {
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            CTF_ASSEMBLY_SUBKEY,
            0,
            PCWSTR::null(),
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        )
        .is_ok()
        {
            let (def_w, profile_w, layout_val) = if is_ede {
                (CLSID_EDE_W, PROFILE_EDE_W, 0u32)
            } else {
                (GUID_NULL_W, GUID_NULL_W, 0x04090409u32)
            };

            let def_slice = def_w.as_wide();
            let _ = RegSetValueExW(
                hkey,
                VAL_DEFAULT,
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    def_slice.as_ptr() as *const u8,
                    (def_slice.len() + 1) * 2,
                )),
            );

            let prof_slice = profile_w.as_wide();
            let _ = RegSetValueExW(
                hkey,
                VAL_PROFILE,
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    prof_slice.as_ptr() as *const u8,
                    (prof_slice.len() + 1) * 2,
                )),
            );

            let _ = RegSetValueExW(
                hkey,
                VAL_KEYBOARD_LAYOUT,
                0,
                REG_DWORD,
                Some(std::slice::from_raw_parts(
                    &layout_val as *const u32 as *const u8,
                    std::mem::size_of::<u32>(),
                )),
            );

            let _ = RegCloseKey(hkey);
        }
    }
}

/// Chuyển đổi TSF Text Service đang hoạt động của Windows sang EDE hoặc ENG.
pub fn set_windows_input_mode(mode: &str) -> Result<(), String> {
    let is_ede = mode.trim().eq_ignore_ascii_case("ede");
    let tip_w = if is_ede { TIP_EDE_W } else { TIP_ENG_W };

    unsafe {
        // 1. Áp dụng qua SetDefaultLayoutOrTip trong input.dll (chuẩn TSF)
        if let Some(set_default) = get_input_dll().set_default {
            let _ = set_default(tip_w, 0);
            let _ = set_default(tip_w, 0x02);
        }

        // 2. Cập nhật Registry CTF Assemblies để Windows Language Flyout nhận diện tức thì
        update_ctf_assembly_registry(is_ede);

        // 3. Đổi input language và kích hoạt profile qua TSF COM interface
        let _com = ComGuard::new();
        if let Ok(profiles) = CoCreateInstance::<_, ITfInputProcessorProfiles>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = profiles.ChangeCurrentLanguage(LANGID_ENGLISH_US);
            if is_ede {
                let _ = profiles.ActivateLanguageProfile(
                    &CHKEY_TSF_CLSID_GUID,
                    LANGID_ENGLISH_US,
                    &crate::profile::CHKEY_TSF_PROFILE_GUID,
                );
            }
        }

        // 4. Gửi thông báo thay đổi TSF intl và Input cho toàn bộ hệ thống
        let _ = PostMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(MSG_INTL_W.as_ptr() as isize),
        );
        let _ = PostMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(MSG_INPUT_W.as_ptr() as isize),
        );
    }

    Ok(())
}

fn read_ctf_assembly_registry() -> Option<&'static str> {
    let mut hkey = HKEY::default();
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, CTF_ASSEMBLY_SUBKEY, 0, KEY_READ, &mut hkey).is_ok() {
            let mut data = [0u16; 64];
            let mut len = (data.len() * 2) as u32;
            let res = RegQueryValueExW(
                hkey,
                VAL_DEFAULT,
                None,
                None,
                Some(data.as_mut_ptr() as *mut u8),
                Some(&mut len),
            );
            let _ = RegCloseKey(hkey);

            if res.is_ok() && len >= 2 {
                let chars = &data[..(len as usize / 2)];
                let ede_wide = CLSID_EDE_W.as_wide();
                let null_wide = GUID_NULL_W.as_wide();
                if eq_ignore_ascii_case_u16(chars, ede_wide) {
                    return Some("EDE");
                } else if eq_ignore_ascii_case_u16(chars, null_wide) {
                    return Some("ENG");
                }
            }
        }
    }
    None
}

#[inline]
fn eq_ignore_ascii_case_u16(a: &[u16], b: &[u16]) -> bool {
    let a_clean = match a.iter().position(|&c| c == 0) {
        Some(idx) => &a[..idx],
        None => a,
    };
    let b_clean = match b.iter().position(|&c| c == 0) {
        Some(idx) => &b[..idx],
        None => b,
    };
    a_clean.len() == b_clean.len()
        && a_clean.iter().zip(b_clean).all(|(&x, &y)| {
            if x <= 127 && y <= 127 {
                (x as u8).eq_ignore_ascii_case(&(y as u8))
            } else {
                x == y
            }
        })
}

/// Lấy chế độ gõ hiện tại của Windows ("EDE" hoặc "ENG") thông qua TSF.
/// - Tối ưu hóa: Dùng stack array cố định thay vì cấp phát heap (Vec) mỗi lần gọi.
pub fn get_windows_input_mode() -> &'static str {
    unsafe {
        // 1. Ưu tiên cao nhất: Kiểm tra cờ LOTP_CURRENT (0x0002) qua EnumEnabledLayoutOrTip trong input.dll.
        // Đây là API chuẩn TSF phản ánh trực tiếp lựa chọn của người dùng trên Language Flyout.
        if let Some(enum_fn) = get_input_dll().enum_enabled {
            let mut buf = [std::mem::zeroed::<LayoutOrTipProfile>(); 16];
            let fetched = enum_fn(PCWSTR::null(), PCWSTR::null(), PCWSTR::null(), buf.as_mut_ptr(), 16);
            if fetched > 0 {
                let count = (fetched as usize).min(16);
                for p in &buf[..count] {
                    if (p.dw_flags & 0x0002) != 0 {
                        if p.langid != LANGID_ENGLISH_US {
                            return "ENG";
                        }
                        return if p.clsid == CHKEY_TSF_CLSID_GUID {
                            "EDE"
                        } else {
                            "ENG"
                        };
                    }
                }
            }
        }

        // 2. Kiểm tra cấu hình Registry CTF Assemblies của 0x0409
        if let Some(mode) = read_ctf_assembly_registry() {
            return mode;
        }

        // 3. Fallback: Kiểm tra active profile của TSF subsystem qua ITfInputProcessorProfileMgr
        let _com = ComGuard::new();
        if let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let mut profile = std::mem::zeroed();
            if mgr.GetActiveProfile(&GUID_TFCAT_TIP_KEYBOARD, &mut profile).is_ok() {
                if profile.langid != 0 && profile.langid != LANGID_ENGLISH_US {
                    return "ENG";
                }
                if profile.clsid == CHKEY_TSF_CLSID_GUID {
                    return "EDE";
                }
            }
        }
    }

    "ENG"
}

static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Lắng nghe sự kiện đổi layout bàn phím / TSF Profile từ Windows (ví dụ qua Win + Space hoặc Taskbar).
/// Khi Windows thay đổi chế độ gõ, gọi callback `F(&'static str)` với `"EDE"` hoặc `"ENG"`.
pub fn watch_windows_input_mode<F>(callback: F)
where
    F: Fn(&'static str) + Send + 'static,
{
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(move || {
        let _com = ComGuard::new();
        let mut last_mode = get_windows_input_mode();
        loop {
            std::thread::sleep(Duration::from_millis(150));
            let current = get_windows_input_mode();
            if current != last_mode {
                last_mode = current;
                callback(current);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_switch_updates_registry() {
        let initial_reg = read_ctf_assembly_registry();

        assert!(set_windows_input_mode("EDE").is_ok());
        assert_eq!(read_ctf_assembly_registry(), Some("EDE"));

        assert!(set_windows_input_mode("ENG").is_ok());
        assert_eq!(read_ctf_assembly_registry(), Some("ENG"));

        // Restore initial
        if let Some(mode) = initial_reg {
            let _ = set_windows_input_mode(mode);
        }
    }

    #[test]
    fn test_get_windows_input_mode_returns_valid() {
        let mode = get_windows_input_mode();
        assert!(mode == "EDE" || mode == "ENG");
    }

    #[test]
    fn test_eq_ignore_ascii_case_u16() {
        let a: Vec<u16> = "AbCd-1234\0".encode_utf16().collect();
        let b: Vec<u16> = "abcd-1234\0".encode_utf16().collect();
        let c: Vec<u16> = "abcd-5678\0".encode_utf16().collect();
        assert!(eq_ignore_ascii_case_u16(&a, &b));
        assert!(!eq_ignore_ascii_case_u16(&a, &c));
    }
}
