//! ffmpeg / ffprobe 薄封装。
//! 支持优先使用项目目录下捆绑的 ffmpeg（Windows 打包时用），
//! 回退到系统 PATH（本地开发时用）。

use crate::error::{AppError, AppResult};
use std::path::Path;
use std::process::Command;

/// 尝试找到捆绑的 ffmpeg 可执行文件路径。
/// 优先顺序：
/// 1. 与当前可执行文件同目录的 resources/ffmpeg(.exe)
/// 2. 当前工作目录的 resources/ffmpeg(.exe)
/// 3. 系统 PATH 中的 ffmpeg
fn resolve_ffmpeg_bin() -> String {
    let name = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };

    // 1. 与当前可执行文件同目录的 resources/
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let bundled = exe_dir.join("resources").join(name);
            if bundled.exists() {
                return bundled.to_string_lossy().to_string();
            }
        }
    }

    // 2. 当前工作目录的 resources/
    let cwd_bundled = std::path::Path::new("resources").join(name);
    if cwd_bundled.exists() {
        return cwd_bundled.to_string_lossy().to_string();
    }

    // 3. 回退到系统 PATH
    name.to_string()
}

fn resolve_ffprobe_bin() -> String {
    let name = if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" };

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let bundled = exe_dir.join("resources").join(name);
            if bundled.exists() {
                return bundled.to_string_lossy().to_string();
            }
        }
    }

    let cwd_bundled = std::path::Path::new("resources").join(name);
    if cwd_bundled.exists() {
        return cwd_bundled.to_string_lossy().to_string();
    }

    name.to_string()
}

/// 运行一条 ffmpeg 命令，stderr 失败时回传完整内容。
pub fn run_ffmpeg(args: &[&str]) -> AppResult<()> {
    let bin = resolve_ffmpeg_bin();
    log::debug!("ffmpeg {}", args.join(" "));
    let output = Command::new(&bin)
        .args(args)
        .output()
        .map_err(|e| AppError::Ffmpeg(format!("启动 ffmpeg 失败 ({}): {e}", bin)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Ffmpeg(format!(
            "ffmpeg 退出码 {}: {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

/// 用 ffprobe 取媒体时长（秒）。
pub fn probe_duration(path: &Path) -> AppResult<f64> {
    let bin = resolve_ffprobe_bin();
    let output = Command::new(&bin)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            to_str(path)?,
        ])
        .output()
        .map_err(|e| AppError::Ffmpeg(format!("启动 ffprobe 失败 ({}): {e}", bin)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Ffmpeg(stderr.into_owned()));
    }
    let s = String::from_utf8_lossy(&output.stdout);
    s.trim()
        .parse::<f64>()
        .map_err(|e| AppError::Ffmpeg(format!("无法解析时长 '{}': {e}", s.trim())))
}

pub fn to_str(p: &Path) -> AppResult<&str> {
    p.to_str()
        .ok_or_else(|| AppError::InvalidArgument(format!("非 UTF-8 路径: {}", p.display())))
}

/// 检测当前 ffmpeg 是否编译进了某个 filter（按名称）。
/// 用于在 drawtext 缺失时优雅降级。
pub fn has_filter(name: &str) -> bool {
    let bin = resolve_ffmpeg_bin();
    let output = match std::process::Command::new(&bin)
        .args(["-hide_banner", "-filters"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };
    let s = String::from_utf8_lossy(&output.stdout);
    // 滤镜行格式：" T. NAME    DESC"，按空格切并取第二个 token 比较
    for line in s.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 2 && cols[1] == name {
            return true;
        }
    }
    false
}

/// 平台默认 CJK 字体路径，drawtext 使用。
pub fn default_cjk_font() -> &'static str {
    if cfg!(target_os = "macos") {
        // macOS 自带 PingFang，FFmpeg+FreeType 通常能解析 .ttc 第一个 face
        "/System/Library/Fonts/PingFang.ttc"
    } else if cfg!(target_os = "windows") {
        "C:/Windows/Fonts/msyh.ttc"
    } else {
        // 常见 Linux 默认（用户没装时需自行 apt install fonts-noto-cjk）
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"
    }
}
