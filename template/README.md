# CHKey Blogger Theme - Môi Trường Phát Triển & Compiler

Môi trường phát triển modular dành cho giao diện **CHKey Blogger Theme**. Toàn bộ mã nguồn giao diện (CSS, HTML Components, JavaScript, Assets) được phân chia rõ ràng theo từng module nhỏ trong thư mục `src/`, sau đó được tự động biên dịch thành file `theme.xml` chuẩn cú pháp XML để nạp trực tiếp vào Google Blogger.

---

## 📁 Cấu Trúc Thư Mục

```text
chkey-template/
├── src/
│   ├── assets/                 # Hình ảnh, logo tĩnh (chkey.png)
│   │   └── chkey.png           # Tự động mã hóa thành Base64 Data URI khi build
│   │
│   ├── components/             # Các khối giao diện HTML/Blogger
│   │   ├── header.html         # Thanh điều hướng <header> & Logo
│   │   ├── hero.html           # Khối giới thiệu mở đầu <section id="hero">
│   │   ├── simulator.html      # Bộ giả lập gõ tiếng Tây Nguyên <section id="simulator">
│   │   ├── cheatsheet.html     # Bảng tra cứu phím gõ Telex <section id="cheatsheet">
│   │   ├── features.html       # 6 thẻ ưu thế vượt trội (Win & Mac) <section id="features">
│   │   ├── guide.html          # 3 bước bắt đầu gõ (Tabs Windows & macOS) <section id="guide">
│   │   ├── download.html       # Khu vực tải về & kiểm tra mã băm SHA <section id="download">
│   │   ├── blog.html           # Khối bài viết Blogger (<b:section id='main'>, <b:widget id='Blog1'>)
│   │   ├── footer.html         # Chân trang <footer> & Bản quyền
│   │   └── toast.html          # Hộp thông báo popup Toast
│   │
│   ├── styles/                 # Toàn bộ CSS (tách module theo số thứ tự nạp)
│   │   ├── 00_variables.css    # Biến màu, phông chữ, glassmorphism, radius
│   │   ├── 01_base.css         # Reset CSS, body, container, badges
│   │   ├── 02_header.css       # Header, navigation bar, logo
│   │   ├── 03_hero.css         # Hero layout, typographic heading, CTA buttons
│   │   ├── 04_simulator.css    # Giao diện khung gõ, bàn phím ảo, sample chips
│   │   ├── 05_cheatsheet.css   # Bảng tra cứu phím tắt Telex
│   │   ├── 06_features.css     # Khối 6 ưu điểm vượt trội
│   │   ├── 07_guide.css        # Tab chuyển đổi hệ điều hành & 3 bước cài đặt
│   │   ├── 08_download.css     # Thẻ tải file, nút download, khung verify SHA-256
│   │   ├── 09_footer.css       # Chân trang & liên kết điều hướng
│   │   ├── 10_responsive.css   # Tối ưu hiển thị Tablet và Mobile (<=640px, <=992px)
│   │   └── 11_blog.css         # Định dạng bài viết Blogger, bình luận, trang chi tiết
│   │
│   ├── scripts/                # JavaScript logic
│   │   ├── 01_utils.js         # Tiện ích: Toast notification, sao chép SHA-256
│   │   ├── 02_simulator.js     # Lõi xử lý Telex, ánh xạ phím gõ tiếng Tây Nguyên
│   │   ├── 03_guide.js         # Hàm chuyển đổi tab Windows / macOS (switchGuide)
│   │   └── 04_detect_os.js     # Tự động nhận diện thiết bị người dùng (detectOS)
│   │
│   └── template.xml            # Khung sườn Blogger gốc (Schema XML, <head>, placeholders)
│
├── build.py                    # Script Python tự động biên dịch & kiểm tra XML
├── build.js                    # Script Node.js tự động biên dịch tương đương
├── theme.xml                   # File thành phẩm hoàn chỉnh để tải lên Blogger
└── README.md                   # Tài liệu hướng dẫn
```

---

## 🚀 Hướng Dẫn Sử Dụng

Bạn có thể sử dụng **Python** hoặc **Node.js** tùy ý, cả hai đều cho ra file `theme.xml` chuẩn cú pháp 100%.

### 1. Sử dụng Python:

- **Biên dịch 1 lần:**
  ```powershell
  python build.py
  ```
  *Script sẽ tự động xác thực XML bằng `xml.etree.ElementTree`. Nếu có lỗi cú pháp, script sẽ báo chính xác dòng và cột bị lỗi.*

- **Chế độ theo dõi tự động (Watch Mode):**
  ```powershell
  python build.py --watch
  ```
  *Mỗi khi bạn lưu thay đổi trong bất kỳ file nào thuộc `src/`, `theme.xml` sẽ được build lại tức thì.*

---

### 2. Sử dụng Node.js:

- **Biên dịch 1 lần:**
  ```powershell
  node build.js
  ```

- **Chế độ theo dõi tự động (Watch Mode):**
  ```powershell
  node build.js --watch
  ```

---

## 💡 Ưu Điểm Của Môi Trường Này

1. **Không phải cuộn trong file 3.000 dòng:** Muốn sửa header vào `src/components/header.html`, sửa tab guide vào `src/components/guide.html`, chỉnh CSS vào `src/styles/...`.
2. **Quản lý Logo thông minh (`{{LOGO_DATA_URI}}`):** Bạn chỉ cần đặt file ảnh mới vào `src/assets/chkey.png`, build script sẽ tự động sinh mã Base64 và cập nhật đồng bộ cho Favicon, Header và Footer.
3. **An toàn cú pháp Blogger XML:** Không sợ quên đóng thẻ XML hay sót ký tự `&amp;`, trình compiler luôn tự động xác thực tính hợp lệ của cây XML trước khi xuất file.
