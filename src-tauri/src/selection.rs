//! 片段挑选：从未使用的片段池中按场景顺序随机抽取，并执行总时长约束。

use crate::types::{Clip, Project};
use rand::seq::SliceRandom;
use std::collections::HashMap;

const TRANSITION_SECS: f64 = 0.5;

/// 按场景顺序随机抽取片段。返回 (选中片段, 是否被裁剪过)。
pub fn select_clips(project: &Project, picks_per_scene: usize) -> Vec<Clip> {
    let mut selected: Vec<Clip> = Vec::new();
    let mut rng = rand::thread_rng();

    for scene in &project.scenes {
        let scene_pool: Vec<&Clip> = project
            .clips
            .iter()
            .filter(|c| !c.used && c.scene_id == *scene)
            .collect();
        let take = picks_per_scene.min(scene_pool.len());
        let chosen: Vec<Clip> = scene_pool
            .choose_multiple(&mut rng, take)
            .map(|&c| c.clone())
            .collect();
        selected.extend(chosen);
    }
    selected
}

/// 总时长（含转场）。
pub fn total_duration_with_transitions(clips: &[Clip]) -> f64 {
    let raw: f64 = clips.iter().map(|c| c.duration_secs).sum();
    let transitions = if clips.len() > 1 {
        (clips.len() - 1) as f64 * TRANSITION_SECS
    } else {
        0.0
    };
    raw + transitions
}

/// 若超过最大时长，自动减少片段：先把每场景超额降到 picks_per_scene，再随机丢弃直到达标。
pub fn auto_reduce_selections(
    selected: &mut Vec<Clip>,
    picks_per_scene: usize,
    max_duration_secs: f64,
) {
    if total_duration_with_transitions(selected) <= max_duration_secs {
        return;
    }

    // 1) 每场景压到 picks_per_scene 以内
    let mut count: HashMap<String, usize> = HashMap::new();
    let mut to_remove: Vec<usize> = Vec::new();
    for (i, c) in selected.iter().enumerate() {
        let entry = count.entry(c.scene_id.clone()).or_insert(0);
        *entry += 1;
        if *entry > picks_per_scene {
            to_remove.push(i);
        }
    }
    for i in to_remove.into_iter().rev() {
        selected.remove(i);
    }

    // 2) 仍超时则随机丢弃，但至少保留 1 个
    while total_duration_with_transitions(selected) > max_duration_secs && selected.len() > 1 {
        let idx = rand::random::<usize>() % selected.len();
        selected.remove(idx);
    }
}

/// 标记选中片段为已使用。
pub fn mark_used(project: &mut Project, selected: &[Clip]) {
    for s in selected {
        if let Some(c) = project.clips.iter_mut().find(|c| c.path == s.path) {
            c.used = true;
        }
    }
}

/// 重置所有片段为未使用。
pub fn reset_all_clips(project: &mut Project) {
    for c in &mut project.clips {
        c.used = false;
    }
}
