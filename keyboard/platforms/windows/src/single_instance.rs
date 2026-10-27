//! Quản lý Single-Instance cho CHKey trên Windows.
//!
//! Khi người dùng mở lại file CHKey.exe trong khi chương trình đã đang chạy (ở khay hệ thống hoặc chạy nền),
//! tiến trình mới sẽ phát hiện tiến trình cũ, kích hoạt hiển thị giao diện UI của tiến trình cũ lên màn hình,
//! và tự động thoát ngay lập tức mà không khởi chạy bản sao thứ hai.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use slint::ComponentHandle;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0,
};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    InitializeSecurityDescriptor, SetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR,
    SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, OpenMutexW, SetEvent, WaitForSingleObject, INFINITE,
    SYNCHRONIZATION_ACCESS_RIGHTS,
};

const MUTEX_NAME: PCWSTR = w!("Local\\CHKey_SingleInstance_Mutex");
const EVENT_NAME: PCWSTR = w!("Local\\CHKey_ShowWindow_Event");

// EVENT_MODIFY_STATE = 0x0002
const EVENT_MODIFY_STATE: u32 = 0x0002;
// SYNCHRONIZE = 0x00100000
const SYNCHRONIZE_ACCESS: u32 = 0x00100000;

unsafe extern "system" {
    fn LocalFree(hmem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

/// Helper quản lý bộ nhớ SECURITY_ATTRIBUTES với Low-Integrity SACL.
/// Cho phép cả tiến trình thường (Medium Integrity) lẫn Admin (High Integrity)
/// có thể mở và gửi tín hiệu SetEvent đến Event liên tiến trình.
struct LowIntegritySecurity {
    sa: SECURITY_ATTRIBUTES,
    sd: PSECURITY_DESCRIPTOR,
    #[allow(dead_code)]
    fallback_sd: Option<Box<SECURITY_DESCRIPTOR>>,
}

impl LowIntegritySecurity {
    fn create() -> Self {
        unsafe {
            let mut sd = PSECURITY_DESCRIPTOR::default();
            // SDDL:
            // D:(A;;GA;;;WD)  -> DACL: Grant Generic All cho Everyone
            // S:(ML;;NW;;;LW) -> SACL: Mandatory Label Low, No-Write-Up policy
            // Đảm bảo tiến trình Medium/Low Integrity có quyền modify/signal đối tượng của tiến trình High Integrity
            let res = ConvertStringSecurityDescriptorToSecurityDescriptorW(
                w!("D:(A;;GA;;;WD)S:(ML;;NW;;;LW)"),
                SDDL_REVISION_1,
                &mut sd,
                None,
            );

            if res.is_ok() && !sd.0.is_null() {
                let sa = SECURITY_ATTRIBUTES {
                    nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                    lpSecurityDescriptor: sd.0 as *mut _,
                    bInheritHandle: false.into(),
                };
                Self {
                    sa,
                    sd,
                    fallback_sd: None,
                }
            } else {
                // Fallback: Tạo NULL DACL cơ bản nếu ConvertStringSecurityDescriptor thất bại
                let mut fallback = Box::new(SECURITY_DESCRIPTOR::default());
                let _ = InitializeSecurityDescriptor(
                    PSECURITY_DESCRIPTOR(fallback.as_mut() as *mut _ as _),
                    1,
                );
                let _ = SetSecurityDescriptorDacl(
                    PSECURITY_DESCRIPTOR(fallback.as_mut() as *mut _ as _),
                    true,
                    None,
                    false,
                );
                let sa = SECURITY_ATTRIBUTES {
                    nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                    lpSecurityDescriptor: fallback.as_mut() as *mut _ as _,
                    bInheritHandle: false.into(),
                };
                Self {
                    sa,
                    sd: PSECURITY_DESCRIPTOR::default(),
                    fallback_sd: Some(fallback),
                }
            }
        }
    }

    fn as_mut_ptr(&mut self) -> *mut SECURITY_ATTRIBUTES {
        &mut self.sa as *mut _
    }
}

impl Drop for LowIntegritySecurity {
    fn drop(&mut self) {
        unsafe {
            if !self.sd.0.is_null() {
                LocalFree(self.sd.0);
            }
        }
    }
}

/// RAII Guard giữ Mutex và Event của tiến trình chính (Instance 1)
pub struct SingleInstanceGuard {
    mutex: HANDLE,
    show_event: HANDLE,
    is_running: Arc<AtomicBool>,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);
        unsafe {
            if !self.show_event.is_invalid() && !self.show_event.0.is_null() {
                let _ = SetEvent(self.show_event);
                let _ = CloseHandle(self.show_event);
            }
            if !self.mutex.is_invalid() && !self.mutex.0.is_null() {
                let _ = CloseHandle(self.mutex);
            }
        }
    }
}

/// Kiểm tra xem CHKey đã đang chạy hay chưa.
/// - Nếu đã chạy: Kích hoạt hiển thị UI của tiến trình cũ và trả về `true` (để tiến trình gọi thoát ngay).
/// - Nếu chưa chạy: Trả về `false`.
pub fn check_already_running_and_wake() -> bool {
    unsafe {
        // 1. Thử mở Event của tiến trình cũ xem có tồn tại không
        if let Ok(event) = OpenEventW(
            SYNCHRONIZATION_ACCESS_RIGHTS(EVENT_MODIFY_STATE),
            false,
            EVENT_NAME,
        ) {
            if !event.is_invalid() && !event.0.is_null() {
                println!("[SingleInstance] Phát hiện CHKey đang chạy qua Event. Đang kích hoạt hiển thị giao diện UI...");
                let _ = SetEvent(event);
                let _ = CloseHandle(event);

                // Cho phép tiến trình cũ có quyền mang cửa sổ lên Foreground
                windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(
                    windows_sys::Win32::UI::WindowsAndMessaging::ASFW_ANY,
                );

                return true;
            }
        }

        // 2. Kiểm tra Mutex xem có tiến trình nào đang nắm giữ không
        let mutex_detected = match OpenMutexW(
            SYNCHRONIZATION_ACCESS_RIGHTS(SYNCHRONIZE_ACCESS),
            false,
            MUTEX_NAME,
        ) {
            Ok(h) => {
                let _ = CloseHandle(h);
                true
            }
            Err(_) => {
                let err = GetLastError();
                err == ERROR_ACCESS_DENIED || err == ERROR_ALREADY_EXISTS
            }
        };

        if mutex_detected {
            println!("[SingleInstance] Phát hiện Mutex của CHKey. Đang chờ kết nối Event để mở UI...");
            // Đợi tối đa 300ms (15 lần x 20ms) để Instance 1 tạo xong Event
            for _ in 0..15 {
                std::thread::sleep(std::time::Duration::from_millis(20));
                if let Ok(event) = OpenEventW(
                    SYNCHRONIZATION_ACCESS_RIGHTS(EVENT_MODIFY_STATE),
                    false,
                    EVENT_NAME,
                ) {
                    if !event.is_invalid() && !event.0.is_null() {
                        let _ = SetEvent(event);
                        let _ = CloseHandle(event);
                        windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(
                            windows_sys::Win32::UI::WindowsAndMessaging::ASFW_ANY,
                        );
                        return true;
                    }
                }
            }

            windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(
                windows_sys::Win32::UI::WindowsAndMessaging::ASFW_ANY,
            );
            return true;
        }

        false
    }
}

/// Đăng ký quyền Single-Instance cho tiến trình chính (sau khi đã có quyền Administrator).
/// Trả về `Some(SingleInstanceGuard)` nếu là tiến trình duy nhất.
/// Trả về `None` nếu đã có tiến trình khác giữ quyền.
pub fn acquire_single_instance() -> Option<SingleInstanceGuard> {
    unsafe {
        let mut sec = LowIntegritySecurity::create();
        let sa_ptr = sec.as_mut_ptr();

        let mutex = match CreateMutexW(Some(&*sa_ptr), true, MUTEX_NAME) {
            Ok(h) => h,
            Err(_) => return None,
        };

        if GetLastError() == ERROR_ALREADY_EXISTS {
            println!("[SingleInstance] Mutex đã tồn tại. Đang gửi tín hiệu mở UI cho tiến trình cũ...");
            if let Ok(event) = OpenEventW(
                SYNCHRONIZATION_ACCESS_RIGHTS(EVENT_MODIFY_STATE),
                false,
                EVENT_NAME,
            ) {
                if !event.is_invalid() && !event.0.is_null() {
                    let _ = SetEvent(event);
                    let _ = CloseHandle(event);
                    windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(
                        windows_sys::Win32::UI::WindowsAndMessaging::ASFW_ANY,
                    );
                }
            }
            let _ = CloseHandle(mutex);
            return None;
        }

        let show_event = match CreateEventW(Some(&*sa_ptr), false, false, EVENT_NAME) {
            Ok(h) => h,
            Err(_) => {
                let _ = CloseHandle(mutex);
                return None;
            }
        };

        Some(SingleInstanceGuard {
            mutex,
            show_event,
            is_running: Arc::new(AtomicBool::new(true)),
        })
    }
}

/// Khởi chạy luồng nền lắng nghe yêu cầu mở cửa sổ từ các tiến trình CHKey.exe khởi động sau
pub fn start_show_window_listener(
    guard: &SingleInstanceGuard,
    window: slint::Weak<crate::MainWindow>,
) {
    let show_event_raw = guard.show_event.0 as isize;
    let is_running = Arc::clone(&guard.is_running);

    std::thread::spawn(move || {
        let show_event = HANDLE(show_event_raw as *mut _);

        while is_running.load(Ordering::SeqCst) {
            let wait_res = unsafe { WaitForSingleObject(show_event, INFINITE) };

            if wait_res == WAIT_OBJECT_0 && is_running.load(Ordering::SeqCst) {
                println!("[SingleInstance] Nhận được tín hiệu mở lại giao diện UI từ người dùng.");
                let window = window.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(win) = window.upgrade() {
                        let _ = win.show();
                        crate::bring_window_to_top_and_foreground(win.window());
                    }
                });
            } else {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance_full_lifecycle() {
        // 1. Thử chiếm quyền SingleInstance
        let guard = acquire_single_instance();
        assert!(guard.is_some(), "Tiến trình đầu tiên phải chiếm được quyền SingleInstance");
        let guard = guard.unwrap();

        // 2. Kiểm tra nhận tín hiệu Event trong khi guard đang giữ quyền
        let event_received = Arc::new(AtomicBool::new(false));
        let event_received_clone = Arc::clone(&event_received);

        let show_event_raw = guard.show_event.0 as isize;
        let is_running = Arc::clone(&guard.is_running);

        let handle = std::thread::spawn(move || {
            let show_event = HANDLE(show_event_raw as *mut _);
            let wait_res = unsafe { WaitForSingleObject(show_event, 2000) };
            if wait_res == WAIT_OBJECT_0 && is_running.load(Ordering::SeqCst) {
                event_received_clone.store(true, Ordering::SeqCst);
            }
        });

        // 3. Trong khi guard còn sống, check_already_running_and_wake phải phát hiện được và gửi tín hiệu
        let detected = check_already_running_and_wake();
        assert!(detected, "Tiến trình thứ hai phải phát hiện tiến trình thứ nhất đang chạy");

        let _ = handle.join();
        assert!(event_received.load(Ordering::SeqCst), "Luồng lắng nghe phải nhận được tín hiệu mở UI");

        // 4. Tiến trình thứ hai thử acquire_single_instance phải trả về None
        let duplicate = acquire_single_instance();
        assert!(duplicate.is_none(), "Tiến trình thứ hai không được phép chiếm quyền SingleInstance");

        // 5. Giải phóng guard
        drop(guard);

        // 6. Sau khi giải phóng, một tiến trình mới phải chiếm lại được quyền
        let guard_after = acquire_single_instance();
        assert!(guard_after.is_some(), "Sau khi thoát, tiến trình mới phải chiếm lại được quyền");
    }
}

