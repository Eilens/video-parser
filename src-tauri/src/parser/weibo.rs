use crate::models::{Author, ImgInfo, VideoParseInfo};
use crate::parser::utils;
use anyhow::{anyhow, Result};
use regex::Regex;
use reqwest::header::{CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use reqwest::Client;
use serde_json::Value;
use url::Url;

pub struct Weibo;

impl Weibo {
    pub async fn parse_share_url(share_url: &str) -> Result<VideoParseInfo> {
        let url_str = if let Some(u) = utils::regexp_match_url_from_string(share_url) {
            u
        } else {
            share_url.to_string()
        };

        let url_info = Url::parse(&url_str)?;

        // Handle video URLs
        if url_str.contains("show?fid=") {
            let video_id = url_info
                .query_pairs()
                .find(|(k, _)| k == "fid")
                .map(|(_, v)| v.to_string())
                .ok_or_else(|| anyhow!("Cannot parse video id from share url"))?;
            return Self::parse_video_id(&video_id).await;
        } else if url_str.contains("/tv/show/") {
            let video_id = url_info.path().replace("/tv/show/", "");
            return Self::parse_video_id(&video_id).await;
        } else {
            // Handle regular post URLs
            let path_parts: Vec<&str> = url_info.path().trim_matches('/').split('/').collect();
            if path_parts.len() >= 2 {
                let post_id = path_parts.last().unwrap().to_string();
                return Self::parse_post_url(&post_id, &url_str).await;
            }
        }

        Err(anyhow!("Unsupported weibo url format"))
    }

    async fn parse_video_id(video_id: &str) -> Result<VideoParseInfo> {
        let req_url = format!(
            "https://h5.video.weibo.com/api/component?page=/show/{}",
            video_id
        );

        let client = Client::new();
        let res = client
            .post(&req_url)
            .header(COOKIE, "login_sid_t=6b652c77c1a4bc50cb9d06b24923210d; cross_origin_proto=SSL; WBStorage=2ceabba76d81138d|undefined; _s_tentry=passport.weibo.com; Apache=7330066378690.048.1625663522444; SINAGLOBAL=7330066378690.048.1625663522444; ULV=1625663522450:1:1:1:7330066378690.048.1625663522444:; TC-V-WEIBO-G0=35846f552801987f8c1e8f7cec0e2230; SUB=_2AkMXuScYf8NxqwJRmf8RzmnhaoxwzwDEieKh5dbDJRMxHRl-yT9jqhALtRB6PDkJ9w8OaqJAbsgjdEWtIcilcZxHG7rw; SUBP=0033WrSXqPxfM72-Ws9jqgMF55529P9D9W5Qx3Mf.RCfFAKC3smW0px0; XSRF-TOKEN=JQSK02Ijtm4Fri-YIRu0-vNj")
            .header(REFERER, format!("https://h5.video.weibo.com/show/{}", video_id))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, utils::DEFAULT_USER_AGENT)
            .body(format!(r#"data={{"Component_Play_Playinfo":{{"oid":"{}"}}}}"#, video_id))
            .send()
            .await?;

        let json: Value = res.json().await?;
        let data = json
            .pointer("/data/Component_Play_Playinfo")
            .ok_or_else(|| anyhow!("Failed to parse video response"))?;

        // Get highest quality video URL
        let mut video_url = String::new();
        if let Some(urls) = data.get("urls").and_then(|v| v.as_object()) {
            if let Some((_, first_url)) = urls.iter().next() {
                if let Some(url_str) = first_url.as_str() {
                    video_url = format!("https:{}", url_str);
                }
            }
        }

        let cover_url = data
            .get("cover_image")
            .and_then(|v| v.as_str())
            .map(|s| format!("https:{}", s))
            .unwrap_or_default();

        let avatar = data
            .get("avatar")
            .and_then(|v| v.as_str())
            .map(|s| format!("https:{}", s))
            .unwrap_or_default();

        Ok(VideoParseInfo {
            author: Author {
                uid: String::new(),
                name: data
                    .get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                avatar,
            },
            title: data
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            video_url,
            music_url: String::new(),
            cover_url,
            images: vec![],
            platform: "weibo".to_string(),
            video_qualities: vec![],
        })
    }

    async fn parse_post_url(post_id: &str, original_url: &str) -> Result<VideoParseInfo> {
        // Try mobile API first
        let req_url = format!("https://m.weibo.cn/statuses/show?id={}", post_id);
        let client = Client::new();

        let res = client
            .get(&req_url)
            .header(USER_AGENT, "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1")
            .header(REFERER, "https://m.weibo.cn/")
            .header(CONTENT_TYPE, "application/json;charset=UTF-8")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await;

        if let Ok(response) = res {
            let json: Value = response.json().await.unwrap_or_default();
            println!("[Weibo] Mobile API Response: {}", serde_json::to_string_pretty(&json).unwrap_or_default());
            if let Some(data) = json.get("data") {
                let mut result = Self::parse_mobile_api_data(data)?;
                
                // If video_url is a video.weibo.com URL, we need to call the video API to get actual stream URL
                if result.video_url.contains("video.weibo.com/show?fid=") {
                    let re = Regex::new(r"fid=([^&\s]+)").unwrap();
                    if let Some(caps) = re.captures(&result.video_url) {
                        if let Some(video_id) = caps.get(1) {
                            println!("[Weibo] Calling video API for: {}", video_id.as_str());
                            // Call video API to get actual stream URL
                            if let Ok(video_info) = Self::parse_video_id(video_id.as_str()).await {
                                result.video_url = video_info.video_url;
                                if result.cover_url.is_empty() {
                                    result.cover_url = video_info.cover_url;
                                }
                            }
                        }
                    }
                }
                
                return Ok(result);
            }
        }

        // Fallback to desktop page parsing
        let res = client
            .get(original_url)
            .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send()
            .await?;

        let html = res.text().await?;
        Self::parse_html_page(&html)
    }

    fn parse_mobile_api_data(data: &Value) -> Result<VideoParseInfo> {
        let raw_text = data.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let title = Self::clean_text(raw_text);
        
        let author_name = data
            .pointer("/user/screen_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let author_avatar = Self::convert_image_url(
            data
                .pointer("/user/avatar_large")
                .and_then(|v| v.as_str())
                .unwrap_or("")
        );

        // Try to get video URL from page_info
        let mut video_url = String::new();
        let mut cover_url = String::new();
        
        // Check page_info for video content
        if let Some(page_info) = data.get("page_info") {
            // Get video URL from media_info or urls
            if let Some(media_info) = page_info.get("media_info") {
                // Try different quality levels
                video_url = media_info.get("stream_url_hd")
                    .or_else(|| media_info.get("stream_url"))
                    .or_else(|| media_info.get("mp4_720p_mp4"))
                    .or_else(|| media_info.get("mp4_hd_url"))
                    .or_else(|| media_info.get("mp4_sd_url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }
            
            // Get cover image - page_pic can be either a string or an object with url field
            if let Some(page_pic) = page_info.get("page_pic") {
                if let Some(url) = page_pic.as_str() {
                    // page_pic is a string
                    cover_url = Self::convert_image_url(url);
                } else if let Some(url) = page_pic.get("url").and_then(|v| v.as_str()) {
                    // page_pic is an object with url field
                    cover_url = Self::convert_image_url(url);
                }
            }
        }
        
        // If no video found in page_info, try to extract video ID from text and call video API
        if video_url.is_empty() && raw_text.contains("video.weibo.com/show") {
            // Extract video ID from text like: href="https://video.weibo.com/show?fid=1034:5258349667876926"
            let re = Regex::new(r#"video\.weibo\.com/show\?fid=([^"&\s]+)"#).unwrap();
            if let Some(caps) = re.captures(raw_text) {
                if let Some(video_id) = caps.get(1) {
                    println!("[Weibo] Found video ID in text: {}", video_id.as_str());
                    // We can't call async from here, so we'll mark this for the caller to handle
                    // For now, return the video show URL as video_url
                    video_url = format!("https://video.weibo.com/show?fid={}", video_id.as_str());
                }
            }
        }

        // Get images from pic_infos (keyed by pic_id)
        let mut images = Vec::new();
        if let Some(pic_infos) = data.get("pic_infos").and_then(|v| v.as_object()) {
            for (_, pic) in pic_infos {
                // Try to get largest image: largest > original > large > bmiddle
                let large_pic_url = pic
                    .pointer("/largest/url")
                    .or_else(|| pic.pointer("/original/url"))
                    .or_else(|| pic.pointer("/large/url"))
                    .or_else(|| pic.pointer("/bmiddle/url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if !large_pic_url.is_empty() {
                    images.push(ImgInfo {
                        url: Self::convert_image_url(large_pic_url),
                        live_photo_url: None,
                    });
                }
            }
        } else if let Some(pics_array) = data.get("pics").and_then(|v| v.as_array()) {
            // Fallback to pics array if pic_infos doesn't exist
            for pic in pics_array {
                let large_pic_url = pic
                    .pointer("/large/url")
                    .or_else(|| pic.pointer("/original/url"))
                    .or_else(|| pic.pointer("/bmiddle/url"))
                    .or_else(|| pic.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if !large_pic_url.is_empty() {
                    images.push(ImgInfo {
                        url: Self::convert_image_url(large_pic_url),
                        live_photo_url: None,
                    });
                }
            }
        }
        
        println!("[Weibo] Video URL: {}", video_url);
        println!("[Weibo] Cover URL: {}", cover_url);
        println!("[Weibo] Extracted {} images", images.len());
        for (i, img) in images.iter().enumerate() {
            println!("[Weibo] Image {}: {}", i, img.url);
        }

        Ok(VideoParseInfo {
            author: Author {
                uid: String::new(),
                name: author_name,
                avatar: author_avatar,
            },
            title,
            video_url,
            cover_url,
            images,
            platform: "weibo".to_string(),
            music_url: "".to_string(),
            video_qualities: vec![],
        })
    }

    fn parse_html_page(html: &str) -> Result<VideoParseInfo> {
        // Try to extract data from $render_data script
        let re = Regex::new(r#"\$render_data\s*=\s*(.*?)\[0\]"#)?;
        let captures = re
            .captures(html)
            .ok_or_else(|| anyhow!("Failed to parse weibo html page"))?;

        let json_str = format!("{}[0]", captures.get(1).unwrap().as_str());
        let data: Value = serde_json::from_str(&json_str).unwrap_or_default();

        let title = Self::clean_text(
            data.pointer("/status/text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
        );
        let author_name = data
            .pointer("/status/user/screen_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let author_avatar = data
            .pointer("/status/user/avatar_large")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Get images
        let mut images = Vec::new();
        if let Some(pics_array) = data
            .pointer("/status/pics")
            .and_then(|v| v.as_array())
        {
            for pic in pics_array {
                let large_pic_url = pic
                    .pointer("/large/url")
                    .or_else(|| pic.pointer("/original/url"))
                    .or_else(|| pic.pointer("/bmiddle/url"))
                    .or_else(|| pic.get("url"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if !large_pic_url.is_empty() {
                    images.push(ImgInfo {
                        url: large_pic_url.to_string(),
                        live_photo_url: None,
                    });
                }
            }
        }

        Ok(VideoParseInfo {
            author: Author {
                uid: String::new(),
                name: author_name,
                avatar: author_avatar,
            },
            title,
            video_url: String::new(),
            music_url: String::new(),
            cover_url: String::new(),
            images,
            platform: "weibo".to_string(),
            video_qualities: vec![],
        })
    }

    fn clean_text(text: &str) -> String {
        // Remove HTML tags
        let re = Regex::new(r"<[^>]*>").unwrap();
        re.replace_all(text, "").trim().to_string()
    }

    // Convert Weibo image URL to use ww subdomain which may bypass hotlink protection
    fn convert_image_url(url: &str) -> String {
        // Try to convert to large.sinaimg.cn format which has no hotlink protection
        // URL format: https://wx3.sinaimg.cn/mw2000/xxx.jpg -> https://wx3.sinaimg.cn/large/xxx.jpg
        // Or extract image ID and use large.sinaimg.cn
        
        // Extract the image path part (after the size prefix like mw2000, orj360, etc.)
        let re = regex::Regex::new(r"https?://[^/]+\.sinaimg\.cn/(?:mw\d+|orj\d+|large|original|bmiddle|thumbnail|crop[^/]*)/(.+)").unwrap();
        
        if let Some(caps) = re.captures(url) {
            if let Some(image_id) = caps.get(1) {
                // Use large.sinaimg.cn format
                return format!("https://ww1.sinaimg.cn/large/{}", image_id.as_str());
            }
        }
        
        // Fallback: just replace subdomains
        url
            .replace("wx1.sinaimg.cn", "ww1.sinaimg.cn")
            .replace("wx2.sinaimg.cn", "ww1.sinaimg.cn")
            .replace("wx3.sinaimg.cn", "ww1.sinaimg.cn")
            .replace("wx4.sinaimg.cn", "ww1.sinaimg.cn")
            .replace("tvax1.sinaimg.cn", "ww1.sinaimg.cn")
            .replace("tvax2.sinaimg.cn", "ww1.sinaimg.cn")
            .replace("tva1.sinaimg.cn", "ww1.sinaimg.cn")
    }
}
