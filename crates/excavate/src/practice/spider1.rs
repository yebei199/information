use anyhow::Result;
use spider::features::chrome_common::{RequestInterceptConfiguration, WaitForDelay};
use spider::website::Website;
use std::time::Duration;

pub struct JobSpider;

impl JobSpider {
    /// 按照官方示例模式执行抓取
    pub async fn crawl_website(url: &str) -> Result<()> {
        // 使用链式调用构建 website 实例
        let mut website: Website = Website::new(url)
            .with_limit(5)
            .with_chrome_intercept(RequestInterceptConfiguration::new(true))
            .with_wait_for_delay(Some(WaitForDelay::new(Some(Duration::from_secs(10))))) // 暂停/等待 10 秒
            .with_stealth(true)
            .with_user_agent(Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"))
            .build()
            .expect("build the website fail");

        // 订阅抓取结果频道
        let mut rx2 = website.subscribe(16).unwrap();

        // 启动后台任务处理页面
        let handle = tokio::spawn(async move {
            while let Ok(page) = rx2.recv().await {
                let html = page.get_html();
                println!(
                    "收到页面: {:?} | 长度:מד {}\n{}",
                    page.get_url(),
                    html.len(),
                    html.as_str()
                );

                if html.contains("用户受限")
                    || html.contains("security-check")
                {
                    println!(
                        "警告：触发反爬验证 (Slider/Security Check)"
                    );
                }
            }
        });

        let start = tokio::time::Instant::now();

        // 开始抓取
        website.crawl().await;

        // 停止订阅
        website.unsubscribe();
        let _ = handle.await;

        let duration = start.elapsed();

        // 获取所有访问过的链接 (异步方法)
        let links = website.get_all_links_visited().await;

        println!(
            "\n抓取完成！URL: {} | 总耗时: {:?} | 总访问页数: {}",
            url,
            duration,
            links.len()
        );

        Ok(())
    }
}

async fn main() -> Result<()> {
    // let url = "https://www.zhipin.com/web/geek/jobs?city=101280100&query=rust%E5%BC%80%E5%8F%91";
    let url = "https://www.zhihu.com";
    JobSpider::crawl_website(url).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test1() -> Result<()> {
        main().await
    }
}
