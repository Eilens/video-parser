use std::fs;
use anyhow::Result;
use reqwest::header::USER_AGENT;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    // 7339794018267122998 is another video ID
    let req_url = "https://www.douyin.com/share/video/7339794018267122998";
    let client = reqwest::Client::new();
    
    let res = client
        .get(req_url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await?;
        
    let res_body = res.text().await?;
    let re = regex::Regex::new(r"window._ROUTER_DATA\s*=\s*(.*?)</script>").unwrap();
    if let Some(caps) = re.captures(&res_body) {
        let json_text = caps.get(1).unwrap().as_str().trim();
        let full_data: Value = serde_json::from_str(json_text)?;
        
        if let Some(loader_data) = full_data.get("loaderData").and_then(|v| v.as_object()) {
            for (key, value) in loader_data {
                if key.contains("/page") && key.starts_with("video_") {
                    if let Some(item) = value.get("videoInfoRes")
                        .and_then(|v| v.get("item_list"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.get(0)) 
                    {
                        if let Some(video) = item.get("video") {
                            println!("Bit Rate Array:");
                            if let Some(bit_rate) = video.get("bit_rate").and_then(|v| v.as_array()) {
                                for br in bit_rate {
                                    let gear_name = br.get("gear_name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                    let quality_type = br.get("quality_type").and_then(|v| v.as_i64()).unwrap_or(-1);
                                    let play_addr = br.get("play_addr").and_then(|v| v.get("url_list")).and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| v.as_str()).unwrap_or("");
                                    println!("Gear: {}, Quality: {}, URL: {}", gear_name, quality_type, play_addr);
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
