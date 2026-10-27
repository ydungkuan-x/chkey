# Hướng dẫn Kỹ thuật dành cho AI (AI Developer Guide)

Tài liệu này cung cấp toàn bộ ngữ cảnh kỹ thuật, kiến trúc luồng dữ liệu và các quy chuẩn cốt lõi của **CHKey Engine** (bộ gõ tiếng các dân tộc thiểu số Tây Nguyên: Ê-đê, Jrai, Bahnar, M'nông...). Bất kỳ trợ lý AI nào đọc tài liệu này đều có thể nhanh chóng nắm bắt và thực hiện review, sửa lỗi hoặc mở rộng tính năng một cách chính xác nhất.

---

## 1. Tổng quan Dự án (Project Overview)
- **Bản chất crate:** `chkey_engine` là một **Library Crate thuần Rust** (không có `main.rs`), độc lập với hệ điều hành và không phụ thuộc thư viện ngoài (zero-dependency).
- **Mục tiêu:** Cung cấp lõi xử lý bàn phím (Input Method Engine - IME) chuẩn hóa, nhận sự kiện phím từ tầng OS Hook (Windows TSF/Hook, macOS InputMethodKit, Linux IBus/Fcitx) và trả về lệnh điều khiển xóa/chèn văn bản chuẩn xác.

---

## 2. Kiến trúc Thư mục (Architecture)

```text
CHKey/engine/
├── Cargo.toml            # Cấu hình crate (edition 2024, version 1.0.0)
├── AI_GUIDE.md           # Tài liệu kỹ thuật chi tiết dành cho AI
├── README.md             # Tài liệu giới thiệu và hướng dẫn người dùng
├── src/
│   ├── lib.rs            # Khai báo các module và hằng số MAX_BUFF = 32
│   ├── keycode/          # Tầng chuẩn hóa mã phím đa nền tảng
│   │   ├── mod.rs        # KeyCode enum, KeyModifier (Shift, Caps), KeyEvent
│   │   ├── win32.rs      # Bảng ánh xạ Windows Virtual Key (KeyCode::from_win32_vk)
│   │   ├── mac.rs        # Bảng ánh xạ macOS Keycode (KeyCode::from_mac_vk)
│   │   └── linux.rs      # Bảng ánh xạ Linux X11 Keycode (KeyCode::from_linux_vk)
│   ├── character.rs      # Tiện ích phân loại chữ cái, nguyên âm, phụ âm, hoa/thường, dấu câu
│   ├── transform.rs      # Bộ máy biến đổi ký tự (Transform::Circumflex, Breve, Horn, BreveMark, Stroke...)
│   ├── state.rs          # Quản lý trạng thái: WordItem, WordState, InputState, KeyHookState, HookCode
│   ├── language/         # Luật ghép vần của từng ngôn ngữ
│   │   ├── mod.rs        # Trait LanguageProfile (key_to_char, match_transform, is_vowel...)
│   │   └── ede.rs        # EdeProfile: Luật gõ Telex tiếng Ê-đê
│   └── processor.rs      # Lõi thực thi chính: Processor<L: LanguageProfile>
└── tests/
    └── ede_typing_test.rs # Bộ Integration Tests tự động kiểm tra gõ thực tế
```

---

## 3. Điểm bắt đầu & Luồng Dữ liệu (Data Flow)

Điểm bắt đầu của toàn bộ engine là hàm `process_event` (hoặc `process_key` / `process`):

```rust
// Tầng OS Hook gọi hàm này khi có sự kiện phím
let hook_state: KeyHookState = processor.process_key(keycode, shift, caps_lock);
```

### Sơ đồ Luồng Thực Thi
```text
[Bàn phím OS] -> (Mã phím thô: VK/Scancode + Shift/Caps)
       │
       ▼
[keycode::from_win32_vk / mac / linux] -> Chuẩn hóa thành KeyEvent
       │
       ▼
[Processor::process_event]
       │
       ├─► Phím ngắt từ (Space, Tab, Enter, Dấu câu, Số):
       │     └─► Reset WordState -> Trả về HookCode::BreakWord (không xóa lẹm từ trước)
       │
       ├─► Phím Backspace:
       │     └─► Xóa WordItem cuối cùng -> Trả về backspace_count tương ứng
       │
       └─► Phím chữ cái:
             ├─► 1. Hỏi LanguageProfile::match_transform(prev_text, next_char)
             │     └─► Khớp luật -> Gọi transform::apply_transform_str
             │     └─► Cập nhật WordItem -> Trả về HookCode::Transform (backspace = 1, text mới)
             │
             ├─► 2. Kiểm tra Revert (Hoàn tác phím):
             │     └─► Nếu bấm lặp lại phím vừa biến đổi -> Trả về nguyên bản các phím gốc
             │
             └─► 3. Gõ ký tự bình thường:
                   └─► Thêm WordItem mới -> Trả về HookCode::DoNothing (OS tự in phím)
```

---

## 4. Các Quy Chuẩn Bắt Buộc Khi Code & Review (Strict Guidelines)

1. **Grapheme Clusters & Ký tự đa mã (Multi-scalar values):**
   - Các chữ cái như `ê̆`, `ơ̆`, `ô̆`, `ư̆`, `â̆` được tạo thành từ nguyên âm cơ sở + Dấu trăng kết hợp Unicode `\u{0306}` (Combining Breve).
   - Kết quả xuất ra luôn được đóng gói dưới dạng `String` (hoặc mảng UTF-32/UTF-16 trong `KeyHookState`), tuyệt đối không ép về kiểu `char`.
   - Mỗi cụm chữ cái có dấu kết hợp trên giao diện soạn thảo hiện đại chỉ tốn **đúng 1 lần nhấn Backspace** để xóa. `WordItem.backspace_count` luôn kiểm soát chính xác số lượng này, tránh xóa lẹm ký tự phía trước.

2. **Bảo tồn Chữ hoa / Chữ thường (Case Preservation):**
   - Luôn xác định chữ hoa thông qua công thức logic: `shift ^ caps_lock`.
   - Khi biến đổi chữ hoa (ví dụ `Shift + A` + `Shift + A`), kết quả phải là `Â`, không được trả về chữ thường `â`.

3. **Cơ chế Hoàn tác Phím (Revert / Undo):**
   - Bộ gõ phải cho phép người dùng sửa sai nhanh chóng: gõ `aa` -> `â`, gõ tiếp `a` -> hoàn tác thành `aa`.
   - `WordItem` lưu lại `raw_keys` để phục vụ hoàn tác tự nhiên mà không cần lưu buffer phức tạp.

4. **Cô lập Ranh giới Từ (Word Boundary Isolation):**
   - Bất kỳ ký tự phân cách nào (dấu phẩy, chấm, chấm phẩy, số, ngoặc...) đều phải lập tức làm mới `WordState` (`clear()`).
   - Tuyệt đối không được gửi lệnh Backspace xóa ngược qua dấu câu hay từ đứng trước.

5. **Đa hình Generic & Zero-Panic:**
   - `Processor<L: LanguageProfile>` dùng generic tĩnh (monomorphization) để đạt hiệu năng tối đa (không tốn chi phí vtable dynamic dispatch).
   - Không được dùng `unwrap()` hay `expect()` bừa bãi trong luồng xử lý phím. Luôn sử dụng pattern matching, `if let`, `?`, và `checked_sub`.

6. **Vị trí viết Tests:**
   - Tuyệt đối không chèn `#[cfg(test)]` vào các file trong `src/`.
   - Toàn bộ test case phải là Integration Tests đặt trong thư mục `tests/` ở thư mục gốc của engine.

---

## 5. Hướng dẫn Thêm Ngôn ngữ Mới (How to Add a Language)

Khi cần thêm ngôn ngữ mới (ví dụ tiếng Jrai, Bahnar, M'nông):
1. Tạo file mới: `src/language/jrai.rs`.
2. Khai báo struct `JraiProfile` và `impl LanguageProfile for JraiProfile`:
   - Cài đặt `key_to_char`: map phím và xử lý Shift/Caps.
   - Cài đặt `match_transform`: khai báo các cặp ký tự kích hoạt `Transform` tương ứng.
3. Export trong `src/language/mod.rs`: `pub mod jrai;`.
4. Tạo file test trong `tests/jrai_typing_test.rs` để kiểm thử toàn diện trước khi bàn giao.
