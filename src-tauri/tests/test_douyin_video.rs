use app_lib::parser::douyin::DouYin;
use app_lib::models::VideoParseInfo;
use anyhow::Result;

#[tokio::test]
async fn test_douyin_video_parsing() {
    let url = "https://v.douyin.com/iANyYmXn/";
    let res: Result<VideoParseInfo> = DouYin::parse_share_url(url).await;
    println!("RESULT: {:?}", res);
    assert!(res.is_ok());
    let info = res.unwrap();
    assert!(!info.video_url.is_empty(), "video_url should not be empty");
}
