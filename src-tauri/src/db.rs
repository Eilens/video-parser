use rusqlite::{Connection, Result};
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

pub fn init_db(app: &tauri::AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    use tauri::Manager;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&app_data_dir)?;
    let db_path = app_data_dir.join("favorites.db");
    println!("[db] Database path: {:?}", db_path);

    let conn = Connection::open(db_path)?;

    // Migration: check if favorites table has user_id column
    let has_user_id = conn
        .prepare("PRAGMA table_info(favorites)")
        .and_then(|mut stmt| {
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            Ok(columns.contains(&"user_id".to_string()))
        })
        .unwrap_or(false);

    if !has_user_id {
        // Drop old table and recreate with new schema
        println!("[db] Migrating: recreating favorites table with user_id");
        conn.execute_batch("DROP TABLE IF EXISTS favorites;")?;
    }

    // Migration: check if downloads table has user_id column
    let has_download_user_id = conn
        .prepare("PRAGMA table_info(downloads)")
        .and_then(|mut stmt| {
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            Ok(columns.contains(&"user_id".to_string()))
        })
        .unwrap_or(false);

    if !has_download_user_id {
        println!("[db] Migrating: recreating downloads table with user_id");
        conn.execute_batch("DROP TABLE IF EXISTS downloads;")?;
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS favorites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL DEFAULT 0,
            url TEXT NOT NULL,
            title TEXT NOT NULL,
            platform TEXT NOT NULL,
            cover_url TEXT DEFAULT '',
            author_name TEXT DEFAULT '',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, url)
        );
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS downloads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL DEFAULT 0,
            url TEXT NOT NULL,
            title TEXT NOT NULL,
            cover_url TEXT DEFAULT '',
            file_path TEXT NOT NULL,
            status TEXT NOT NULL,
            total_size INTEGER DEFAULT 0,
            downloaded_size INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    println!("[db] Database initialized successfully");
    Ok(conn)
}
