use reqwest::Client;
use anyhow::Result;

#[tokio::test]
async fn test_redirect() -> Result<()> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let res = client.get("https://v.douyin.com/iANyYmXn/").send().await?;
    let loc = res.headers().get("location").unwrap().to_str()?;
    println!("LOCATION: {}", loc);
    Ok(())
}
