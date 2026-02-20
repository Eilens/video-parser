use anyhow::Result;
use regex::Regex;
use reqwest::header::USER_AGENT;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    let video_id = "7605439172665996282";
    // Check old behavior (which had SSL issue for user):
    let req_url = format!("https://www.douyin.com/share/video/{}", video_id);
    let client = reqwest::Client::new();
    
    let res = client
        .get(&req_url)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await?;
        
    let res_body = res.text().await?;
    let re = Regex::new(r"window._ROUTER_DATA\s*=\s*(.*?)</script>").unwrap();
    if let Some(caps) = re.captures(&res_body) {
        println!("SUCCESS: FOUND ROUTER DATA");
    } else {
        println!("FAIL: NO ROUTER DATA FOUND");
        // Print snippets
        println!("Body snippet: {}", &res_body[0..std::cmp::min(200, res_body.len())]);
    }

    Ok(())
}
