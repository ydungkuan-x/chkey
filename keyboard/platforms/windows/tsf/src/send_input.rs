use std::mem::size_of;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_BACK,
};

pub const INJECTED_MAGIC: usize = 0x43484B45;

#[derive(Default)]
pub struct InputSender;

impl InputSender {

    /// Sends backspaces and unicode characters to the active application window.
    pub fn send_key_events(backspace_count: usize, new_chars: &str) {
        let mut inputs: Vec<INPUT> = Vec::new();

        // 1. Send backspace key events
        for _ in 0..backspace_count {
            // Key down VK_BACK
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_BACK,
                        wScan: 0,
                        dwFlags: Default::default(),
                        time: 0,
                        dwExtraInfo: INJECTED_MAGIC,
                    },
                },
            });
            // Key up VK_BACK
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_BACK,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: INJECTED_MAGIC,
                    },
                },
            });
        }

        // 2. Send Unicode character key events
        for utf16_unit in new_chars.encode_utf16() {
            // Key down Unicode
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: Default::default(),
                        wScan: utf16_unit,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: INJECTED_MAGIC,
                    },
                },
            });
            // Key up Unicode
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: Default::default(),
                        wScan: utf16_unit,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: INJECTED_MAGIC,
                    },
                },
            });
        }

        if !inputs.is_empty() {
            unsafe {
                let _ = SendInput(&inputs, size_of::<INPUT>() as i32);
            }
        }
    }
}
