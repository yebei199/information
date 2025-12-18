use futures::StreamExt;

use chromiumoxide::browser::{Browser, BrowserConfig};
use tokio::time::sleep;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 需要创建tokio运行时来运行异步代码
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_native())
}

async fn run_native() -> Result<(), Box<dyn std::error::Error>> {
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

    // 等待足够长的时间确保元素加载
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // 多次尝试查找元素直到成功
    let search_input = loop {
        match page.find_element("input#kw").await {
            Ok(element) => break element,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                continue;
            }
        }
    };

    // 确保元素可交互
    search_input.scroll_into_view().await.ok();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 分开执行操作，不使用链式调用
    search_input.click().await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    search_input.type_str("Rust编程语言").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 同样多次尝试查找搜索按钮
    let search_button = loop {
        match page.find_element("input#su").await {
            Ok(element) => break element,
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                continue;
            }
        }
    };

    search_button.click().await?;

    // 等待搜索结果页面加载
    page.wait_for_navigation().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

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
        run_native()
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }
}