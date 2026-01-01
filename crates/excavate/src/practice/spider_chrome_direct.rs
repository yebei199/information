use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::env;
use std::path::Path;

async fn run_direct() -> anyhow::Result<()> {
    // Setup env
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    if env::var("CHROME_BIN").is_err() {
        let possible_paths = [
            "/run/current-system/sw/bin/google-chrome-stable",
            "/usr/bin/google-chrome-stable",
        ];
        for path in possible_paths {
            if Path::new(path).exists() {
                unsafe {
                    env::set_var("CHROME_BIN", path);
                }
                println!("Chrome path set to: {}", path);
                break;
            }
        }
    }

    println!("Launching browser directly...");

    // Explicitly disable sandbox which is often needed
    let builder = BrowserConfig::builder()
        .with_head() // Try with head first (though in CI/ssh it might fail, but let's see) or remove for headless
        .args(vec!["--headless=new"]);

    let (mut browser, mut handler) = Browser::launch(
        builder.build().map_err(|e| anyhow::anyhow!(e))?,
    )
    .await?;

    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    println!("Navigating...");
    let page = browser.new_page("https://www.zhipin.com/web/geek/jobs?city=101280100&query=rust%E5%BC%80%E5%8F%91").await?;
    page.wait_for_navigation().await?;

    let content = page.content().await?;
    println!("Content fetched length: {}", content.len());

    browser.close().await?;
    handle.await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_direct() -> anyhow::Result<()> {
        run_direct().await
    }
}
