use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_subtitle_position() -> String { "50,95".into() }
fn default_orientation() -> String { "portrait".into() }
fn default_original_volume() -> f32 { 0.08 }
fn default_narration_volume() -> f32 { 1.0 }
fn default_bgm_volume() -> f32 { 0.3 }
fn default_tts_voice() -> String { "auto".into() }
fn default_batch_count() -> u32 { 1 }
fn default_title_font_size() -> u32 { 48 }
fn default_title_style() -> String { "simple".into() }
fn default_auto_subtitle() -> bool { false }
fn default_trans_secs() -> f64 { 0.5 }
fn default_bgm_fade_secs() -> f32 { 1.5 }

/// 用户导入的原始素材。一段长视频，归属一个场景。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAsset {
    pub path: PathBuf,
    pub duration_secs: f64,
    pub scene_id: String,
}

/// 从原始素材中切出来的固定长度小片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub path: PathBuf,
    pub duration_secs: f64,
    pub scene_id: String,
    pub used: bool,
}

/// 模板结构：保存一组常用参数，快速切换风格。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub clip_duration_secs: f64,
    pub picks_per_scene: usize,
    pub max_duration_secs: f64,
    pub orientation: String,
    pub title_font_size: u32,
    pub title_style: String,
    pub title2_font_size: u32,
    pub title2_style: String,
    pub title3_font_size: u32,
    pub title3_style: String,
    pub original_volume: f32,
    pub narration_volume: f32,
    pub bgm_volume: f32,
    pub tts_voice: String,
    pub batch_count: u32,
    pub auto_subtitle: bool,
    pub trans_secs: f64,
    pub bgm_fade_secs: f32,
}

impl Default for Template {
    fn default() -> Self {
        Self {
            name: "默认模板".into(),
            clip_duration_secs: 3.0,
            picks_per_scene: 1,
            max_duration_secs: 60.0,
            orientation: "portrait".into(),
            title_font_size: 48,
            title_style: "simple".into(),
            title2_font_size: 48,
            title2_style: "simple".into(),
            title3_font_size: 48,
            title3_style: "simple".into(),
            original_volume: 0.08,
            narration_volume: 1.0,
            bgm_volume: 0.3,
            tts_voice: "auto".into(),
            batch_count: 1,
            auto_subtitle: false,
            trans_secs: 0.5,
            bgm_fade_secs: 1.5,
        }
    }
}

/// 整个项目的完整状态，前端可序列化展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub scenes: Vec<String>,
    pub raw_assets: Vec<RawAsset>,
    pub clips: Vec<Clip>,
    pub clip_duration_secs: f64,
    pub picks_per_scene: usize,

    // 标题1
    pub title_text: String,
    pub title_position: String,
    #[serde(default = "default_title_font_size")]
    pub title_font_size: u32,
    #[serde(default = "default_title_style")]
    pub title_style: String,

    // 标题2
    #[serde(default)]
    pub title2_text: String,
    #[serde(default)]
    pub title2_position: String,
    #[serde(default = "default_title_font_size")]
    pub title2_font_size: u32,
    #[serde(default = "default_title_style")]
    pub title2_style: String,

    // 标题3
    #[serde(default)]
    pub title3_text: String,
    #[serde(default)]
    pub title3_position: String,
    #[serde(default = "default_title_font_size")]
    pub title3_font_size: u32,
    #[serde(default = "default_title_style")]
    pub title3_style: String,

    #[serde(default = "default_subtitle_position")]
    pub subtitle_position: String,
    pub narration_text: String,
    pub bg_music_path: Option<PathBuf>,
    /// "portrait"（1080×1920）或 "landscape"（1920×1080）
    #[serde(default = "default_orientation")]
    pub orientation: String,
    pub output_resolution: (u32, u32),
    pub output_fps: u32,
    pub max_duration_secs: f64,
    /// 三轨音量（0.0–2.0），原片段声 / 配音 / 背景音乐
    #[serde(default = "default_original_volume")]
    pub original_volume: f32,
    #[serde(default = "default_narration_volume")]
    pub narration_volume: f32,
    #[serde(default = "default_bgm_volume")]
    pub bgm_volume: f32,
    /// "auto" / "female" / "male" / "news_female" / "loli" / "uncle" / "youth_male" / "sweet_female" / "deep_male"
    #[serde(default = "default_tts_voice")]
    pub tts_voice: String,
    /// 批量生成数量（1–50）
    #[serde(default = "default_batch_count")]
    pub batch_count: u32,
    /// 是否启用自动语音字幕（ASS 时间轴）
    #[serde(default = "default_auto_subtitle")]
    pub auto_subtitle: bool,
    /// 转场时长（秒）
    #[serde(default = "default_trans_secs")]
    pub trans_secs: f64,
    /// BGM 淡入淡出秒数
    #[serde(default = "default_bgm_fade_secs")]
    pub bgm_fade_secs: f32,
    /// 模板列表
    #[serde(default)]
    pub templates: Vec<Template>,
    /// 用于存放切片/中间产物的工作目录。
    pub work_dir: Option<PathBuf>,
    /// 上一次成功生成的输出文件路径（用于前端显示）。
    pub last_output: Option<PathBuf>,
    /// 批量生成的所有输出路径。
    pub batch_outputs: Vec<PathBuf>,
    /// 项目落盘文件路径。None 表示尚未保存。
    pub project_file: Option<PathBuf>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: String::new(),
            scenes: Vec::new(),
            raw_assets: Vec::new(),
            clips: Vec::new(),
            clip_duration_secs: 3.0,
            picks_per_scene: 1,
            title_text: String::new(),
            title_position: "5,5".into(),
            title_font_size: 48,
            title_style: "simple".into(),
            title2_text: String::new(),
            title2_position: String::new(),
            title2_font_size: 48,
            title2_style: "simple".into(),
            title3_text: String::new(),
            title3_position: String::new(),
            title3_font_size: 48,
            title3_style: "simple".into(),
            subtitle_position: "50,95".into(),
            narration_text: String::new(),
            bg_music_path: None,
            orientation: "portrait".into(),
            output_resolution: (1080, 1920),
            output_fps: 30,
            max_duration_secs: 60.0,
            original_volume: 0.08,
            narration_volume: 1.0,
            bgm_volume: 0.3,
            tts_voice: "auto".into(),
            batch_count: 1,
            auto_subtitle: false,
            trans_secs: 0.5,
            bgm_fade_secs: 1.5,
            templates: Vec::new(),
            work_dir: None,
            last_output: None,
            batch_outputs: Vec::new(),
            project_file: None,
        }
    }
}

/// 前端 generate_mix 时传入的参数集合。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MixParams {
    pub clip_duration_secs: f64,
    pub picks_per_scene: usize,

    pub title_text: String,
    pub title_position: String,
    pub title_font_size: u32,
    pub title_style: String,

    pub title2_text: String,
    pub title2_position: String,
    pub title2_font_size: u32,
    pub title2_style: String,

    pub title3_text: String,
    pub title3_position: String,
    pub title3_font_size: u32,
    pub title3_style: String,

    pub subtitle_position: String,
    pub narration_text: String,
    pub bg_music_path: Option<PathBuf>,
    pub max_duration_secs: f64,
    pub original_volume: f32,
    pub narration_volume: f32,
    pub bgm_volume: f32,
    pub tts_voice: String,
    pub batch_count: u32,
    pub auto_subtitle: bool,
    pub trans_secs: f64,
    pub bgm_fade_secs: f32,
}

/// 单条生成结果摘要。
#[derive(Debug, Clone, Serialize)]
pub struct MixResult {
    pub output_path: PathBuf,
    pub duration_secs: f64,
    pub clip_count: usize,
}

/// 批量生成结果。
#[derive(Debug, Clone, Serialize)]
pub struct BatchMixResult {
    pub outputs: Vec<MixResult>,
    pub total_count: usize,
}
