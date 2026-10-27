#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn log(msg: &str) {
    println!("[CHKey Installer] {msg}");
    let log_path = env::temp_dir().join("CHKey").join("installer.log");
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = writeln!(file, "[{}] {msg}", chrono_time_now());
    }
}

fn chrono_time_now() -> String {
    // Đơn giản hóa định dạng thời gian để không cần thêm crate ngoài
    format!("{:?}", std::time::SystemTime::now())
}

fn main() {
    log("Installer bắt đầu thực thi");

    let args: Vec<String> = env::args().collect();
    let mut pid: Option<u32> = None;
    let mut target_path: Option<PathBuf> = None;
    let mut new_path: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pid" if i + 1 < args.len() => {
                pid = args[i + 1].parse().ok();
                i += 2;
            }
            "--target" if i + 1 < args.len() => {
                target_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--new" if i + 1 < args.len() => {
                new_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            _ => i += 1,
        }
    }

    let (Some(pid), Some(target), Some(new_file)) = (pid, target_path, new_path) else {
        log("Thiếu tham số! Cần: --pid <PID> --target <target_path> --new <new_path>");
        eprintln!("Thiếu tham số! Cần: --pid <PID> --target <target_path> --new <new_path>");
        return;
    };

    log(&format!(
        "Tham số: PID={}, Target={}, New={}",
        pid,
        target.display(),
        new_file.display()
    ));

    // 1. Chờ tiến trình chính thoát hẳn để giải phóng file lock
    log(&format!("Đang chờ tiến trình chính PID {} thoát...", pid));
    wait_for_process_exit(pid, Duration::from_secs(15));
    log("Tiến trình chính đã thoát");

    // 2. Chờ thêm tối đa 10 giây để Windows nhả hoàn toàn file lock trên target
    let backup_path = target.with_extension("exe.old");
    if !safe_rename_file(&target, &backup_path, Duration::from_secs(10)) {
        log(&format!(
            "Không thể backup/đổi tên file cũ: {}",
            target.display()
        ));
        return;
    }
    log(&format!(
        "Đã đổi tên file cũ thành backup: {}",
        backup_path.display()
    ));

    // 3. Sao chép file mới vào file đích target
    log(&format!(
        "Đang sao chép file mới vào: {}",
        target.display()
    ));
    if let Err(e) = fs::copy(&new_file, &target) {
        log(&format!("Lỗi sao chép file mới: {e}. Đang khôi phục file cũ..."));
        let _ = fs::rename(&backup_path, &target);
        return;
    }
    log("Sao chép file mới thành công");

    // 4. Dọn dẹp file tạm
    let _ = fs::remove_file(&new_file);
    let _ = fs::remove_file(&backup_path);
    log("Đã dọn dẹp file tạm");

    // 5. Khởi động ứng dụng mới
    log(&format!("Đang khởi động lại ứng dụng: {}", target.display()));
    relaunch_app(&target);

    log("Hoàn tất quy trình cập nhật. Installer thoát.");
}

/// Đợi tiến trình PID kết thúc
fn wait_for_process_exit(pid: u32, timeout: Duration) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
        };

        let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !handle.is_null() {
            let millis = timeout.as_millis() as u32;
            WaitForSingleObject(handle, millis);
            CloseHandle(handle);
            // Nghỉ thêm 300ms để OS nhả file lock
            sleep(Duration::from_millis(300));
            return;
        }
    }

    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        sleep(Duration::from_millis(300));
    }
}

/// Thử đổi tên file an toàn với retry loop phòng trường hợp Windows nhả file chậm
fn safe_rename_file(from: &Path, to: &Path, timeout: Duration) -> bool {
    if !from.exists() {
        return true;
    }

    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if fs::rename(from, to).is_ok() {
            return true;
        }
        sleep(Duration::from_millis(400));
    }

    false
}

/// Khởi chạy ứng dụng mới một cách độc lập
fn relaunch_app(target: &Path) {
    let mut cmd = Command::new(target);
    if let Some(parent) = target.parent() {
        cmd.current_dir(parent);
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(DETACHED_PROCESS);
    }

    match cmd.spawn() {
        Ok(child) => {
            log("Khởi động ứng dụng mới thành công");
            #[cfg(target_os = "windows")]
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(child.id());
            }
        }
        Err(e) => {
            log(&format!("Lỗi khi khởi động lại ứng dụng: {e}"));
        }
    }
}
