use crate::models::{Author, ImgInfo, VideoParseInfo, VideoPreview};
use regex::Regex;
use serde_json::Value;

use crate::parser::utils;

pub struct Xiaohongshu;

impl Xiaohongshu {
    pub async fn parse_share_url(share_url: &str) -> Result<VideoParseInfo, String> {
        let url = if let Some(u) = utils::regexp_match_url_from_string(share_url) {
            u
        } else {
            share_url.to_string()
        };

        let client = reqwest::Client::new();
        let res = client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .header("Cookie", "abRequestId=0000; webId=0000; gibberish=0000;") // Sometimes needed
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let html = res.text().await.map_err(|e| e.to_string())?;

        // Extract __INITIAL_STATE__
        let re = Regex::new(r"window\.__INITIAL_STATE__\s*=\s*(\{.*?\})\s*</script>").map_err(|e| e.to_string())?;
        let captures = re.captures(&html).ok_or("Could not find __INITIAL_STATE__ in page")?;
        let json_str = captures.get(1).map(|m| m.as_str()).ok_or("Could not find JSON content")?;
        
        // Replace "undefined" which is invalid JSON but often in JS objects
        let clean_json = json_str.replace("undefined", "null");

        let data: Value = serde_json::from_str(&clean_json).map_err(|e| format!("JSON decode error: {}, content snippet: {}", e, &clean_json.chars().take(100).collect::<String>()))?;

        // Navigate JSON path directly to note data
        // Typically: note.noteDetailMap[noteId].note
        // Or sometimes directly in `note` if SSR structure varies.
        // Let's try to find the note object.
        
        let note_data = if let Some(note) = data.get("note").and_then(|n| n.get("note")) {
             // Direct structure
             note
        } else if let Some(note_detail_map) = data.get("note").and_then(|n| n.get("noteDetailMap")) {
            // Map structure, need first key
            if let Some(first_key) = note_detail_map.as_object().and_then(|m| m.keys().next()) {
                note_detail_map.get(first_key).and_then(|v| v.get("note")).ok_or("Empty note detail")?
            } else {
                return Err("Note detail map is empty".to_string());
            }
        } else {
             return Err("Could not find note data in JSON".to_string());
        };

        // Extract Title
        let title = note_data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let desc = note_data.get("desc").and_then(|v| v.as_str()).unwrap_or("");
        let full_title = if title.is_empty() { desc.chars().take(30).collect() } else { title };

        // Extract Author
        let user = note_data.get("user").ok_or("No user info")?;
        let author = Author {
            uid: user.get("userId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: user.get("nickname").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            avatar: user.get("avatar").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        };

        // Extract Cover
        // Usually imageList[0] or cover
        let cover_url = if let Some(il) = note_data.get("imageList").and_then(|v| v.as_array()) {
            if let Some(first) = il.first() {
                first.get("urlDefault").and_then(|v| v.as_str()).unwrap_or("").replace("http://", "https://")
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // Check Type: Video or Image
        let note_type = note_data.get("type").and_then(|v| v.as_str()).unwrap_or("normal");
        
        let mut video_url = String::new();
        let mut images = Vec::new();

        if note_type == "video" {
            // Video
            if let Some(video) = note_data.get("video") {
                 // Try media.stream.h264 -> [0].masterUrl
                 // Or consumer.originVideoKey
                 // Or direct url in older structures
                 
                 // Strategy 1: consumer generic
                 if let Some(consumer) = video.get("consumer") {
                     if let Some(_) = consumer.get("originVideoKey").and_then(|v| v.as_str()) {
                         // This is often just a key, requires host CDN construction? 
                         // Actually consumer often has `http...` url in recent versions?
                         // Let's look for `media` field for direct URLs.
                     }
                 }
                 
                 // Strategy 2: media.stream
                 if let Some(stream) = video.get("media").and_then(|m| m.get("stream")) {
                     if let Some(h264) = stream.get("h264").and_then(|v| v.as_array()) {
                         if let Some(first_fmt) = h264.first() {
                             video_url = first_fmt.get("masterUrl").and_then(|v| v.as_str()).unwrap_or("").replace("http://", "https://");
                         }
                     }
                 }
            }
        } else {
            // Images
            if let Some(img_list) = note_data.get("imageList").and_then(|v| v.as_array()) {
                for img in img_list {
                    let url = img.get("urlDefault").and_then(|v| v.as_str()).unwrap_or("").replace("http://", "https://");
                     // High res might be 'urlPre' or inferred?
                     // urlDefault is usually decent. 
                     // Some logic replaces `!nd_` parts for raw?
                     // Let's stick to urlDefault for now.
                    if !url.is_empty() {
                        images.push(ImgInfo {
                            url,
                            live_photo_url: None
                        });
                    }
                }
            }
        }

        Ok(VideoParseInfo {
            title: full_title,
            author,
            video_url,
            cover_url,
            music_url: "".to_string(),
            images,
            platform: "xhs".to_string(),
            video_qualities: vec![],
        })
    }

    pub async fn fetch_posts(user_id: &str) -> Result<Vec<VideoPreview>, String> {
        let url = format!("https://www.xiaohongshu.com/user/profile/{}", user_id);
        
        let client = reqwest::Client::new();
        let res = client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .header("Cookie", "abRequestId=0000; webId=0000; gibberish=0000;") 
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let html = res.text().await.map_err(|e| e.to_string())?;

        let re = Regex::new(r"window\.__INITIAL_STATE__\s*=\s*(\{.*?\})\s*</script>").map_err(|e| e.to_string())?;
        let captures = re.captures(&html).ok_or("Could not find __INITIAL_STATE__ in page")?;
        let json_str = captures.get(1).map(|m| m.as_str()).ok_or("Could not find JSON content")?;
        
        let clean_json = json_str.replace("undefined", "null");
        let data: Value = serde_json::from_str(&clean_json).map_err(|e| format!("JSON decode error: {}, content snippet: {}", e, &clean_json.chars().take(100).collect::<String>()))?;

        let mut previews = Vec::new();
        
        // Try to find notes list
        // user.notes or similar
        let notes_array = data.get("user").and_then(|u| u.get("notes")).and_then(|v| v.as_array())
             .or_else(|| data.get("user").and_then(|u| u.get("feed")).and_then(|v| v.as_array())); 

        if let Some(list) = notes_array {
            for item in list {
                let id = item.get("noteId").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let title = item.get("displayTitle").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let cover_url = item.get("cover").and_then(|v| v.get("urlDefault").or(v.get("url"))).and_then(|v| v.as_str()).unwrap_or("").replace("http://", "https://");
                let type_str = item.get("type").and_then(|v| v.as_str()).unwrap_or("normal");
                let is_video = type_str == "video";
                
                if !id.is_empty() {
                    previews.push(VideoPreview {
                        id,
                        title,
                        cover_url,
                        video_url: "".to_string(),
                        is_video,
                        platform: "xhs".to_string(),
                    });
                }
            }
        }

        Ok(previews)
    }
}
