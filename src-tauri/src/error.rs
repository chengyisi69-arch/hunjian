use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("ffmpeg 调用失败: {0}")]
    Ffmpeg(String),

    #[error("TTS 失败: {0}")]
    Tts(String),

    #[error("找不到资源: {0}")]
    NotFound(String),

    #[error("参数无效: {0}")]
    InvalidArgument(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("其他: {0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

/// 暴露给前端时序列化为字符串。
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
