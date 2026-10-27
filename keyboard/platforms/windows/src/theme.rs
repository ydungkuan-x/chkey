use windows::Win32::System::Registry::{
    RegCloseKey, RegNotifyChangeKeyValue, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_NOTIFY, KEY_READ, REG_NOTIFY_CHANGE_LAST_SET, REG_VALUE_TYPE,
};
use windows::core::w;

/// Kiểm tra chế độ màu hiện tại của Windows thông qua Registry:
/// - `AppsUseLightTheme` = 0 -> Dark Mode (`true`)
/// - `AppsUseLightTheme` = 1 -> Light Mode (`false`)
pub fn is_dark_mode() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey).is_err() {
            return false;
        }

        let value_name = w!("AppsUseLightTheme");
        let mut data: u32 = 1;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;

        let res = RegQueryValueExW(
            hkey,
            value_name,
            None,
            Some(&mut REG_VALUE_TYPE(0)),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey);

        if res.is_ok() {
            data == 0
        } else {
            false
        }
    }
}

/// Lắng nghe sự kiện đổi theme realtime qua `RegNotifyChangeKeyValue` của Windows.
/// Hàm chạy trên một thread nền độc lập và gọi callback ngay khi có thay đổi.
pub fn watch_theme_changes<F>(callback: F)
where
    F: Fn(bool) + Send + 'static,
{
    std::thread::spawn(move || unsafe {
        let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");

        loop {
            let mut hkey = HKEY::default();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey,
                0,
                KEY_READ | KEY_NOTIFY,
                &mut hkey,
            )
            .is_err()
            {
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }

            // Chờ Windows gửi tín hiệu khi Registry key này bị thay đổi (block thread không tốn CPU)
            let notify_res = RegNotifyChangeKeyValue(
                hkey,
                false,
                REG_NOTIFY_CHANGE_LAST_SET,
                None,
                false,
            );

            let _ = RegCloseKey(hkey);

            if notify_res.is_ok() {
                // Đợi 50ms để Windows kịp ghi xong giá trị mới vào Registry
                std::thread::sleep(std::time::Duration::from_millis(50));
                let dark = is_dark_mode();
                callback(dark);
            } else {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });
}
