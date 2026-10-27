//! CHKey-Setup: Bộ cài đặt và gỡ cài đặt duy nhất cho CHKey và chkey_ime.dll.
//!
//! Đóng gói CHKey.exe và chkey_ime.dll, cài đặt vào thư mục ổn định %LOCALAPPDATA%\Programs\CHKey,
//! đăng ký COM/TSF, tạo shortcut Start Menu và hỗ trợ gỡ cài đặt sạch sẽ.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use windows::core::{Interface, GUID, PCWSTR, Result};
use windows::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitialize, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegSetValueExW,
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::TextServices::{
    CLSID_TF_CategoryMgr, CLSID_TF_InputProcessorProfiles, GUID_TFCAT_TIP_KEYBOARD,
    ITfCategoryMgr, ITfInputProcessorProfiles,
};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, HWND_BROADCAST, WM_SETTINGCHANGE};

// Nhúng trực tiếp bản build CHKey.exe và chkey_tsf.dll
const EMBEDDED_EXE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/CHKey.exe"));
const EMBEDDED_DLL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/chkey_tsf.dll"));
const EMBEDDED_ICO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/chkey.ico"));
const EMBEDDED_RAD_ICO: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rad.ico"));

// GUID đồng bộ với chkey_ime
const CHKEY_TSF_CLSID_GUID: GUID = GUID::from_u128(0xa3f2b801_1234_4567_89ab_cdef01234567);
const CHKEY_TSF_PROFILE_GUID: GUID = GUID::from_u128(0xa3f2b802_1234_4567_89ab_cdef01234567);
const LANGID_ENGLISH_US: u16 = 0x0409;
const LANGID_VIETNAMESE: u16 = 0x042A;

type LayoutOrTipFn = unsafe extern "system" fn(PCWSTR, u32) -> BOOL;

fn log_setup(msg: &str) {
    println!("[CHKey-Setup] {msg}");
    let log_dir = env::temp_dir().join("CHKey");
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("setup.log");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(f, "[{}] {msg}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    }
}

/// RAII Guard bảo vệ COM apartment
struct ComGuard(bool);
impl ComGuard {
    fn new() -> Self {
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

fn guid_to_string(guid: &GUID) -> String {
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

fn get_install_dir() -> PathBuf {
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("Programs").join("CHKey")
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
}

fn is_elevated() -> bool {
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

fn set_registry_sz(hkey: HKEY, subkey: &str, value_name: &str, data: &str) -> Result<()> {
    unsafe {
        let mut subkey_handle = HKEY::default();
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
        let val_w: Vec<u16> = if value_name.is_empty() {
            vec![0]
        } else {
            format!("{}\0", value_name).encode_utf16().collect()
        };
        let data_w: Vec<u16> = format!("{}\0", data).encode_utf16().collect();

        RegCreateKeyExW(
            hkey,
            PCWSTR(subkey_w.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            None,
            &mut subkey_handle,
            None,
        ).ok()?;

        let val_ptr = if value_name.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(val_w.as_ptr())
        };

        RegSetValueExW(
            subkey_handle,
            val_ptr,
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                data_w.as_ptr() as *const u8,
                data_w.len() * 2,
            )),
        ).ok()?;

        let _ = RegCloseKey(subkey_handle);
        Ok(())
    }
}

unsafe fn call_input_dll_func(func_name: &str, langid: u16) {
    unsafe {
        if let Ok(lib) = LoadLibraryA(windows::core::s!("input.dll")) {
            let name_c = std::ffi::CString::new(func_name).unwrap();
            if let Some(proc) = GetProcAddress(lib, windows::core::PCSTR(name_c.as_ptr() as *const u8)) {
                let func: LayoutOrTipFn = std::mem::transmute(proc);
                let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);
                let profile_str = guid_to_string(&CHKEY_TSF_PROFILE_GUID);
                let tip_string = format!("{:04x}:{{{}}}{{{}}}", langid, clsid_str, profile_str);
                let tip_w: Vec<u16> = format!("{}\0", tip_string).encode_utf16().collect();
                let _ = func(PCWSTR(tip_w.as_ptr()), 0);
            }
        }
    }
}

fn safe_write_file(target: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = target.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let tmp_path = target.with_extension("tmp");
    {
        let mut f = File::create(&tmp_path).map_err(|e| {
            windows::core::Error::new(windows::core::HRESULT(-1), format!("Lỗi tạo file tmp: {e}"))
        })?;
        f.write_all(content).map_err(|e| {
            windows::core::Error::new(windows::core::HRESULT(-1), format!("Lỗi ghi file tmp: {e}"))
        })?;
        f.sync_all().map_err(|e| {
            windows::core::Error::new(windows::core::HRESULT(-1), format!("Lỗi flush file tmp: {e}"))
        })?;
    }

    // Nếu file đích đang tồn tại, thử đổi tên sang .old phòng trường hợp bị khóa
    if target.exists() {
        let old_path = target.with_extension("old");
        let _ = fs::remove_file(&old_path);
        let _ = fs::rename(target, &old_path);
    }

    // Đưa file mới vào
    if let Err(e) = fs::rename(&tmp_path, target) {
        log_setup(&format!("Lỗi đổi tên file tmp sang đích {}: {e}", target.display()));
        let _ = fs::copy(&tmp_path, target);
        let _ = fs::remove_file(&tmp_path);
    }

    Ok(())
}

fn create_shortcut(target_exe: &Path, shortcut_path: &Path, description: &str) -> Result<()> {
    unsafe {
        let _com_guard = ComGuard::new();

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        let target_w: Vec<u16> = format!("{}\0", target_exe.display()).encode_utf16().collect();
        shell_link.SetPath(PCWSTR(target_w.as_ptr()))?;

        if let Some(dir) = target_exe.parent() {
            let dir_w: Vec<u16> = format!("{}\0", dir.display()).encode_utf16().collect();
            shell_link.SetWorkingDirectory(PCWSTR(dir_w.as_ptr()))?;
        }

        let desc_w: Vec<u16> = format!("{}\0", description).encode_utf16().collect();
        shell_link.SetDescription(PCWSTR(desc_w.as_ptr()))?;

        let persist_file: IPersistFile = shell_link.cast()?;
        if let Some(parent) = shortcut_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let lnk_w: Vec<u16> = format!("{}\0", shortcut_path.display()).encode_utf16().collect();
        persist_file.Save(PCWSTR(lnk_w.as_ptr()), true)?;

        Ok(())
    }
}

fn register_com_and_tsf(dll_path: &Path) -> Result<()> {
    let dll_str = dll_path.to_string_lossy();
    let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

    log_setup(&format!("Đang đăng ký COM InProcServer32 cho DLL: {dll_str}"));

    // 1. Ghi COM InProcServer32 vào HKCU (User Scope)
    let hkcu_clsid_key = format!("Software\\Classes\\CLSID\\{{{}}}", clsid_str);
    let hkcu_inproc_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32", clsid_str);
    let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_clsid_key, "", "CHKey Êđê Text Service");
    let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_inproc_key, "", &dll_str);
    let _ = set_registry_sz(HKEY_CURRENT_USER, &hkcu_inproc_key, "ThreadingModel", "Apartment");

    // 2. Nếu có quyền Admin, ghi thêm vào HKCR
    if is_elevated() {
        let clsid_key = format!("CLSID\\{{{}}}", clsid_str);
        let inproc_key = format!("CLSID\\{{{}}}\\InProcServer32", clsid_str);
        let _ = set_registry_sz(HKEY_CLASSES_ROOT, &clsid_key, "", "CHKey Êđê Text Service");
        let _ = set_registry_sz(HKEY_CLASSES_ROOT, &inproc_key, "", &dll_str);
        let _ = set_registry_sz(HKEY_CLASSES_ROOT, &inproc_key, "ThreadingModel", "Apartment");
    }

    log_setup("Đã ghi thông tin COM InProcServer32");

    // 3. Đăng ký TSF Input Processor Profile
    unsafe {
        let _com_guard = ComGuard::new();

        if let Ok(profiles) = CoCreateInstance::<_, ITfInputProcessorProfiles>(
            &CLSID_TF_InputProcessorProfiles,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = profiles.Register(&CHKEY_TSF_CLSID_GUID);

            let desc: Vec<u16> = "Native (Rade)\0".encode_utf16().collect();

            // Gỡ bỏ cấu hình cũ của tiếng Việt (0x042A) nếu có để tránh hiển thị trùng lặp
            let _ = profiles.RemoveLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_VIETNAMESE,
                &CHKEY_TSF_PROFILE_GUID,
            );
            call_input_dll_func("UninstallLayoutOrTip", LANGID_VIETNAMESE);

            // Gỡ bỏ profile English US (0x0409) cũ nếu có để cập nhật thông tin mới nhất
            let _ = profiles.RemoveLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_ENGLISH_US,
                &CHKEY_TSF_PROFILE_GUID,
            );

            let icon_file: Vec<u16> = format!("{}\0", dll_str).encode_utf16().collect();

            let _ = profiles.AddLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_ENGLISH_US,
                &CHKEY_TSF_PROFILE_GUID,
                &desc,
                &icon_file,
                0,
            );
            let _ = profiles.EnableLanguageProfile(
                &CHKEY_TSF_CLSID_GUID,
                LANGID_ENGLISH_US,
                &CHKEY_TSF_PROFILE_GUID,
                true,
            );
            call_input_dll_func("InstallLayoutOrTip", LANGID_ENGLISH_US);
        }

        // Đăng ký category bàn phím
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

        // 4. Thông báo Windows cập nhật Text Service
        let intl_w: Vec<u16> = "intl\0".encode_utf16().collect();
        let _ = PostMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), LPARAM(intl_w.as_ptr() as isize));
    }

    log_setup("Đăng ký TSF profile hoàn tất.");
    Ok(())
}

fn register_uninstall_entry(install_dir: &Path, uninstaller_exe: &Path, exe_path: &Path) {
    let uninstall_key = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CHKey";
    let _ = set_registry_sz(HKEY_CURRENT_USER, uninstall_key, "DisplayName", "CHKey - Bộ gõ tiếng Êđê & Tây Nguyên");
    let _ = set_registry_sz(HKEY_CURRENT_USER, uninstall_key, "DisplayVersion", "0.1.0");
    let _ = set_registry_sz(HKEY_CURRENT_USER, uninstall_key, "Publisher", "CHKey Team");
    let _ = set_registry_sz(HKEY_CURRENT_USER, uninstall_key, "InstallLocation", &install_dir.to_string_lossy());
    let _ = set_registry_sz(HKEY_CURRENT_USER, uninstall_key, "DisplayIcon", &format!("{},0", exe_path.to_string_lossy()));
    let _ = set_registry_sz(HKEY_CURRENT_USER, uninstall_key, "UninstallString", &format!("\"{}\" --uninstall", uninstaller_exe.to_string_lossy()));
    log_setup("Đã đăng ký mục Gỡ cài đặt trong Windows Settings / Control Panel");
}

fn run_install() -> Result<()> {
    log_setup("=== BẮT ĐẦU CÀI ĐẶT CHKEY ===");

    let install_dir = get_install_dir();
    log_setup(&format!("Thư mục cài đặt: {}", install_dir.display()));

    fs::create_dir_all(&install_dir).map_err(|e| {
        windows::core::Error::new(windows::core::HRESULT(-1), format!("Không thể tạo thư mục cài đặt: {e}"))
    })?;

    // 1. Sao chép CHKey.exe
    let target_exe = install_dir.join("CHKey.exe");
    log_setup(&format!("Trích xuất CHKey.exe ({:.2} MB)...", EMBEDDED_EXE.len() as f64 / 1_048_576.0));
    safe_write_file(&target_exe, EMBEDDED_EXE)?;

    // 2. Sao chép chkey_tsf.dll và chkey.ico
    let target_dll = install_dir.join("chkey_tsf.dll");
    log_setup(&format!("Trích xuất chkey_tsf.dll ({:.2} KB)...", EMBEDDED_DLL.len() as f64 / 1024.0));
    safe_write_file(&target_dll, EMBEDDED_DLL)?;

    let target_ico = install_dir.join("chkey.ico");
    let _ = safe_write_file(&target_ico, EMBEDDED_ICO);

    let target_rad_ico = install_dir.join("rad.ico");
    let _ = safe_write_file(&target_rad_ico, EMBEDDED_RAD_ICO);
    let target_rade_ico = install_dir.join("rade.ico");
    let _ = safe_write_file(&target_rade_ico, EMBEDDED_RAD_ICO);

    // 3. Sao chép chính file setup thành Uninstall.exe
    let uninstaller_exe = install_dir.join("Uninstall.exe");
    if let Ok(current_exe) = env::current_exe() {
        let _ = fs::copy(&current_exe, &uninstaller_exe);
    }

    // 4. Đăng ký COM Server và TSF profile
    if let Err(e) = register_com_and_tsf(&target_dll) {
        log_setup(&format!("Cảnh báo đăng ký TSF: {e}"));
    }

    // 5. Tạo Shortcut trong Start Menu
    if let Ok(app_data) = env::var("APPDATA") {
        let start_menu_lnk = PathBuf::from(app_data)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("CHKey.lnk");
        if let Err(e) = create_shortcut(&target_exe, &start_menu_lnk, "CHKey - Bộ gõ tiếng Êđê & Tây Nguyên") {
            log_setup(&format!("Không thể tạo shortcut Start Menu: {e}"));
        } else {
            log_setup(&format!("Đã tạo shortcut tại: {}", start_menu_lnk.display()));
        }
    }

    // 6. Đăng ký mục gỡ cài đặt vào Windows Programs and Features
    register_uninstall_entry(&install_dir, &uninstaller_exe, &target_exe);

    log_setup("Cài đặt thành công!");

    // 7. Khởi chạy ứng dụng CHKey.exe
    if !env::args().any(|arg| arg == "--no-launch") {
        log_setup(&format!("Đang khởi chạy: {}", target_exe.display()));
        let mut cmd = Command::new(&target_exe);
        cmd.current_dir(&install_dir);
        let _ = cmd.spawn();
    }

    Ok(())
}

fn run_uninstall() -> Result<()> {
    log_setup("=== BẮT ĐẦU GỠ CÀI ĐẶT CHKEY ===");

    let install_dir = get_install_dir();
    let clsid_str = guid_to_string(&CHKEY_TSF_CLSID_GUID);

    // 1. Gỡ Layout/TIP khỏi người dùng
    unsafe {
        for &langid in &[LANGID_ENGLISH_US, LANGID_VIETNAMESE] {
            call_input_dll_func("UninstallLayoutOrTip", langid);
        }

        let _com_guard = ComGuard::new();

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

        let intl_w: Vec<u16> = "intl\0".encode_utf16().collect();
        let _ = PostMessageW(HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), LPARAM(intl_w.as_ptr() as isize));
    }

    // 2. Xóa Registry COM
    unsafe {
        let hkcu_inproc_key = format!("Software\\Classes\\CLSID\\{{{}}}\\InProcServer32\0", clsid_str);
        let hkcu_clsid_key = format!("Software\\Classes\\CLSID\\{{{}}}\0", clsid_str);
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(hkcu_inproc_key.encode_utf16().collect::<Vec<_>>().as_ptr()));
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(hkcu_clsid_key.encode_utf16().collect::<Vec<_>>().as_ptr()));

        let inproc_key = format!("CLSID\\{{{}}}\\InProcServer32\0", clsid_str);
        let clsid_key = format!("CLSID\\{{{}}}\0", clsid_str);
        let _ = RegDeleteKeyW(HKEY_CLASSES_ROOT, PCWSTR(inproc_key.encode_utf16().collect::<Vec<_>>().as_ptr()));
        let _ = RegDeleteKeyW(HKEY_CLASSES_ROOT, PCWSTR(clsid_key.encode_utf16().collect::<Vec<_>>().as_ptr()));

        // Xóa Registry Uninstall
        let uninst_key = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CHKey\0";
        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR(uninst_key.encode_utf16().collect::<Vec<_>>().as_ptr()));
    }
    log_setup("Đã xóa Registry COM và Uninstall");

    // 3. Xóa Start Menu shortcut
    if let Ok(app_data) = env::var("APPDATA") {
        let start_menu_lnk = PathBuf::from(app_data)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("CHKey.lnk");
        let _ = fs::remove_file(start_menu_lnk);
    }

    // 4. Xóa CHKey.exe và chkey_tsf.dll (cùng file legacy nếu có)
    let _ = fs::remove_file(install_dir.join("CHKey.exe"));
    let _ = fs::remove_file(install_dir.join("chkey_tsf.dll"));
    let _ = fs::remove_file(install_dir.join("chkey_tsf.dll.old"));
    let _ = fs::remove_file(install_dir.join("chkey_ime.dll"));
    let _ = fs::remove_file(install_dir.join("chkey_ime.dll.old"));
    let _ = fs::remove_file(install_dir.join("CHKey.exe.old"));
    log_setup("Đã xóa các file ứng dụng");

    // 5. Lên lịch tự xóa Uninstall.exe
    let uninstaller_exe = install_dir.join("Uninstall.exe");
    if uninstaller_exe.exists() {
        let script = format!(
            "ping 127.0.0.1 -n 2 > nul & del \"{}\" & rmdir \"{}\"",
            uninstaller_exe.display(),
            install_dir.display()
        );
        let _ = Command::new("cmd").args(["/C", &script]).spawn();
    }

    log_setup("Gỡ cài đặt hoàn tất.");
    println!("Gỡ cài đặt CHKey thành công!");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--uninstall") {
        if let Err(e) = run_uninstall() {
            eprintln!("Lỗi gỡ cài đặt: {e}");
        }
    } else if let Err(e) = run_install() {
        eprintln!("Lỗi cài đặt: {e}");
    }
}
