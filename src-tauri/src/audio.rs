//! 三轨音频混合：原片段声 + 配音 + BGM。各轨音量由前端参数指定。
//! BGM 支持按视频时长自动裁剪，并加 afade 淡入淡出。

use crate::error::AppResult;
use crate::ffmpeg::{run_ffmpeg, to_str};
use std::path::Path;

/// 默认音量（用于没有外部参数时的回退）。
pub const DEFAULT_NARRATION_VOLUME: f32 = 1.0;
pub const DEFAULT_BGM_VOLUME: f32 = 0.3;
pub const DEFAULT_ORIGINAL_VOLUME: f32 = 0.08;

/// 混合视频原声 + 配音（可选） + 背景音乐（可选），输出最终带音轨的视频。
/// 输出长度对齐到 video_in（-shortest）。
/// bgm_fade_secs: BGM 淡入淡出秒数。
pub fn mix_audio(
    video_in: &Path,
    narration: Option<&Path>,
    bg_music: Option<&Path>,
    output: &Path,
    original_volume: f32,
    narration_volume: f32,
    bgm_volume: f32,
    bgm_fade_secs: f32,
) -> AppResult<()> {
    // 没有 narration 和 bg_music 时，原片段声直接保留
    if narration.is_none() && bg_music.is_none() {
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

    // 输入按顺序：[0]=视频本体，[1]=narration（如有），[2]=bgm（如有）
    let mut input_args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
        "-i".into(),
        to_str(video_in)?.into(),
    ];
    let mut idx = 1usize;
    let nar_idx = if let Some(p) = narration {
        input_args.push("-i".into());
        input_args.push(to_str(p)?.into());
        let cur = idx;
        idx += 1;
        Some(cur)
    } else {
        None
    };
    let bgm_idx = if let Some(p) = bg_music {
        input_args.push("-i".into());
        input_args.push(to_str(p)?.into());
        Some(idx)
    } else {
        None
    };
    let _ = idx;

    // 构造 filter_complex
    let mut filter = String::new();
    filter.push_str(&format!(
        "[0:a]aresample=48000,volume={original_volume}[ao];"
    ));
    let mut amix_inputs: Vec<&str> = vec!["[ao]"];
    let nar_label;
    if let Some(i) = nar_idx {
        nar_label = format!("[an]");
        filter.push_str(&format!(
            "[{i}:a]aresample=48000,volume={narration_volume}{nar_label};"
        ));
        amix_inputs.push("[an]");
    }
    let bgm_label;
    if let Some(i) = bgm_idx {
        bgm_label = format!("[ab]");
        // BGM 加淡入淡出 + 音量
        let fade_in = bgm_fade_secs.min(2.0);
        let fade_out = bgm_fade_secs.min(2.0);
        filter.push_str(&format!(
            "[{i}:a]aresample=48000,afade=t=in:st=0:d={fade_in},afade=t=out:st=0:d={fade_out}:curve=tri,volume={bgm_volume}{bgm_label};"
        ));
        amix_inputs.push("[ab]");
    }
    let n = amix_inputs.len();
    filter.push_str(&format!(
        "{joined}amix=inputs={n}:duration=longest:dropout_transition=0:normalize=0[aout]",
        joined = amix_inputs.join("")
    ));

    let mut args = input_args;
    args.push("-filter_complex".into());
    args.push(filter);
    args.push("-map".into());
    args.push("0:v".into());
    args.push("-map".into());
    args.push("[aout]".into());
    args.push("-c:v".into());
    args.push("copy".into());
    args.push("-c:a".into());
    args.push("aac".into());
    args.push("-ar".into());
    args.push("48000".into());
    args.push("-b:a".into());
    args.push("160k".into());
    args.push("-shortest".into());
    args.push(to_str(output)?.into());

    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    run_ffmpeg(&argv)
}
