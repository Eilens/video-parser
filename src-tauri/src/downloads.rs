use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{State, Manager};
use crate::db::DbState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: i64,
    pub user_id: i64,
    pub url: String,
    pub title: String,
    pub cover_url: String,
    pub file_path: String,
    pub status: String,
    pub total_size: i64,
    pub downloaded_size: i64,
    pub created_at: String,
}

#[tauri::command]
pub async fn get_downloads(
    state: State<'_, DbState>,
    user_id: i64,
) -> Result<Vec<DownloadRecord>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare("SELECT id, user_id, url, title, cover_url, file_path, status, total_size, downloaded_size, created_at FROM downloads WHERE user_id = ? ORDER BY id DESC")
        .map_err(|e| e.to_string())?;
        
    let iter = stmt
        .query_map([user_id], |row| {
            Ok(DownloadRecord {
                id: row.get(0)?,
                user_id: row.get(1)?,
                url: row.get(2)?,
                title: row.get(3)?,
                cover_url: row.get(4)?,
                file_path: row.get(5)?,
                status: row.get(6)?,
                total_size: row.get(7)?,
                downloaded_size: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut downloads = Vec::new();
    for download in iter {
        if let Ok(dl) = download {
            downloads.push(dl);
        }
    }
    Ok(downloads)
}

#[tauri::command]
pub async fn remove_download_record(
    state: State<'_, DbState>,
    id: i64,
    delete_file: bool,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    
    if delete_file {
        // First get the file_path
        let file_path: Result<String, _> = conn.query_row(
            "SELECT file_path FROM downloads WHERE id = ?",
            [id],
            |row| row.get(0),
        );
        
        if let Ok(path_str) = file_path {
            if !path_str.is_empty() {
                // Ignore error if file doesn't exist or we don't have permission
                let _ = std::fs::remove_file(path_str);
            }
        }
    }

    conn.execute(
        "DELETE FROM downloads WHERE id = ?",
        [id],
    )
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

// Helper function to insert a new download record
pub fn create_download_record(
    conn: &mut std::sync::MutexGuard<'_, rusqlite::Connection>,
    user_id: i64,
    url: &str,
    title: &str,
    cover_url: &str,
    file_path: &str,
    status: &str,
) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO downloads (user_id, url, title, cover_url, file_path, status, total_size, downloaded_size) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0)",
        rusqlite::params![user_id, url, title, cover_url, file_path, status],
    )?;
    Ok(conn.last_insert_rowid())
}

// Helper function to update download progress
pub fn update_download_progress(
    conn: &mut std::sync::MutexGuard<'_, rusqlite::Connection>,
    id: i64,
    downloaded_size: i64,
    total_size: i64,
    status: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE downloads SET downloaded_size = ?1, total_size = ?2, status = ?3 WHERE id = ?4",
        rusqlite::params![downloaded_size, total_size, status, id],
    )?;
    Ok(())
}
