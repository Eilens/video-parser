use crate::models::{Author, ImgInfo, VideoParseInfo};
use crate::parser::utils;
use anyhow::{anyhow, Result};
use reqwest::header::{USER_AGENT, REFERER};
use reqwest::Client;
use serde_json::Value; // Make sure to use Value from serde_json
use std::time::Duration;
use url::Url;

pub struct Bilibili;

impl Bilibili {
    const BILI_USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36";

    pub async fn parse_share_url(share_url: &str) -> Result<VideoParseInfo> {
        let url_str = if let Some(u) = utils::regexp_match_url_from_string(share_url) {
            u
        } else {
            share_url.to_string()
        };
        
        let mut bvid = String::new();
        
        // 1. Handle b23.tv redirects
        if url_str.contains("b23.tv") {
           bvid = Self::get_bvid_from_short_url(&url_str).await?;
        } else {
           bvid = Self::get_bvid_from_url(&url_str)?;
        }
        
        if bvid.is_empty() {
            return Err(anyhow!("Could not find BVID in URL"));
        }

        let client = Client::new();

        // 2. Get Video Metadata (View API)
        let view_api = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid);
        let view_res = client.get(&view_api)
            .header(USER_AGENT, Self::BILI_USER_AGENT)
            .send()
            .await?;
            
        let view_json: Value = view_res.json().await?;
        
        if view_json["code"].as_i64().unwrap_or(-1) != 0 {
             return Err(anyhow!("Bilibili API Error: {}", view_json["message"]));
        }
        
        let data = &view_json["data"];
        let title = data["title"].as_str().unwrap_or("").to_string();
        let pic = data["pic"].as_str().unwrap_or("").to_string();
        let cid = data["cid"].as_i64().or_else(|| data["pages"][0]["cid"].as_i64()).unwrap_or(0);
        let owner = &data["owner"];
        let author_name = owner["name"].as_str().unwrap_or("").to_string();
        let author_avatar = owner["face"].as_str().unwrap_or("").to_string();
        let _mid = owner["mid"].as_i64().unwrap_or(0).to_string();

        // 3. Get Video Stream URL (Play API)
        // qn=80 (1080P), platform=html5
        let play_api = format!(
            "https://api.bilibili.com/x/player/playurl?otype=json&fnver=0&fnval=0&qn=80&bvid={}&cid={}&platform=html5",
            bvid, cid
        );
        
        let play_res = client.get(&play_api)
             .header(USER_AGENT, Self::BILI_USER_AGENT)
             .header(REFERER, "https://www.bilibili.com/") // Important for some videos
             .send()
             .await?;
             
        let play_json: Value = play_res.json().await?;
        
        let mut video_url = String::new();
        if let Some(durl) = play_json["data"]["durl"].as_array() {
            if let Some(first_url) = durl.get(0) {
                 video_url = first_url["url"].as_str().unwrap_or("").to_string();
            }
        }
        
        if video_url.is_empty() {
             return Err(anyhow!("Could not find video stream URL"));
        }

        Ok(VideoParseInfo {
            author: Author {
                uid: _mid,
                name: author_name,
                avatar: author_avatar,
            },
            title,
            video_url,
            cover_url: pic,
            images: vec![],
            platform: "bilibili".to_string(),
            music_url: "".to_string(),
            video_qualities: vec![],
        })
    }
    
    // Extracted from reference: b23.tv redirection
    async fn get_bvid_from_short_url(short_url: &str) -> Result<String> {
         let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
            
         // Check if it's already a full URL? No, ensure we have a protocol
         let url = if !short_url.starts_with("http") {
             format!("https://{}", short_url)
         } else {
             short_url.to_string()
         };

         let res = client.get(&url)
            .header(USER_AGENT, Self::BILI_USER_AGENT)
            .send()
            .await?;
            
         if res.status().is_redirection() {
             if let Some(loc) = res.headers().get("location") {
                 let full_url = loc.to_str()?.to_string();
                 return Self::get_bvid_from_url(&full_url);
             }
         }
         
         Err(anyhow!("Could not resolve b23.tv short link"))
    }
    
    fn get_bvid_from_url(full_url: &str) -> Result<String> {
        let parsed = Url::parse(full_url)?;
        // Path should be like /video/BVxxxxxxxx
        if let Some(path_segments) = parsed.path_segments() {
            let segments: Vec<&str> = path_segments.collect();
            // find "video" segment, next one is BV
            for (i, segment) in segments.iter().enumerate() {
                if *segment == "video" && i + 1 < segments.len() {
                    let bvid = segments[i+1];
                    if bvid.starts_with("BV") {
                        // Strip query params handled by Url parse, but BVID might have trailing slash?
                        // segments are split by slash, so it should be clean.
                        return Ok(bvid.to_string());
                    }
                }
            }
            
            // Sometimes path is just /BVxxxx (if mobile?)
            for segment in segments {
                if segment.starts_with("BV") {
                    return Ok(segment.to_string());
                }
            }
        }
        
        Err(anyhow!("Could not find BVID in path"))
    }
}
