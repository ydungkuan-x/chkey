use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=rade.ico");

    #[cfg(target_os = "windows")]
    {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let ico_path = manifest_dir.join("rade.ico");

        if ico_path.exists() {
            if let Some(rc_exe) = find_rc_exe() {
                let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
                let rc_path = out_dir.join("chkey_tsf.rc");
                let res_path = out_dir.join("chkey_tsf.res");

                let rc_content = format!(
                    "1 ICON \"{}\"\n",
                    ico_path.to_string_lossy().replace('\\', "/")
                );

                if fs::write(&rc_path, rc_content).is_ok() {
                    let status = Command::new(&rc_exe)
                        .args(["/nologo", "/fo"])
                        .arg(&res_path)
                        .arg(&rc_path)
                        .status();

                    if let Ok(st) = status
                        && st.success()
                        && res_path.exists()
                    {
                        // Chỉ liên kết tài nguyên này khi build file cdylib (chkey_tsf.dll).
                        // Không truyền vào rlib để tránh gây lỗi xung đột CVT1100 (duplicate resource)
                        // khi chkey_windows (CHKey.exe) liên kết với crate này.
                        println!("cargo:rustc-cdylib-link-arg={}", res_path.display());
                    }
                }
            }
        }
    }
}

fn find_rc_exe() -> Option<PathBuf> {
    // 1. Thử lệnh rc.exe trong PATH
    if let Ok(output) = Command::new("rc.exe").arg("/?").output() {
        if output.status.success() || !output.stdout.is_empty() || !output.stderr.is_empty() {
            return Some(PathBuf::from("rc.exe"));
        }
    }

    // 2. Quét các thư mục Windows Kits 10 tiêu chuẩn
    let kits_roots = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];

    for root_str in &kits_roots {
        let root = Path::new(root_str);
        if let Ok(entries) = fs::read_dir(root) {
            let mut versions: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.join(r"x64\rc.exe").exists())
                .collect();
            versions.sort();
            if let Some(latest) = versions.last() {
                return Some(latest.join(r"x64\rc.exe"));
            }
        }
    }

    None
}
