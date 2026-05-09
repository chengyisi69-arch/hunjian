# 应用图标

Tauri 打包需要以下图标文件：
- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

生成方法：
1. 准备一张 1024x1024 的 PNG 图片
2. 安装 Tauri CLI: `cargo install tauri-cli`
3. 运行: `cargo tauri icon /path/to/your/icon.png`

或手动放置对应格式的图标文件到此目录。
