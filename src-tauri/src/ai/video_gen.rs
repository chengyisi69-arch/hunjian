use crate::error::AppResult;
use crate::ffmpeg::run_ffmpeg;
use std::path::PathBuf;

/// 使用 FFmpeg 将单张图片转为带动态效果的视频片段。
/// effect: "zoom-in" | "ken-burns" | "pan-left" | "pan-right"
pub fn image_to_motion_video(
    image_path: &PathBuf,
    duration: f64,
    effect: &str,
    w: u32,
    h: u32,
    output: &PathBuf,
) -> AppResult<()> {
    let frames = (duration * 30.0).round() as u32; // 30fps

    let filter = match effect {
        "zoom-in" => {
            // 从 1.0 缓慢缩放到 1.3
            format!(
                "zoompan=z='min(zoom+0.0015,1.5)':d={frames}:s={w}x{h}:fps=30,\
                trim=duration={duration},setpts=PTS-STARTPTS,format=yuv420p",
                frames = frames,
                w = w,
                h = h,
                duration = duration
            )
        }
        "ken-burns" => {
            // 缓慢从左上角缩放到右下角（经典 Ken Burns）
            format!(
                "zoompan=z='min(zoom+0.0012,1.4)':x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)':d={frames}:s={w}x{h}:fps=30,\
                trim=duration={duration},setpts=PTS-STARTPTS,format=yuv420p",
                frames = frames,
                w = w,
                h = h,
                duration = duration
            )
        }
        "pan-left" => {
            // 从右向左平移
            format!(
                "crop=iw*0.8:ih:(iw-iw*0.8)*min(t/{dur},1):0,scale={w}:{h},fps=30,format=yuv420p",
                dur = duration,
                w = w,
                h = h
            )
        }
        "pan-right" => {
            // 从左向右平移
            format!(
                "crop=iw*0.8:ih:(iw-iw*0.8)*(1-min(t/{dur},1)):0,scale={w}:{h},fps=30,format=yuv420p",
                dur = duration,
                w = w,
                h = h
            )
        }
        _ => {
            // 默认 zoom-in
            format!(
                "zoompan=z='min(zoom+0.0015,1.5)':d={frames}:s={w}x{h}:fps=30,\
                trim=duration={duration},setpts=PTS-STARTPTS,format=yuv420p",
                frames = frames,
                w = w,
                h = h,
                duration = duration
            )
        }
    };

    run_ffmpeg(&[
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-loop",
        "1",
        "-i",
        image_path.to_str().ok_or_else(|| {
            crate::error::AppError::Other("图片路径包含非法字符".into())
        })?,
        "-vf",
        &filter,
        "-t",
        &format!("{duration:.2}"),
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "23",
        "-pix_fmt",
        "yuv420p",
        "-an",
        output.to_str().ok_or_else(|| {
            crate::error::AppError::Other("输出路径包含非法字符".into())
        })?,
    ])
}
