use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub uid: String,
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImgInfo {
    pub url: String,
    pub live_photo_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoQuality {
    pub quality: String,
    pub video_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoParseInfo {
    pub author: Author,
    pub title: String,
    pub video_url: String,
    pub music_url: String,
    pub cover_url: String,
    pub images: Vec<ImgInfo>,
    pub platform: String,
    pub video_qualities: Vec<VideoQuality>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoPreview {
    pub id: String,
    pub title: String,
    pub cover_url: String,
    pub video_url: String,
    pub is_video: bool,
    pub platform: String,
}
