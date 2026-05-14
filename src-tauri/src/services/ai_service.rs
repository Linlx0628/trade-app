use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatRequest {
    pub config: AiConfig,
    pub messages: Vec<AiMessage>,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_temperature() -> f64 { 0.7 }
fn default_max_tokens() -> u32 { 2000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatResponse {
    pub content: String,
    pub model: String,
    pub usage: AiUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub struct AiService;

impl AiService {
    pub async fn chat(req: AiChatRequest) -> Result<AiChatResponse, AppError> {
        if req.config.api_key.trim().is_empty() {
            return Err(AppError::Validation("API Key 不能为空".to_string()));
        }

        match req.config.provider.as_str() {
            "anthropic" | "claude" => Self::call_anthropic(&req).await,
            "openai" => Self::call_openai(&req).await,
            _ => Err(AppError::Validation(format!("不支持的 AI 提供商: {}", req.config.provider))),
        }
    }

    async fn call_anthropic(req: &AiChatRequest) -> Result<AiChatResponse, AppError> {
        let base_url = if req.config.base_url.is_empty() {
            "https://api.anthropic.com".to_string()
        } else {
            req.config.base_url.clone()
        };
        let model = if req.config.model.is_empty() { "claude-sonnet-4-20250514".to_string() } else { req.config.model.clone() };

        let messages: Vec<serde_json::Value> = req.messages.iter().map(|m| {
            serde_json::json!({ "role": m.role, "content": m.content })
        }).collect();

        let body = serde_json::json!({
            "model": model,
            "max_tokens": req.max_tokens,
            "temperature": req.temperature,
            "messages": messages,
        });

        let client = reqwest::Client::new();
        let resp = client.post(format!("{}/v1/messages", base_url))
            .header("x-api-key", &req.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("AI API 请求失败: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Database(format!("AI API 返回错误 ({}): {}", status, text)));
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| AppError::Database(format!("解析 AI 响应失败: {}", e)))?;

        let content = json["content"][0]["text"].as_str().unwrap_or("").to_string();
        let usage_input = json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
        let usage_output = json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(AiChatResponse {
            content,
            model,
            usage: AiUsage { prompt_tokens: usage_input, completion_tokens: usage_output },
        })
    }

    async fn call_openai(req: &AiChatRequest) -> Result<AiChatResponse, AppError> {
        let base_url = if req.config.base_url.is_empty() {
            "https://api.openai.com".to_string()
        } else {
            req.config.base_url.clone()
        };
        let model = if req.config.model.is_empty() { "gpt-4o".to_string() } else { req.config.model.clone() };

        let messages: Vec<serde_json::Value> = req.messages.iter().map(|m| {
            serde_json::json!({ "role": m.role, "content": m.content })
        }).collect();

        let body = serde_json::json!({
            "model": model,
            "max_tokens": req.max_tokens,
            "temperature": req.temperature,
            "messages": messages,
        });

        let client = reqwest::Client::new();
        let resp = client.post(format!("{}/v1/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", req.config.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("AI API 请求失败: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Database(format!("AI API 返回错误 ({}): {}", status, text)));
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| AppError::Database(format!("解析 AI 响应失败: {}", e)))?;

        let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
        let usage_input = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let usage_output = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(AiChatResponse {
            content,
            model,
            usage: AiUsage { prompt_tokens: usage_input, completion_tokens: usage_output },
        })
    }
}
