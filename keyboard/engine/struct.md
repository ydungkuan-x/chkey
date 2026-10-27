# Kiến Trúc Cấu Trúc Lõi (Engine Architecture)

Tài liệu này mô tả chi tiết kiến trúc kỹ thuật của **CHKey Engine**.

---

## 1. Phân Chia Tầng (Layered Architecture)

Engine được xây dựng theo mô hình phân tầng chặt chẽ:

```text
[OS Hook Layer] (Windows TSF / macOS IMK / Linux IBus)
       │
       ▼
[Keycode Abstraction Layer] (src/keycode/)
       │  Chuẩn hóa mã phím phần cứng thành `KeyEvent`
       ▼
[Processor Engine] (src/processor.rs)
       │  Quản lý máy trạng thái, bộ đệm từ, và hoàn tác
       ├───► [WordState & InputState] (src/state.rs)
       │
       ├───► [Language Profile Trait] (src/language/)
       │        └─► So khớp mẫu Telex -> trả về `Transform`
       │
       └───► [Character & Transform Engine] (src/character.rs & src/transform.rs)
                └─► Xử lý ký tự Unicode & biến đổi Grapheme cluster
       │
       ▼
[KeyHookState Output]
       │  Báo cho OS Hook: số lượng Backspace và chuỗi Unicode cần chèn
       ▼
[OS Native Text Control]
```

---

## 2. Trách Nhiệm Từng Module

| File / Thư mục | Trách nhiệm chính |
| :--- | :--- |
| `src/lib.rs` | Khai báo các module và hằng số toàn cục (`MAX_BUFF`). |
| `src/keycode/mod.rs` | Enum `KeyCode`, struct `KeyModifier` (Shift, CapsLock), struct `KeyEvent`. |
| `src/keycode/win32.rs` | Chuyển đổi Windows Virtual Key (VK) sang `KeyCode`. |
| `src/keycode/mac.rs` | Chuyển đổi macOS Virtual Keycode sang `KeyCode`. |
| `src/keycode/linux.rs` | Chuyển đổi Linux X11 Keycode sang `KeyCode`. |
| `src/character.rs` | Phân loại nguyên âm, phụ âm, hoa/thường, nhận diện dấu câu và ngắt từ. |
| `src/transform.rs` | Thực thi các phép biến đổi hình thái ký tự (`Circumflex`, `Breve`, `Horn`, `BreveMark`, `Stroke`, `Caron`, `Tilde`, `Remove`). |
| `src/state.rs` | Quản lý `WordItem`, `WordState`, `InputState` và cấu trúc kết quả `KeyHookState` kèm `HookCode`. |
| `src/language/mod.rs` | Định nghĩa trait `LanguageProfile`. |
| `src/language/ede.rs` | Cài đặt luật gõ Telex tiếng Ê-đê (`EdeProfile`). |
| `src/processor.rs` | Điều phối luồng gõ phím `Processor<L: LanguageProfile>`. |
| `tests/` | Chứa các bài kiểm thử tích hợp (Integration Tests) tự động. |
