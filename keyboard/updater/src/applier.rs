use std::path::Path;
use std::process::Command;

/// Kích hoạt chương trình trung gian chkey_installer để thay thế file và khởi động lại.
/// Hàm này sau khi khởi chạy chkey_installer sẽ lập tức thoát tiến trình hiện tại.
pub fn apply_update(new_file_path: impl AsRef<Path>) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Không lấy được đường dẫn exe hiện tại: {e}"))?;
    let pid = std::process::id();
    let new_path = new_file_path.as_ref().to_path_buf();

    if !new_path.exists() {
        return Err(format!("File cập nhật mới không tồn tại: {}", new_path.display()));
    }

    // Tìm chkey_installer trong cùng thư mục với exe hiện tại
    let exe_dir = current_exe.parent().ok_or("Không xác định được thư mục exe")?;
    let installer_name = if cfg!(target_os = "windows") {
        "chkey_installer.exe"
    } else {
        "chkey_installer"
    };

    let installer_exe = exe_dir.join(installer_name);

    let mut cmd = if installer_exe.exists() {
        let mut c = Command::new(&installer_exe);
        c.args([
            "--pid",
            &pid.to_string(),
            "--target",
            &current_exe.to_string_lossy(),
            "--new",
            &new_path.to_string_lossy(),
        ]);
        c
    } else {
        // new_path chính là file bộ cài đặt Setup/Installer tải về (như windows-x64.exe / CHKey_Setup.exe)
        Command::new(&new_path)
    };

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        cmd.creation_flags(DETACHED_PROCESS);
    }

    cmd.spawn()
        .map_err(|e| format!("Không khởi chạy được installer: {e}"))?;

    // Thoát ứng dụng hiện tại để giải phóng file lock
    std::process::exit(0);
}
