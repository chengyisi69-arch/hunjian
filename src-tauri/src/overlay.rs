//! 用 ffmpeg drawtext / subtitles 滤镜在视频上叠加文字：
//!  - 最多 3 个标题（drawtext）
//!  - 静态字幕或自动语音字幕（ASS 时间轴）
//!
//! 文案通过 textfile 参数传入，避免 CJK / 引号 / 冒号 转义麻烦。
//!
//! 位置格式：
//!   - 旧别名：top-left / top-center / top-right / bottom-center
//!   - 新坐标："x,y" 百分比，如 "50,50" 表示画面中央
//!
//! 标题样式：
//!   simple      — 白字 + 黑半透明底（默认）
//!   outline     — 白字 + 黑描边
//!   shadow      — 白字 + 投影阴影
//!   neon        — 黄字 + 红描边 + 阴影
//!   gradient    — 白字 + 蓝紫描边
//!   vintage     — 米黄字 + 褐色描边（复古）
//!   tech        — 青色字 + 蓝绿描边（科技）
//!   comic       — 白字 + 粗黑描边（漫画）
//!   golden      — 金色字 + 深棕描边
//!   clean       — 白字无边框无背景（极简）

use crate::error::AppResult;
use crate::ffmpeg::{default_cjk_font, has_filter, run_ffmpeg, to_str};
use std::path::{Path, PathBuf};

/// 解析位置字符串为 drawtext 的 (x, y) 表达式。
pub fn position_expr(pos: &str) -> (String, String) {
    if pos.contains(',') {
        let parts: Vec<&str> = pos.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(x), Ok(y)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                let x_expr = format!("(w-text_w)*{x}/100");
                let y_expr = format!("(h-text_h)*{y}/100");
                return (x_expr, y_expr);
            }
        }
    }
    let (x, y) = match pos {
        "top-left" => ("40", "40"),
        "top-center" => ("(w-text_w)/2", "40"),
        "top-right" => ("w-text_w-40", "40"),
        "bottom-center" => ("(w-text_w)/2", "h-text_h-40"),
        _ => ("40", "40"),
    };
    (x.to_string(), y.to_string())
}

fn style_params(style: &str) -> String {
    match style {
        "outline" => "borderw=3:bordercolor=black".to_string(),
        "shadow" => "shadowcolor=black@0.6:shadowx=4:shadowy=4".to_string(),
        "neon" => "borderw=4:bordercolor=red:shadowcolor=red@0.4:shadowx=2:shadowy=2".to_string(),
        "gradient" => "borderw=3:bordercolor=#8B5CF6".to_string(),
        "vintage" => "borderw=3:bordercolor=#8B4513".to_string(),
        "tech" => "borderw=3:bordercolor=#00CED1".to_string(),
        "comic" => "borderw=5:bordercolor=black".to_string(),
        "golden" => "borderw=3:bordercolor=#B8860B".to_string(),
        "clean" => "".to_string(),
        _ => "box=1:boxcolor=black@0.5:boxborderw=12".to_string(),
    }
}

fn style_color(style: &str) -> &'static str {
    match style {
        "neon" => "yellow",
        "vintage" => "#F5F5DC",
        "tech" => "#00FFFF",
        "golden" => "#FFD700",
        _ => "white",
    }
}

/// 单条标题配置（辅助结构）。
pub struct TitleCfg<'a> {
    pub text: &'a str,
    pub pos: &'a str,
    pub size: u32,
    pub style: &'a str,
}

/// 给视频叠加最多 3 个标题 + 可选 ASS 字幕。
/// 所有文本为空且不需要 ASS 时直接复制。
pub fn overlay_titles_and_subtitle(
    video_in: &Path,
    titles: &[TitleCfg<'_>],
    subtitle_text: &str,
    subtitle_pos: &str,
    text_dir: &Path,
    output: &Path,
    font_override: Option<&str>,
    auto_subtitle: bool,
    video_duration_secs: f64,
) -> AppResult<()> {
    let has_text = titles.iter().any(|t| !t.text.trim().is_empty())
        || !subtitle_text.trim().is_empty()
        || auto_subtitle;

    if !has_text {
        return run_ffmpeg(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            to_str(video_in)?,
            "-c",
            "copy",
            to_str(output)?,
        ]);
    }

    if !has_filter("drawtext") {
        log::warn!("ffmpeg 未编译 drawtext 滤镜，跳过文字层");
        return run_ffmpeg(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            to_str(video_in)?,
            "-c",
            "copy",
            to_str(output)?,
        ]);
    }

    std::fs::create_dir_all(text_dir)?;
    let font: &str = match font_override {
        Some(f) => f,
        None => default_cjk_font(),
    };

    let mut filters: Vec<String> = Vec::new();

    // 标题（最多 3 个）
    for (idx, t) in titles.iter().enumerate() {
        let txt = t.text.trim();
        if txt.is_empty() {
            continue;
        }
        let p = text_dir.join(format!("title{idx}.txt"));
        std::fs::write(&p, txt)?;
        let (x, y) = position_expr(t.pos);
        let color = style_color(t.style);
        let extra = style_params(t.style);
        filters.push(drawtext_filter(font, &p, &x, &y, t.size, color, &extra));
    }

    // 字幕层：优先 ASS 自动字幕，否则静态 drawtext
    if auto_subtitle && !subtitle_text.trim().is_empty() {
        let ass_path = text_dir.join("karaoke.ass");
        generate_karaoke_ass(subtitle_text, video_duration_secs, &ass_path)?;
        // subtitles 滤镜需要在最外层，且 drawtext 在其前
        let joined = filters.join(",");
        let vf = if joined.is_empty() {
            format!("subtitles='{}'", ffescape(&ass_path.to_string_lossy()))
        } else {
            format!("{joined},subtitles='{}'", ffescape(&ass_path.to_string_lossy()))
        };
        return run_ffmpeg(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            to_str(video_in)?,
            "-vf",
            &vf,
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "copy",
            to_str(output)?,
        ]);
    }

    if !subtitle_text.trim().is_empty() {
        let p = text_dir.join("subtitle.txt");
        std::fs::write(&p, subtitle_text)?;
        let (x, y) = position_expr(subtitle_pos);
        let extra = style_params("simple");
        filters.push(drawtext_filter(font, &p, &x, &y, 32, "white", &extra));
    }

    let vf = filters.join(",");
    run_ffmpeg(&[
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        to_str(video_in)?,
        "-vf",
        &vf,
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "23",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "copy",
        to_str(output)?,
    ])
}

fn drawtext_filter(
    font: &str,
    textfile: &PathBuf,
    x: &str,
    y: &str,
    size: u32,
    color: &str,
    extra: &str,
) -> String {
    let font_e = ffescape(font);
    let textfile_e = ffescape(&textfile.to_string_lossy());
    if extra.is_empty() {
        format!(
            "drawtext=fontfile='{font_e}':textfile='{textfile_e}':\
             x={x}:y={y}:fontsize={size}:fontcolor={color}"
        )
    } else {
        format!(
            "drawtext=fontfile='{font_e}':textfile='{textfile_e}':\
             x={x}:y={y}:fontsize={size}:fontcolor={color}:{extra}"
        )
    }
}

/// 生成 Karaoke 风格 ASS 字幕：按句子切分，等长分配时间。
fn generate_karaoke_ass(text: &str, duration_secs: f64, out: &Path) -> AppResult<()> {
    let sentences: Vec<&str> = text
        .split(|c| c == '。' || c == '！' || c == '？' || c == '.' || c == '!' || c == '?')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if sentences.is_empty() {
        // 整段作为一句
        return write_ass(&[text], duration_secs, out);
    }
    write_ass(&sentences, duration_secs, out)
}

fn write_ass(sentences: &[&str], duration_secs: f64, out: &Path) -> AppResult<()> {
    let n = sentences.len().max(1);
    let seg = duration_secs / n as f64;

    let mut body = String::new();
    body.push_str("[Script Info]\nTitle:AutoSubtitle\nScriptType:v4.00+\nCollisions:Normal\nPlayResX:1920\nPlayResY:1080\nTimer:100.0000\n\n");
    body.push_str("[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    body.push_str("Style: Default,PingFang SC,48,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,40,40,40,1\n\n");
    body.push_str("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for (i, sent) in sentences.iter().enumerate() {
        let start = seg * i as f64;
        let end = seg * (i + 1) as f64;
        body.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            fmt_time(start),
            fmt_time(end),
            sent.replace(',', "，")
        ));
    }

    std::fs::write(out, body)?;
    Ok(())
}

fn fmt_time(s: f64) -> String {
    let h = (s / 3600.0) as u32;
    let m = ((s % 3600.0) / 60.0) as u32;
    let sec = s % 60.0;
    format!("{:01}:{:02}:{:05.2}", h, m, sec)
}

fn ffescape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}
