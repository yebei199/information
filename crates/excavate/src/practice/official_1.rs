use futures::StreamExt;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::dom::GetBoxModelParams;
use chromiumoxide::cdp::browser_protocol::input::MouseButton;
use chromiumoxide::layout::Point;
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
    // 注意：这里必须传入 node_id
    let result = page
        .execute(GetBoxModelParams::builder().node_id(ele1.node_id).build())
        .await?;
    let model = &result.model;
    let q = &model.content; // 假设这就是那 8 个数的数组
    // 尝试使用 std::mem::transmute 将 Quad 转换为 [f64; 8]
    let coords: &[f64; 8] = unsafe { std::mem::transmute(q) };
    // 计算中心点 X = (x1 + x2 + x3 + x4) / 4
    let center_x = (coords[0] + coords[2] + coords[4] + coords[6]) / 4.0;
    // 计算中心点 Y = (y1 + y2 + y3 + y4) / 4
    let center_y = (coords[1] + coords[3] + coords[5] + coords[7]) / 4.0;
    dbg!(center_x, center_y);
    // 1. 移动鼠标到目标位置
    page.move_mouse(Point::new(center_x, center_y)).await?;

    // 2. 点击 (这一步其实封装了 down + up，但在底层没有延迟，速度极快)
    page.click(Point::new(center_x, center_y)).await?;

    let html = ele1
        .inner_text()
        .await?;
    debug!("Element text: {}", html.unwrap_or_else(|| "No text found".to_string()));
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
