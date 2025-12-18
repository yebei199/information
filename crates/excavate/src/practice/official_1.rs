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

    // 等待搜索框元素出现并可见，使用更长的时间
    tokio::time::sleep(tokio::time::Duration::from_secs(3))
        .await;

    // 先聚焦到搜索框
    let search_input = page
        .find_element("input#kw")
        .await
        .expect("Failed to find search input");

    // 单独执行每个操作，并添加适当的延迟
    search_input.click().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    search_input.type_str("Rust编程语言").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 找到搜索按钮并点击，而不是按回车键
    let search_button = page
        .find_element("input#su")
        .await
        .expect("Failed to find search button");
    search_button.click().await?;

    // 等待搜索结果页面加载
    page.wait_for_navigation().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let html = page.content().await.expect("Failed to get page content");

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
