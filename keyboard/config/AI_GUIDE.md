# Hướng dẫn Kỹ thuật dành cho AI (AI Developer Guide) - `chkey_config`

Tài liệu này cung cấp toàn bộ ngữ cảnh kỹ thuật, kiến trúc luồng dữ liệu, đặc tả API và các quy chuẩn bắt buộc của module **CHKey Config** (`chkey_config`). Bất kỳ trợ lý AI nào đọc tài liệu này đều có thể nhanh chóng nắm bắt và thực hiện review, sửa lỗi hoặc mở rộng tính năng cấu hình một cách chính xác nhất.

---

## 1. Tổng quan Dự án (Project Overview)
- **Bản chất crate:** `chkey_config` là một **Library Crate thuần Rust** (không có `main.rs`), độc lập hoàn toàn với giao diện người dùng (Zero UI Dependency) và cung cấp khả năng quản lý cấu hình TOML đa nền tảng (Windows, macOS, Linux).
- **Mục tiêu:** Cung cấp nguồn cấu hình tập trung duy nhất (Single Source of Truth) cho toàn bộ ứng dụng CHKey:
  - Ứng dụng chính & Giao diện (`chkey_windows`, `chkey_ui`)
  - Bàn phím hệ thống Text Services Framework IME DLL (`chkey_ime`)
  - Trình cài đặt / gỡ cài đặt (`chkey_setup`)
  - Cơ chế tự động cập nhật (`chkey_updater`)
- **Nguyên tắc an toàn dữ liệu:** Bảo đảm tính toàn vẹn (Data Integrity) qua cơ chế ghi nguyên tử (Atomic Write), tự động bổ sung giá trị mặc định cho trường thiếu (Missing Fields Defaulting), và tự động di chuyển dữ liệu cũ (Auto-migration).

---

## 2. Kiến trúc Thư mục (Architecture)

```text
CHKey/config/
├── Cargo.toml          # Khai báo dependencies: serde, toml, directories
├── AI_GUIDE.md         # Tài liệu kỹ thuật chi tiết dành cho AI Developer
├── README.md           # Hướng dẫn sử dụng file cấu hình dành cho Người dùng cuối
└── src/
    ├── lib.rs          # Khai báo public exports và bộ 9 Unit Tests toàn diện
    ├── model.rs        # Định nghĩa struct AppConfig với serde default attributes
    ├── defaults.rs     # Hằng số phiên bản & các hàm khởi tạo giá trị mặc định
    ├── path.rs         # Xác định đường dẫn thư mục/file cấu hình chuẩn & Migration
    └── storage.rs      # Đọc, ghi nguyên tử (Atomic Write), reset và xử lý lỗi ConfigError
```

### Trách nhiệm của từng tệp:
- **`model.rs`**: Khai báo struct `AppConfig`. Mọi trường đều có `#[serde(default = "defaults::...")]` để đảm bảo deserialization luôn thành công ngay cả khi người dùng xóa bớt dòng trong file TOML.
- **`defaults.rs`**: Tập trung toàn bộ giá trị mặc định (`CURRENT_CONFIG_VERSION = 1`, `language = "vi"`, `dark_mode = true`,...).
- **`path.rs`**: Giải quyết đường dẫn theo OS. Trên Windows, ưu tiên `%LOCALAPPDATA%\CHKey`, đồng thời có logic tự động sao chép cấu hình cũ từ `%APPDATA%\CHKey` nếu có.
- **`storage.rs`**: Cung cấp các hàm `load_config`, `save_config`, `reset_config`. Quá trình ghi sử dụng file tạm `.config.tmp-{PID}-{Timestamp}` rồi mới `rename` để chống hỏng file khi crash/mất điện.
- **`lib.rs`**: Re-export API công khai phẳng tại `chkey_config::*` và chứa bộ kiểm thử unit tests.

---

## 3. Điểm bắt đầu & Luồng Dữ liệu (Data Flow)

### 3.1. Luồng Đọc Cấu hình (`load_config`)
```text
load_config() -> get_config_path()
       │
       ▼
Kiểm tra file tồn tại?
       ├─► [Chưa tồn tại]:
       │     └─► Tạo AppConfig::default() -> save_config_to_path() -> Trả về cấu hình mặc định
       │
       └─► [Đã tồn tại]:
             ├─► Đọc nội dung file TOML sang chuỗi
             ├─► Deserialization qua toml::from_str
             │     └─► Trường nào thiếu? -> Tự lấy giá trị từ defaults.rs
             ├─► Kiểm tra config_version:
             │     ├─► config_version > CURRENT_CONFIG_VERSION -> Báo lỗi UnsupportedVersion
             │     └─► config_version < CURRENT_CONFIG_VERSION -> Tự động migrate lên bản mới
             └─► Trả về Ok(AppConfig)
```

### 3.2. Luồng Ghi Cấu hình Nguyên tử (`save_config`)
```text
save_config(&config) -> get_config_path()
       │
       ▼
Đảm bảo thư mục cha tồn tại (fs::create_dir_all)
       │
       ▼
Tuần tự hóa sang chuỗi TOML (toml::to_string_pretty)
       │
       ▼
Ghi vào file tạm cùng thư mục: `.config.tmp-{PID}-{Timestamp}`
       │
       ▼
Đổi tên nguyên tử (fs::rename(temp, target))
       ├─► Thành công -> Hoàn tất an toàn
       └─► Thất bại (hệ thống file không hỗ trợ rename đè):
             └─► Fallback: fs::copy(temp, target) -> fs::remove_file(temp)
```

---

## 4. Các Quy Chuẩn Bắt Buộc Khi Code & Review (Strict Guidelines)

1. **Tuyệt đối không đưa UI hoặc OS-specific API nặng vào Crate:**
   - Crate này phải biên dịch được ở mọi nơi mà Rust hỗ trợ.
   - Không import Slint, Windows API (win32), COM hay các crate đồ họa.
2. **Luôn sử dụng `#[serde(default = "...")]` cho mọi trường mới:**
   - Khi thêm bất kỳ field nào vào `AppConfig`, bắt buộc phải tạo hàm getter giá trị mặc định trong `defaults.rs` và gắn thuộc tính serde default vào `model.rs`.
   - Điều này ngăn chặn 100% lỗi runtime panic khi người dùng dùng bản cập nhật mới mở file cấu hình cũ chưa có trường đó.
3. **Bảo toàn File khi gặp TOML hỏng (Preserve Corrupted File):**
   - Nếu người dùng chỉnh sửa file làm hỏng cú pháp TOML, `load_config` phải trả về `Err(ConfigError::Deserialize(...))`.
   - **Tuyệt đối KHÔNG tự ý ghi đè file hỏng bằng file mặc định**, vì sẽ làm mất dữ liệu tùy chỉnh của người dùng trước khi họ kịp sửa sai.
4. **Đường dẫn chuẩn trên Windows là `%LOCALAPPDATA%\CHKey`:**
   - Bộ gõ hoạt động ở mức per-machine/per-user không đồng bộ mạng Roaming.
   - Luôn duy trì logic migration tự động từ `%APPDATA%\CHKey` trong `path.rs` để không làm mất cài đặt của người dùng các phiên bản trước.
5. **Đảm bảo tính Nguyên tử khi Ghi File (Atomic Replacement):**
   - Không được dùng `fs::write` trực tiếp lên file đích `config.toml` vì nếu ứng dụng bị kill giữa chừng, file sẽ bị rỗng (0 bytes).
   - Luôn ghi qua file tạm có tiền tố ẩn `.config.tmp-` rồi mới rename.
6. **Không dùng `unwrap()` / `expect()` bừa bãi:**
   - Mọi lỗi I/O, path, serialize phải được gom về `ConfigError` để tầng gọi (caller) tự quyết định cách xử lý hoặc fallback.

---

## 5. Bảng Đặc tả Kiểu Dữ liệu & API (Public API Reference)

### Struct `AppConfig`
```rust
pub struct AppConfig {
    pub config_version: u32,       // Mặc định: 1
    pub language: String,          // Mặc định: "vi" ("vi", "en", "ede")
    pub encoding: String,          // Mặc định: "unicode"
    pub input_method: String,      // Mặc định: "ede"
    pub keyboard_enabled: bool,    // Mặc định: true
    pub run_on_startup: bool,      // Mặc định: true
    pub show_dialog_on_startup: bool, // Mặc định: true
    pub auto_update: bool,         // Mặc định: true
    pub dark_mode: bool,           // Mặc định: true
}
```

### Các Hàm Quản lý Cấu hình
```rust
pub fn load_config() -> Result<AppConfig, ConfigError>;
pub fn save_config(config: &AppConfig) -> Result<(), ConfigError>;
pub fn reset_config() -> Result<AppConfig, ConfigError>;

pub fn load_config_from_path(path: &Path) -> Result<AppConfig, ConfigError>;
pub fn save_config_to_path(path: &Path, config: &AppConfig) -> Result<(), ConfigError>;
pub fn reset_config_at_path(path: &Path) -> Result<AppConfig, ConfigError>;

pub fn get_config_dir() -> Result<PathBuf, PathError>;
pub fn get_config_path() -> Result<PathBuf, PathError>;
```

### Enum `ConfigError`
```rust
pub enum ConfigError {
    Path(PathError),
    Io(std::io::Error),
    Serialize(toml::ser::Error),
    Deserialize(toml::de::Error),
    UnsupportedVersion { version: u32, max_supported: u32 },
}
```

---

## 6. Hướng dẫn Thêm Trường Cấu hình Mới (How to Add a Config Field)

Khi ứng dụng cần bổ sung thiết lập mới (ví dụ `beep_on_error: bool`):

1. **Bước 1: Khai báo hàm mặc định trong `src/defaults.rs`**:
   ```rust
   pub fn default_beep_on_error() -> bool {
       false
   }
   ```
2. **Bước 2: Thêm trường vào `AppConfig` trong `src/model.rs`**:
   ```rust
   #[serde(default = "defaults::default_beep_on_error")]
   pub beep_on_error: bool,
   ```
3. **Bước 3: Cập nhật hàm `Default for AppConfig` trong `src/model.rs`**:
   ```rust
   beep_on_error: defaults::default_beep_on_error(),
   ```
4. **Bước 4: Cập nhật Unit Tests trong `src/lib.rs`**:
   - Thêm assertion kiểm tra giá trị mặc định trong `test_default_config`.
   - Chạy `cargo test -p chkey_config` để đảm bảo 100% test vượt qua.
5. **Bước 5: Cập nhật tài liệu người dùng trong `README.md`** để người dùng biết ý nghĩa tham số mới.
