//! Tauri 命令：前端 invoke 的入口。

use crate::error::{AppError, AppResult};
use crate::ffmpeg;
use crate::pipeline;
use crate::preprocess;
use crate::state::AppState;
use crate::types::{BatchMixResult, MixParams, Project, RawAsset};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn add_scene(state: State<'_, AppState>, name: String) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    if p.scenes.iter().any(|s| s == &name) {
        return Err(AppError::InvalidArgument(format!("场景已存在: {name}")));
    }
    p.scenes.push(name);
    Ok(())
}

#[tauri::command]
pub fn remove_scene(state: State<'_, AppState>, name: String) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    p.scenes.retain(|s| s != &name);
    p.raw_assets.retain(|a| a.scene_id != name);
    p.clips.retain(|c| c.scene_id != name);
    Ok(())
}

#[tauri::command]
pub fn import_assets(
    state: State<'_, AppState>,
    scene_id: String,
    paths: Vec<PathBuf>,
) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    if !p.scenes.iter().any(|s| s == &scene_id) {
        return Err(AppError::NotFound(format!("场景不存在: {scene_id}")));
    }
    for path in paths {
        let duration = ffmpeg::probe_duration(&path).unwrap_or(0.0);
        p.raw_assets.push(RawAsset {
            path,
            duration_secs: duration,
            scene_id: scene_id.clone(),
        });
    }
    Ok(())
}

#[tauri::command]
pub fn preprocess(
    state: State<'_, AppState>,
    clip_duration_secs: f64,
    orientation: String,
) -> AppResult<usize> {
    // 在锁内拷贝必要数据，离开锁后跑 ffmpeg，结束再回写
    let mut p_local: Project;
    {
        let mut p = state.project.lock().unwrap();
        p.clip_duration_secs = clip_duration_secs;
        p.orientation = orientation.clone();
        p.output_resolution = match orientation.as_str() {
            "landscape" => (1920, 1080),
            _ => (1080, 1920),
        };
        pipeline::clean_for_reprocess(&mut p)?;
        pipeline::ensure_work_dir(&mut p)?;
        p_local = p.clone();
    }
    preprocess::preprocess_all_assets(&mut p_local)?;
    let n = p_local.clips.len();
    *state.project.lock().unwrap() = p_local;
    Ok(n)
}

#[tauri::command]
pub fn generate_mix(
    state: State<'_, AppState>,
    app: AppHandle,
    params: MixParams,
) -> AppResult<BatchMixResult> {
    // 1) 锁内更新参数 + 拷贝出本地副本
    let p_local: Project = {
        let mut p = state.project.lock().unwrap();
        p.picks_per_scene = params.picks_per_scene;
        p.title_text = params.title_text.clone();
        p.title_position = params.title_position.clone();
        p.title_font_size = params.title_font_size;
        p.title_style = params.title_style.clone();
        p.title2_text = params.title2_text.clone();
        p.title2_position = params.title2_position.clone();
        p.title2_font_size = params.title2_font_size;
        p.title2_style = params.title2_style.clone();
        p.title3_text = params.title3_text.clone();
        p.title3_position = params.title3_position.clone();
        p.title3_font_size = params.title3_font_size;
        p.title3_style = params.title3_style.clone();
        p.subtitle_position = params.subtitle_position.clone();
        p.narration_text = params.narration_text.clone();
        p.bg_music_path = params.bg_music_path.clone();
        p.max_duration_secs = params.max_duration_secs;
        p.original_volume = params.original_volume;
        p.narration_volume = params.narration_volume;
        p.bgm_volume = params.bgm_volume;
        p.tts_voice = params.tts_voice.clone();
        p.batch_count = params.batch_count;
        p.auto_subtitle = params.auto_subtitle;
        p.trans_secs = params.trans_secs;
        p.bgm_fade_secs = params.bgm_fade_secs;
        if p.clip_duration_secs <= 0.0 {
            p.clip_duration_secs = params.clip_duration_secs;
        }
        p.clone()
    };

    // 2) 锁外跑流水线
    let emit = |progress: pipeline::Progress| {
        let _ = app.emit("progress", progress);
    };
    let result = pipeline::run_pipeline(&p_local, &params, &emit)?;

    // 3) 记录输出路径
    {
        let mut p = state.project.lock().unwrap();
        if let Some(first) = result.outputs.first() {
            p.last_output = Some(first.output_path.clone());
        }
        p.batch_outputs = result.outputs.iter().map(|r| r.output_path.clone()).collect();
    }
    Ok(result)
}

#[tauri::command]
pub fn reset_clips(state: State<'_, AppState>) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    crate::selection::reset_all_clips(&mut p);
    Ok(())
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>) -> AppResult<Project> {
    Ok(state.project.lock().unwrap().clone())
}

#[tauri::command]
pub fn save_project(state: State<'_, AppState>, path: PathBuf) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    pipeline::save_project(&p, &path)?;
    p.project_file = Some(path);
    Ok(())
}

#[tauri::command]
pub fn load_project(state: State<'_, AppState>, path: PathBuf) -> AppResult<Project> {
    let loaded = pipeline::load_project(&path)?;
    *state.project.lock().unwrap() = loaded.clone();
    Ok(loaded)
}

#[tauri::command]
pub fn export_video(src: PathBuf, dst: PathBuf) -> AppResult<()> {
    std::fs::copy(&src, &dst).map_err(|e| {
        AppError::Other(format!(
            "复制视频失败: {e}  (src={} dst={})",
            src.display(),
            dst.display()
        ))
    })?;
    Ok(())
}

#[tauri::command]
pub fn new_project(state: State<'_, AppState>) -> AppResult<()> {
    *state.project.lock().unwrap() = Project::default();
    Ok(())
}

// ── 模板命令 ──

#[tauri::command]
pub fn save_template(state: State<'_, AppState>, template: crate::types::Template) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    // 同名覆盖
    p.templates.retain(|t| t.name != template.name);
    p.templates.push(template);
    Ok(())
}

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> AppResult<Vec<crate::types::Template>> {
    let p = state.project.lock().unwrap();
    Ok(p.templates.clone())
}

#[tauri::command]
pub fn delete_template(state: State<'_, AppState>, name: String) -> AppResult<()> {
    let mut p = state.project.lock().unwrap();
    p.templates.retain(|t| t.name != name);
    Ok(())
}

#[tauri::command]
pub fn apply_template(state: State<'_, AppState>, name: String) -> AppResult<crate::types::Template> {
    let mut p = state.project.lock().unwrap();
    let t = p.templates.iter().find(|t| t.name == name)
        .ok_or_else(|| AppError::NotFound(format!("模板不存在: {name}")))?
        .clone();
    p.clip_duration_secs = t.clip_duration_secs;
    p.picks_per_scene = t.picks_per_scene;
    p.max_duration_secs = t.max_duration_secs;
    p.orientation = t.orientation.clone();
    p.title_font_size = t.title_font_size;
    p.title_style = t.title_style.clone();
    p.title2_font_size = t.title2_font_size;
    p.title2_style = t.title2_style.clone();
    p.title3_font_size = t.title3_font_size;
    p.title3_style = t.title3_style.clone();
    p.original_volume = t.original_volume;
    p.narration_volume = t.narration_volume;
    p.bgm_volume = t.bgm_volume;
    p.tts_voice = t.tts_voice.clone();
    p.batch_count = t.batch_count;
    p.auto_subtitle = t.auto_subtitle;
    p.trans_secs = t.trans_secs;
    p.bgm_fade_secs = t.bgm_fade_secs;
    Ok(t)
}

// ── AI 命令 ──

#[tauri::command]
pub async fn get_ai_config(state: State<'_, AppState>) -> AppResult<crate::ai::config::AiConfig> {
    Ok(state.ai_config.lock().unwrap().clone())
}

#[tauri::command]
pub async fn set_ai_config(
    state: State<'_, AppState>,
    config: crate::ai::config::AiConfig,
) -> AppResult<()> {
    let mut cfg = state.ai_config.lock().unwrap();
    *cfg = config;
    cfg.save()?;
    Ok(())
}

#[tauri::command]
pub async fn ai_generate_title(
    state: State<'_, AppState>,
    description: String,
) -> AppResult<String> {
    let cfg = state.ai_config.lock().unwrap().clone();
    crate::ai::llm::generate_title(&description, &cfg).await
}

#[tauri::command]
pub async fn ai_generate_script(
    state: State<'_, AppState>,
    description: String,
) -> AppResult<String> {
    let cfg = state.ai_config.lock().unwrap().clone();
    crate::ai::llm::generate_script(&description, &cfg).await
}

#[tauri::command]
pub async fn ai_generate_image(
    state: State<'_, AppState>,
    prompt: String,
) -> AppResult<String> {
    let cfg = state.ai_config.lock().unwrap().clone();
    let path = crate::ai::image_gen::generate_image(&prompt, &cfg).await?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn image_to_video(
    image_path: String,
    duration: f64,
    effect: String,
    orientation: String,
) -> AppResult<String> {
    let input = PathBuf::from(&image_path);
    let (w, h) = match orientation.as_str() {
        "landscape" => (1920u32, 1080u32),
        _ => (1080u32, 1920u32),
    };
    let out = std::env::temp_dir().join(format!(
        "hunjian-motion-{}.mp4",
        rand::random::<u32>()
    ));
    crate::ai::video_gen::image_to_motion_video(&input, duration, &effect, w, h, &out)?;
    Ok(out.to_string_lossy().to_string())
}

/// 将图片作为素材添加到指定场景（转换为视频片段或直接加入）
#[tauri::command]
pub fn add_image_as_asset(
    state: State<'_, AppState>,
    scene_id: String,
    image_path: String,
    duration: f64,
    orientation: String,
) -> AppResult<String> {
    let mut p = state.project.lock().unwrap();
    if !p.scenes.iter().any(|s| s == &scene_id) {
        return Err(AppError::NotFound(format!("场景不存在: {scene_id}")));
    }

    let work = match &p.work_dir {
        Some(w) => w.clone(),
        None => {
            let w = std::env::temp_dir().join(format!("hunjian-{}", rand::random::<u32>()));
            std::fs::create_dir_all(&w)?;
            p.work_dir = Some(w.clone());
            w
        }
    };

    let input = PathBuf::from(&image_path);
    let (w, h) = match orientation.as_str() {
        "landscape" => (1920u32, 1080u32),
        _ => (1080u32, 1920u32),
    };
    let out = work.join(format!("ai_img_{}.mp4", rand::random::<u32>()));

    crate::ai::video_gen::image_to_motion_video(&input, duration, "zoom-in", w, h, &out)?;

    p.raw_assets.push(RawAsset {
        path: out.clone(),
        duration_secs: duration,
        scene_id: scene_id.clone(),
    });

    Ok(out.to_string_lossy().to_string())
}
