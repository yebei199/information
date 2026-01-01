use anyhow::Result;
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;
use std::env;
use std::path::Path;

pub struct JobSpider;

impl JobSpider {
    /// 环境初始化：自动检测 Chrome 路径
    pub fn setup_environment() {
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
                    // SAFETY: 初始化阶段设置环境变量
                    unsafe {
                        env::set_var("CHROME_BIN", path);
                    }
                    println!(
                        "已自动设置 Chrome 路径: {}",
                        path
                    );
                    break;
                }
            }
        }
    }

    /// 按照官方示例模式执行抓取
    pub async fn crawl_website(url: &str) -> Result<()> {
        // 使用链式调用构建 website 实例
        // 注意：根据 spider v2 签名，with_chrome_intercept 通常需要两个参数
        let mut website: Website = Website::new(url)
            .with_limit(10)
            .with_chrome_intercept(RequestInterceptConfiguration::new(true) )
            .with_stealth(true)
            .with_user_agent(Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"))
            .build()
            .unwrap();

        // 订阅抓取结果频道
        let mut rx2 = website.subscribe(16).unwrap();

        // 启动后台任务处理页面
        let handle = tokio::spawn(async move {
            while let Ok(page) = rx2.recv().await {
                let html = page.get_html();
                println!(
                    "收到页面: {:?} | 长度:מד {}",
                    page.get_url(),
                    html.len()
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
            "\n抓取完成！URL: {} | 总耗时: {:?} | 总访问页数: מד {}",
            url,
            duration,
            links.len()
        );

        Ok(())
    }
}

async fn main() -> Result<()> {
    JobSpider::setup_environment();
    let url = "https://www.zhipin.com/web/geek/jobs?city=101280100&query=rust%E5%BC%80%E5%8F%91";
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
