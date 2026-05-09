//! 离线 TTS 配音。
//!
//! 默认实现用 espeak-ng，安装方式见 README。
//! 中文效果一般，后续可替换为 piper 或 coqui-tts。
//!
//! 音色差异通过 pitch（-p 0–99）+ speed（-s words-per-minute）实现，
//! 因为 espeak-ng 的 +f/+m 变体对中文 cmn 几乎无区别。

use crate::error::{AppError, AppResult};
use std::path::Path;
use std::process::Command;

struct VoiceCfg {
    suffix: &'static str,
    pitch: u8,
    speed: u16,
}

fn voice_cfg(style: &str) -> VoiceCfg {
    match style {
        "female" => VoiceCfg {
            suffix: "+f3",
            pitch: 65,
            speed: 170,
        },
        "male" => VoiceCfg {
            suffix: "+m3",
            pitch: 35,
            speed: 150,
        },
        "news_female" => VoiceCfg {
            suffix: "+f2",
            pitch: 55,
            speed: 160,
        },
        "loli" => VoiceCfg {
            suffix: "+f5",
            pitch: 88,
            speed: 230,
        },
        "uncle" => VoiceCfg {
            suffix: "+m1",
            pitch: 22,
            speed: 125,
        },
        "youth_male" => VoiceCfg {
            suffix: "+m4",
            pitch: 48,
            speed: 185,
        },
        "sweet_female" => VoiceCfg {
            suffix: "+f4",
            pitch: 78,
            speed: 200,
        },
        "deep_male" => VoiceCfg {
            suffix: "+m2",
            pitch: 28,
            speed: 135,
        },
        _ => VoiceCfg {
            suffix: "",
            pitch: 50,
            speed: 175,
        },
    }
}

pub fn resolve_voice(lang_prefix: &str, style: &str) -> String {
    let cfg = voice_cfg(style);
    format!("{}{}", lang_prefix, cfg.suffix)
}

pub fn text_to_speech(
    text: &str,
    output_wav: &Path,
    lang: &str,
    voice: &str,
) -> AppResult<()> {
    if text.trim().is_empty() {
        return Err(AppError::InvalidArgument("配音文案为空".into()));
    }
    let out = output_wav
        .to_str()
        .ok_or_else(|| AppError::InvalidArgument(format!("非法路径: {}", output_wav.display())))?;

    let cfg = voice_cfg(voice);
    let voice_str = format!("{}{}", lang, cfg.suffix);

    let status = Command::new("espeak-ng")
        .args([
            "-v",
            &voice_str,
            "-p",
            &cfg.pitch.to_string(),
            "-s",
            &cfg.speed.to_string(),
            "-w",
            out,
            text,
        ])
        .status()
        .map_err(|e| AppError::Tts(format!("启动 espeak-ng 失败 (是否已安装?): {e}")))?;
    if !status.success() {
        return Err(AppError::Tts("espeak-ng 退出码非零".into()));
    }
    Ok(())
}
