use reqwest;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.get("https://www.douyin.com/share/video/7605439172665996282")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send().await?;
    let text = res.text().await?;
    if text.contains("_ROUTER_DATA") {
        println!("SUCCESS: found _ROUTER_DATA");
    } else {
        println!("FAILED: no _ROUTER_DATA");
    }
    Ok(())
}
