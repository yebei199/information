use futures::StreamExt;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Page;
use log::{debug, error};

async fn main()
-> anyhow::Result<(), Box<dyn std::error::Error>> {
    utils::tools::log::init_logger();
    // create a `Browser` that spawns a `chromium` process running with UI (`with_head()`, headless is default)
    // and the handler that drives the websocket etc.
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder().with_head().build()?,
    )
    .await?;

    // spawn a new task that continuously polls the handler
    let handle = tokio::task::spawn(async move {
        loop {
            match handler.next().await {
                Some(Ok(_event)) => {
                    // Handle events if needed
                }
                Some(Err(e)) => {
                    error!("Error: {:?}", e);
                    break;
                }
                None => break,
            }
        }
    });

    // create a new browser page and navigate to the url
    let page = browser
        .new_page("https://www.baidu.com")
        .await
        .expect("Failed to create page");
    page.wait_for_navigation().await?;
    let ele1 = page.find_element("#hotsearch-content-wrapper li:nth-of-type(5) a span:nth-of-type(2)").await?;

    let html = ele1
        .inner_text()
        .await
        .expect("Failed to get element text")
        .expect("Failed to get element text");
    debug!("hhh{}", html);
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
