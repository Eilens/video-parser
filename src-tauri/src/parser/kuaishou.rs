use crate::models::{Author, ImgInfo, VideoParseInfo};
use crate::parser::utils;
use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::header::{USER_AGENT, ACCEPT, COOKIE};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct Kuaishou;

impl Kuaishou {
    const KUAISHOU_USER_AGENT: &'static str = "Mozilla/5.0 (iPhone; CPU iPhone OS 26_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Mobile/15E148 Safari/604.1";

    pub async fn parse_share_url(share_url: &str) -> Result<VideoParseInfo> {
        let url_str = if let Some(u) = utils::regexp_match_url_from_string(share_url) {
            u
        } else {
            share_url.to_string()
        };

        // 1. Initial request to get the redirect location
        // We do this manually or allow redirects until we hit the interesting part.
        // The reference repo sets a custom redirect policy. 
        // Simplest way in Rust: Request with redirects enabled, but inspect final URL?
        // Or disable redirects and loop manually.
        // Kuaishou 'v.kuaishou.com' usually redirects 302.
        
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none()) // Handle redirects manually
            .build()?;

        let mut current_url = url_str.clone();
        let mut final_url = String::new();
        
        // Manual redirect loop to mimic the specific logic if needed, 
        // or just to catch the 'fs/long-video' case before final fetch.
        for _ in 0..5 {
            let res = client.get(&current_url)
                .header(USER_AGENT, Self::KUAISHOU_USER_AGENT)
                .send()
                .await?;

            if res.status().is_redirection() {
                if let Some(loc) = res.headers().get("location") {
                    let mut next_url = loc.to_str()?.to_string();
                    
                    // Specific logic from reference: replace /fw/long-video/ with /fw/photo/
                     if next_url.contains("/fw/long-video/") {
                        next_url = next_url.replace("/fw/long-video/", "/fw/photo/");
                    }
                    
                    current_url = next_url;
                    continue;
                }
            } else {
                final_url = current_url.clone();
                break;
            }
        }
        
        if final_url.is_empty() {
             // If loop finished without breaking (too many redirects?), use the last one
             final_url = current_url;
        }

        // 2. Fetch the final page content
        let res = client.get(&final_url)
            .header(USER_AGENT, Self::KUAISHOU_USER_AGENT)
            .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header(COOKIE, "did=web_d1326127361a7a02596e1e273063544d; didv=1686713337000;") // Basic cookie often helps
            .send()
            .await?;

        let html = res.text().await?;
        
        Self::parse_html(&html)
    }

    fn parse_html(html: &str) -> Result<VideoParseInfo> {
        // Extract window.INIT_STATE JSON
        // Regex: window.INIT_STATE\s*=\s*(.*?)</script>
        let re = Regex::new(r"window\.INIT_STATE\s*=\s*(.*?)</script>")?;
        let cap = re.captures(html).ok_or_else(|| anyhow!("Could not find window.INIT_STATE in Kuaishou page"))?;
        
        let json_str = cap.get(1).unwrap().as_str().trim();
        let data: Value = serde_json::from_str(json_str)?;
        
        // Kuaishou JSON structure:
        // Root is usually a map. We need to find the item that has "photo" and "result" fields?
        // Reference logic: Iterate keys, find one with "photo" and "result".
        // gjson parses bytes map. 
        // Here we have `data` which might be a large Object.
        
        let mut video_data = &Value::Null;
        
        // The structure might be: { "visionVideoDetail": { "photo": ..., "result": 1 }, ... }
        // Or directly the object?
        // Reference iterates over the map.
        
        if let Some(obj) = data.as_object() {
             for (_, v) in obj {
                 if v.get("photo").is_some() && v.get("result").is_some() {
                     video_data = v;
                     break;
                 }
             }
        }
        
        if video_data.is_null() {
             // Fallback: check "visionVideoDetail" directly
             if let Some(v) = data.get("visionVideoDetail") {
                 video_data = v;
             } else {
                 return Err(anyhow!("Could not find video info in Kuaishou JSON"));
             }
        }
        
        let photo = video_data.get("photo").ok_or_else(|| anyhow!("No photo field in Kuaishou data"))?;
        
        let title = photo.get("caption").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let author_name = photo.get("userName").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let author_avatar = photo.get("headUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
        
        let video_url = photo.get("mainMvUrls").and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
            
        let cover_url = photo.get("coverUrls").and_then(|v| v.as_array())
             .and_then(|arr| arr.get(0))
             .and_then(|v| v.get("url"))
             .and_then(|v| v.as_str())
             .unwrap_or("")
             .to_string();

        // Images (Atlas)
        let mut images = Vec::new();
        let image_cdn = photo.get("ext_params").and_then(|v| v.get("atlas")).and_then(|v| v.get("cdn")).and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0)).and_then(|v| v.as_str()).unwrap_or("");
            
        if let Some(list) = photo.get("ext_params").and_then(|v| v.get("atlas")).and_then(|v| v.get("list")).and_then(|v| v.as_array()) {
            if !image_cdn.is_empty() {
                for item in list {
                    if let Some(path) = item.as_str() {
                        let full_url = format!("https://{}/{}", image_cdn, path);
                        images.push(ImgInfo {
                            url: full_url,
                            live_photo_url: None
                        });
                    }
                }
            }
        }

        Ok(VideoParseInfo {
            author: Author {
                uid: "".to_string(), // uid extraction logic if needed
                name: author_name,
                avatar: author_avatar,
            },
            title,
            video_url,
            cover_url,
            images,
            platform: "kuaishou".to_string(),
            music_url: "".to_string(),
            video_qualities: vec![],
        })
    }
}
