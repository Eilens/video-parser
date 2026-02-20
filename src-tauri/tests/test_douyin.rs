use app_lib::parser::douyin::DouYin;
use app_lib::models::VideoParseInfo;
use anyhow::Result;

#[tokio::test]
async fn test_douyin_parsing() {
    let res: Result<VideoParseInfo> = DouYin::parse_share_url("https://v.douyin.com/aiozpqHvSIg/").await;
    println!("RESULT: {:?}", res);
    assert!(res.is_ok());
}
