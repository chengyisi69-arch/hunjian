use crate::ai::config::AiConfig;
use crate::error::AppResult;
use reqwest::Client;
use serde_json::json;

pub async fn generate_title(description: &str, config: &AiConfig) -> AppResult<String> {
    if config.llm_api_key.is_empty() {
        return Err(crate::error::AppError::InvalidArgument(
            "LLM API Key 未配置，请先在设置中填写".into(),
        ));
    }
    let prompt = format!(
        "根据以下视频内容描述，生成3个吸引人的短视频标题，每个标题不超过20字，直接输出标题列表，每行一个，不要编号和多余解释：\n{}",
        description
    );
    call_llm(&prompt, config).await
}

pub async fn generate_script(description: &str, config: &AiConfig) -> AppResult<String> {
    if config.llm_api_key.is_empty() {
        return Err(crate::error::AppError::InvalidArgument(
            "LLM API Key 未配置，请先在设置中填写".into(),
        ));
    }
    let prompt = format!(
        "根据以下主题，写一段30秒短视频的中文旁白文案，风格生动有趣，适合口语朗读，字数约150字，直接输出文案内容不要多余解释：\n{}",
        description
    );
    call_llm(&prompt, config).await
}

async fn call_llm(prompt: &str, config: &AiConfig) -> AppResult<String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| crate::error::AppError::Other(format!("HTTP 客户端创建失败: {e}")))?;

    let body = json!({
        "model": config.llm_model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.8,
    });

    let response = client
        .post(&config.llm_api_url)
        .header("Authorization", format!("Bearer {}", config.llm_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("LLM API 请求失败: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误响应".into());
        return Err(crate::error::AppError::Other(format!(
            "LLM API 返回错误 ({}): {}",
            status, text
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("LLM 响应解析失败: {e}")))?;

    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        return Err(crate::error::AppError::Other("LLM 返回空内容".into()));
    }

    Ok(text)
}
