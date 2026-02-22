use app_lib::parser::douyin::DouYin;

#[tokio::test]
async fn test_parse_id() {
    let res = DouYin::parse_video_id("7464082269229042971").await;
    println!("RESULT: {:?}", res);
    assert!(res.is_ok());
    let info = res.unwrap();
    println!("VIDEO URL: {}", info.video_url);
    for q in info.video_qualities {
        println!("  - QUALITY: {}, URL: {}", q.quality, q.video_url);
    }
}
