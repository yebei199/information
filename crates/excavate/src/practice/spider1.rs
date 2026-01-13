use anyhow::Result;
use spider::configuration::{ChromeEventTracker, WebAutomation};
use spider::features::chrome_common::{RequestInterceptConfiguration, Viewport};
use spider::hashbrown::HashMap;
use spider::page::Page;
use spider::website::Website;

pub struct JobSpider;

impl JobSpider {
    /// 使用 WebAutomation 执行复杂页面交互的抓取示例
    pub async fn crawl_website(url: &str) -> Result<()> {
        // 1. 定义自动化脚本
        // 我们可以为不同的 URL 路径定义不同的操作序列
        let mut automation_scripts = HashMap::new();

        automation_scripts.insert(
            "/".into(), // 匹配根路径
            Vec::from([
                // 等待搜索框出现
                WebAutomation::WaitFor(".css-19ol0kv".into()), // 这里的选择器仅为示例
                // 输入关键词
                WebAutomation::Fill {
                    selector: "input.Input".into(),
                    value: "Rust 编程".into(),
                },
                // 等待一下
                WebAutomation::Wait(1000),
                // 执行自定义 JS (例如高亮某个元素)
                WebAutomation::Evaluate(
                    r#"document.querySelector('input.Input').style.border = '5px solid red';"#.into(),
                ),
                // 模拟点击搜索按钮 (或者按回车)
                WebAutomation::Click(".SearchBar-searchButton".into()),
                // 等待跳转完成
                WebAutomation::Wait(3000),
                // 滚动页面
                WebAutomation::ScrollY(1000),
            ]),
        );

        // 2. 配置事件追踪器 (必须开启 automation)
        let mut tracker = ChromeEventTracker::new(true, true);
        tracker.automation = true;

        // 3. 构建 Website 实例
        let mut website: Website = Website::new(url)
            .with_limit(1) // 限制抓取页面数量
            .with_chrome_intercept(RequestInterceptConfiguration::new(true))
            .with_viewport(Some(Viewport::new(1920, 1080))) // 设置桌面级窗口
            .with_event_tracker(Some(tracker))
            .with_automation_scripts(Some(automation_scripts))
            .with_stealth(true)
            .build()
            .expect("build the website fail");

        // 订阅结果
        let mut rx2 = website.subscribe(16).unwrap();

        let handle = tokio::spawn(async move {
            while let Ok(page) = rx2.recv().await {
                let page: Page = page;
                println!(
                    "收到页面: {:?} | 长度: {}",
                    page.get_url(),
                    page.get_html().len()
                );
            }
        });

        println!("开始自动化抓取: {}", url);
        website.crawl().await;

        website.unsubscribe();
        let _ = handle.await;

        Ok(())
    }
}

async fn main() -> Result<()> {
    // 以知乎为例 (仅作演示自动化流程)
    let url = "https://www.zhihu.com";
    JobSpider::crawl_website(url).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test1() -> anyhow::Result<()> {
        main().await?;
        Ok(())
    }
}
