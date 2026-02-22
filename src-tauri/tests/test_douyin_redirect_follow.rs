use reqwest::{Client, header};

#[tokio::test]
async fn test_with_ac_nonce() {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1"));
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    
    let client = Client::builder()
        .cookie_store(true)
        .default_headers(headers)
        .build().unwrap();
        
    let res = client.get("https://v.douyin.com/iANyYmXn/").send().await.unwrap();
    println!("FINAL URL: {}", res.url());
}
