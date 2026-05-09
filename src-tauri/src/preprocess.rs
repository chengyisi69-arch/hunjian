//! 预处理：把每个原始素材切成等长片段，统一编码为 1080x1920(竖屏) 或 1920x1080(横屏) @30fps H.264 + AAC。
//!
//! 优化点：
//! - 竖屏时用智能裁剪（crop 中心区域）替代 pad 黑边，更适合短视频平台。
//! - 音频用 loudnorm 统一响度（EBU R128）。
//! - 视频加 eq 轻微增强对比度/饱和度，让短视频更鲜明。
//! - 使用 rayon 并行切割多个原始素材。

use crate::error::{AppError, AppResult};
use crate::ffmpeg::{run_ffmpeg, to_str};
use crate::types::{Clip, Project};
use rayon::prelude::*;
use std::path::Path;

/// 给定项目，将所有 raw_assets 切成 clip_duration_secs 长度的片段。
/// 切片文件落在 project.work_dir/clips/ 下。
pub fn preprocess_all_assets(project: &mut Project) -> AppResult<()> {
    if project.raw_assets.is_empty() {
        return Err(AppError::InvalidArgument("没有任何原始素材".into()));
    }
    if project.clip_duration_secs <= 0.0 {
        return Err(AppError::InvalidArgument("片段长度必须 > 0".into()));
    }

    // 确保 work_dir 存在
    if project.work_dir.is_none() {
        let dir = std::env::temp_dir().join(format!("hunjian-{}", rand::random::<u32>()));
        std::fs::create_dir_all(&dir)?;
        project.work_dir = Some(dir);
    }
    let work = project.work_dir.as_ref().unwrap();
    let clips_dir = work.join("clips");
    std::fs::create_dir_all(&clips_dir)?;

    let (w, h) = project.output_resolution;
    let fps = project.output_fps;
    let clip_len = project.clip_duration_secs;
    let orientation = project.orientation.clone();

    // 每个原始素材 → N 个切片任务（asset_idx, segment_idx）
    let mut tasks: Vec<(usize, usize)> = Vec::new();
    for (ai, asset) in project.raw_assets.iter().enumerate() {
        let n = (asset.duration_secs / clip_len).floor() as usize;
        for si in 0..n {
            tasks.push((ai, si));
        }
    }
    if tasks.is_empty() {
        return Err(AppError::InvalidArgument(
            "素材时长不足以切出任何片段".into(),
        ));
    }

    log::info!(
        "preprocess: {} 个原始素材 → {} 个切片，目标 {}x{}@{}fps，方向={}",
        project.raw_assets.len(),
        tasks.len(),
        w,
        h,
        fps,
        orientation
    );

    // 并行切割
    let assets = project.raw_assets.clone();
    let results: Vec<AppResult<Clip>> = tasks
        .par_iter()
        .map(|&(ai, si)| {
            let asset = &assets[ai];
            let start = si as f64 * clip_len;
            let out = clips_dir.join(format!("a{ai:03}_s{si:03}.mp4"));
            cut_and_normalize(
                &asset.path,
                start,
                clip_len,
                w,
                h,
                fps,
                &orientation,
                &out,
            )?;
            Ok(Clip {
                path: out,
                duration_secs: clip_len,
                scene_id: asset.scene_id.clone(),
                used: false,
            })
        })
        .collect();

    let mut clips = Vec::with_capacity(results.len());
    for r in results {
        clips.push(r?);
    }
    project.clips = clips;
    Ok(())
}

/// 从原始素材切一段并强制重编码到统一参数。
fn cut_and_normalize(
    input: &Path,
    start: f64,
    duration: f64,
    w: u32,
    h: u32,
    fps: u32,
    orientation: &str,
    output: &Path,
) -> AppResult<()> {
    // 视频滤镜：
    // 1) 智能裁剪（竖屏时 crop 中心区域，横屏时 pad 或 scale）
    // 2) fps 统一
    // 3) eq 轻微增强（对比度+10%，饱和度+15%，亮度+2%）
    // 4) format=yuv420p
    let video_filter = if orientation == "portrait" {
        // 竖屏：先统一缩放到目标宽，然后 crop 出目标高（从画面中心）
        // 如果原视频比目标更竖，则先缩放到目标高再 crop 宽
        format!(
            "scale={w}:-2,crop={w}:{h}:(iw-{w})/2:(ih-{h})/2,fps={fps},eq=contrast=1.1:saturation=1.15:brightness=0.02,format=yuv420p"
        )
    } else {
        // 横屏：保持原有逻辑（scale+pad）
        format!(
            "scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:black,fps={fps},eq=contrast=1.1:saturation=1.15:brightness=0.02,format=yuv420p"
        )
    };

    // 音频滤镜：loudnorm 统一响度 + aresample 统一采样率
    let audio_filter = "loudnorm=I=-16:TP=-1.5:LRA=11,aresample=48000";

    run_ffmpeg(&[
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-ss",
        &format!("{start}"),
        "-i",
        to_str(input)?,
        "-t",
        &format!("{duration}"),
        "-vf",
        &video_filter,
        "-r",
        &fps.to_string(),
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "23",
        "-pix_fmt",
        "yuv420p",
        "-af",
        audio_filter,
        "-c:a",
        "aac",
        "-ar",
        "48000",
        "-ac",
        "2",
        "-b:a",
        "128k",
        "-shortest",
        to_str(output)?,
    ])
}

/// 清空 work_dir/clips 下旧切片（重新预处理时调用）。
pub fn clean_clips_dir(project: &Project) -> AppResult<()> {
    if let Some(work) = &project.work_dir {
        let clips_dir = work.join("clips");
        if clips_dir.exists() {
            std::fs::remove_dir_all(&clips_dir)?;
        }
    }
    Ok(())
}
