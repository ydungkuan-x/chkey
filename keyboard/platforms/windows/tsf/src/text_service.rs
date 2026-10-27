//! TSF Text Service COM Implementation (`ITfTextInputProcessor`, `ITfKeyEventSink`).

use std::cell::RefCell;

use chkey_engine::keycode::KeyCode;
use chkey_engine::language::ede::EdeProfile;
use chkey_engine::processor::Processor as EngineProcessor;

use windows::core::{Interface, Result, implement};
use windows::Win32::Foundation::{BOOL, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::TextServices::{
    ITfKeyEventSink, ITfKeyEventSink_Impl, ITfKeystrokeMgr, ITfTextInputProcessor,
    ITfTextInputProcessor_Impl, ITfThreadMgr,
};
use windows::Win32::UI::WindowsAndMessaging::GetMessageExtraInfo;

use crate::send_input::{InputSender, INJECTED_MAGIC};
use super::composition::CompositionManager;

#[inline]
fn log_tsf(msg: &str) {
    crate::utils::log_tsf_msg("chkey_tsf.log", msg);
}

/// Kiểm tra nhanh các phím bổ trợ (Ctrl, Alt, Win) bằng GetKeyState (đọc từ thread message queue, cực nhanh)
#[inline(always)]
fn is_shortcut_modifier_active() -> bool {
    unsafe {
        (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000 != 0)
            || (GetKeyState(VK_MENU.0 as i32) as u16 & 0x8000 != 0)
            || (GetKeyState(VK_LWIN.0 as i32) as u16 & 0x8000 != 0)
            || (GetKeyState(VK_RWIN.0 as i32) as u16 & 0x8000 != 0)
    }
}

/// TSF Text Service COM implementation.
#[implement(ITfTextInputProcessor, ITfKeyEventSink)]
pub struct TsfTextService {
    client_id: RefCell<u32>,
    thread_mgr: RefCell<Option<ITfThreadMgr>>,
    engine: RefCell<EngineProcessor<EdeProfile>>,
    composition: RefCell<CompositionManager>,
}

impl Default for TsfTextService {
    fn default() -> Self {
        Self::new()
    }
}

impl TsfTextService {
    /// Creates a new `TsfTextService` instance initialized with the Êđê language profile.
    pub fn new() -> Self {
        Self {
            client_id: RefCell::new(0),
            thread_mgr: RefCell::new(None),
            engine: RefCell::new(EngineProcessor::new(EdeProfile)),
            composition: RefCell::new(CompositionManager::new()),
        }
    }
}

impl ITfTextInputProcessor_Impl for TsfTextService_Impl {
    fn Activate(&self, thread_mgr: Option<&ITfThreadMgr>, client_id: u32) -> Result<()> {
        log_tsf(&format!("=== Activate START === client_id={}", client_id));

        let Some(tm) = thread_mgr else {
            log_tsf("Activate: thread_mgr is None, returning Ok");
            return Ok(());
        };

        *self.client_id.borrow_mut() = client_id;
        *self.thread_mgr.borrow_mut() = Some(tm.clone());

        // Register key event sink with TSF Keystroke Manager
        if let Ok(keystroke_mgr) = tm.cast::<ITfKeystrokeMgr>()
            && let Ok(sink) = unsafe { self.cast::<ITfKeyEventSink>() }
        {
            unsafe {
                let _ = keystroke_mgr.AdviseKeyEventSink(client_id, &sink, true);
            }
        }

        log_tsf("=== Activate END ===");
        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        log_tsf("=== Deactivate START ===");
        let client_id = *self.client_id.borrow();

        if let Some(tm) = self.thread_mgr.borrow().as_ref()
            && let Ok(keystroke_mgr) = tm.cast::<ITfKeystrokeMgr>()
        {
            unsafe {
                let _ = keystroke_mgr.UnadviseKeyEventSink(client_id);
            }
        }

        *self.client_id.borrow_mut() = 0;
        *self.thread_mgr.borrow_mut() = None;
        self.engine.borrow_mut().clear();
        self.composition.borrow_mut().clear();

        log_tsf("=== Deactivate END ===");
        Ok(())
    }
}

impl ITfKeyEventSink_Impl for TsfTextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _context: Option<&windows::Win32::UI::TextServices::ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let extra_info = unsafe { GetMessageExtraInfo() }.0 as usize;
        if extra_info == INJECTED_MAGIC {
            return Ok(BOOL::from(false));
        }

        let vk = wparam.0 as u32;
        // Fast short-circuit: Chỉ xử lý chữ cái (A-Z), số (0-9), Backspace, Space, Enter, Tab
        let should_handle = matches!(vk, 0x41..=0x5A | 0x30..=0x39 | 0x08 | 0x20 | 0x0D | 0x09);
        if !should_handle {
            return Ok(BOOL::from(false));
        }

        // Nếu người dùng đang giữ Ctrl / Alt / Win (phím tắt) -> không can thiệp
        if is_shortcut_modifier_active() {
            return Ok(BOOL::from(false));
        }

        Ok(BOOL::from(true))
    }

    fn OnTestKeyUp(
        &self,
        _context: Option<&windows::Win32::UI::TextServices::ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }

    fn OnKeyDown(
        &self,
        _context: Option<&windows::Win32::UI::TextServices::ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let extra_info = unsafe { GetMessageExtraInfo() }.0 as usize;
            if extra_info == INJECTED_MAGIC {
                return BOOL::from(false);
            }

            if is_shortcut_modifier_active() {
                return BOOL::from(false);
            }

            let shift = unsafe { GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0 };
            let caps_lock = unsafe { GetKeyState(VK_CAPITAL.0 as i32) as u16 & 0x0001 != 0 };

            let keycode = KeyCode::from_win32_vk(wparam.0 as u32);
            let mut engine = self.engine.borrow_mut();
            let action = engine.process_key(keycode, shift, caps_lock);
            let new_chars = action.output_string();

            if action.backspace_count > 0 {
                InputSender::send_key_events(action.backspace_count as usize, &new_chars);
                BOOL::from(true) // Swallowed physical key
            } else if !new_chars.is_empty() {
                // Normal typing without backspaces. Cho Windows xử lý tiếp phím gốc.
                BOOL::from(false)
            } else {
                BOOL::from(false)
            }
        }));

        match res {
            Ok(b) => Ok(b),
            Err(_) => {
                log_tsf("PANIC in OnKeyDown!");
                Ok(BOOL::from(false))
            }
        }
    }

    fn OnKeyUp(
        &self,
        _context: Option<&windows::Win32::UI::TextServices::ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }

    fn OnPreservedKey(
        &self,
        _context: Option<&windows::Win32::UI::TextServices::ITfContext>,
        _guid: *const windows::core::GUID,
    ) -> Result<BOOL> {
        Ok(BOOL::from(false))
    }
}
