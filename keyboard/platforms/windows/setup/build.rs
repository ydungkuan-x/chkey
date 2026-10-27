use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CHKEY_TSF_DLL_PATH");
    println!("cargo:rerun-if-env-changed=CHKEY_IME_DLL_PATH");
    println!("cargo:rerun-if-env-changed=CHKEY_EXE_PATH");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x86_64".to_string());
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));

    // 1. Tìm kiếm và nhúng DLL TSF (ưu tiên chkey_tsf.dll, fallback chkey_ime.dll)
    let dll_path = try_locate_artifact(
        "chkey_tsf.dll",
        "CHKEY_TSF_DLL_PATH",
        &target_arch,
        &manifest_dir,
        &out_dir,
    )
    .or_else(|| {
        try_locate_artifact(
            "chkey_ime.dll",
            "CHKEY_IME_DLL_PATH",
            &target_arch,
            &manifest_dir,
            &out_dir,
        )
    })
    .unwrap_or_else(|| {
        panic!(
            "Không tìm thấy file nhị phân hợp lệ 'chkey_tsf.dll' hoặc 'chkey_ime.dll' cho kiến trúc '{}'.",
            target_arch
        );
    });

    let out_tsf_dll = out_dir.join("chkey_tsf.dll");
    let out_ime_dll = out_dir.join("chkey_ime.dll");
    let _ = fs::copy(&dll_path, &out_tsf_dll);
    let _ = fs::copy(&dll_path, &out_ime_dll);
    println!("cargo:rerun-if-changed={}", dll_path.display());

    let ico_path = manifest_dir.join("../resources/chkey.ico");
    if ico_path.exists() {
        let _ = fs::copy(&ico_path, out_dir.join("chkey.ico"));
        println!("cargo:rerun-if-changed={}", ico_path.display());
    }

    let rad_ico_path = manifest_dir.join("../tsf/rade.ico");
    let rad_ico_res = manifest_dir.join("../resources/rade.ico");
    let rad_ico_old = manifest_dir.join("../tsf/rad.ico");
    let src_rad = if rad_ico_path.exists() {
        Some(rad_ico_path)
    } else if rad_ico_res.exists() {
        Some(rad_ico_res)
    } else if rad_ico_old.exists() {
        Some(rad_ico_old)
    } else {
        None
    };
    if let Some(src) = src_rad {
        let _ = fs::copy(&src, out_dir.join("rad.ico"));
        let _ = fs::copy(&src, out_dir.join("rade.ico"));
        println!("cargo:rerun-if-changed={}", src.display());
    } else {
        // Fallback dummy file để include_bytes! không bao giờ bị lỗi
        let _ = fs::write(out_dir.join("rad.ico"), b"");
        let _ = fs::write(out_dir.join("rade.ico"), b"");
    }

    // 2. Tìm kiếm CHKey.exe (TÙY CHỌN, chỉ dùng nếu build CHKey-Setup.exe)
    if let Some(exe_path) = try_locate_artifact(
        "CHKey.exe",
        "CHKEY_EXE_PATH",
        &target_arch,
        &manifest_dir,
        &out_dir,
    ) {
        let out_exe = out_dir.join("CHKey.exe");
        let _ = fs::copy(&exe_path, &out_exe);
        println!("cargo:rerun-if-changed={}", exe_path.display());
    } else {
        // Tạo file dummy nếu không tìm thấy CHKey.exe để tránh compile error cho CHKey-Setup
        let out_exe = out_dir.join("CHKey.exe");
        if !out_exe.exists() {
            let _ = fs::write(&out_exe, b"");
        }
    }

    // winres is not compiled here to avoid CVT1100 duplicate resource collision
    // when chkey_setup is linked into the main CHKey executable (which already compiles resources).
}

fn try_locate_artifact(
    filename: &str,
    env_var: &str,
    target_arch: &str,
    manifest_dir: &Path,
    out_dir: &Path,
) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(custom) = env::var(env_var) {
        let p = PathBuf::from(custom);
        if p.exists() {
            candidates.push(p);
        }
    }

    if let Some(profile_dir) = out_dir.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
        candidates.push(profile_dir.join(filename));
        candidates.push(profile_dir.join("deps").join(filename));
    }

    let ws_target = manifest_dir.join("../../../target");
    candidates.push(ws_target.join("debug").join(filename));
    candidates.push(ws_target.join("release").join(filename));
    candidates.push(ws_target.join("x86_64-pc-windows-msvc/debug").join(filename));
    candidates.push(ws_target.join("x86_64-pc-windows-msvc/release").join(filename));

    let res = manifest_dir.join("../resources").join(filename);
    if res.exists() {
        candidates.push(res);
    }

    for candidate in &candidates {
        if candidate.exists() && validate_pe_architecture(candidate, target_arch).is_ok() {
            return Some(candidate.clone());
        }
    }

    None
}

#[allow(dead_code)]
fn locate_artifact(
    filename: &str,
    env_var: &str,
    target_arch: &str,
    manifest_dir: &Path,
    out_dir: &Path,
) -> PathBuf {
    if let Some(path) = try_locate_artifact(filename, env_var, target_arch, manifest_dir, out_dir) {
        path
    } else {
        panic!(
            "Không tìm thấy file nhị phân hợp lệ '{}' cho kiến trúc '{}'.",
            filename, target_arch
        );
    }
}

fn validate_pe_architecture(path: &Path, target_arch: &str) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("Không thể mở file: {e}"))?;

    let mut dos_header = [0u8; 64];
    file.read_exact(&mut dos_header)
        .map_err(|e| format!("Lỗi đọc DOS header: {e}"))?;

    if dos_header[0] != b'M' || dos_header[1] != b'Z' {
        return Err("File không phải định dạng PE (thiếu chữ ký MZ)".to_string());
    }

    let e_lfanew = u32::from_le_bytes([
        dos_header[60],
        dos_header[61],
        dos_header[62],
        dos_header[63],
    ]) as u64;

    file.seek(SeekFrom::Start(e_lfanew))
        .map_err(|e| format!("Lỗi seek đến PE header: {e}"))?;

    let mut pe_signature = [0u8; 4];
    file.read_exact(&mut pe_signature)
        .map_err(|e| format!("Lỗi đọc PE signature: {e}"))?;

    if pe_signature != [b'P', b'E', 0, 0] {
        return Err("File không có PE signature hợp lệ".to_string());
    }

    let mut coff_header = [0u8; 20];
    file.read_exact(&mut coff_header)
        .map_err(|e| format!("Lỗi đọc COFF header: {e}"))?;

    let machine = u16::from_le_bytes([coff_header[0], coff_header[1]]);

    let expected_machine = match target_arch {
        "x86_64" => 0x8664u16,
        "aarch64" => 0xAA64u16,
        "x86" => 0x014Cu16,
        other => return Err(format!("Kiến trúc target chưa được hỗ trợ: {other}")),
    };

    if machine != expected_machine {
        return Err(format!(
            "Kiến trúc PE machine (0x{:04X}) không khớp với target '{}' (kỳ vọng 0x{:04X})",
            machine, target_arch, expected_machine
        ));
    }

    Ok(())
}
