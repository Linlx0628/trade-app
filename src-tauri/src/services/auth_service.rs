use crate::db::user_repo::UserRepo;
use crate::db::DbState;
use crate::error::AppError;
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserBrief};
use tauri::State;

pub struct AuthService;

impl AuthService {
    pub fn ensure_default_user(state: &State<'_, DbState>) -> Result<User, AppError> {
        if let Some(user) = UserRepo::find_default(&state.conn)? {
            return Ok(user);
        }

        let id = uuid::Uuid::new_v4().to_string();
        UserRepo::create(&state.conn, &id, "default", "默认用户", true)
    }

    pub fn get_current_user(state: &State<'_, DbState>, user_id: &str) -> Result<User, AppError> {
        UserRepo::find_by_id(&state.conn, user_id)?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))
    }

    pub fn login(state: &State<'_, DbState>, username: &str) -> Result<User, AppError> {
        let user = UserRepo::find_by_username(&state.conn, username)?
            .ok_or_else(|| AppError::NotFound(format!("用户不存在: {}", username)))?;
        UserRepo::update_login(&state.conn, &user.id)?;
        Ok(user)
    }

    pub fn create_user(state: &State<'_, DbState>, dto: &CreateUserDto) -> Result<User, AppError> {
        if dto.username.trim().is_empty() || dto.display_name.trim().is_empty() {
            return Err(AppError::Validation("用户名和显示名称不能为空".to_string()));
        }

        if UserRepo::find_by_username(&state.conn, &dto.username)?.is_some() {
            return Err(AppError::Validation(format!("用户名已存在: {}", dto.username)));
        }

        let id = uuid::Uuid::new_v4().to_string();
        UserRepo::create(&state.conn, &id, &dto.username, &dto.display_name, false)
    }

    pub fn update_user(state: &State<'_, DbState>, dto: &UpdateUserDto) -> Result<User, AppError> {
        let conn = &state.conn;
        {
            let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
            let now = chrono::Utc::now().to_rfc3339();
            if let Some(ref name) = dto.display_name {
                conn.execute(
                    "UPDATE user SET display_name = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![name, now, dto.id],
                ).map_err(|e| AppError::Database(format!("更新用户失败: {}", e)))?;
            }
        }

        UserRepo::find_by_id(conn, &dto.id)?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))
    }

    pub fn delete_user(state: &State<'_, DbState>, id: &str) -> Result<(), AppError> {
        let user = UserRepo::find_by_id(&state.conn, id)?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;
        if user.is_default {
            return Err(AppError::Validation("不能删除默认用户".to_string()));
        }
        UserRepo::delete(&state.conn, id)
    }

    pub fn list_users(state: &State<'_, DbState>) -> Result<Vec<UserBrief>, AppError> {
        UserRepo::list_all(&state.conn)
    }
}
