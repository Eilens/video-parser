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
    pub size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoStatistics {
    pub likes: Option<u64>,
    pub views: Option<u64>,
    pub favorites: Option<u64>,
    pub shares: Option<u64>,
    pub comments: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MusicInfo {
    pub title: String,
    pub author: String,
    pub url: String,
    pub cover_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoParseInfo {
    pub author: Author,
    pub title: String,
    pub video_url: String,
    pub music_url: String, // Keeping this for backward compatibility or we can just keep music_info
    pub cover_url: String,
    pub images: Vec<ImgInfo>,
    pub platform: String,
    pub video_qualities: Vec<VideoQuality>,
    pub statistics: Option<VideoStatistics>,
    pub tags: Option<Vec<String>>,
    pub music_info: Option<MusicInfo>,
    pub create_time: Option<u64>,
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
