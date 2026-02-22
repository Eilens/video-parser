use app_lib::parser::douyin::DouYin;

#[tokio::test]
async fn test_pc_url() {
    let url = "https://www.douyin.com/video/7464082269229042971";
    let res = DouYin::parse_share_url(url).await;
    println!("RESULT: {:?}", res);
    assert!(res.is_ok());
}
