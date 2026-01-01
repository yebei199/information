use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::handler::viewport::Viewport;
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

    // 配置浏览器：有头模式，桌面分辨率
    let builder = BrowserConfig::builder()
        .with_head() // 开启有头模式 (显示浏览器窗口)
        .viewport(Viewport {
            width: 1920,
            height: 1080,
            device_scale_factor: Some(1.0),
            emulating_mobile: false,
            is_landscape: true,
            has_touch: false,
        })
        .build()
        .map_err(|e| anyhow::anyhow!(e))?;

    let (mut browser, mut handler) = Browser::launch(builder).await?;

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

    // --- 调试技巧 ---
    // 在这里暂停，允许你手动检查浏览器状态，或者在 IDE 中设置断点。
    println!("页面已加载。按 [回车键] 继续执行，或在此处打断点进行调试...");
    let _ = tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    })
    .await;
    // ----------------

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
