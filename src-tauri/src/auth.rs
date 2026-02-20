use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[tauri::command]
pub fn register(
    state: State<DbState>,
    username: String,
    password: String,
    email: String,
) -> Result<UserInfo, String> {
    if username.trim().is_empty() || password.trim().is_empty() || email.trim().is_empty() {
        return Err("All fields are required".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Check if username already exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM users WHERE username = ?1",
            rusqlite::params![username],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if exists {
        return Err("Username already exists".to_string());
    }

    conn.execute(
        "INSERT INTO users (username, password, email) VALUES (?1, ?2, ?3)",
        rusqlite::params![username, password, email],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    println!("[auth] User registered: {} (id: {})", username, id);

    Ok(UserInfo {
        id,
        username,
        email,
    })
}

#[tauri::command]
pub fn login(
    state: State<DbState>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    if username.trim().is_empty() || password.trim().is_empty() {
        return Err("Username and password are required".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let user = conn
        .query_row(
            "SELECT id, username, email FROM users WHERE username = ?1 AND password = ?2",
            rusqlite::params![username, password],
            |row| {
                Ok(UserInfo {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                })
            },
        )
        .map_err(|_| "Invalid username or password".to_string())?;

    println!("[auth] User logged in: {} (id: {})", user.username, user.id);
    Ok(user)
}

#[tauri::command]
pub fn update_profile(
    state: State<DbState>,
    id: i64,
    new_username: Option<String>,
    new_password: Option<String>,
) -> Result<UserInfo, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(ref name) = new_username {
        if name.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        // Check if new username is taken by another user
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM users WHERE username = ?1 AND id != ?2",
                rusqlite::params![name, id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if exists {
            return Err("Username already exists".to_string());
        }
        conn.execute(
            "UPDATE users SET username = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref pwd) = new_password {
        if pwd.trim().is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        conn.execute(
            "UPDATE users SET password = ?1 WHERE id = ?2",
            rusqlite::params![pwd, id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Return updated user info
    let user = conn
        .query_row(
            "SELECT id, username, email FROM users WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                Ok(UserInfo {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    println!(
        "[auth] Profile updated: {} (id: {})",
        user.username, user.id
    );
    Ok(user)
}

#[tauri::command]
pub fn reset_password(
    state: State<DbState>,
    username: String,
    email: String,
    new_password: String,
) -> Result<(), String> {
    if username.trim().is_empty() || email.trim().is_empty() || new_password.trim().is_empty() {
        return Err("All fields are required".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let affected = conn
        .execute(
            "UPDATE users SET password = ?1 WHERE username = ?2 AND email = ?3",
            rusqlite::params![new_password, username, email],
        )
        .map_err(|e| e.to_string())?;

    if affected == 0 {
        return Err("User not found or email mismatch".to_string());
    }

    println!("[auth] Password reset for user: {}", username);
    Ok(())
}
