// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod models;
mod parser;
mod db;
mod favorites;
mod auth;

use crate::models::VideoParseInfo;
use crate::parser::{douyin::DouYin, xhs::Xiaohongshu, pipixia::PiPiXia, weibo::Weibo, kuaishou::Kuaishou, bilibili::Bilibili, xigua::XiGua};
use tauri::Manager;

#[tauri::command]
async fn parse_video(app: tauri::AppHandle, url: String) -> Result<VideoParseInfo, String> {
    if url.contains("douyin.com") || url.contains("iesdouyin.com") {
        // Use HTTP-based parsing (no webview needed)
        DouYin::parse_share_url(&url).await.map_err(|e| e.to_string())
        
    } else if url.contains("xhslink.com") || url.contains("xiaohongshu.com") {
        Xiaohongshu::parse_share_url(&url).await.map_err(|e| e.to_string())
    } else if url.contains("pipix.com") {
        PiPiXia::parse_share_url(&url).await.map_err(|e| e.to_string())
    } else if url.contains("weibo.com") || url.contains("weibo.cn") {
        Weibo::parse_share_url(&url).await.map_err(|e| e.to_string())
    } else if url.contains("kuaishou.com") || url.contains("chenzhongtech.com") {
        Kuaishou::parse_share_url(&url).await.map_err(|e| e.to_string())
    } else if url.contains("bilibili.com") || url.contains("b23.tv") {
        Bilibili::parse_share_url(&url).await.map_err(|e| e.to_string())
    } else if url.contains("ixigua.com") {
        XiGua::parse_share_url(&url).await.map_err(|e| e.to_string())
    } else {
        Err("Unsupported URL".to_string())
    }
}

#[tauri::command]
async fn download_file(url: String, save_path: String) -> Result<String, String> {
    use std::io::Write;

    let client = reqwest::Client::new();
    let res = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let content = res.bytes().await.map_err(|e| e.to_string())?;
    
    let mut file = std::fs::File::create(&save_path).map_err(|e| e.to_string())?;
    file.write_all(&content).map_err(|e| e.to_string())?;

    Ok(save_path)
}

// Proxy image through backend to bypass hotlink protection
// Returns base64 data URL that can be used in img src
#[tauri::command]
async fn proxy_image(url: String) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};
    
    println!("[proxy_image] Proxying URL: {}", url);
    
    let client = reqwest::Client::new();
    let res = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", "https://weibo.com/")
        .send()
        .await
        .map_err(|e| {
            println!("[proxy_image] Request failed: {}", e);
            e.to_string()
        })?;

    let status = res.status();
    println!("[proxy_image] Response status: {}", status);
    
    if !status.is_success() {
        return Err(format!("HTTP error: {}", status));
    }

    let content_type = res.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = res.bytes().await.map_err(|e| e.to_string())?;
    println!("[proxy_image] Downloaded {} bytes, content-type: {}", bytes.len(), content_type);
    
    let base64_data = general_purpose::STANDARD.encode(&bytes);
    
    // Return as data URL
    Ok(format!("data:{};base64,{}", content_type, base64_data))
}

#[tauri::command]
async fn cache_video(app: tauri::AppHandle, url: String) -> Result<String, String> {
    use std::io::Write;
    use tauri::Manager;
    
    println!("[cache_video] Caching URL: {}", url);
    
    // Create a temp directory for videos if not exists
    let temp_dir = app.path().temp_dir().map_err(|e| e.to_string())?.join("tauri_video_cache");
    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    }
    
    // Generate filename from URL hash
    use md5::{Md5, Digest};
    let mut hasher = Md5::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    let filename = format!("{:x}.mp4", result);
    let file_path = temp_dir.join(&filename);
    
    // Return existing file if already cached
    if file_path.exists() {
        println!("[cache_video] Video already cached: {:?}", file_path);
        return Ok(file_path.to_string_lossy().to_string());
    }
    
    let client = reqwest::Client::new();
    let res = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", "https://weibo.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Download failed: {}", res.status()));
    }
    
    let content = res.bytes().await.map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(&content).map_err(|e| e.to_string())?;
    
    println!("[cache_video] Video cached to: {:?}", file_path);
    Ok(file_path.to_string_lossy().to_string())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let conn = db::init_db(&app.handle()).expect("Failed to initialize database");
            app.manage(db::DbState(std::sync::Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_video,
            download_file,
            proxy_image,
            cache_video,
            favorites::add_favorite,
            favorites::remove_favorite,
            favorites::get_favorites,
            favorites::is_favorited,
            auth::register,
            auth::login,
            auth::update_profile,
            auth::reset_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
