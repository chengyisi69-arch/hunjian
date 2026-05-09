use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub llm_api_key: String,
    pub llm_api_url: String,
    pub llm_model: String,
    pub image_api_key: String,
    pub image_api_url: String,
    pub image_model: String,
    pub video_api_key: String,
    pub video_api_url: String,
    pub video_model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            llm_api_key: "".into(),
            llm_api_url: "https://api.openai.com/v1/chat/completions".into(),
            llm_model: "gpt-4o-mini".into(),
            image_api_key: "".into(),
            image_api_url: "https://api.openai.com/v1/images/generations".into(),
            image_model: "dall-e-3".into(),
            video_api_key: "".into(),
            video_api_url: "".into(),
            video_model: "".into(),
        }
    }
}

impl AiConfig {
    pub fn load() -> crate::error::AppResult<Self> {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            let data = std::fs::read_to_string(config_path)
                .map_err(|e| crate::error::AppError::Other(format!("读取配置失败: {e}")))?;
            serde_json::from_str(&data)
                .map_err(|e| crate::error::AppError::Other(format!("解析配置失败: {e}")))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> crate::error::AppResult<()> {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::error::AppError::Other(format!("创建配置目录失败: {e}")))?;
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::AppError::Other(format!("序列化配置失败: {e}")))?;
        std::fs::write(config_path, data)
            .map_err(|e| crate::error::AppError::Other(format!("写入配置失败: {e}")))?;
        Ok(())
    }

    fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("hunjian")
            .join("ai_config.json")
    }
}
