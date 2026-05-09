# 打包发布指南

## 方案一：GitHub Actions 自动打包（推荐）

已在 `.github/workflows/build.yml` 配置好 CI/CD，推送标签后自动构建 Windows 安装包。

### 操作步骤：

1. **将代码推送到 GitHub 仓库**（如果还没推）：
   ```bash
   git init
   git add .
   git commit -m "v0.1.0"
   git remote add origin https://github.com/你的用户名/hunjian.git
   git push -u origin main
   ```

2. **打标签触发构建**：
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **等待构建完成**：
   - 打开 GitHub 仓库 → Actions 页面
   - 等待 `Build and Release` 工作流完成（约 5-10 分钟）
   - 构建完成后，Release 页面会自动出现 `.msi` 和 `.exe` 安装包

4. **下载安装包**：
   - 打开 Release 页面
   - 下载 Windows 用户对应的 `.msi`（安装版）或 `.exe`（便携版）

### 产物说明：
- `.msi` — Windows 标准安装包，双击安装到系统
- `.exe`（NSIS）— 单文件安装程序，体积更小

### FFmpeg 已内置
Windows 安装包已自动捆绑 FFmpeg，用户**无需手动安装 FFmpeg**，开箱即用。

---

## 方案二：本地 Windows 电脑打包

如果有一台 Windows 电脑，可以直接本地构建：

### Windows 环境准备：
1. 安装 [Node.js 20+](https://nodejs.org/)
2. 安装 [Rust](https://rustup.rs/)
3. 安装 Tauri 依赖：
   ```powershell
   # 安装 WebView2（Windows 10/11 通常已自带）
   # 安装 Visual Studio Build Tools（含 MSVC）
   ```

### 构建命令：
```powershell
cd 项目目录
npm install
cargo tauri build
```

构建完成后，安装包位于：
```
src-tauri\target\release\bundle\msi\*.msi
src-tauri\target\release\bundle\nsis\*.exe
```

---

## 方案三：本地 macOS 打包（供 Mac 用户用）

当前 Mac 可直接打包 macOS 版本：

```bash
cd /Users/chengyisi/AI开发/项目/hunjian
npm install
cargo tauri build
```

产物位于：
```
src-tauri/target/release/bundle/dmg/*.dmg
```

### macOS 开发说明
macOS 本地开发**继续使用 Homebrew 安装的 FFmpeg**，不受任何影响。Windows 捆绑的 FFmpeg 仅在打包 Windows 安装包时由 CI 自动下载，不会污染本地环境。

---

## 图标配置

项目已配置图标生成。GitHub Actions 构建时会自动根据 `src-tauri/icons/icon.png` 生成所有平台所需图标格式。

如需更换图标，替换 `src-tauri/icons/icon.png`（推荐 1024x1024 PNG），然后重新打包即可。

---

## 跨平台构建限制说明

**为什么不能直接在 Mac 上打包 Windows 安装包？**

Tauri 打包 Windows `.msi` / `.exe` 需要 Windows 专用工具链：
- MSVC 编译器（Windows C++ 工具链）
- WiX Toolset（生成 MSI）
- NSIS（生成安装程序）

这些工具无法在 macOS 上运行，因此必须通过 GitHub Actions 的 Windows Runner 或本地 Windows 电脑来构建。

Rust 交叉编译可以生成 Windows 可执行文件，但无法生成安装包（MSI/NSIS）。
