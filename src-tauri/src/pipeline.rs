//! 完整流水线：select → compose → overlay → tts → audio_mix。
//! 每个阶段通过 emit_progress 推一个事件，前端可显示进度。

use crate::audio;
use crate::compose;
use crate::error::{AppError, AppResult};
use crate::overlay;
use crate::preprocess;
use crate::selection;
use crate::tts;
use crate::types::{BatchMixResult, Clip, MixParams, MixResult, Project};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub stage: &'static str,
    pub percent: u32,
    pub message: String,
}

/// 执行单条流水线。emitter 是一个回调，让 commands 层注入 Tauri AppHandle。
fn run_pipeline_single<F>(
    project: &Project,
    params: &MixParams,
    emit: &F,
    output_path: &Path,
) -> AppResult<(MixResult, Vec<Clip>)>
where
    F: Fn(Progress),
{
    if project.clips.is_empty() {
        return Err(AppError::InvalidArgument(
            "片段池为空，请先预处理素材".into(),
        ));
    }
    let work = project
        .work_dir
        .as_ref()
        .ok_or_else(|| AppError::InvalidArgument("work_dir 未初始化".into()))?;

    // 1) 选片
    emit(Progress {
        stage: "select",
        percent: 5,
        message: "随机抽取片段…".into(),
    });
    let mut selected = selection::select_clips(project, params.picks_per_scene);
    selection::auto_reduce_selections(
        &mut selected,
        params.picks_per_scene,
        params.max_duration_secs,
    );
    if selected.is_empty() {
        return Err(AppError::InvalidArgument(
            "可用片段不足，请重置或导入更多素材".into(),
        ));
    }
    emit(Progress {
        stage: "select",
        percent: 15,
        message: format!("选中 {} 个片段", selected.len()),
    });

    // 2) 拼接 + 转场
    let composed = work.join("composed.mp4");
    emit(Progress {
        stage: "compose",
        percent: 25,
        message: "拼接片段并应用转场…".into(),
    });
    compose::concat_with_transitions(&selected, &composed, params.trans_secs)?;

    // 3) 叠加 标题 + 字幕
    let with_text = work.join("with_text.mp4");
    emit(Progress {
        stage: "overlay",
        percent: 55,
        message: "叠加标题与字幕…".into(),
    });
    let video_duration_secs = selection::total_duration_with_transitions(&selected);
    let titles = [
        overlay::TitleCfg {
            text: &params.title_text,
            pos: &params.title_position,
            size: params.title_font_size,
            style: &params.title_style,
        },
        overlay::TitleCfg {
            text: &params.title2_text,
            pos: &params.title2_position,
            size: params.title2_font_size,
            style: &params.title2_style,
        },
        overlay::TitleCfg {
            text: &params.title3_text,
            pos: &params.title3_position,
            size: params.title3_font_size,
            style: &params.title3_style,
        },
    ];
    overlay::overlay_titles_and_subtitle(
        &composed,
        &titles,
        &params.narration_text,
        &params.subtitle_position,
        &work.join("text"),
        &with_text,
        None,
        params.auto_subtitle,
        video_duration_secs,
    )?;

    // 4) TTS（可选）
    let narration_path = if params.narration_text.trim().is_empty() {
        None
    } else {
        emit(Progress {
            stage: "tts",
            percent: 70,
            message: "合成配音…".into(),
        });
        let p = work.join("narration.wav");
        let lang = if has_cjk(&params.narration_text) {
            "cmn"
        } else {
            "en"
        };
        match tts::text_to_speech(&params.narration_text,
            &p,
            lang,
            &params.tts_voice,
        ) {
            Ok(()) => Some(p),
            Err(e) => {
                log::warn!("TTS 失败，将跳过配音：{e}");
                emit(Progress {
                    stage: "tts",
                    percent: 75,
                    message: format!("TTS 失败，跳过配音：{e}"),
                });
                None
            }
        }
    };

    // 5) 三轨混音
    emit(Progress {
        stage: "audio",
        percent: 85,
        message: "混合音轨…".into(),
    });
    audio::mix_audio(
        &with_text,
        narration_path.as_deref(),
        params.bg_music_path.as_deref(),
        output_path,
        params.original_volume,
        params.narration_volume,
        params.bgm_volume,
        params.bgm_fade_secs,
    )?;

    emit(Progress {
        stage: "done",
        percent: 100,
        message: format!("完成: {}", output_path.display()),
    });

    Ok((
        MixResult {
            output_path: output_path.to_path_buf(),
            duration_secs: video_duration_secs,
            clip_count: selected.len(),
        },
        selected,
    ))
}

/// 批量执行流水线。循环生成 batch_count 个视频，每次随机选片。
pub fn run_pipeline<F>(
    project: &Project,
    params: &MixParams,
    emit: &F,
) -> AppResult<BatchMixResult>
where
    F: Fn(Progress),
{
    let batch_count = params.batch_count.max(1).min(50) as usize;
    let work = project
        .work_dir
        .as_ref()
        .ok_or_else(|| AppError::InvalidArgument("work_dir 未初始化".into()))?;

    let mut outputs = Vec::with_capacity(batch_count);

    for i in 0..batch_count {
        // 每次重置所有 clips 的 used 状态，保证可重复抽取
        let mut project_clone = project.clone();
        for c in &mut project_clone.clips {
            c.used = false;
        }

        let output_path = work.join(format!("output_{:03}.mp4", i + 1));
        emit(Progress {
            stage: "batch",
            percent: ((i * 100) / batch_count) as u32,
            message: format!("正在生成第 {} / {} 个视频…", i + 1, batch_count),
        });

        let (result, _used) =
            run_pipeline_single(&project_clone, params, emit, &output_path)?;
        outputs.push(result);
    }

    emit(Progress {
        stage: "done",
        percent: 100,
        message: format!("全部完成，共生成 {} 个视频", outputs.len()),
    });

    let total_count = outputs.len();
    Ok(BatchMixResult {
        outputs,
        total_count,
    })
}

fn has_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        let u = c as u32;
        (0x4E00..=0x9FFF).contains(&u)
            || (0x3000..=0x303F).contains(&u)
            || (0x3040..=0x30FF).contains(&u)
    })
}

/// 把 Project 序列化到 JSON 文件。
pub fn save_project(project: &Project, path: &Path) -> AppResult<()> {
    let s = serde_json::to_string_pretty(project)
        .map_err(|e| AppError::Other(format!("项目序列化失败: {e}")))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, s)?;
    Ok(())
}

pub fn load_project(path: &Path) -> AppResult<Project> {
    let s = std::fs::read_to_string(path)?;
    let mut p: Project = serde_json::from_str(&s)
        .map_err(|e| AppError::Other(format!("项目反序列化失败: {e}")))?;
    p.project_file = Some(path.to_path_buf());
    Ok(p)
}

/// 用于命令层在锁外初始化 work_dir（避免 preprocess::preprocess_all_assets 内部生成副作用）。
pub fn ensure_work_dir(project: &mut Project) -> AppResult<PathBuf> {
    if project.work_dir.is_none() {
        let dir = std::env::temp_dir().join(format!("hunjian-{}", rand::random::<u32>()));
        std::fs::create_dir_all(&dir)?;
        project.work_dir = Some(dir);
    }
    Ok(project.work_dir.clone().unwrap())
}

/// 重新预处理前清理旧切片。
pub fn clean_for_reprocess(project: &mut Project) -> AppResult<()> {
    preprocess::clean_clips_dir(project)?;
    project.clips.clear();
    Ok(())
}

#[cfg(test)]
mod e2e_tests {
    //! 端到端集成测试：用 ffmpeg lavfi 合成 3 个场景的测试素材，
    //! 跑完整 pipeline，校验输出文件真实可读且时长合理。
    //!
    //! 需要本机有 ffmpeg；espeak-ng 缺失时 TTS 自动跳过（pipeline 已处理）。

    use super::*;
    use crate::ffmpeg::{probe_duration, run_ffmpeg};
    use crate::types::RawAsset;
    use std::cell::RefCell;
    use std::path::PathBuf;

    fn synth_asset(out: &Path, color: &str, freq: u32, duration: u32) {
        run_ffmpeg(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("color=c={color}:s=320x240:d={duration}:r=30"),
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency={freq}:duration={duration}:sample_rate=48000"),
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
            out.to_str().unwrap(),
        ])
        .expect("synth_asset failed");
    }

    #[test]
    fn full_pipeline_runs_and_produces_output() {
        let tmp = std::env::temp_dir().join(format!("hunjian-test-{}", rand::random::<u32>()));
        std::fs::create_dir_all(&tmp).unwrap();
        let assets_dir = tmp.join("inputs");
        std::fs::create_dir_all(&assets_dir).unwrap();

        // 3 个场景，各一段 12 秒素材，颜色/音调不同方便人眼/耳分辨
        let scenes = ["intro", "product", "outro"];
        let palette = [("red", 440), ("green", 660), ("blue", 880)];
        let mut raw_assets: Vec<RawAsset> = Vec::new();
        for (i, scene) in scenes.iter().enumerate() {
            let p: PathBuf = assets_dir.join(format!("{scene}.mp4"));
            synth_asset(&p, palette[i].0, palette[i].1, 12);
            let dur = probe_duration(&p).unwrap();
            raw_assets.push(RawAsset {
                path: p,
                duration_secs: dur,
                scene_id: scene.to_string(),
            });
        }

        // 构造 Project
        let mut project = Project {
            scenes: scenes.iter().map(|s| s.to_string()).collect(),
            raw_assets,
            clip_duration_secs: 3.0,
            picks_per_scene: 1,
            title_text: "Hunjian Test".into(),
            title_position: "top-left".into(),
            subtitle_position: "bottom-center".into(),
            narration_text: "Hello world".into(),
            bg_music_path: None,
            orientation: "landscape".into(),
            output_resolution: (640, 360),
            output_fps: 30,
            max_duration_secs: 30.0,
            original_volume: 0.08,
            narration_volume: 1.0,
            bgm_volume: 0.3,
            tts_voice: "auto".into(),
            batch_count: 1,
            title_font_size: 48,
            title_style: "simple".into(),
            ..Project::default()
        };
        project.work_dir = Some(tmp.clone());

        // 1) 预处理
        preprocess::preprocess_all_assets(&mut project).expect("preprocess failed");
        // 12s / 3s = 4 切片 × 3 场景 = 12
        assert_eq!(project.clips.len(), 12, "expected 12 clips");
        for c in &project.clips {
            assert!(c.path.exists(), "clip not on disk: {}", c.path.display());
        }

        // 2) 流水线
        let params = MixParams {
            clip_duration_secs: 3.0,
            picks_per_scene: 1,
            title_text: project.title_text.clone(),
            title_position: project.title_position.clone(),
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
            subtitle_position: project.subtitle_position.clone(),
            narration_text: project.narration_text.clone(),
            bg_music_path: None,
            max_duration_secs: 30.0,
            original_volume: project.original_volume,
            narration_volume: project.narration_volume,
            bgm_volume: project.bgm_volume,
            tts_voice: project.tts_voice.clone(),
            batch_count: 1,
            auto_subtitle: false,
            trans_secs: 0.5,
            bgm_fade_secs: 1.5,
        };
        let events: RefCell<Vec<(&'static str, u32)>> = RefCell::new(Vec::new());
        let emit = |p: Progress| {
            events.borrow_mut().push((p.stage, p.percent));
        };
        let result = run_pipeline(&project, &params, &emit).expect("pipeline failed");

        // 校验输出
        assert_eq!(result.outputs.len(), 1, "should have 1 output");
        assert!(result.outputs[0].output_path.exists(), "output missing");
        let dur = probe_duration(&result.outputs[0].output_path).expect("probe output");
        // 3 个 3 秒片段 + 2 个 0.5 秒转场重叠 ≈ 9 - 1 = 8 秒
        assert!(
            (7.0..=10.0).contains(&dur),
            "output duration out of range: {dur}"
        );

        // 阶段事件确实推送了关键节点
        let evs = events.borrow().clone();
        let stages: Vec<&str> = evs.iter().map(|(s, _)| *s).collect();
        for needed in ["select", "compose", "overlay", "audio", "done"] {
            assert!(stages.contains(&needed), "missing stage {needed}");
        }

        // 3) JSON 持久化往返
        let pf = tmp.join("project.json");
        save_project(&project, &pf).expect("save_project failed");
        let loaded = load_project(&pf).expect("load_project failed");
        assert_eq!(loaded.scenes, project.scenes);
        assert_eq!(loaded.clips.len(), project.clips.len());

        // 清理
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn xfade_filter_offsets_correct() {
        // 委托 compose 模块的 unit test 覆盖；这里再做一次集成层校验
        let f = crate::compose::build_xfade_filter(
            &[
                Clip {
                    path: PathBuf::new(),
                    duration_secs: 4.0,
                    scene_id: "a".into(),
                    used: false,
                },
                Clip {
                    path: PathBuf::new(),
                    duration_secs: 4.0,
                    scene_id: "b".into(),
                    used: false,
                },
            ],
            &["fade"],
            0.5,
        );
        assert!(f.contains("offset=3.500"), "got {f}");
    }
}
