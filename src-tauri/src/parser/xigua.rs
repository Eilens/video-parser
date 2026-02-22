use crate::models::{Author, VideoParseInfo, VideoQuality};
use crate::parser::utils;
use anyhow::{anyhow, Result};
use base64::Engine;
use reqwest::header::USER_AGENT;
use reqwest::redirect::Policy;
use reqwest::Client;
use serde_json::Value;

pub struct XiGua;

impl XiGua {
    pub async fn parse_share_url(share_url: &str) -> Result<VideoParseInfo> {
        let url_str = if let Some(u) = utils::regexp_match_url_from_string(share_url) {
            u
        } else {
            share_url.to_string()
        };

        // Step 1: Follow redirect to get the numeric item_id
        let client = Client::builder()
            .redirect(Policy::none())
            .build()?;

        let res = client
            .get(&url_str)
            .header(USER_AGENT, utils::DEFAULT_USER_AGENT)
            .send()
            .await?;

        let item_id = if res.status().is_redirection() {
            if let Some(loc) = res.headers().get("location") {
                let location = loc.to_str()?.to_string();
                let path = location.split('?').next().unwrap_or(&location);
                let path = path.trim_end_matches('/');
                path.rsplit('/').next().unwrap_or("").to_string()
            } else {
                return Err(anyhow!("No redirect location found"));
            }
        } else {
            let path = url_str.split('?').next().unwrap_or(&url_str);
            let path = path.trim_end_matches('/');
            path.rsplit('/').next().unwrap_or("").to_string()
        };

        if item_id.is_empty() {
            return Err(anyhow!("Could not parse video ID from URL"));
        }

        Self::parse_video_id(&item_id).await
    }

    async fn parse_video_id(item_id: &str) -> Result<VideoParseInfo> {
        let client = Client::new();

        // Step 1: Get metadata + play_auth_token from Toutiao API
        let api_url = format!("https://m.toutiao.com/i{}/info/", item_id);
        let res = client
            .get(&api_url)
            .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;

        let json: Value = res.json().await?;
        if json.get("success").and_then(|v| v.as_bool()) != Some(true) {
            return Err(anyhow!("Toutiao API returned error"));
        }

        let data = json.get("data").ok_or_else(|| anyhow!("No data in response"))?;
        let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let cover_url = data.get("poster_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let source = data.get("detail_source").and_then(|v| v.as_str()).unwrap_or("").to_string();

        // Step 2: Use play_auth_token_v2 to call Bytedance VOD API
        let mut video_url = String::new();
        let mut video_qualities = Vec::new();

        if let Some(play_auth_token) = data.get("play_auth_token_v2").and_then(|v| v.as_str()) {
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(play_auth_token) {
                if let Ok(token_str) = String::from_utf8(decoded) {
                    if let Ok(token_json) = serde_json::from_str::<Value>(&token_str) {
                        if let Some(get_play_info_token) = token_json.get("GetPlayInfoToken").and_then(|v| v.as_str()) {
                            let vod_url = format!("https://vod.bytedanceapi.com/?{}", get_play_info_token);
                            if let Ok(vod_res) = client
                                .get(&vod_url)
                                .header(USER_AGENT, "Mozilla/5.0")
                                .send()
                                .await
                            {
                                if let Ok(vod_json) = vod_res.json::<Value>().await {
                                    if let Some(play_info_list) = vod_json.get("Result")
                                        .and_then(|v| v.get("Data"))
                                        .and_then(|v| v.get("PlayInfoList"))
                                        .and_then(|v| v.as_array())
                                    {
                                        // Collect all qualities
                                        for info in play_info_list {
                                            let definition = info.get("Definition").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            let main_play_url = info.get("MainPlayUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            
                                            if !main_play_url.is_empty() {
                                                video_qualities.push(VideoQuality {
                                                    quality: definition,
                                                    video_url: main_play_url,
                                                    size: None,
                                                });
                                            }
                                        }
                                        
                                        // Set the best quality as default video_url
                                        // Sort by height (resolution) descending
                                        let mut sorted_infos: Vec<&Value> = play_info_list.iter().collect();
                                        sorted_infos.sort_by(|a, b| {
                                            let h_b = b.get("Height").and_then(|v| v.as_i64()).unwrap_or(0);
                                            let h_a = a.get("Height").and_then(|v| v.as_i64()).unwrap_or(0);
                                            h_b.cmp(&h_a)
                                        });
                                        
                                        if let Some(best) = sorted_infos.first() {
                                            video_url = best.get("MainPlayUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Get author info from media_user
        let author = if let Some(user) = data.get("media_user").and_then(|v| v.as_object()) {
            Author {
                uid: user.get("user_id").map(|v| v.to_string()).unwrap_or_default(),
                name: user.get("screen_name").and_then(|v| v.as_str()).unwrap_or(&source).to_string(),
                avatar: user.get("avatar_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }
        } else {
            Author {
                uid: "".to_string(),
                name: source,
                avatar: "".to_string(),
            }
        };

        Ok(VideoParseInfo {
            author,
            title,
            video_url,
            music_url: "".to_string(),
            cover_url,
            images: vec![],
            platform: "xigua".to_string(),
            video_qualities,
            statistics: None,
            tags: None,
            music_info: None,
            create_time: None,
        })
    }
}
