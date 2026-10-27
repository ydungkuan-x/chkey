# Hướng dẫn Cập nhật Phiên bản CHKey (`chkey_updater`)

Tài liệu này giải thích chi tiết cách thức hoạt động của tính năng kiểm tra và tự động cập nhật phiên bản mới trên bộ gõ **CHKey**.

> **Dành cho Lập trình viên & AI:** Nếu bạn cần tài liệu kiến trúc kỹ thuật, luồng dữ liệu async, đặc tả API và triển khai server cập nhật, vui lòng xem tệp [AI_GUIDE.md](file:///c:/Users/ydung/Desktop/Keyboard/CHKey/updater/AI_GUIDE.md).

---

## 1. Tính năng Cập nhật Hoạt động như thế nào?

Bộ gõ CHKey được tích hợp cơ chế cập nhật tự động an toàn và tiện lợi, giúp bạn luôn được trải nghiệm các tính năng mới nhất, sửa lỗi và cập nhật từ vựng tiếng các dân tộc Tây Nguyên:

1. **Tự động kiểm tra:** Mỗi khi bạn mở CHKey, ứng dụng sẽ gửi yêu cầu nhẹ nhàng đến máy chủ để kiểm tra xem đã có phiên bản mới hơn phiên bản bạn đang dùng hay chưa.
2. **Kiểm tra thủ công:** Bạn có thể chủ động bấm nút **"Cập nhật"** (Updates) trên thanh công cụ của CHKey bất kỳ lúc nào để kiểm tra ngay lập tức.
3. **Tiến trình trực quan:** Hộp thoại cập nhật sẽ hiển thị phần trăm (%) và dung lượng (MB) tải về theo thời gian thực.
4. **Xác thực an toàn:** Mọi tệp tải về đều được kiểm tra tính toàn vẹn bằng thuật toán mã hóa SHA-256. Nếu tệp bị tải ngắt quãng hoặc bị lỗi mạng, hệ thống sẽ tự hủy tệp hỏng để bảo vệ máy tính của bạn.
5. **Cài đặt & Khởi động lại tự động:** Khi tải xong, CHKey sẽ tự động đóng lại, tiến hành nâng cấp các tệp và khởi động lại phiên bản mới mà bạn không cần phải tự tay gỡ cài đặt bản cũ.

---

## 2. Quy trình Cập nhật Từng bước

Khi có phiên bản mới, màn hình cập nhật sẽ xuất hiện:

```text
+---------------------------------------------------+
|  Cập nhật phiên bản                      -  □  ✕  |
+---------------------------------------------------+
|                                                   |
|  Kiểm tra bản cập nhật                            |
|  Đã có phiên bản mới: v1.2.0 (Hiện tại: v1.0.0)    |
|                                                   |
|                         [ Cập nhật ]    [ Đóng ]  |
+---------------------------------------------------+
```

- **Bước 1 (Nhận thông báo):** Bấm nút **"Cập nhật"** (Update) để bắt đầu tải về.
- **Bước 2 (Đang tải xuống):** Ứng dụng sẽ hiển thị tiến trình: `Đang tải xuống: 45%` (hoặc `12.5 MB`). Nút bấm sẽ chuyển sang trạng thái đang tải.
- **Bước 3 (Hoàn tất & Cài đặt):** Khi tải xong 100%, hệ thống hiển thị thông báo `Tải xong! Đang khởi động lại để cập nhật...` và sau 1.5 giây (hoặc khi bạn bấm **"Cài đặt ngay"**), CHKey sẽ tự khởi động bộ cài để hoàn tất.

---

## 3. Cách Bật hoặc Tắt Tự động Cập nhật

Nếu bạn không muốn CHKey tự động kiểm tra phiên bản mới mỗi khi mở máy:

### Cách 1: Từ giao diện phần mềm
1. Mở cửa sổ chính của CHKey.
2. Tìm mục **Cài đặt** (Settings).
3. Bỏ dấu tích tại ô **"Tự động cập nhật"** (Automatic updates).

### Cách 2: Chỉnh sửa file `config.toml`
Mở file `config.toml` (tại `%LOCALAPPDATA%\CHKey\config.toml`) và đổi:
```toml
auto_update = false
```

---

## 4. Tệp Cập nhật Lưu ở Đâu?

Trong quá trình tải, tệp cài đặt mới được lưu tạm thời tại thư mục Temp của máy tính:
- **Windows:** `%TEMP%\CHKey\` (ví dụ: `C:\Users\<Tên_bạn>\AppData\Local\Temp\CHKey\windows-x64.exe`).

Sau khi cài đặt xong, bạn có thể xóa thư mục này nếu muốn giải phóng dung lượng ổ đĩa.

---

## 5. Câu hỏi Thường gặp (FAQ)

### Q: Cập nhật phiên bản mới có làm mất các cài đặt của tôi không?
**Không.** Toàn bộ cấu hình cá nhân (chế độ gõ, ngôn ngữ, phím tắt, giao diện sáng/tối) được lưu trữ độc lập tại thư mục `%LOCALAPPDATA%\CHKey\config.toml`. Quá trình nâng cấp chỉ thay thế file chương trình và giữ nguyên 100% dữ liệu của bạn.

### Q: Tôi bấm "Cập nhật" nhưng báo "Không thể kết nối server"?
Hãy kiểm tra lại kết nối Internet của bạn hoặc kiểm tra xem tường lửa (Firewall) / phần mềm diệt virus có đang chặn kết nối mạng của CHKey hay không.

### Q: Báo lỗi "SHA-256 không khớp"?
Điều này xảy ra khi đường truyền Internet bị chập chờn khiến tệp tải về không trọn vẹn. Hãy bấm nút **"Cập nhật"** một lần nữa để tải lại tệp mới.
