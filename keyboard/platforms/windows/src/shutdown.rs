//! Quản lý sự kiện tắt máy tính (Windows Shutdown/Restart/Logoff) và đóng ứng dụng.
//!
//! Đảm bảo TSF Text Service luôn được hủy đăng ký và chế độ gõ trả về ENG khi:
//! 1. Windows tắt máy, khởi động lại, hoặc đăng xuất người dùng (WM_QUERYENDSESSION, WM_ENDSESSION, CTRL_SHUTDOWN_EVENT, CTRL_LOGOFF_EVENT).
//! 2. Người dùng đóng chương trình (menu Thoát khay hệ thống, nút Thoát giao diện, Ctrl+C, v.v.).

use std::sync::atomic::{AtomicBool, Ordering};
use raw_window_handle::HasWindowHandle;

use windows::core::w;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::SetProcessShutdownParameters;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassExW,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HCURSOR, HICON, HMENU,
    WINDOW_EX_STYLE, WNDCLASSEXW, WS_OVERLAPPED,
    WM_DESTROY, WM_ENDSESSION, WM_QUERYENDSESSION,
};
use windows::Win32::Graphics::Gdi::HBRUSH;

use chkey_tsf as tsf;

static TSF_CLEANED_UP: AtomicBool = AtomicBool::new(false);

/// Dọn dẹp: Đổi chế độ gõ về ENG và hủy đăng ký TSF khi thoát ứng dụng hoặc tắt Windows.
/// Hàm này an toàn để gọi nhiều lần (idempotent), chỉ thực thi 1 lần duy nhất nhờ AtomicBool.
pub fn cleanup_tsf_on_exit() {
    if TSF_CLEANED_UP.swap(true, Ordering::SeqCst) {
        return;
    }
    println!("[TSF] Bắt đầu dọn dẹp khi thoát: Chuyển về ENG và hủy đăng ký TSF...");

    // 1. Chuyển chế độ gõ Windows về US English
    if let Err(err) = tsf::layout_switcher::set_windows_input_mode("ENG") {
        eprintln!("[TSF] Cảnh báo đổi sang ENG khi thoát: {err}");
    }

    // 2. Hủy đăng ký TSF Text Service khỏi Windows
    if let Err(err) = tsf::uninstall_tsf() {
        eprintln!("[TSF] Lỗi khi hủy đăng ký TSF khi thoát: {err}");
    } else {
        println!("[TSF] Đã hủy đăng ký TSF thành công khi đóng chương trình / tắt máy.");
    }
}

/// Handler xử lý tín hiệu console & hệ thống (Ctrl+C, Logoff, Shutdown)
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        // CTRL_C_EVENT = 0, CTRL_BREAK_EVENT = 1, CTRL_CLOSE_EVENT = 2,
        // CTRL_LOGOFF_EVENT = 5, CTRL_SHUTDOWN_EVENT = 6
        0 | 1 | 2 | 5 | 6 => {
            println!("[Shutdown] Nhận sự kiện hệ thống (ctrl_type = {ctrl_type}). Đang hủy đăng ký TSF...");
            cleanup_tsf_on_exit();
            BOOL(1)
        }
        _ => BOOL(0),
    }
}

/// Window Procedure cho cửa sổ ẩn chuyên lắng nghe thông điệp tắt Windows
unsafe extern "system" fn shutdown_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_QUERYENDSESSION => {
            println!("[Shutdown] Nhận WM_QUERYENDSESSION: Windows đang chuẩn bị tắt/khởi động lại...");
            cleanup_tsf_on_exit();
            // Trả về 1 (TRUE) để xác nhận sẵn sàng cho Windows tắt máy
            LRESULT(1)
        }
        WM_ENDSESSION => {
            if wparam.0 != 0 {
                println!("[Shutdown] Nhận WM_ENDSESSION: Windows xác nhận tắt máy...");
                cleanup_tsf_on_exit();
            }
            LRESULT(0)
        }
        WM_DESTROY => LRESULT(0),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Window Subclass Procedure gắn vào MainWindow của Slint để đón thông điệp tắt máy
unsafe extern "system" fn main_window_subclass_proc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _refdata: usize,
) -> LRESULT {
    match umsg {
        WM_QUERYENDSESSION => {
            println!("[Shutdown] MainWindow nhận WM_QUERYENDSESSION: Hủy đăng ký TSF...");
            cleanup_tsf_on_exit();
            LRESULT(1)
        }
        WM_ENDSESSION => {
            if wparam.0 != 0 {
                println!("[Shutdown] MainWindow nhận WM_ENDSESSION: Hủy đăng ký TSF...");
                cleanup_tsf_on_exit();
            }
            LRESULT(0)
        }
        _ => unsafe { DefSubclassProc(hwnd, umsg, wparam, lparam) },
    }
}

/// Gắn Subclass vào MainWindow của Slint để nhận trực tiếp các thông điệp tắt máy từ Windows
pub fn hook_window_shutdown_messages(window: &slint::Window) {
    let wh = window.window_handle();
    if let Ok(h) = wh.window_handle()
        && let raw_window_handle::RawWindowHandle::Win32(w32) = h.as_raw()
    {
        let hwnd = HWND(w32.hwnd.get() as _);
        unsafe {
            let _ = SetWindowSubclass(hwnd, Some(main_window_subclass_proc), 1001, 0);
        }
    }
}

/// Handler khi tiến trình thoát qua libc/C-runtime atexit
extern "C" fn atexit_handler() {
    cleanup_tsf_on_exit();
}

/// Tạo cửa sổ cấp cao ẩn (top-level hidden window) chuyên lắng nghe thông điệp WM_QUERYENDSESSION
fn create_shutdown_listener_window() -> Option<HWND> {
    unsafe {
        let class_name = w!("CHKeyShutdownListenerClass");
        let h_instance = GetModuleHandleW(None).unwrap_or_default();

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(shutdown_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance.into(),
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: HICON::default(),
        };

        let _ = RegisterClassExW(&wnd_class);

        let hwnd_res = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("CHKeyShutdownListener"),
            WS_OVERLAPPED,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0,
            0,
            HWND::default(),
            HMENU::default(),
            h_instance,
            None,
        );

        match hwnd_res {
            Ok(hwnd) if !hwnd.is_invalid() => Some(hwnd),
            _ => None,
        }
    }
}

/// Khởi tạo toàn bộ cơ chế bảo vệ dọn dẹp khi tắt máy hoặc đóng chương trình
pub fn init_shutdown_handlers() {
    // 1. Đăng ký atexit để bắt mọi trường hợp exit() thông thường
    unsafe {
        unsafe extern "C" {
            fn atexit(f: extern "C" fn()) -> i32;
        }
        let _ = atexit(atexit_handler);
    }

    // 2. Thiết lập độ ưu tiên tắt máy cao hơn ứng dụng thông thường (0x380)
    // Giúp CHKey nhận thông báo shutdown sớm và hoàn tất hủy đăng ký TSF trước khi dịch vụ hệ thống dừng
    unsafe {
        let _ = SetProcessShutdownParameters(0x380, 0);
    }

    // 3. Đăng ký Windows Console Ctrl Handler (bắt CTRL_SHUTDOWN_EVENT, CTRL_LOGOFF_EVENT, Ctrl+C)
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(console_ctrl_handler), true);
    }

    // 4. Tạo cửa sổ top-level ẩn để nhận thông điệp WM_QUERYENDSESSION và WM_ENDSESSION từ Windows
    let _ = create_shutdown_listener_window();
}

/// RAII Guard đảm bảo TSF luôn được hủy đăng ký khi biến guard bị Drop
pub struct TsfExitGuard;

impl Drop for TsfExitGuard {
    fn drop(&mut self) {
        cleanup_tsf_on_exit();
    }
}
