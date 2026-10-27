//! Context menu cho System Tray (right-click).
//!
//! Build menu và trả về IDs để [`crate::platform::windows::system_tray::events`]
//! có thể so sánh khi user click vào menu item.

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};

use crate::MainWindow;
use slint::{ComponentHandle, Weak};

/// IDs của các menu item để dispatch trong event handler.
pub struct MenuIds {
    pub open: tray_icon::menu::MenuId,
    pub repair: tray_icon::menu::MenuId,
    pub quit: tray_icon::menu::MenuId,
}

/// Tạo context menu và trả về `(Menu, MenuIds)`.
pub fn build_menu() -> Result<(Menu, MenuIds), Box<dyn std::error::Error>> {
    let item_open = MenuItem::new("Mở CHKey", true, None);
    let item_repair = MenuItem::new("Kiểm tra / Sửa lỗi TSF", true, None);
    let item_quit = MenuItem::new("Thoát", true, None);

    let ids = MenuIds {
        open: item_open.id().clone(),
        repair: item_repair.id().clone(),
        quit: item_quit.id().clone(),
    };

    let menu = Menu::new();

    menu.append(&item_open)?;
    menu.append(&item_repair)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&item_quit)?;

    Ok((menu, ids))
}

/// Đăng ký handler cho MenuEvent.
/// Mở cửa sổ, sửa lỗi TSF, hoặc thoát app tuỳ item được click.
pub fn register_menu_handler(window_weak: Weak<MainWindow>, ids: MenuIds) {
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let id = event.id;
        let w = window_weak.clone();

        if id == ids.open {
            slint::invoke_from_event_loop(move || {
                if let Some(win) = w.upgrade() {
                    let _ = win.show();
                    #[cfg(target_os = "windows")]
                    crate::bring_window_to_top_and_foreground(win.window());
                }
            })
            .ok();
        } else if id == ids.repair {
            match chkey_tsf::check_tsf_status() {
                chkey_tsf::TsfStatus::Valid(path) => {
                    println!("[TSF] TSF đang hoạt động bình thường tại: {}", path.display());
                }
                _ => {
                    println!("[TSF] Đang thực hiện sửa chữa / đăng ký lại TSF...");
                    if let Err(e) = chkey_tsf::repair_tsf() {
                        eprintln!("[TSF] Đăng ký trực tiếp thất bại: {e}");
                        eprintln!("[TSF] Đang kích hoạt quyền Administrator để sửa chữa TSF toàn hệ thống...");
                        let _ = chkey_tsf::request_elevated_repair();
                    } else {
                        println!("[TSF] Sửa chữa và đăng ký TSF thành công!");
                    }
                }
            }
        } else if id == ids.quit {
            crate::shutdown::cleanup_tsf_on_exit();
            slint::invoke_from_event_loop(move || {
                if let Some(win) = w.upgrade() {
                    win.hide().ok();
                }
                slint::quit_event_loop().ok();
            })
            .ok();
        }
    }));
}
