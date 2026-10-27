# CHKey Engine ⌨️

**CHKey Engine** là thư viện lõi (Core Input Method Engine) mã nguồn mở viết bằng **Rust**, chịu trách nhiệm xử lý ghép vần và biến đổi ký tự dành cho các ngôn ngữ dân tộc thiểu số vùng Tây Nguyên (Ê-đê, Jrai, Bahnar, M'nông...).

Engine được thiết kế độc lập hoàn toàn với hệ điều hành (OS-agnostic), không phụ thuộc bên ngoài (Zero-dependency), đảm bảo độ trễ thấp nhất và tính an toàn bộ nhớ tuyệt đối.

---

## 🌟 Tính Năng Nổi Bật

- **Hỗ trợ đầy đủ tiếng Ê-đê:** Hỗ trợ chuẩn xác các ký tự đặc thù như nguyên âm ngắn (`ĕ`, `ĭ`, `ŏ`, `ŭ`), nguyên âm trăng kết hợp (`ê̆`, `ơ̆`, `ô̆`, `ư̆`, `â̆`), và các phụ âm gạch ngang/móc (`ƀ`, `č`, `đ`, `ñ`).
- **Hỗ trợ Shift & Caps Lock:** Gõ chính xác chữ hoa có dấu (`Â`, `Ă`, `Ê`, `Đ`, `Ƀ`, `Č`, `Ñ`...).
- **Cơ chế Hoàn tác Phím (Revert / Undo):** Bấm lặp lại phím biến đổi để thoát dấu hoặc lấy lại ký tự nguyên thủy (ví dụ: `aa` -> `â`, gõ tiếp `a` -> `aa`).
- **Tối ưu Backspace cục bộ:** Chỉ xóa đúng số ký tự cần biến đổi thay vì xóa lại cả từ, triệt tiêu hiện tượng giật nháy màn hình (screen flickering).
- **Cách ly dấu câu an toàn:** Dấu phẩy, chấm, số hay ký tự đặc biệt lập tức ngắt từ, bảo đảm không bao giờ xóa lẹm sang các từ đã gõ trước đó.
- **Kiến trúc mở rộng dễ dàng:** Triển khai trait `LanguageProfile` để bổ sung ngôn ngữ mới trong vài chục dòng code.

---

## 📋 Bảng Quy Tắc Gõ Telex Tiếng Ê-đê

| Ký tự kết quả | Phím gõ Telex | Ví dụ từ thực tế |
| :---: | :---: | :--- |
| **â** | `aa` | *m'nâng* |
| **ă** | `ax` | *hdră* |
| **â̆** | `aax` hoặc `â` + `x` | *h'â̆* |
| **ê** | `ee` | *klei* |
| **ĕ** | `ex` | *măt dĕ* |
| **ê̆** | `eex` hoặc `ê` + `x` | *kê̆* |
| **ô** | `oo` | *m'nông* |
| **ŏ** | `ox` | *čŏ* |
| **ô̆** | `oox` hoặc `ô` + `x` | *ô̆* |
| **ơ** | `ow` | *kơ* |
| **ơ̆** | `owx` hoặc `ơ` + `x` | *kơ̆* |
| **ư** | `uu` hoặc `uw` | *čư* |
| **ŭ** | `ux` | *mŭ* |
| **ư̆** | `uwx` hoặc `ư` + `x` | *ư̆* |
| **ĭ** | `ix` | *čĭ* |
| **č** | `cx` | *čih* |
| **ƀ** / **Ƀ** | `bb` | *ƀuôn* / *Ƀuôn* |
| **ñ** / **Ñ** | `nn` | *ñu* / *Ñu* |
| **đ** / **Đ** | `dd` | *đao* / *Đao* |

*Mẹo: Gõ tiếp phím biến đổi một lần nữa để hoàn tác lại chữ cái ban đầu.*

---

## 🏗️ Cấu Trúc Thư Mục

```text
CHKey/engine/
├── Cargo.toml                 # Cấu hình package
├── README.md                  # Hướng dẫn dành cho người dùng và lập trình viên
├── AI_GUIDE.md                # Tài liệu quy chuẩn dành cho các trợ lý AI
├── src/
│   ├── lib.rs                 # Điểm xuất phát của thư viện crate
│   ├── keycode/               # Chuẩn hóa mã phím đa nền tảng
│   │   ├── mod.rs             # Enum KeyCode, struct KeyModifier, struct KeyEvent
│   │   ├── win32.rs           # Ánh xạ Virtual Key Windows (KeyCode::from_win32_vk)
│   │   ├── mac.rs             # Ánh xạ Keycode macOS (KeyCode::from_mac_vk)
│   │   └── linux.rs           # Ánh xạ X11 Keycode Linux (KeyCode::from_linux_vk)
│   ├── character.rs           # Tiện ích phân loại chữ, nguyên âm, phụ âm, case
│   ├── transform.rs           # Bộ máy biến đổi ký tự (Circumflex, Breve, Horn...)
│   ├── state.rs               # Quản lý phiên gõ, từ, và trạng thái trả về Hook
│   ├── language/              # Các hồ sơ ngôn ngữ
│   │   ├── mod.rs             # Trait LanguageProfile
│   │   └── ede.rs             # Hồ sơ tiếng Ê-đê (EdeProfile)
│   └── processor.rs           # Bộ xử lý trung tâm (Processor<L: LanguageProfile>)
└── tests/
    └── ede_typing_test.rs     # Bộ Integration Tests kiểm thử gõ thực tế
```

---

## 💻 Hướng Dẫn Sử Dụng Chi Tiết (Usage Guide)

### 1. Khai báo phụ thuộc (Cargo.toml)

Thêm `chkey_engine` vào file `Cargo.toml` của ứng dụng hoặc crate nền tảng:

```toml
[dependencies]
chkey_engine = { path = "../engine" }
```

---

### 2. Các thành phần và API cốt lõi

- **`Processor<L: LanguageProfile>`** ([`src/processor.rs`](src/processor.rs)): Lõi xử lý gõ phím trung tâm.
  - `Processor::new(language: L)`: Tạo bộ xử lý với cấu hình ngôn ngữ chỉ định (ví dụ `EdeProfile`).
  - `processor.process_key(key: KeyCode, shift: bool, caps_lock: bool) -> KeyHookState`: Xử lý phím cùng trạng thái Shift và Caps Lock.
  - `processor.process(key: KeyCode) -> KeyHookState`: Xử lý phím cơ bản (không kèm bổ trợ).
  - `processor.process_event(event: KeyEvent) -> KeyHookState`: Xử lý sự kiện bàn phím đóng gói đầy đủ.
  - `processor.clear()`: Đặt lại (reset) bộ đệm từ và phiên gõ (khi chuyển focus, chuột click hoặc ngắt văn bản).
- **`KeyCode`** ([`src/keycode/`](src/keycode/)): Chuẩn hóa mã phím đa nền tảng:
  - Windows: `KeyCode::from_win32_vk(vk: u32)`
  - macOS: `KeyCode::from_mac_vk(code: u32)`
  - Linux: `KeyCode::from_linux_vk(code: u32)`
- **`KeyHookState` & `HookCode`** ([`src/state.rs`](src/state.rs)): Kết quả trả về cho tầng OS Hook:
  - `action.code`: Mã lệnh điều khiển:
    - `HookCode::DoNothing (0)`: Ký tự thường, cho phép hệ điều hành in tự nhiên.
    - `HookCode::WillProcess (1)`: Engine đã xử lý phím (như phím xóa Backspace nội bộ).
    - `HookCode::BreakWord (2)`: Phím ngắt từ (Space, Tab, Enter, dấu câu, chữ số...).
    - `HookCode::Transform (3)`: Ký tự biến đổi (cần gửi Backspace và chèn chuỗi mới).
    - `HookCode::Revert (4)`: Hoàn tác phím (khi người dùng gõ lặp lại phím biến đổi).
  - `action.backspace_count`: Số lần phím xóa Backspace mà tầng Hook cần mô phỏng.
  - `action.output_string()`: Chuỗi ký tự Unicode kết quả cần chèn vào văn bản.

---

### 3. Ví dụ Tích hợp Hoàn chỉnh (Integration Example)

Dưới đây là luồng tích hợp chuẩn giữa Engine và tầng bắt phím hệ điều hành:

```rust
use chkey_engine::keycode::KeyCode;
use chkey_engine::language::ede::EdeProfile;
use chkey_engine::processor::Processor;
use chkey_engine::state::HookCode;

fn main() {
    // 1. Khởi tạo Processor với cấu hình ngôn ngữ Ê-đê
    let mut processor = Processor::new(EdeProfile);

    // 2. Khi nhận sự kiện phím từ hệ điều hành (ví dụ: Windows Virtual Key 0x41 cho phím 'A')
    let vk = 0x41;
    let shift_pressed = false;
    let caps_lock_on = false;

    // Chuyển đổi mã phím từ hệ điều hành sang KeyCode chuẩn của Engine
    let keycode = KeyCode::from_win32_vk(vk);

    // 3. Đưa vào Engine để phân tích và xử lý ngữ âm
    let action = processor.process_key(keycode, shift_pressed, caps_lock_on);

    // 4. Xử lý chỉ dẫn trả về từ Engine
    match action.code {
        c if c == HookCode::Transform as u8 || c == HookCode::Revert as u8 => {
            // Có biến đổi hoặc hoàn tác: gửi Backspace và chèn chuỗi ký tự mới
            let backspaces = action.backspace_count;
            let new_text = action.output_string();

            if backspaces > 0 {
                println!("OS Hook: Gửi {} lần phím Backspace", backspaces);
            }
            println!("OS Hook: Chèn chuỗi mới: \"{}\"", new_text);
        }
        c if c == HookCode::BreakWord as u8 => {
            // Gặp dấu ngắt từ (khoảng trắng, enter, dấu câu...)
            println!("Ngắt từ: Bộ đệm từ đã được làm mới tự động.");
        }
        c if c == HookCode::WillProcess as u8 => {
            // Phím điều khiển đã được engine ghi nhận vào bộ nhớ từ
            println!("Engine đã cập nhật trạng thái từ.");
        }
        _ => {
            // HookCode::DoNothing (0): Ký tự thông thường
            println!("Ký tự bình thường, hệ điều hành tự in phím vật lý.");
        }
    }
}
```

---

### 4. Diễn Giải Luồng Xử Lý Khi Gõ Thực Tế

| Ký tự nhập | `action.code` | `action.backspace_count` | `action.output_string()` | Mô tả hành vi |
| :---: | :---: | :---: | :---: | :--- |
| **`a`** (lần 1) | `HookCode::DoNothing` | `0` | `"a"` | Ký tự cơ sở, cho phép OS in tự nhiên. |
| **`a`** (lần 2) | `HookCode::Transform` | `1` | `"â"` | Xóa `'a'`, chèn `'â'`. |
| **`a`** (lần 3) | `HookCode::Revert` | `1` | `"aa"` | Gõ tiếp phím biến đổi -> hoàn tác thành 2 chữ `'aa'`. |
| **`x`** (sau `â`) | `HookCode::Transform` | `1` | `"â̆"` | Ghép thêm dấu trăng kết hợp Unicode `\u{0306}`. |
| **`Space` / Dấu câu** | `HookCode::BreakWord` | `0` | `" "` | Ngắt ranh giới từ an toàn, tránh xóa lẹm từ đứng trước. |

---

### 5. Hướng Dẫn Mở Rộng Ngôn Ngữ Mới

Để hỗ trợ thêm ngôn ngữ mới (như Jrai, Bahnar, M'nông...), chỉ cần tạo struct mới và triển khai trait `LanguageProfile`:

```rust
use chkey_engine::keycode::{KeyCode, KeyModifier};
use chkey_engine::language::LanguageProfile;
use chkey_engine::transform::Transform;

pub struct JraiProfile;

impl LanguageProfile for JraiProfile {
    fn key_to_char(&self, key: KeyCode, modifier: KeyModifier) -> Option<char> {
        // Ánh xạ phím vật lý sang ký tự và xử lý Shift/Caps
        // ...
        None
    }

    fn match_transform(&self, prev_text: &str, next_char: char) -> Option<Transform> {
        // Khai báo các cặp ký tự kích hoạt Transform tương ứng
        // ...
        None
    }

    fn is_vowel(&self, ch: char) -> bool {
        // Phân loại nguyên âm
        false
    }

    fn is_consonant(&self, ch: char) -> bool {
        // Phân loại phụ âm
        false
    }
}
```

---

## 🧪 Chạy Kiểm Thử (Testing)

Engine đi kèm bộ Integration Tests kiểm tra đầy đủ các kịch bản gõ thực tế:

```bash
cargo test
```

---

## 📄 Bản Quyền (License)

Dự án thuộc bộ gõ **CHKey** (Central Highlands Keyboard).
Mọi đóng góp và mã nguồn được phát triển vì mục đích bảo tồn và phát triển chữ viết các ngôn ngữ dân tộc Tây Nguyên.
