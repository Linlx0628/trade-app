use crate::db::DbState;
use crate::error::AppError;
use crate::services::search_service::SearchResult;
use crate::services::SearchService;
use tauri::State;

#[tauri::command]
pub fn global_search(
    state: State<'_, DbState>,
    account_id: String,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, AppError> {
    SearchService::search(&state, &account_id, &query, limit.unwrap_or(20))
}
