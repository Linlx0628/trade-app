use crate::db::DbState;
use crate::error::AppError;
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserBrief};
use crate::services::auth_service::AuthService;
use tauri::State;

#[tauri::command]
pub fn get_current_user(state: State<'_, DbState>, user_id: String) -> Result<User, AppError> {
    AuthService::get_current_user(&state, &user_id)
}

#[tauri::command]
pub fn login(state: State<'_, DbState>, username: String) -> Result<User, AppError> {
    AuthService::login(&state, &username)
}

#[tauri::command]
pub fn create_user(state: State<'_, DbState>, dto: CreateUserDto) -> Result<User, AppError> {
    AuthService::create_user(&state, &dto)
}

#[tauri::command]
pub fn update_user(state: State<'_, DbState>, dto: UpdateUserDto) -> Result<User, AppError> {
    AuthService::update_user(&state, &dto)
}

#[tauri::command]
pub fn delete_user(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    AuthService::delete_user(&state, &id)
}

#[tauri::command]
pub fn list_users(state: State<'_, DbState>) -> Result<Vec<UserBrief>, AppError> {
    AuthService::list_users(&state)
}

#[tauri::command]
pub fn ensure_default_user(state: State<'_, DbState>) -> Result<User, AppError> {
    AuthService::ensure_default_user(&state)
}
