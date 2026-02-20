use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Favorite {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub platform: String,
    pub cover_url: String,
    pub author_name: String,
    pub created_at: String,
}

#[tauri::command]
pub fn add_favorite(
    state: State<DbState>,
    user_id: i64,
    url: String,
    title: String,
    platform: String,
    cover_url: String,
    author_name: String,
) -> Result<Favorite, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO favorites (user_id, url, title, platform, cover_url, author_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![user_id, url, title, platform, cover_url, author_name],
    ).map_err(|e| e.to_string())?;

    // Return the inserted/existing favorite
    let fav = conn.query_row(
        "SELECT id, url, title, platform, cover_url, author_name, created_at FROM favorites WHERE user_id = ?1 AND url = ?2",
        rusqlite::params![user_id, url],
        |row| {
            Ok(Favorite {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                platform: row.get(3)?,
                cover_url: row.get(4)?,
                author_name: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    println!(
        "[favorites] Added favorite for user {}: {} ({})",
        user_id, fav.title, fav.platform
    );
    Ok(fav)
}

#[tauri::command]
pub fn remove_favorite(state: State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM favorites WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    println!("[favorites] Removed favorite id: {}", id);
    Ok(())
}

#[tauri::command]
pub fn get_favorites(
    state: State<DbState>,
    user_id: i64,
    platform: Option<String>,
) -> Result<Vec<Favorite>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &platform {
        Some(p) if !p.is_empty() && p != "all" => (
            "SELECT id, url, title, platform, cover_url, author_name, created_at FROM favorites WHERE user_id = ?1 AND platform = ?2 ORDER BY created_at DESC",
            vec![Box::new(user_id), Box::new(p.clone())],
        ),
        _ => (
            "SELECT id, url, title, platform, cover_url, author_name, created_at FROM favorites WHERE user_id = ?1 ORDER BY created_at DESC",
            vec![Box::new(user_id)],
        ),
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let favorites = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(Favorite {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                platform: row.get(3)?,
                cover_url: row.get(4)?,
                author_name: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(favorites)
}

#[tauri::command]
pub fn is_favorited(state: State<DbState>, user_id: i64, url: String) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM favorites WHERE user_id = ?1 AND url = ?2",
            rusqlite::params![user_id, url],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}
