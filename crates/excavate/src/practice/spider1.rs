//noinspection
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;

async fn main() {
    let url = "https://www.zhipin.com/web/geek/jobs?city=101280100&query=rust%E5%BC%80%E5%8F%91";
    let mut website = Website::new(url);

    let intercept_conf =
        RequestInterceptConfiguration::new(true);

    website.configuration
        .with_chrome_intercept(intercept_conf, &None)
        .with_stealth(true) // 绕过 WebDriver 检测
        .with_wait_for_delay(Some(spider::features::chrome_common::WaitForDelay::new(Some(std::time::Duration::from_secs(1)))))
        .with_user_agent(Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")); // 模拟真实 UA

    println!("正在启动 Headless Chrome 抓取...");

    // 启动抓取
    website.crawl().await;

    // 检查结果
    if let Some(pages) = website.get_pages() {
        if pages.is_empty() {
            println!("列表为空，尝试直接访问第一页内容...");
        } else {
            for page in pages {
                let html = page.get_html();
                if html.contains("用户受限")
                    || html.contains("security-check")
                {
                    println!(
                        "失败：触发了 IP 封禁或验证码。内容片段: {}",
                        &html[..200]
                    );
                } else {
                    println!(
                        "成功获取！URL: {}",
                        page.get_url()
                    );
                    println!("HTML 长度: {}", html.len());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test1() -> anyhow::Result<()> {
        main().await;
        Ok(())
    }
}
