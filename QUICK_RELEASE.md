# 快速发布 Windows 版本

## 一键发布流程

### 第 1 步：确保代码在 Git 仓库中

```bash
cd /Users/chengyisi/AI开发/项目/hunjian
git add .
git commit -m "v0.2.0 - AI智能创作 + Windows打包"
```

### 第 2 步：推送到 GitHub

如果没有远程仓库：
```bash
git remote add origin https://github.com/你的用户名/hunjian.git
git push -u origin main
```

已有远程仓库：
```bash
git push origin main
```

### 第 3 步：打标签触发自动构建

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 第 4 步：下载安装包

1. 打开浏览器访问 `https://github.com/你的用户名/hunjian/releases`
2. 等待约 5-10 分钟（GitHub Actions 构建中）
3. 看到 `v0.2.0` 版本发布页面
4. 下载附件中的 `.msi` 或 `.exe` 文件
5. 发给 Windows 用户即可

---

## Windows 用户安装说明

1. 下载 `.msi` 文件
2. 双击安装（像安装普通软件一样）
3. 安装完成后桌面会出现"混剪生成器"图标
4. 双击打开即可使用，**无需额外安装 FFmpeg**

---

## 常见问题

**Q: 安装包有多大？**
A: 约 180-200MB（已包含 FFmpeg），Windows 用户无需额外下载任何东西。

**Q: macOS 本地开发受影响吗？**
A: 完全不受影响。你继续使用 `npm run tauri dev` 和 Homebrew 的 FFmpeg 进行开发。

**Q: 如何更新版本？**
A: 修改代码后，重复上述步骤，打新标签如 `v0.2.1`，GitHub Actions 会自动构建新版本。
