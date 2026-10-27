//! System Tray cho Windows.
//!
//! Module được tổ chức thành các submodule theo trách nhiệm:
//!
//! | Submodule  | Trách nhiệm                                      |
//! |------------|--------------------------------------------------|
//! | [`icon`]   | Load, process PNG → tray_icon::Icon, thread cache|
//! | [`tray`]   | Build TrayIconBuilder, lưu vào thread_local      |
//! | [`menu`]   | Build context menu, đăng ký MenuEvent handler    |
//! | [`events`] | Xử lý click / double-click trên tray icon        |

mod events;
mod icon;
mod menu;
mod tray;

use crate::MainWindow;
use slint::Weak;

// ── AppTray ───────────────────────────────────────────────────────────────────

/// Guard giữ System Tray sống trong suốt vòng đời app.
/// Drop đối tượng này = tray biến mất.
pub struct AppTray;

// ── Public entry point ────────────────────────────────────────────────────────

/// Khởi tạo toàn bộ System Tray.
///
/// Thứ tự:
/// 1. Load + cache icon (on / off)  
/// 2. Build context menu  
/// 3. Tạo và lưu TrayIcon  
/// 4. Đăng ký MenuEvent handler  
/// 5. Đăng ký TrayIconEvent handler  
pub fn init_tray(window_weak: Weak<MainWindow>) -> Result<AppTray, Box<dyn std::error::Error>> {
    // 1. Icons
    let icon = icon::init_icon()?;

    // 2. Context menu
    let (context_menu, menu_ids) = menu::build_menu()?;

    // 3. TrayIcon
    tray::build_and_store(icon, context_menu)?;

    // 4. Menu events (right-click items)
    menu::register_menu_handler(window_weak.clone(), menu_ids);

    // 5. Tray click events
    events::register_tray_event_handler(window_weak);

    Ok(AppTray)
}
