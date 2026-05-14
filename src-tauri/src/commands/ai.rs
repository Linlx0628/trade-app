use crate::error::AppError;
use crate::services::ai_service::{AiChatRequest, AiChatResponse, AiService};

#[tauri::command]
pub async fn ai_chat(req: AiChatRequest) -> Result<AiChatResponse, AppError> {
    AiService::chat(req).await
}
