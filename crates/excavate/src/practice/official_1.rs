use futures::StreamExt;

use chromiumoxide::browser::{Browser, BrowserConfig};
use tokio::time::sleep;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create a `Browser` that spawns a `chromium` process running with UI (`with_head()`, headless is default)
    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder().with_head().build()?,
    )
    .await?;

    // spawn a new task that continuously polls the handler
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    // create a new browser page and navigate to the url
    let page = browser
        .new_page("https://www.baidu.com")
        .await
        .expect("Failed to create page");

    // 等待页面加载完成
    page.wait_for_navigation().await?;

    // 等待足够长时间确保页面加载完毕
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 使用JavaScript直接设置搜索框的值并提交表单
    println!("Setting search value via JavaScript...");
    page.evaluate_expression(r#"document.querySelector('input#kw').value = 'Rust编程语言'"#)
        .await?;
    println!("Search value set");
    
    println!("Clicking search button via JavaScript...");
    page.evaluate_expression(r#"document.querySelector('input#su').click()"#)
        .await?;
    println!("Search button clicked");

    // 等待搜索结果页面加载
    println!("Waiting for navigation...");
    page.wait_for_navigation().await?;
    println!("Navigation completed");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("Getting page content...");
    let html = page.content().await.expect("Failed to get page content");
    println!("Got page content, length: {}", html.len());

    browser.close().await?;
    handle.await?;
    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test1() -> anyhow::Result<()> {
        main()
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }
}
