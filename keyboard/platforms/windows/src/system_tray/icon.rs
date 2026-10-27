//! Xử lý icon cho System Tray.
//!
//! - [`make_icon`]: Tạo icon màu đầy đủ.
//! - [`init_icon`]: Khởi tạo và cache icon vào thread_local.

use image::ImageReader;
use std::io::Cursor;

/// PNG bytes nhúng trực tiếp vào binary.
const ICON_BYTES: &[u8] = include_bytes!("../../../../chkey.png");

// ── Internal helper ──────────────────────────────────────────────────────────

/// Decode PNG bytes → (raw RGBA pixels, width, height).
fn decode_rgba(
    bytes: &[u8],
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?
        .into_rgba8();

    let (width, height) = img.dimensions();

    Ok((img.into_raw(), width, height))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Tạo icon màu đầy đủ.
pub fn make_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    let (raw, width, height) = decode_rgba(ICON_BYTES)?;

    Ok(tray_icon::Icon::from_rgba(raw, width, height)?)
}

/// Khởi tạo icon tray.
pub fn init_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    make_icon()
}