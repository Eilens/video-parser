use anyhow::Result;

#[tokio::test]
async fn debug_douyin_mobile_ua() -> Result<()> {
    let video_id = "7227408198167186721";
    let mobile_ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1";
    
    // Use douyin.com (not iesdouyin.com)
    let url = format!("https://www.douyin.com/share/video/{}", video_id);
    println!("Testing: {}", url);
    
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, mobile_ua)
        .send()
        .await?;
    
    let body = res.text().await?;
    println!("Body length: {}", body.len());
    println!("Has _ROUTER_DATA: {}", body.contains("_ROUTER_DATA"));
    
    if body.contains("_ROUTER_DATA") {
        let re = regex::Regex::new(r"window._ROUTER_DATA\s*=\s*(.*?)</script>").unwrap();
        if let Some(caps) = re.captures(&body) {
            if let Some(json_str) = caps.get(1) {
                let json_text = json_str.as_str().trim();
                let full_data: serde_json::Value = serde_json::from_str(json_text)?;
                if let Some(loader_data) = full_data.get("loaderData").and_then(|v| v.as_object()) {
                    for (key, value) in loader_data {
                        if key.contains("/page") {
                            println!("Key: {}", key);
                            if let Some(item) = value.get("videoInfoRes")
                                .and_then(|v| v.get("item_list"))
                                .and_then(|v| v.as_array())
                                .and_then(|arr| arr.get(0))
                            {
                                println!("SUCCESS!");
                                println!("desc: {}", item.get("desc").and_then(|v| v.as_str()).unwrap_or("N/A"));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
