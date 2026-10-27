#!/usr/bin/env python3
"""
CHKey Blogger Theme Compiler & Builder
======================================
Tự động ghép nối các module trong thư mục src/ (styles, components, scripts, assets)
thành file theme.xml hoàn chỉnh, chuẩn cú pháp XML cho nền tảng Google Blogger.

Cách dùng:
    python build.py            # Build 1 lần
    python build.py --watch    # Theo dõi thư mục src/ và tự động build lại khi có thay đổi
"""

import os
import sys
import time
import base64
import xml.etree.ElementTree as ET

import re

# Thiết lập encoding UTF-8 cho console
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
SRC_DIR = os.path.join(BASE_DIR, "src")
ASSETS_DIR = os.path.join(SRC_DIR, "assets")
OUTPUT_FILE = os.path.join(BASE_DIR, "theme.xml")

MIME_TYPES = {
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".svg": "image/svg+xml",
    ".webp": "image/webp",
    ".gif": "image/gif",
    ".ico": "image/x-icon",
    ".avif": "image/avif",
}

def resolve_asset_to_base64(src_val):
    """
    Nếu src_val trỏ tới file ảnh trong src/assets, chuyển thành Base64 Data URI.
    Nếu src_val là URL bên ngoài (http://, https://, //, data:), giữ nguyên.
    """
    trimmed = src_val.strip()
    if not trimmed or trimmed.startswith(("http://", "https://", "//", "data:", "blob:", "mailto:", "javascript:")):
        return src_val  # URL bên ngoài hoặc data URI -> Giữ nguyên
    
    # Kiểm tra các dạng đường dẫn khả dĩ tới thư mục assets
    clean_val = trimmed.replace("\\", "/")
    filename_only = os.path.basename(clean_val)
    clean_rel = re.sub(r"^(?:\.\/|\.\.\/|\/)?(?:src\/)?assets\/", "", clean_val, flags=re.IGNORECASE)

    possible_paths = []
    # 1. Đường dẫn tuyệt đối
    if os.path.isabs(trimmed) and os.path.exists(trimmed):
        possible_paths.append(trimmed)
    
    # 2. Tìm theo tên file trong src/assets/
    possible_paths.append(os.path.join(ASSETS_DIR, filename_only))
    # 3. Tìm theo đường dẫn tương đối trong assets/
    possible_paths.append(os.path.join(ASSETS_DIR, clean_rel))
    # 4. Tìm từ SRC_DIR hoặc BASE_DIR
    possible_paths.append(os.path.normpath(os.path.join(SRC_DIR, trimmed.lstrip("/\\"))))
    possible_paths.append(os.path.normpath(os.path.join(BASE_DIR, trimmed.lstrip("/\\"))))

    target_file = None
    for p in possible_paths:
        if os.path.isfile(p):
            target_file = p
            break
            
    if not target_file:
        return src_val  # Không tìm thấy file trong assets, giữ nguyên

    ext = os.path.splitext(target_file)[1].lower()
    mime = MIME_TYPES.get(ext, "image/png")
    
    try:
        with open(target_file, "rb") as f:
            b64_data = base64.b64encode(f.read()).decode("ascii")
            return f"data:{mime};base64,{b64_data}"
    except Exception as e:
        print(f"⚠️ Cảnh báo: Không thể đọc file {target_file}: {e}")
        return src_val

def transform_images(html_content):
    """
    Tự động quét qua tất cả các thẻ <img>, <image> (SVG) và <link rel='icon'>
    chuyển các ảnh lấy từ src/assets thành Base64, giữ nguyên ảnh URL bên ngoài.
    """
    def replace_attrs(tag_match):
        tag_str = tag_match.group(0)
        def attr_repl(m):
            attr_name = m.group(1)
            quote = m.group(2)
            url_val = m.group(3)
            converted = resolve_asset_to_base64(url_val)
            return f"{attr_name}={quote}{converted}{quote}"
        return re.sub(r"(src|href|xlink:href)=([\"'])(.*?)\2", attr_repl, tag_str)

    # 1. Thẻ <img>
    html_content = re.sub(r"<img\b[^>]*?>", replace_attrs, html_content, flags=re.IGNORECASE)
    # 2. Thẻ SVG <image>
    html_content = re.sub(r"<image\b[^>]*?>", replace_attrs, html_content, flags=re.IGNORECASE)
    # 3. Thẻ <link rel='icon'> hoặc favicon
    html_content = re.sub(r"<link\b[^>]*?rel=[\"'](?:icon|shortcut icon|apple-touch-icon)[\"'][^>]*?>", replace_attrs, html_content, flags=re.IGNORECASE)
    return html_content

def get_logo_data_uri():
    """Đọc file ảnh chkey.png và chuyển thành Base64 Data URI (Hỗ trợ tương thích ngược)."""
    return resolve_asset_to_base64("chkey.png")

def load_styles():
    """Ghép toàn bộ các file CSS trong thư mục src/styles theo thứ tự tên file."""
    styles_dir = os.path.join(SRC_DIR, "styles")
    if not os.path.exists(styles_dir):
        return ""
    
    css_files = sorted([f for f in os.listdir(styles_dir) if f.endswith(".css")])
    css_chunks = []
    for f in css_files:
        fpath = os.path.join(styles_dir, f)
        with open(fpath, "r", encoding="utf-8") as file:
            content = file.read().strip()
            if content:
                css_chunks.append(f"/* === Module: {f} === */\n{content}")
    
    return "\n\n".join(css_chunks)

def load_scripts():
    """Ghép toàn bộ các file JS trong thư mục src/scripts theo thứ tự tên file."""
    scripts_dir = os.path.join(SRC_DIR, "scripts")
    if not os.path.exists(scripts_dir):
        return ""
    
    js_files = sorted([f for f in os.listdir(scripts_dir) if f.endswith(".js")])
    js_chunks = []
    for f in js_files:
        fpath = os.path.join(scripts_dir, f)
        with open(fpath, "r", encoding="utf-8") as file:
            content = file.read().strip()
            if content:
                js_chunks.append(f"// === Module: {f} ===\n{content}")
    
    return "\n\n".join(js_chunks)

def load_component(name):
    """Đọc nội dung một component trong thư mục src/components/."""
    comp_path = os.path.join(SRC_DIR, "components", name)
    if os.path.exists(comp_path):
        with open(comp_path, "r", encoding="utf-8") as f:
            return f.read().strip()
    return f"<!-- Component {name} not found -->"

def build():
    """Thực hiện build theme.xml từ src/ và xác thực XML."""
    print("⏳ Đang build theme.xml từ thư mục src/...")
    start_time = time.time()
    
    template_path = os.path.join(SRC_DIR, "template.xml")
    if not os.path.exists(template_path):
        print(f"❌ Lỗi: Không tìm thấy file khung {template_path}")
        return False
    
    with open(template_path, "r", encoding="utf-8") as f:
        template = f.read()
    
    logo_uri = get_logo_data_uri()
    styles = load_styles()
    scripts = load_scripts()
    
    components = {
        "{{HEADER}}": load_component("header.html"),
        "{{HERO}}": load_component("hero.html"),
        "{{SIMULATOR}}": load_component("simulator.html"),
        "{{CHEATSHEET}}": load_component("cheatsheet.html"),
        "{{FEATURES}}": load_component("features.html"),
        "{{GUIDE}}": load_component("guide.html"),
        "{{DOWNLOAD}}": load_component("download.html"),
        "{{BLOG}}": load_component("blog.html"),
        "{{FOOTER}}": load_component("footer.html"),
        "{{TOAST}}": load_component("toast.html"),
    }
    
    # 1. Thay thế CSS & JS
    output = template.replace("{{STYLES}}", styles)
    output = output.replace("{{SCRIPTS}}", scripts)
    
    # 2. Thay thế Components
    for placeholder, comp_code in components.items():
        output = output.replace(placeholder, comp_code)
    
    # 3. Thay thế Logo Data URI (tương thích ngược)
    output = output.replace("{{LOGO_DATA_URI}}", logo_uri)
    
    # 4. Tự động chuyển đổi các ảnh từ src/assets thành Base64 trong các thẻ <img> và <image>
    # (Ảnh từ URL bên ngoài http/https được giữ nguyên)
    output = transform_images(output)
    
    # 5. Kiểm tra tính hợp lệ của XML với ElementTree
    try:
        ET.fromstring(output)
        print("✅ Kiểm tra XML thành công: Chuẩn cú pháp Blogger XML 100%!")
    except ET.ParseError as e:
        print(f"❌ Lỗi cú pháp XML trong bản build: {e}")
        line_num, col = e.position
        lines = output.splitlines()
        start_l = max(0, line_num - 3)
        end_l = min(len(lines), line_num + 3)
        for idx in range(start_l, end_l):
            prefix = ">> " if idx == line_num - 1 else "   "
            print(f"{prefix}{idx+1}: {lines[idx]}")
        return False
    except Exception as e:
        print(f"❌ Lỗi kiểm tra XML: {e}")
        return False
    
    # 5. Ghi file theme.xml
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.write(output)
    
    elapsed = (time.time() - start_time) * 1000
    file_size_kb = os.path.getsize(OUTPUT_FILE) / 1024
    print(f"🎉 Build hoàn tất trong {elapsed:.1f}ms! File: {OUTPUT_FILE} ({file_size_kb:.1f} KB)\n")
    return True

def watch():
    """Theo dõi các thay đổi trong src/ và build lại tự động."""
    print("👀 Đang chạy ở chế độ Watch Mode. Nhấn Ctrl+C để dừng...")
    build()
    
    last_mtime = 0
    def get_max_mtime():
        max_t = 0
        for root, _, files in os.walk(SRC_DIR):
            for file in files:
                fpath = os.path.join(root, file)
                try:
                    t = os.path.getmtime(fpath)
                    if t > max_t:
                        max_t = t
                except OSError:
                    pass
        return max_t
    
    last_mtime = get_max_mtime()
    try:
        while True:
            time.sleep(1)
            cur_mtime = get_max_mtime()
            if cur_mtime > last_mtime:
                last_mtime = cur_mtime
                print("🔄 Phát hiện thay đổi trong src/...")
                build()
    except KeyboardInterrupt:
        print("\n👋 Đã dừng chế độ Watch.")

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] in ("--watch", "-w"):
        watch()
    else:
        success = build()
        if not success:
            sys.exit(1)
