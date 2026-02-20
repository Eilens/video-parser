use crate::models::{Author, ImgInfo, VideoParseInfo};
use crate::parser::utils;
use anyhow::{anyhow, Result};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use serde_json::Value;

pub struct PiPiXia;

impl PiPiXia {
    pub async fn parse_share_url(share_url: &str) -> Result<VideoParseInfo> {
        // Extract URL from share text if needed
        let url_str = if let Some(u) = utils::regexp_match_url_from_string(share_url) {
            u
        } else {
            share_url.to_string()
        };

        // Follow redirect to get video ID
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let res = client
            .get(&url_str)
            .header(USER_AGENT, utils::DEFAULT_USER_AGENT)
            .send()
            .await?;

        let location = res
            .headers()
            .get("location")
            .ok_or_else(|| anyhow!("No location header found"))?
            .to_str()?;

        // Parse video ID from path: /ppx/item/{videoId} or /item/{videoId}
        let url_parsed = url::Url::parse(location)?;
        let path = url_parsed.path().trim_matches('/');
        // Remove common prefixes
        let video_id = path
            .replace("ppx/item/", "")
            .replace("item/", "");

        println!("[Pipixia] Location: {}", location);
        println!("[Pipixia] Path: {}", path);

        if video_id.is_empty() {
            return Err(anyhow!("Failed to parse video ID from URL"));
        }

        println!("[Pipixia] Video ID: {}", video_id);
        Self::parse_video_id(&video_id).await
    }

    async fn parse_video_id(video_id: &str) -> Result<VideoParseInfo> {
        let api_url = format!(
            "https://api.pipix.com/bds/cell/cell_comment/?offset=0&cell_type=1&api_version=1&cell_id={}&ac=wifi&channel=huawei_1319_64&aid=1319&app_name=super",
            video_id
        );

        println!("[Pipixia] API URL: {}", api_url);

        let client = Client::new();
        let res = client
            .get(&api_url)
            .header(USER_AGENT, utils::DEFAULT_USER_AGENT)
            .send()
            .await?;

        let json: Value = res.json().await?;

        println!("[Pipixia] API Response: {}", serde_json::to_string_pretty(&json).unwrap_or_default());

        // data.cell_comments.0.comment_info.item
        let data = json
            .pointer("/data/cell_comments/0/comment_info/item")
            .ok_or_else(|| anyhow!("Failed to get item data from API response"))?;

        let author_id = data
            .pointer("/author/id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Get images from note.multi_image
        let mut images = Vec::new();
        if let Some(img_array) = data.pointer("/note/multi_image").and_then(|v| v.as_array()) {
            for img in img_array {
                if let Some(url) = img.pointer("/url_list/0/url").and_then(|v| v.as_str()) {
                    if !url.is_empty() {
                        images.push(ImgInfo {
                            url: url.to_string(),
                            live_photo_url: None,
                        });
                    }
                }
            }
        }

        // Get video URL - try video.video_high.url_list.0.url
        let mut video_url = data
            .pointer("/video/video_high/url_list/0/url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Try to get watermark-free video from comments
        if let Some(comments) = data.get("comments").and_then(|v| v.as_array()) {
            for comment in comments {
                let comment_author_id = comment
                    .pointer("/item/author/id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if comment_author_id == author_id {
                    if let Some(comment_video_url) = comment
                        .pointer("/item/video/video_high/url_list/0/url")
                        .and_then(|v| v.as_str())
                    {
                        if !comment_video_url.is_empty() {
                            video_url = comment_video_url.to_string();
                            break;
                        }
                    }
                }
            }
        }

        // If has images, clear video URL (same logic as reference)
        if !images.is_empty() {
            video_url = String::new();
        }

        let title = data
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let cover_url = data
            .pointer("/cover/url_list/0/url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let author = Author {
            uid: author_id,
            name: data
                .pointer("/author/name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            avatar: data
                .pointer("/author/avatar/download_list/0/url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        };

        Ok(VideoParseInfo {
            author,
            title,
            video_url,
            cover_url,
            images,
            platform: "pipixia".to_string(),
            music_url: "".to_string(),
            video_qualities: vec![],
        })
    }
}
