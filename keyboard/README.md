A Ă Â B Ƀ Č D Đ E Ĕ Ê Ê̆ G H I Ĭ J K L M N Ñ O Ŏ Ơ Ơ̆ Ô Ô̆ P R S T U Ŭ Ư Ư̆ W Y
a ă â b ƀ č d đ e ĕ ê ê̆ g h i ĭ j k l m n ñ o ơ ŏ ơ̆ ô ô̆ p r s t u ŭ ư ư̆ w y

          Keyboard Hook
                │
                ▼
       ┌─────────────────┐
       │ nhận KeyDown    │
       └────────┬────────┘
                │
                ▼
       xác định InputType
          /           \
       Telex           VNI
         │              │
         ▼              ▼

is_special_key() is_number_key()
│ │
└──────┬───────┘
▼
Phân tích từ đang gõ
│
▼
thêm/xóa/thay ký tự
│
▼
KeyHookState
┌──────────────┐
│ code │
│ backspace │
│ new chars │
│ char_data[] │
└──────┬───────┘
▼
Keyboard injection

# Bản Debug

cargo build -p chkey_windows --lib

# Hoặc bản Release (tối ưu hóa, khuyên dùng khi đăng ký DLL vào Windows)

cargo build -p chkey_windows --lib --release

# Chạy với quyền Administrator

regsvr32 "C:\Users\ydung\Desktop\Keyboard\CHKey\target\release\chkey_tsf.dll"

# Chạy với quyền Administrator (cờ /u là unregister)

regsvr32 /u "C:\Users\ydung\Desktop\Keyboard\CHKey\target\release\chkey_tsf.dll"

# Build arm

Cài gói MSVC v142 - VS 2019 C++ ARM64 build tools (v14.29-16.11)

# 1. Build TSF DLL trước

cargo build -p chkey_tsf --release --target aarch64-pc-windows-msvc

# 2. Build ứng dụng chính CHKey

cargo build -p chkey_windows --release --target aarch64-pc-windows-msvc

# 3. (Tùy chọn) Build bộ cài đặt CHKey_Setup

cargo build -p chkey_setup --bin CHKey_Setup --release --target aarch64-pc-windows-msvc
