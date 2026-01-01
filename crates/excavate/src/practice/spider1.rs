use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;
use std::env;
use std::path::Path;
use std::time::Duration;

pub struct JobSpider {
    website: Website,
}

impl JobSpider {
    pub fn new(url: &str) -> Self {
        Self {
            website: Website::new(url),
        }
    }

    /// 设置环境变量，确保能找到 Chrome
    /// # Safety
    /// modifying environment variables is not thread safe.
    /// ensure this is called before spawning threads.
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
                    // SAFETY: 在 Rust 2024 中 set_var 被标记为 unsafe，因为它在多线程环境下不安全。
                    // 这里我们只在程序初始化阶段调用它。
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

    pub fn configure(&mut self) {
        let intercept_conf =
            RequestInterceptConfiguration::new(true);

        self.website
            .configuration
            .with_respect_robots_txt(false) // 忽略 robots.txt
            .with_chrome_intercept(intercept_conf, &None)
            .with_stealth(true) // 绕过 WebDriver 检测
            .with_wait_for_delay(Some(spider::features::chrome_common::WaitForDelay::new(
                Some(Duration::from_secs(6)),
            )))
            .with_user_agent(Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")); // 模拟真实 UA
    }

    pub async fn run(&mut self) {
        println!("正在启动 Headless Chrome 抓取...");
        self.website.crawl().await;
        self.process_results();
    }

    fn process_results(&self) {
        if let Some(pages) = self.website.get_pages() {
            if pages.is_empty() {
                println!(
                    "列表为空，尝试直接访问第一页内容..."
                );
            } else {
                for page in pages {
                    let html = page.get_html();
                    if html.contains("用户受限")
                        || html.contains("security-check")
                    {
                        let preview_len =
                            200.min(html.len());
                        println!(
                            "失败：触发了 IP 封禁或验证码。内容片段: {}",
                            &html[..preview_len]
                        );
                    } else {
                        println!(
                            "成功获取！URL: {}",
                            page.get_url()
                        );
                        println!(
                            "HTML 长度: {}",
                            html.len()
                        );
                    }
                }
            }
        }
    }
}

async fn main() {
    JobSpider::setup_environment();
    let url = "https://www.zhipin.com/web/geek/jobs?city=101280100&query=rust%E5%BC%80%E5%8F%91";
    let mut spider = JobSpider::new(url);
    spider.configure();
    spider.run().await;
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
