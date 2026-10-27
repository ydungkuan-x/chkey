//! Quản lý vòng đời TrayIcon.
//!
//! Tạo System Tray icon và lưu trữ trong thread_local.

use std::cell::RefCell;
use tray_icon::{TrayIcon, TrayIconBuilder};

thread_local! {
    /// Lưu TrayIcon trên main thread vì TrayIcon không implement Send.
    pub static TRAY_ICON: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
}

/// Tooltip mặc định của CHKey.
const TOOLTIP: &str = "CHKey";

/// Tạo TrayIcon với icon, tooltip và context menu.
///
/// `icon` được tạo từ `super::icon::init_icon`.
/// `menu` được tạo từ `super::menu::build_menu`.
pub fn build_and_store(
    icon: tray_icon::Icon,
    menu: tray_icon::menu::Menu,
) -> Result<(), Box<dyn std::error::Error>> {
    let tray = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip(TOOLTIP)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()?;

    TRAY_ICON.with(|tray_cell| {
        *tray_cell.borrow_mut() = Some(tray);
    });

    Ok(())
}
