use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;

async fn main() {
    let url = "https://www.zhipin.com/web/geek/jobs?city=101280100&query=rust%E5%BC%80%E5%8F%91";
    let mut website = Website::new(url);

    // 获取配置句柄
    website.configuration
        .with_chrome_intercept(
            RequestInterceptConfiguration::new(true),
            &None  // 使用None作为第二个参数，类型为&Option<Box<url::Url>>
        )    // 启用无头浏览器模式
        .with_stealth(true)         // 必须开启！绕过 Boss 直聘的 WebDriver 检测
        .with_user_agent(Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")); // 模拟真实 UA

    // 启动抓取
    website.crawl().await;

    // 打印结果
    let pages = website.get_pages();
    if let Some(p) = pages {
        for page in p {
            println!(
                "成功获取页面内容，长度: {}",
                page.get_html().len()
            );
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
