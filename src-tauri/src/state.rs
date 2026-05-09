use crate::ai::config::AiConfig;
use crate::types::Project;
use std::sync::Mutex;

/// 全局应用状态。MVP 阶段直接用 Mutex 保护一个 Project 实例。
#[derive(Default)]
pub struct AppState {
    pub project: Mutex<Project>,
    pub ai_config: Mutex<AiConfig>,
}

impl AppState {
    pub fn new() -> Self {
        let ai_config = AiConfig::load().unwrap_or_default();
        Self {
            project: Mutex::new(Project::default()),
            ai_config: Mutex::new(ai_config),
        }
    }
}
