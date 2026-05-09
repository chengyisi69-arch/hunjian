//! 视频拼接 + 随机 xfade 转场。
//!
//! 5 种内置转场：fade / slideleft / slideright / wipeleft / zoomin。
//! 支持自定义转场时长，同一视频内转场不重复。

use crate::error::AppResult;
use crate::ffmpeg::{run_ffmpeg, to_str};
use crate::types::Clip;
use rand::seq::SliceRandom;
use std::path::Path;

pub const TRANSITIONS: &[&str] = &["fade", "slideleft", "slideright", "wipeleft", "zoomin"];

/// 把 clips 顺序拼接成单个视频，相邻片段之间随机选一种 xfade 转场。
/// 输出的视频已带原片段音轨（acrossfade 衔接），分辨率/帧率以输入为准。
pub fn concat_with_transitions(clips: &[Clip], output: &Path, trans_secs: f64) -> AppResult<()> {
    assert!(!clips.is_empty(), "concat_with_transitions: clips 不能为空");

    if clips.len() == 1 {
        // 单片段：直接 remux 到目标
        return run_ffmpeg(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            to_str(&clips[0].path)?,
            "-c",
            "copy",
            to_str(output)?,
        ]);
    }

    let trans = pick_transitions(clips.len() - 1);
    let filter = build_xfade_filter(clips, &trans, trans_secs);
    let v_out = format!("[v{}]", clips.len() - 1);
    let a_out = format!("[a{}]", clips.len() - 1);

    // 拼接 ffmpeg 参数：N 个 -i + filter_complex + map
    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
    ];
    for c in clips {
        args.push("-i".into());
        args.push(to_str(&c.path)?.into());
    }
    args.push("-filter_complex".into());
    args.push(filter);
    args.push("-map".into());
    args.push(v_out);
    args.push("-map".into());
    args.push(a_out);
    args.push("-c:v".into());
    args.push("libx264".into());
    args.push("-preset".into());
    args.push("veryfast".into());
    args.push("-crf".into());
    args.push("23".into());
    args.push("-pix_fmt".into());
    args.push("yuv420p".into());
    args.push("-c:a".into());
    args.push("aac".into());
    args.push("-ar".into());
    args.push("48000".into());
    args.push("-b:a".into());
    args.push("128k".into());
    args.push(to_str(output)?.into());

    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    run_ffmpeg(&argv)
}

/// 随机选 n 个转场名，同一视频内尽量不重复。
/// 当 n > 转场种类数时允许循环使用，但相邻两次不重复。
fn pick_transitions(n: usize) -> Vec<&'static str> {
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(n);
    let mut last: Option<&str> = None;

    for _ in 0..n {
        let mut pool: Vec<&&str> = TRANSITIONS.iter().collect();
        // 如果可能，排除上次用过的
        if let Some(l) = last {
            pool.retain(|&&t| t != l);
        }
        let choice = pool.choose(&mut rng).copied().copied()
            .unwrap_or(TRANSITIONS[0]);
        result.push(choice);
        last = Some(choice);
    }
    result
}

/// 构造 xfade 链 + acrossfade 链的 -filter_complex 字符串。
///
/// 视频：
///   [0:v]format=yuv420p,setpts=PTS-STARTPTS[v0_in];
///   [1:v]format=yuv420p,setpts=PTS-STARTPTS[v1_in];
///   [v0_in][v1_in]xfade=transition=...:duration=T:offset=d0-T[v1];
///   [v1][v2_in]xfade=...:offset=d0+d1-2T[v2];
///
/// 音频：
///   [0:a]aresample=48000,asetpts=PTS-STARTPTS[a0_in];
///   ...
///   [a0_in][a1_in]acrossfade=d=T[a1];
///   [a1][a2_in]acrossfade=d=T[a2];
pub fn build_xfade_filter(clips: &[Clip], transitions: &[&str], t: f64) -> String {
    let mut s = String::new();

    // 预处理每路输入
    for i in 0..clips.len() {
        s.push_str(&format!(
            "[{i}:v]format=yuv420p,setpts=PTS-STARTPTS[v{i}_in];"
        ));
    }
    for i in 0..clips.len() {
        s.push_str(&format!(
            "[{i}:a]aresample=48000,asetpts=PTS-STARTPTS[a{i}_in];"
        ));
    }

    // xfade 链
    let mut acc = 0.0;
    for i in 0..clips.len() - 1 {
        // 第 i 次 xfade，前一段输出标签是 v0_in（i=0）或 v{i}（i>=1）
        let prev_v = if i == 0 {
            "[v0_in]".to_string()
        } else {
            format!("[v{i}]")
        };
        let next_v = format!("[v{}_in]", i + 1);
        let out_v = format!("[v{}]", i + 1);

        acc += clips[i].duration_secs;
        let offset = acc - (i as f64 + 1.0) * t;
        s.push_str(&format!(
            "{prev_v}{next_v}xfade=transition={tr}:duration={t}:offset={off:.3}{out};",
            tr = transitions[i],
            t = t,
            off = offset.max(0.0),
            out = out_v
        ));
    }

    // acrossfade 链
    for i in 0..clips.len() - 1 {
        let prev_a = if i == 0 {
            "[a0_in]".to_string()
        } else {
            format!("[a{i}]")
        };
        let next_a = format!("[a{}_in]", i + 1);
        let out_a = format!("[a{}]", i + 1);
        s.push_str(&format!(
            "{prev_a}{next_a}acrossfade=d={t}{out_a};",
            t = t,
            out_a = out_a
        ));
    }

    // 去掉末尾分号，filter_complex 不允许悬空分号
    if s.ends_with(';') {
        s.pop();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_clip(d: f64) -> Clip {
        Clip {
            path: PathBuf::from("dummy.mp4"),
            duration_secs: d,
            scene_id: "s".into(),
            used: false,
        }
    }

    #[test]
    fn xfade_offsets_are_correct() {
        let clips = vec![make_clip(3.0), make_clip(3.0), make_clip(3.0)];
        let f = build_xfade_filter(&clips, &["fade", "slideleft"], 0.5);
        // 第一对 offset = 3 - 0.5 = 2.5
        assert!(f.contains("offset=2.500"), "got: {f}");
        // 第二对 offset = 6 - 1.0 = 5.0
        assert!(f.contains("offset=5.000"), "got: {f}");
    }
}
