use crate::ai::config::AiConfig;
use crate::error::AppResult;
use reqwest::Client;
use serde_json::json;
use std::path::PathBuf;

pub async fn generate_image(prompt: &str, config: &AiConfig) -> AppResult<PathBuf> {
    if config.image_api_key.is_empty() {
        return Err(crate::error::AppError::InvalidArgument(
            "图片 API Key 未配置，请先在设置中填写".into(),
        ));
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| crate::error::AppError::Other(format!("HTTP 客户端创建失败: {e}")))?;

    let body = json!({
        "model": config.image_model,
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024",
    });

    let response = client
        .post(&config.image_api_url)
        .header("Authorization", format!("Bearer {}", config.image_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("图片 API 请求失败: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "无法读取错误响应".into());
        return Err(crate::error::AppError::Other(format!(
            "图片 API 返回错误 ({}): {}",
            status, text
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("图片响应解析失败: {e}")))?;

    let image_url = json["data"][0]["url"]
        .as_str()
        .ok_or_else(|| crate::error::AppError::Other("图片响应中缺少 URL".into()))?;

    // 下载图片
    let img_data = client
        .get(image_url)
        .send()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("下载图片失败: {e}")))?
        .bytes()
        .await
        .map_err(|e| crate::error::AppError::Other(format!("读取图片数据失败: {e}")))?;

    // 保存到临时目录
    let tmp_dir = std::env::temp_dir().join(format!("hunjian-ai-{}-img.jpg", rand::random::<u32>()));
    std::fs::write(&tmp_dir, &img_data)
        .map_err(|e| crate::error::AppError::Other(format!("保存图片失败: {e}")))?;

    Ok(tmp_dir)
}
