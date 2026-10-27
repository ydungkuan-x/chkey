//! Kiểm tra trạng thái runtime DLL và đăng ký TSF của CHKey.

use std::path::PathBuf;
use crate::extract;
use crate::registration;

/// Các trạng thái có thể có của CHKey runtime và TSF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupStatus {
    /// File DLL runtime tồn tại, SHA-256 khớp với bản nhúng, và TSF COM đã đăng ký trỏ đúng đường dẫn.
    UpToDate {
        dll_path: PathBuf,
        sha256: String,
    },
    /// Chưa có file DLL trong thư mục runtime.
    MissingDll {
        expected_path: PathBuf,
    },
    /// File DLL tồn tại nhưng SHA-256 khác với bản nhúng trong EXE (cần cập nhật bản mới).
    OutdatedDll {
        dll_path: PathBuf,
        current_sha256: String,
        embedded_sha256: String,
    },
    /// File DLL tồn tại và đúng SHA-256, nhưng TSF chưa được đăng ký hoặc COM InProcServer32 trỏ sai đường dẫn.
    NotRegistered {
        dll_path: PathBuf,
        registered_path: Option<String>,
    },
}

/// Kiểm tra toàn diện trạng thái cài đặt runtime của CHKey.
pub fn check_status() -> Result<SetupStatus, String> {
    let dll_path = extract::get_runtime_dll_path()?;

    // 1. Kiểm tra sự tồn tại của file DLL
    if !dll_path.exists() {
        return Ok(SetupStatus::MissingDll {
            expected_path: dll_path,
        });
    }

    // 2. Xác minh tính toàn vẹn và phiên bản qua mã băm SHA-256
    let current_sha256 = extract::file_sha256_hex(&dll_path)?;
    let embedded_sha256 = extract::embedded_dll_sha256_hex();

    if current_sha256 != embedded_sha256 {
        return Ok(SetupStatus::OutdatedDll {
            dll_path,
            current_sha256,
            embedded_sha256,
        });
    }

    // 3. Kiểm tra đăng ký COM InProcServer32 và TSF Profiles
    if !registration::is_registered_with_dll(&dll_path) {
        let registered_path = registration::get_registered_dll_path();
        return Ok(SetupStatus::NotRegistered {
            dll_path,
            registered_path,
        });
    }

    Ok(SetupStatus::UpToDate {
        dll_path,
        sha256: current_sha256,
    })
}
