# Hướng dẫn Cấu hình CHKey (`config.toml`)

Tài liệu này hướng dẫn người dùng cách tìm, chỉnh sửa, sao lưu và khôi phục cài đặt cấu hình của bộ gõ tiếng Tây Nguyên **CHKey**.

> **Dành cho Lập trình viên & AI:** Nếu bạn cần tài liệu kiến trúc kỹ thuật, đặc tả API và quy chuẩn mã nguồn, vui lòng xem tệp [AI_GUIDE.md](file:///c:/Users/ydung/Desktop/Keyboard/CHKey/config/AI_GUIDE.md).

---

## 1. Vị trí Lưu File Cấu hình

File cấu hình có tên là `config.toml` và được lưu tại các thư mục chuẩn theo từng hệ điều hành:

| Hệ điều hành | Đường dẫn thư mục | Đường dẫn tập tin đầy đủ |
| :--- | :--- | :--- |
| **Windows** | `%LOCALAPPDATA%\CHKey` | `C:\Users\<Tên_bạn>\AppData\Local\CHKey\config.toml` |
| **macOS** | `~/Library/Application Support/CHKey` | `~/Library/Application Support/CHKey/config.toml` |
| **Linux** | `~/.config/chkey` hoặc `$XDG_CONFIG_HOME/chkey` | `~/.config/chkey/config.toml` |

### Mẹo mở nhanh trên Windows:
1. Nhấn tổ hợp phím `Windows + R` trên bàn phím để mở hộp thoại Run.
2. Nhập hoặc dán nội dung sau vào ô:
   ```text
   %LOCALAPPDATA%\CHKey
   ```
3. Nhấn **Enter** hoặc bấm **OK**. Cửa sổ thư mục chứa file `config.toml` sẽ hiện ra ngay lập tức.

---

## 2. Mẫu File Cấu hình Chuẩn (`config.toml`)

Dưới đây là nội dung mẫu đầy đủ của file `config.toml`:

```toml
config_version = 1
language = "vi"
encoding = "unicode"
input_method = "ede"
keyboard_enabled = true
run_on_startup = true
show_dialog_on_startup = true
auto_update = true
dark_mode = true
```

---

## 3. Ý Nghĩa Chi Tiết Từng Tùy Chọn

| Tên tham số | Kiểu giá trị | Mặc định | Ý nghĩa & Các giá trị hợp lệ |
| :--- | :---: | :---: | :--- |
| `config_version` | Số | `1` | **Phiên bản cấu trúc file.** Giữ nguyên giá trị này để ứng dụng nhận diện chính xác. |
| `language` | Chuỗi | `"vi"` | **Ngôn ngữ hiển thị của giao diện:**<br>• `"vi"`: Tiếng Việt<br>• `"en"`: Tiếng Anh (English)<br>• `"ede"`: Tiếng Êđê |
| `encoding` | Chuỗi | `"unicode"` | **Bảng mã ký tự văn bản:**<br>• `"unicode"`: Unicode dựng sẵn chuẩn quốc tế. |
| `input_method` | Chuỗi | `"ede"` | **Kiểu gõ ký tự:**<br>• `"ede"`: Bộ gõ tiếng Ê-đê & các dân tộc Tây Nguyên. |
| `keyboard_enabled` | Đúng / Sai | `true` | **Bật hoặc tắt bộ gõ:**<br>• `true`: Đang bật gõ tiếng dân tộc.<br>• `false`: Tạm tắt bộ gõ (gõ tiếng Anh thông thường). |
| `run_on_startup` | Đúng / Sai | `true` | **Khởi động cùng máy tính:**<br>• `true`: CHKey tự động chạy mỗi khi bạn bật máy tính.<br>• `false`: Không tự chạy, chỉ mở khi người dùng bấm mở. |
| `show_dialog_on_startup` | Đúng / Sai | `true` | **Hiển thị bảng điều khiển khi mở máy:**<br>• `true`: Tự hiện cửa sổ CHKey khi máy tính khởi động.<br>• `false`: Chạy ẩn dưới khay hệ thống (System Tray). |
| `auto_update` | Đúng / Sai | `true` | **Tự động cập nhật:**<br>• `true`: Tự kiểm tra phiên bản mới từ máy chủ khi khởi động.<br>• `false`: Tắt tự động kiểm tra. |
| `dark_mode` | Đúng / Sai | `true` | **Chế độ giao diện Sáng / Tối:**<br>• `true`: Giao diện tối (Dark Mode - êm mắt).<br>• `false`: Giao diện sáng (Light Mode). |

---

## 4. Hướng dẫn Chỉnh sửa Cấu hình Thủ công

Bạn có thể thay đổi các cài đặt bằng cách chỉnh sửa trực tiếp file `config.toml`:

1. **Đóng hoàn toàn ứng dụng CHKey** (nhấp chuột phải vào biểu tượng CHKey dưới khay hệ thống và chọn **Thoát**).
2. Mở file `config.toml` bằng ứng dụng soạn thảo như **Notepad**, **VS Code** hoặc bất kỳ trình soạn thảo văn bản nào.
3. Thay đổi các giá trị bạn muốn (ví dụ đổi `dark_mode = false` để chuyển sang nền sáng).
4. Nhấn `Ctrl + S` để lưu file lại.
5. Mở lại ứng dụng CHKey để cài đặt mới có hiệu lực.

---

## 5. Sao lưu, Khôi phục và Đặt lại về Mặc định

### Cách đặt lại cài đặt về mặc định ban đầu (Reset)
Nếu bạn lỡ sửa nhầm làm file bị lỗi hoặc muốn đưa mọi cài đặt về như lúc mới cài đặt:
- **Cách 1 (Từ giao diện):** Mở bảng điều khiển CHKey, nhấn vào nút **"Mặc định"** (hoặc *Restore*).
- **Cách 2 (Bằng tay):** Thoát ứng dụng CHKey, sau đó **xóa hẳn file `config.toml`**. Khi bạn mở lại CHKey, chương trình sẽ tự động tạo một file `config.toml` mới nguyên bản.

### Cách sao lưu cài đặt (Backup)
- Sao chép file `config.toml` cất vào USB hoặc dịch vụ lưu trữ đám mây.
- Khi cài lại Windows hoặc chuyển sang máy tính mới, chỉ cần cài CHKey rồi dán đè file `config.toml` đã sao lưu vào thư mục `%LOCALAPPDATA%\CHKey`.

---

## 6. Xử lý Sự cố Thường gặp (Troubleshooting)

- **Q: Tôi sửa file nhưng khi mở ứng dụng lên cài đặt không đổi?**  
  *A:* Hãy đảm bảo bạn đã thoát hoàn toàn CHKey trước khi sửa file và lưu lại (`Ctrl + S`), vì nếu CHKey đang chạy nó có thể lưu đè cài đặt hiện tại lên file.

- **Q: Tôi chỉnh sửa sai cú pháp (quên dấu ngoặc kép, gõ sai chữ) thì sao?**  
  *A:* CHKey có cơ chế tự bảo vệ an toàn: nếu file bị lỗi cú pháp, ứng dụng sẽ không xóa file của bạn mà sẽ tạm thời dùng cấu hình mặc định an toàn cho đến khi bạn sửa lại file cho đúng.
