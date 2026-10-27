//! Xử lý sự kiện click trên System Tray.
//!
//! - Nhấp chuột trái: Mở cửa sổ chính.

use tray_icon::{MouseButton, TrayIconEvent};
use slint::{ComponentHandle, Weak};
use crate::MainWindow;

/// Đăng ký handler xử lý sự kiện click trên System Tray.
pub fn register_tray_event_handler(window_weak: Weak<MainWindow>) {
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            ..
        } = event
        {
            handle_open_window(&window_weak);
        }
    }));
}

/// Mở cửa sổ chính trên main thread.
fn handle_open_window(window_weak: &Weak<MainWindow>) {
    let window_weak = window_weak.clone();

    slint::invoke_from_event_loop(move || {
        if let Some(window) = window_weak.upgrade() {
            let _ = window.show();
            #[cfg(target_os = "windows")]
            crate::bring_window_to_top_and_foreground(window.window());
        }
    })
    .ok();
}
