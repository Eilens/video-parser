use reqwest::{Client, header};

#[tokio::test]
async fn test_iesdouyin_redirect() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build().unwrap();
        
    let res = client.get("https://v.douyin.com/iANyYmXn/").send().await.unwrap();
    let location = res.headers().get("location").unwrap().to_str().unwrap();
    println!("LOCATION 1: {}", location);
    
    // Now if it is www.douyin.com, wait. The location is ONLY www.douyin.com...
    // WAIT. If it redirects to www.douyin.com, where did the video ID go?
}
