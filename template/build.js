#!/usr/bin/env node
/**
 * CHKey Blogger Theme Compiler (Node.js version)
 * ==============================================
 * Tự động ghép nối các module trong thư mục src/ (styles, components, scripts, assets)
 * thành file theme.xml hoàn chỉnh cho Google Blogger.
 *
 * Cách dùng:
 *     node build.js            # Build 1 lần
 *     node build.js --watch    # Tự động build lại khi file trong src/ thay đổi
 */

const fs = require('fs');
const path = require('path');

const BASE_DIR = __dirname;
const SRC_DIR = path.join(BASE_DIR, 'src');
const ASSETS_DIR = path.join(SRC_DIR, 'assets');
const OUTPUT_FILE = path.join(BASE_DIR, 'theme.xml');

const MIME_TYPES = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.svg': 'image/svg+xml',
  '.webp': 'image/webp',
  '.gif': 'image/gif',
  '.ico': 'image/x-icon',
  '.avif': 'image/avif',
};

function resolveAssetToBase64(srcVal) {
  const trimmed = (srcVal || '').trim();
  if (!trimmed || /^(https?:|\/\/|data:|blob:|mailto:|javascript:)/i.test(trimmed)) {
    return srcVal; // URL bên ngoài hoặc data URI -> Giữ nguyên
  }

  const cleanVal = trimmed.replace(/\\/g, '/');
  const filenameOnly = path.basename(cleanVal);
  const cleanRel = cleanVal.replace(/^(?:\.\/|\.\.\/|\/)?(?:src\/)?assets\//i, '');

  const possiblePaths = [];
  if (path.isAbsolute(trimmed) && fs.existsSync(trimmed)) {
    possiblePaths.push(trimmed);
  }
  possiblePaths.push(path.join(ASSETS_DIR, filenameOnly));
  possiblePaths.push(path.join(ASSETS_DIR, cleanRel));
  possiblePaths.push(path.normalize(path.join(SRC_DIR, trimmed.replace(/^[/\\]+/, ''))));
  possiblePaths.push(path.normalize(path.join(BASE_DIR, trimmed.replace(/^[/\\]+/, ''))));

  for (const p of possiblePaths) {
    if (fs.existsSync(p) && fs.statSync(p).isFile()) {
      const ext = path.extname(p).toLowerCase();
      const mime = MIME_TYPES[ext] || 'image/png';
      const b64 = fs.readFileSync(p).toString('base64');
      return `data:${mime};base64,${b64}`;
    }
  }

  return srcVal;
}

function transformImages(htmlContent) {
  const replaceAttrs = (tagStr) => {
    return tagStr.replace(/(src|href|xlink:href)=(["'])(.*?)\2/gi, (match, attrName, quote, urlVal) => {
      const converted = resolveAssetToBase64(urlVal);
      return `${attrName}=${quote}${converted}${quote}`;
    });
  };

  htmlContent = htmlContent.replace(/<img\b[^>]*?>/gi, replaceAttrs);
  htmlContent = htmlContent.replace(/<image\b[^>]*?>/gi, replaceAttrs);
  htmlContent = htmlContent.replace(/<link\b[^>]*?rel=["'](?:icon|shortcut icon|apple-touch-icon)["'][^>]*?>/gi, replaceAttrs);
  return htmlContent;
}

function getLogoDataUri() {
  return resolveAssetToBase64('chkey.png');
}

function loadStyles() {
  const stylesDir = path.join(SRC_DIR, 'styles');
  if (!fs.existsSync(stylesDir)) return '';

  const files = fs.readdirSync(stylesDir).filter(f => f.endsWith('.css')).sort();
  return files.map(f => {
    const content = fs.readFileSync(path.join(stylesDir, f), 'utf-8').trim();
    return `/* === Module: ${f} === */\n${content}`;
  }).join('\n\n');
}

function loadScripts() {
  const scriptsDir = path.join(SRC_DIR, 'scripts');
  if (!fs.existsSync(scriptsDir)) return '';

  const files = fs.readdirSync(scriptsDir).filter(f => f.endsWith('.js')).sort();
  return files.map(f => {
    const content = fs.readFileSync(path.join(scriptsDir, f), 'utf-8').trim();
    return `// === Module: ${f} ===\n${content}`;
  }).join('\n\n');
}

function loadComponent(name) {
  const compPath = path.join(SRC_DIR, 'components', name);
  if (fs.existsSync(compPath)) {
    return fs.readFileSync(compPath, 'utf-8').trim();
  }
  return `<!-- Component ${name} not found -->`;
}

function build() {
  console.log('⏳ Đang build theme.xml từ thư mục src/...');
  const startTime = Date.now();

  const templatePath = path.join(SRC_DIR, 'template.xml');
  if (!fs.existsSync(templatePath)) {
    console.error(`❌ Lỗi: Không tìm thấy file khung ${templatePath}`);
    return false;
  }

  let output = fs.readFileSync(templatePath, 'utf-8');
  const logoUri = getLogoDataUri();
  const styles = loadStyles();
  const scripts = loadScripts();

  const components = {
    '{{HEADER}}': loadComponent('header.html'),
    '{{HERO}}': loadComponent('hero.html'),
    '{{SIMULATOR}}': loadComponent('simulator.html'),
    '{{CHEATSHEET}}': loadComponent('cheatsheet.html'),
    '{{FEATURES}}': loadComponent('features.html'),
    '{{GUIDE}}': loadComponent('guide.html'),
    '{{DOWNLOAD}}': loadComponent('download.html'),
    '{{BLOG}}': loadComponent('blog.html'),
    '{{FOOTER}}': loadComponent('footer.html'),
    '{{TOAST}}': loadComponent('toast.html'),
  };

  // 1. Thay thế CSS & JS
  output = output.replace('{{STYLES}}', styles);
  output = output.replace('{{SCRIPTS}}', scripts);

  // 2. Thay thế Components
  for (const [placeholder, code] of Object.entries(components)) {
    output = output.split(placeholder).join(code);
  }

  // 3. Thay thế Logo Data URI (tương thích ngược)
  output = output.split('{{LOGO_DATA_URI}}').join(logoUri);

  // 4. Tự động chuyển đổi các ảnh từ src/assets thành Base64 trong các thẻ <img> và <image>
  // (Ảnh từ URL bên ngoài http/https được giữ nguyên)
  output = transformImages(output);

  // 5. Ghi file theme.xml
  fs.writeFileSync(OUTPUT_FILE, output, 'utf-8');

  const elapsed = Date.now() - startTime;
  const sizeKb = (fs.statSync(OUTPUT_FILE).size / 1024).toFixed(1);
  console.log(`🎉 Build hoàn tất trong ${elapsed}ms! File: ${OUTPUT_FILE} (${sizeKb} KB)\n`);
  return true;
}

function watch() {
  console.log('👀 Đang chạy ở chế độ Watch Mode (Node.js). Nhấn Ctrl+C để dừng...');
  build();

  fs.watch(SRC_DIR, { recursive: true }, (eventType, filename) => {
    if (filename) {
      console.log(`🔄 Phát hiện thay đổi trong ${filename}...`);
      build();
    }
  });
}

const args = process.argv.slice(2);
if (args.includes('--watch') || args.includes('-w')) {
  watch();
} else {
  const success = build();
  if (!success) process.exit(1);
}
