#![windows_subsystem = "windows"]

pub mod shutdown;
mod single_instance;
mod startup;
mod system_tray;
mod theme;

use chkey_tsf as tsf;
pub use shutdown::cleanup_tsf_on_exit;

use raw_window_handle::HasWindowHandle;
use std::process::Command;

pub use chkey_ui::{ComponentHandle, MainWindow, UpdateWindow};
use chkey_updater::{
    DownloadProgress, UpdateInfo, UpdateStatus, apply_update, check_update, download_update,
};

fn open_link(url: &str) {
    let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
}

#[cfg(target_os = "windows")]
pub fn setup_window_icon(window: &slint::Window) {
    let wh = window.window_handle();
    if let Ok(h) = wh.window_handle()
        && let raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw()
    {
        let hwnd = windows::Win32::Foundation::HWND(w32.hwnd.get() as _);
        unsafe {
            use windows::Win32::System::LibraryLoader::GetModuleHandleW;
            use windows::Win32::UI::WindowsAndMessaging::{
                CreateIconFromResourceEx, GetSystemMetrics, HICON, ICON_BIG, ICON_SMALL,
                IMAGE_ICON, LR_DEFAULTCOLOR, LR_SHARED, LoadImageW, SM_CXICON, SM_CXSMICON,
                SM_CYICON, SM_CYSMICON, SendMessageW, WM_SETICON,
            };
            use windows::core::PCWSTR;

            let h_instance = GetModuleHandleW(None).unwrap_or_default();
            let cx_big = GetSystemMetrics(SM_CXICON);
            let cy_big = GetSystemMetrics(SM_CYICON);
            let cx_small = GetSystemMetrics(SM_CXSMICON);
            let cy_small = GetSystemMetrics(SM_CYSMICON);

            // 1. Load icon đã nhúng trong tài nguyên Win32 PE (ID 1)
            let mut icon_big = LoadImageW(
                h_instance,
                PCWSTR(1 as _),
                IMAGE_ICON,
                cx_big,
                cy_big,
                LR_SHARED,
            )
            .ok()
            .map(|h| HICON(h.0));

            let mut icon_small = LoadImageW(
                h_instance,
                PCWSTR(1 as _),
                IMAGE_ICON,
                cx_small,
                cy_small,
                LR_SHARED,
            )
            .ok()
            .map(|h| HICON(h.0));

            // 2. Dự phòng: Nếu chưa load được từ PE resource, tạo HICON từ PNG bytes
            const PNG_BYTES: &[u8] = include_bytes!("../../../chkey.png");
            if icon_big.is_none() {
                if let Ok(h) = CreateIconFromResourceEx(
                    PNG_BYTES,
                    true,
                    0x00030000,
                    cx_big,
                    cy_big,
                    LR_DEFAULTCOLOR,
                ) {
                    icon_big = Some(h);
                }
            }
            if icon_small.is_none() {
                if let Ok(h) = CreateIconFromResourceEx(
                    PNG_BYTES,
                    true,
                    0x00030000,
                    cx_small,
                    cy_small,
                    LR_DEFAULTCOLOR,
                ) {
                    icon_small = Some(h);
                }
            }

            // 3. Gửi thông điệp WM_SETICON để Taskbar và Titlebar hiển thị đúng icon
            if let Some(hicon) = icon_big {
                SendMessageW(
                    hwnd,
                    WM_SETICON,
                    windows::Win32::Foundation::WPARAM(ICON_BIG as _),
                    windows::Win32::Foundation::LPARAM(hicon.0 as _),
                );
            }
            if let Some(hicon) = icon_small {
                SendMessageW(
                    hwnd,
                    WM_SETICON,
                    windows::Win32::Foundation::WPARAM(ICON_SMALL as _),
                    windows::Win32::Foundation::LPARAM(hicon.0 as _),
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn bring_window_to_top_and_foreground(window: &slint::Window) {
    setup_window_icon(window);
    shutdown::hook_window_shutdown_messages(window);

    let wh = window.window_handle();
    if let Ok(h) = wh.window_handle()
        && let raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw()
    {
        let hwnd = windows::Win32::Foundation::HWND(w32.hwnd.get() as _);
        unsafe {
            use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
            use windows::Win32::UI::WindowsAndMessaging::{
                BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, HWND_NOTOPMOST,
                HWND_TOPMOST, SW_RESTORE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE, SetForegroundWindow,
                SetWindowPos, ShowWindow,
            };

            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = ShowWindow(hwnd, SW_SHOW);

            let foreground_hwnd = GetForegroundWindow();
            let foreground_thread_id = GetWindowThreadProcessId(foreground_hwnd, None);
            let current_thread_id = GetCurrentThreadId();

            if foreground_thread_id != 0 && foreground_thread_id != current_thread_id {
                let _ = AttachThreadInput(foreground_thread_id, current_thread_id, true);
                let _ = BringWindowToTop(hwnd);
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                let _ = SetForegroundWindow(hwnd);
                let _ = AttachThreadInput(foreground_thread_id, current_thread_id, false);
            } else {
                let _ = BringWindowToTop(hwnd);
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
}

pub use shutdown::TsfExitGuard;

thread_local! {
    static UPDATE_DIALOG_HOLDER: std::cell::RefCell<Option<UpdateWindow>> = const { std::cell::RefCell::new(None) };
}

/// Mở hộp thoại Cập nhật phiên bản (UpdateWindow).
/// - `initial_info`: Nếu có (từ luồng tự động kiểm tra), hiển thị ngay kết quả cập nhật mà không cần kiểm tra lại.
/// - Nếu `initial_info` là `None` (người dùng bấm nút thủ công), kích hoạt kiểm tra với server.
fn open_update_dialog(main_win: &MainWindow, initial_info: Option<UpdateInfo>) {
    let already_open = UPDATE_DIALOG_HOLDER.with(|holder| holder.borrow().is_some());
    if already_open {
        UPDATE_DIALOG_HOLDER.with(|holder| {
            if let Some(existing) = holder.borrow().as_ref() {
                let _ = existing.show();
                #[cfg(target_os = "windows")]
                bring_window_to_top_and_foreground(existing.window());
            }
        });
        return;
    }

    if let Ok(dialog) = UpdateWindow::new() {
        main_win.set_overlay_visible(true);

        dialog.set_i18n(main_win.get_i18n());
        let is_dark = theme::is_dark_mode();
        dialog.set_dark_mode(is_dark);
        let current_ver = env!("CARGO_PKG_VERSION");
        dialog.set_current_version(current_ver.into());

        let update_info: std::sync::Arc<std::sync::Mutex<Option<UpdateInfo>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let downloaded_file: std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));

        if let Some(info) = initial_info {
            dialog.set_is_checking(false);
            dialog.set_latest_version(info.latest_version.clone().into());
            if let Ok(mut lock) = update_info.lock() {
                *lock = Some(info);
            }
        } else {
            dialog.set_is_checking(true);
            let update_info_for_check = update_info.clone();
            let dialog_weak_update = dialog.as_weak();

            tokio::spawn(async move {
                let result = check_update().await;

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(dlg) = dialog_weak_update.upgrade() {
                        dlg.set_is_checking(false);
                        match result {
                            Ok(UpdateStatus::UpToDate) => {
                                println!("CHKey đang sử dụng phiên bản mới nhất.");
                                dlg.set_latest_version(current_ver.into());
                            }
                            Ok(UpdateStatus::UpdateAvailable(info)) => {
                                println!("Có phiên bản mới của CHKey: v{}", info.latest_version);
                                if let Ok(mut lock) = update_info_for_check.lock() {
                                    *lock = Some(info.clone());
                                }
                                dlg.set_latest_version(info.latest_version.into());
                            }
                            Err(error) => {
                                eprintln!("Lỗi kiểm tra cập nhật: {}", error);
                                let i18n = dlg.get_i18n();
                                dlg.set_status_message(
                                    format!("{}{error}", i18n.update_check_error).into(),
                                );
                            }
                        }
                    }
                });
            });
        }

        #[cfg(target_os = "windows")]
        unsafe {
            let wh = main_win.window().window_handle();
            if let Ok(h) = wh.window_handle()
                && let raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw()
            {
                let hwnd = windows::Win32::Foundation::HWND(w32.hwnd.get() as _);
                let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(hwnd, false);
            }
        }

        let main_weak = main_win.as_weak();
        let dialog_weak = dialog.as_weak();

        let re_enable_main_window = {
            let main_weak = main_weak.clone();
            let dialog_weak = dialog_weak.clone();
            move || {
                if let Some(d) = dialog_weak.upgrade() {
                    let _ = d.hide();
                }
                UPDATE_DIALOG_HOLDER.with(|holder| {
                    *holder.borrow_mut() = None;
                });

                #[cfg(target_os = "windows")]
                if let Some(m) = main_weak.upgrade() {
                    m.set_overlay_visible(false);
                    unsafe {
                        let wh = m.window().window_handle();
                        if let Ok(h) = wh.window_handle()
                            && let raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw()
                        {
                            let hwnd = windows::Win32::Foundation::HWND(w32.hwnd.get() as _);
                            let _ = windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow(
                                hwnd, true,
                            );
                            let _ =
                                windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
                        }
                    }
                }
            }
        };

        let on_close = re_enable_main_window.clone();
        dialog.on_close_dialog(move || {
            on_close();
        });

        let update_info_for_download = update_info.clone();
        let downloaded_file_for_save = downloaded_file.clone();
        let dialog_weak_for_download = dialog.as_weak();

        dialog.on_update(move || {
            let info_opt = update_info_for_download.lock().ok().and_then(|u| u.clone());
            if let Some(info) = info_opt {
                if let Some(dlg) = dialog_weak_for_download.upgrade() {
                    if dlg.get_is_downloading() {
                        return;
                    }
                    let i18n = dlg.get_i18n();
                    dlg.set_is_downloading(true);
                    dlg.set_status_message(i18n.update_download_starting.clone());
                }

                let dlg_weak_progress = dialog_weak_for_download.clone();
                let dlg_weak_done = dialog_weak_for_download.clone();
                let downloaded_file_dest = downloaded_file_for_save.clone();

                tokio::spawn(async move {
                    let file_name = std::path::Path::new(&info.download_url)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("chkey_setup.exe");
                    let destination = std::env::temp_dir().join("CHKey").join(file_name);

                    let dl_res = download_update(&info, &destination, move |progress| {
                        let dlg_weak = dlg_weak_progress.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(d) = dlg_weak.upgrade() {
                                match progress {
                                    DownloadProgress::Downloading {
                                        percent,
                                        downloaded,
                                        ..
                                    } => {
                                        let i18n = d.get_i18n();
                                        if let Some(pct) = percent {
                                            d.set_status_message(
                                                format!(
                                                    "{}{:.0}%",
                                                    i18n.update_downloading_progress, pct
                                                )
                                                .into(),
                                            );
                                        } else {
                                            let mb = downloaded as f64 / (1024.0 * 1024.0);
                                            d.set_status_message(
                                                format!(
                                                    "{}{:.1} MB",
                                                    i18n.update_downloading_progress, mb
                                                )
                                                .into(),
                                            );
                                        }
                                    }
                                    DownloadProgress::Completed { .. } => {}
                                }
                            }
                        });
                    })
                    .await;

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(d) = dlg_weak_done.upgrade() {
                            d.set_is_downloading(false);
                            match dl_res {
                                Ok(saved_path) => {
                                    println!("Tải xuống thành công: {}", saved_path.display());
                                    if let Ok(mut f) = downloaded_file_dest.lock() {
                                        *f = Some(saved_path.clone());
                                    }
                                    let i18n = d.get_i18n();
                                    d.set_download_success(true);
                                    d.set_status_message(
                                        format!(
                                            "{}\n{}",
                                            i18n.update_download_success,
                                            saved_path.display()
                                        )
                                        .into(),
                                    );

                                    let path_for_auto = saved_path.clone();
                                    let dlg_weak_err = dlg_weak_done.clone();
                                    tokio::spawn(async move {
                                        tokio::time::sleep(std::time::Duration::from_millis(1500))
                                            .await;
                                        if let Err(e) = apply_update(&path_for_auto) {
                                            eprintln!("Lỗi kích hoạt cập nhật: {e}");
                                            let _ = slint::invoke_from_event_loop(move || {
                                                if let Some(d) = dlg_weak_err.upgrade() {
                                                    d.set_status_message(format!("{e}").into());
                                                }
                                            });
                                        }
                                    });
                                }
                                Err(e) => {
                                    eprintln!("Tải xuống thất bại: {e}");
                                    let i18n = d.get_i18n();
                                    d.set_status_message(
                                        format!("{}{e}", i18n.update_download_failed).into(),
                                    );
                                }
                            }
                        }
                    });
                });
            }
        });

        let downloaded_file_for_install = downloaded_file.clone();
        let dialog_weak_install = dialog.as_weak();
        dialog.on_install_now(move || {
            let path_opt = downloaded_file_for_install
                .lock()
                .ok()
                .and_then(|f| f.clone());
            if let Some(path) = path_opt {
                println!("Người dùng bấm Cài đặt ngay...");
                if let Err(e) = apply_update(&path) {
                    eprintln!("Lỗi kích hoạt cập nhật: {e}");
                    if let Some(d) = dialog_weak_install.upgrade() {
                        d.set_status_message(format!("{e}").into());
                    }
                }
            }
        });

        let downloaded_file_for_open = downloaded_file.clone();
        dialog.on_open_download_dir(move || {
            let path_opt = downloaded_file_for_open.lock().ok().and_then(|f| f.clone());
            if let Some(path) = path_opt {
                let _ = std::process::Command::new("explorer")
                    .arg(format!("/select,{}", path.display()))
                    .spawn();
            }
        });

        let on_close_req = re_enable_main_window.clone();
        dialog.window().on_close_requested(move || {
            on_close_req();
            slint::CloseRequestResponse::HideWindow
        });

        let _ = dialog.show();
        #[cfg(target_os = "windows")]
        bring_window_to_top_and_foreground(dialog.window());
        UPDATE_DIALOG_HOLDER.with(|holder| {
            *holder.borrow_mut() = Some(dialog);
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    // Đăng ký AppUserModelID để Windows Taskbar nhận diện và hiển thị đúng icon ứng dụng
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(
            windows::core::w!("CHKey.Keyboard.Vietnamese"),
        );
    }

    // 0. Kiểm tra Single-Instance: Nếu CHKey đã đang chạy, báo cho tiến trình cũ mở UI và thoát ngay mà không hiện UAC
    let is_maintenance =
        std::env::args().any(|arg| arg == "--repair-tsf" || arg == "--uninstall-tsf");
    if !is_maintenance && single_instance::check_already_running_and_wake() {
        println!(
            "[SingleInstance] CHKey đã đang chạy. Đã kích hoạt hiển thị giao diện UI và thoát tiến trình mới."
        );
        std::process::exit(0);
    }

    // Luôn yêu cầu quyền Administrator khi khởi chạy ứng dụng
    #[cfg(target_os = "windows")]
    if !tsf::is_elevated() {
        println!(
            "[UAC] Ứng dụng yêu cầu quyền Administrator. Đang kích hoạt UAC prompt để nâng quyền..."
        );
        if let Ok(exe_path) = std::env::current_exe() {
            let args: Vec<String> = std::env::args().skip(1).collect();
            let args_str = args.join(" ");

            let verb: Vec<u16> = "runas\0".encode_utf16().collect();
            let file: Vec<u16> = format!("{}\0", exe_path.display()).encode_utf16().collect();
            let params: Vec<u16> = format!("{}\0", args_str).encode_utf16().collect();

            let res = unsafe {
                windows_sys::Win32::UI::Shell::ShellExecuteW(
                    std::ptr::null_mut(),
                    verb.as_ptr(),
                    file.as_ptr(),
                    if args.is_empty() {
                        std::ptr::null()
                    } else {
                        params.as_ptr()
                    },
                    std::ptr::null(),
                    windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
                )
            };

            if res as usize > 32 {
                std::process::exit(0);
            } else {
                eprintln!(
                    "[UAC] Người dùng từ chối quyền Administrator (Mã lỗi: {res:?}). Thoát chương trình."
                );
                std::process::exit(1);
            }
        }
    }

    println!("[UAC] Đang chạy với quyền Administrator.");

    // Đăng ký quyền Single-Instance cho tiến trình chính (Instance 1)
    let _single_instance_guard = if !is_maintenance {
        match single_instance::acquire_single_instance() {
            Some(guard) => Some(guard),
            None => {
                println!(
                    "[SingleInstance] Phát hiện tiến trình CHKey khác đã chạy. Đang kích hoạt mở UI và thoát."
                );
                single_instance::check_already_running_and_wake();
                std::process::exit(0);
            }
        }
    } else {
        None
    };

    // 0. Xử lý các cờ bảo trì từ dòng lệnh
    if std::env::args().any(|arg| arg == "--repair-tsf") {
        println!("[Setup] Đang thực hiện sửa chữa / đăng ký lại TSF Text Service...");
        match chkey_setup::repair_setup() {
            Ok(()) => println!("[Setup] Đăng ký / sửa chữa TSF thành công."),
            Err(err) => eprintln!("[Setup] Thất bại khi sửa chữa TSF: {err}"),
        }
        return Ok(());
    }

    if std::env::args().any(|arg| arg == "--uninstall-tsf") {
        println!("[Setup] Đang gỡ cài đặt CHKey TSF...");
        match chkey_setup::uninstall_setup() {
            Ok(()) => println!("[Setup] Gỡ cài đặt CHKey TSF thành công."),
            Err(err) => eprintln!("[Setup] Lỗi khi gỡ cài đặt CHKey TSF: {err}"),
        }
        return Ok(());
    }

    // 1. Tự động kiểm tra runtime DLL nhúng, cấu hình config.toml và đăng ký TSF Text Service
    println!("[Setup] Kiểm tra runtime DLL nhúng và trạng thái TSF...");
    match chkey_setup::ensure_setup() {
        Ok(dll_path) => println!(
            "[Setup] Runtime DLL và TSF đã sẵn sàng tại: {}",
            dll_path.display()
        ),
        Err(err) => eprintln!("[Setup] Cảnh báo khởi tạo runtime: {err}"),
    }

    // Khởi tạo các cơ chế dọn dẹp và hủy đăng ký TSF khi tắt máy hoặc thoát app
    let _tsf_guard = shutdown::TsfExitGuard;
    shutdown::init_shutdown_handlers();

    let window = MainWindow::new()?;
    shutdown::hook_window_shutdown_messages(window.window());

    // Bắt đầu lắng nghe yêu cầu hiển thị UI từ các lần mở CHKey.exe tiếp theo
    if let Some(ref guard) = _single_instance_guard {
        single_instance::start_show_window_listener(guard, window.as_weak());
    }

    let startup_mode = std::env::args().any(|arg| arg == "--startup");

    // 1. Tải cấu hình từ %APPDATA%\CHKey\config.toml (tự tạo mặc định nếu lần đầu mở)
    let mut initial_config = chkey_config::load_config().unwrap_or_else(|err| {
        eprintln!("Lỗi đọc cấu hình: {err}, dùng mặc định");
        chkey_config::AppConfig::default()
    });

    // Đồng bộ với chế độ gõ thực tế hiện tại của Windows (EDE hoặc ENG)
    let initial_sys_mode = tsf::layout_switcher::get_windows_input_mode();
    println!("Windows active input mode on startup: {}", initial_sys_mode);
    initial_config.input_method = initial_sys_mode.to_lowercase();

    // Đồng bộ trạng thái khởi động cùng Windows từ hệ thống thực tế
    let startup_enabled_sys = startup::is_startup_enabled();
    println!(
        "Windows startup enabled on startup: {}",
        startup_enabled_sys
    );
    initial_config.run_on_startup = startup_enabled_sys;

    // Helper chuyển đổi từ chkey_config::AppConfig sang chkey_ui::AppConfig (Slint struct)
    let to_slint_config = |cfg: &chkey_config::AppConfig| -> chkey_ui::AppConfig {
        chkey_ui::AppConfig {
            encoding: cfg.encoding.clone().into(),
            input_mode: cfg.input_method.to_uppercase().into(),
            auto_update: cfg.auto_update,
            launch_at_startup: cfg.run_on_startup,
            show_dialog_on_startup: cfg.show_dialog_on_startup,
            lang: cfg.language.clone().into(),
        }
    };

    // 2. Đồng bộ giá trị cấu hình ban đầu vào UI
    window.set_app_config(to_slint_config(&initial_config));
    window.set_lang(initial_config.language.clone().into());
    window.set_i18n(chkey_i18n::load_ui_texts(&initial_config.language));

    // Quyết định hiển thị cửa sổ:
    // - Khi khởi động cùng Windows (--startup):
    //     + Chỉ hiển thị hộp thoại nếu CẢ HAI điều kiện: `run_on_startup` và `show_dialog_on_startup` đều được bật.
    //     + Nếu `show_dialog_on_startup` tắt: ứng dụng chạy ngầm dưới khay hệ thống (System Tray).
    // - Khi người dùng mở app thủ công (không có cờ --startup):
    //     + Luôn hiển thị cửa sổ giao diện chính.
    let should_show_window = if startup_mode {
        initial_config.run_on_startup && initial_config.show_dialog_on_startup
    } else {
        true
    };

    if should_show_window {
        window.show()?;
        #[cfg(target_os = "windows")]
        bring_window_to_top_and_foreground(window.window());
    }

    // 3. Tự động kiểm tra cập nhật nếu tính năng "Tự động cập nhật" được BẬT
    if initial_config.auto_update {
        println!(
            "[AutoUpdate] Tính năng tự động cập nhật đang BẬT. Đang kiểm tra phiên bản mới ngầm..."
        );
        let window_weak_auto = window.as_weak();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            match check_update().await {
                Ok(UpdateStatus::UpdateAvailable(info)) => {
                    println!(
                        "[AutoUpdate] Phát hiện bản cập nhật mới: v{}! Hiển thị hộp thoại nhắc người dùng.",
                        info.latest_version
                    );
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(main_win) = window_weak_auto.upgrade() {
                            open_update_dialog(&main_win, Some(info));
                        }
                    });
                }
                Ok(UpdateStatus::UpToDate) => {
                    println!(
                        "[AutoUpdate] CHKey đang ở phiên bản mới nhất. Không làm phiền người dùng."
                    );
                }
                Err(err) => {
                    println!(
                        "[AutoUpdate] Kiểm tra tự động thất bại (không làm phiền người dùng): {err}"
                    );
                }
            }
        });
    } else {
        println!("[AutoUpdate] Tính năng tự động cập nhật đang TẮT. Bỏ qua kiểm tra.");
    }

    let config_state = std::sync::Arc::new(std::sync::Mutex::new(initial_config));

    // Lắng nghe thay đổi chế độ gõ realtime từ Windows (Win + Space hoặc Taskbar)
    let window_weak_mode = window.as_weak();
    let config_for_watch = std::sync::Arc::clone(&config_state);
    tsf::layout_switcher::watch_windows_input_mode(move |new_mode| {
        println!("Windows layout changed realtime: {}", new_mode);
        if let Ok(mut cfg) = config_for_watch.lock()
            && !cfg.input_method.eq_ignore_ascii_case(&new_mode)
        {
            cfg.input_method = new_mode.to_lowercase();
            let _ = chkey_config::save_config(&cfg);
            let slint_cfg = to_slint_config(&cfg);
            let w = window_weak_mode.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(win) = w.upgrade() {
                    win.set_app_config(slint_cfg);
                }
            });
        }
    });

    window.set_current_version(env!("CARGO_PKG_VERSION").into());

    // In trạng thái theme hiện tại
    let initial_dark_mode = theme::is_dark_mode();
    window.set_dark_mode(initial_dark_mode);
    println!(
        "window mode: {}",
        if initial_dark_mode {
            "Dark Mode (Tối)"
        } else {
            "Light Mode (Sáng)"
        }
    );

    // Lắng nghe thay đổi theme realtime từ Windows và cập nhật UI Slint
    let window_weak_theme = window.as_weak();
    theme::watch_theme_changes(move |dark_mode| {
        println!(
            "window mode realtime change: {}",
            if dark_mode {
                "Dark Mode (Tối)"
            } else {
                "Light Mode (Sáng)"
            }
        );
        let w = window_weak_theme.clone();
        slint::invoke_from_event_loop(move || {
            if let Some(win) = w.upgrade() {
                win.set_dark_mode(dark_mode);
            }
        })
        .ok();
    });

    let window_weak_action = window.as_weak();
    let config_for_action = std::sync::Arc::clone(&config_state);
    window.on_action_clicked(move |action| match action.as_str() {
        "close" => {
            if let Some(win) = window_weak_action.upgrade() {
                let _ = win.hide();
            }
        }
        "exit" => {
            shutdown::cleanup_tsf_on_exit();
            let _ = slint::quit_event_loop();
        }
        "guide" => {
            open_link("https://chkey.org/guide");
        }
        "update" => {
            println!("Kiểm tra phiên bản mới...");
            if let Some(main_win) = window_weak_action.upgrade() {
                open_update_dialog(&main_win, None);
            }
        }
        "default" | "restore" => {
            println!("Khôi phục mặc định...");
            if let Ok(default_cfg) = chkey_config::reset_config() {
                if let Ok(mut cfg) = config_for_action.lock() {
                    *cfg = default_cfg.clone();
                }
                if let Some(win) = window_weak_action.upgrade() {
                    win.set_app_config(to_slint_config(&default_cfg));
                    win.set_lang(default_cfg.language.clone().into());
                    win.set_i18n(chkey_i18n::load_ui_texts(&default_cfg.language));
                }
            }
        }
        other => {
            println!("Action clicked: {}", other);
        }
    });

    let config_for_setting = std::sync::Arc::clone(&config_state);
    let window_weak_setting = window.as_weak();
    window.on_setting_changed(move |key, val| {
        println!("Setting changed: {} = {}", key, val);
        if let Ok(mut cfg) = config_for_setting.lock() {
            match key.as_str() {
                "run-on-startup" => {
                    cfg.run_on_startup = val;
                    if let Err(err) = startup::set_startup_enabled(val) {
                        eprintln!("Lỗi cập nhật khởi động cùng Windows: {err}");
                    }
                }
                "show-on-startup" => {
                    // Yêu cầu chức năng khởi động cùng Windows mới có thể hoạt động tính năng này
                    if cfg.run_on_startup {
                        cfg.show_dialog_on_startup = val;
                    } else {
                        println!("Bỏ qua show-on-startup vì chức năng khởi động cùng Windows đang tắt");
                        cfg.show_dialog_on_startup = false;
                    }
                }
                "check-for-updates" => {
                    cfg.auto_update = val;
                    if val {
                        println!("[AutoUpdate] Người dùng vừa bật tự động cập nhật. Đang kiểm tra cập nhật...");
                        let window_weak_toggle = window_weak_setting.clone();
                        tokio::spawn(async move {
                            match check_update().await {
                                Ok(UpdateStatus::UpdateAvailable(info)) => {
                                    println!("[AutoUpdate] Phát hiện bản cập nhật mới: v{}! Hiển thị hộp thoại nhắc người dùng.", info.latest_version);
                                    let _ = slint::invoke_from_event_loop(move || {
                                        if let Some(main_win) = window_weak_toggle.upgrade() {
                                            open_update_dialog(&main_win, Some(info));
                                        }
                                    });
                                }
                                Ok(UpdateStatus::UpToDate) => {
                                    println!("[AutoUpdate] CHKey đang ở phiên bản mới nhất.");
                                }
                                Err(err) => {
                                    println!("[AutoUpdate] Lỗi kiểm tra cập nhật: {err}");
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
            if let Err(err) = chkey_config::save_config(&cfg) {
                eprintln!("Lỗi lưu config: {err}");
            }
            if let Some(win) = window_weak_setting.upgrade() {
                win.set_app_config(to_slint_config(&cfg));
            }
        }
    });

    let config_for_input = std::sync::Arc::clone(&config_state);
    let window_weak_input = window.as_weak();
    window.on_input_settings_changed(move |key, val| {
        println!("Input settings changed: {} = {}", key, val);
        if let Ok(mut cfg) = config_for_input.lock() {
            match key.as_str() {
                "input_mode" => {
                    let mode_str = val.to_string();
                    cfg.input_method = mode_str.to_lowercase();
                    if let Err(err) = tsf::layout_switcher::set_windows_input_mode(&mode_str) {
                        eprintln!("Lỗi đổi chế độ gõ Windows sang {mode_str}: {err}");
                    }
                }
                "encoding" => {
                    cfg.encoding = val.to_lowercase().to_string();
                }
                _ => {}
            }
            if let Err(err) = chkey_config::save_config(&cfg) {
                eprintln!("Lỗi lưu config: {err}");
            }
            if let Some(win) = window_weak_input.upgrade() {
                win.set_app_config(to_slint_config(&cfg));
            }
        }
    });

    let window_weak_lang = window.as_weak();
    let config_for_lang = std::sync::Arc::clone(&config_state);
    window.on_language_changed(move |lang| {
        println!("language changed: {}", lang);
        if let Some(win) = window_weak_lang.upgrade() {
            win.set_lang(lang.clone());
            win.set_i18n(chkey_i18n::load_ui_texts(lang.as_str()));
        }
        if let Ok(mut cfg) = config_for_lang.lock() {
            cfg.language = lang.to_string();
            if let Err(err) = chkey_config::save_config(&cfg) {
                eprintln!("Lỗi lưu config: {err}");
            }
        }
    });

    // Khi người dùng nhấn X trên cửa sổ: ẩn cửa sổ thay vì thoát app
    let window_weak_close = window.as_weak();
    window.window().on_close_requested(move || {
        if let Some(win) = window_weak_close.upgrade() {
            let _ = win.hide();
        }
        slint::CloseRequestResponse::HideWindow
    });

    // Khởi tạo Windows System Tray
    // _tray phải được giữ sống đến cuối hàm main để tray không bị drop
    #[cfg(target_os = "windows")]
    let _tray = system_tray::init_tray(window.as_weak()).expect("Không thể khởi tạo System Tray");

    // Vòng lặp sự kiện KHÔNG tự thoát khi cửa sổ ẩn → app vẫn sống trong system tray.
    // Chỉ thoát khi gọi slint::quit_event_loop() (từ menu Thoát trong tray).
    let loop_res = slint::run_event_loop_until_quit();

    // Hủy đăng ký TSF khi thoát ứng dụng
    cleanup_tsf_on_exit();

    loop_res?;
    Ok(())
}
