use crate::error::AppError;
use crate::models::user::{User, UserBrief};
use rusqlite::params;
use std::sync::Mutex;
use rusqlite::Connection;

pub struct UserRepo;

impl UserRepo {
    pub fn find_by_username(conn: &Mutex<Connection>, username: &str) -> Result<Option<User>, AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, avatar, is_default, last_login_at, created_at, updated_at FROM user WHERE username = ?1"
        )?;

        let result = stmt.query_row(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                avatar: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
                last_login_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }

    pub fn find_by_id(conn: &Mutex<Connection>, id: &str) -> Result<Option<User>, AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, avatar, is_default, last_login_at, created_at, updated_at FROM user WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                avatar: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
                last_login_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }

    pub fn find_default(conn: &Mutex<Connection>) -> Result<Option<User>, AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, avatar, is_default, last_login_at, created_at, updated_at FROM user WHERE is_default = 1"
        )?;

        let result = stmt.query_row([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                avatar: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
                last_login_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }

    pub fn list_all(conn: &Mutex<Connection>) -> Result<Vec<UserBrief>, AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, avatar, is_default FROM user ORDER BY created_at"
        )?;

        let users = stmt.query_map([], |row| {
            Ok(UserBrief {
                id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                avatar: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    pub fn create(conn: &Mutex<Connection>, id: &str, username: &str, display_name: &str, is_default: bool) -> Result<User, AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO user (id, username, display_name, is_default, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, username, display_name, is_default as i32, now, now],
        ).map_err(|e| AppError::Database(format!("创建用户失败: {}", e)))?;

        Ok(User {
            id: id.to_string(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            avatar: None,
            is_default,
            last_login_at: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn update_login(conn: &Mutex<Connection>, id: &str) -> Result<(), AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE user SET last_login_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        ).map_err(|e| AppError::Database(format!("更新登录时间失败: {}", e)))?;
        Ok(())
    }

    pub fn delete(conn: &Mutex<Connection>, id: &str) -> Result<(), AppError> {
        let conn = conn.lock().map_err(|e| AppError::Database(format!("锁失败: {}", e)))?;
        conn.execute("DELETE FROM user WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(format!("删除用户失败: {}", e)))?;
        Ok(())
    }
}
