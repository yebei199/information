//! crawl astroturfers from X_based_china
use super::to_db::Model;
use scraper::Selector;

struct XCrawl;
impl XCrawl {
    fn url_1(page: u32) -> anyhow::Result<String> {
        let base_url =
            "https://pluto0x0.github.io/X_based_china";
        let url = if page == 1 {
            format!("{}/", base_url)
        } else {
            format!("{}/page{}.html", base_url, page)
        };
        Ok(url)
    }
    fn all_url() -> anyhow::Result<Vec<String>> {
        let max_page = 48;
        let urls = (1..=max_page)
            .map(Self::url_1)
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(urls)
    }
}

struct ParseHtml {
    url: String,
    user_card_selector: Selector,
    name_selector: Selector,
    handle_selector: Selector,
    id_selector: Selector,
    profile_url_selector: Selector,
    avatar_selector: Selector,
    meta_selector: Selector,
}
impl ParseHtml {
    async fn muti_url_parse(
        urls: Vec<String>,
    ) -> anyhow::Result<Vec<Model>> {
        use futures::stream::{self, StreamExt};

        let mut results = Vec::new();
        let parsers = urls.into_iter().map(ParseHtml::new);

        let mut stream = stream::iter(parsers)
            .map(|parser| async move {
                parser.parse_html().await
            })
            .buffer_unordered(10);

        while let Some(result) = stream.next().await {
            let items = result?;
            results.extend(items);
        }

        Ok(results)
    }

    fn new(url: String) -> Self {
        Self {
            url,
            user_card_selector: Selector::parse(
                "article.user-card",
            )
            .unwrap(),
            name_selector: Selector::parse("h2.user-name")
                .unwrap(),
            handle_selector: Selector::parse(
                "div.user-handle",
            )
            .unwrap(),
            id_selector: Selector::parse("div.user-id")
                .unwrap(),
            profile_url_selector: Selector::parse(
                ".user-avatar-wrap a",
            )
            .unwrap(),
            avatar_selector: Selector::parse(
                "img.user-avatar",
            )
            .unwrap(),
            meta_selector: Selector::parse(
                "div.user-meta span",
            )
            .unwrap(),
        }
    }
    async fn parse_html(
        &self,
    ) -> anyhow::Result<Vec<Model>> {
        let client = reqwest::Client::new();
        let res = client.get(&self.url).send().await?;
        let doc = scraper::Html::parse_document(
            &res.text().await?,
        );

        let mut astroturfers_list = Vec::new();

        for element in doc.select(&self.user_card_selector)
        {
            let name = element
                .select(&self.name_selector)
                .next()
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
                .unwrap_or_default();

            let handle = element
                .select(&self.handle_selector)
                .next()
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
                .unwrap_or_default();

            let id = element
                .select(&self.id_selector)
                .next()
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .trim()
                        .replace("ID: ", "")
                })
                .unwrap_or_default();

            let profile_url = element
                .select(&self.profile_url_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .map(String::from)
                .unwrap_or_default();

            let avatar = element
                .select(&self.avatar_selector)
                .next()
                .and_then(|e| e.value().attr("src"))
                .map(String::from)
                .unwrap_or_default();

            let meta: Vec<String> = element
                .select(&self.meta_selector)
                .map(|e| {
                    e.text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
                .collect();

            let mut register_time = String::new();
            let mut changed_name_count = 0;

            for item in meta {
                if item.starts_with("注册：") {
                    register_time =
                        item.replace("注册：", "");
                } else if item.starts_with("改名次数：")
                {
                    changed_name_count = item
                        .replace("改名次数：", "")
                        .parse()
                        .unwrap_or(0);
                }
            }

            let astroturfers = Model {
                name,
                handle,
                user_id: id,
                profile_url,
                avatar,
                register_time,
                changed_name_count,
            };

            astroturfers_list.push(astroturfers);
        }

        Ok(astroturfers_list)
    }
}
struct EndToDB;
impl EndToDB {
    async fn end() -> anyhow::Result<()> {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("PG_DB").expect("PG_DB must be set");
        let db = sea_orm::Database::connect(&database_url).await?;

        let urls = XCrawl::all_url()?;

        use futures::stream::{self, StreamExt};

        // Create a stream that processes pages concurrently
        let mut stream = stream::iter(urls)
            .map(|url| {
                let parser = ParseHtml::new(url);
                async move { parser.parse_html().await }
            })
            .buffer_unordered(5); // Adjust concurrency limit as needed

        while let Some(result) = stream.next().await {
            match result {
                Ok(models) => {
                    if !models.is_empty() {
                        println!("Inserting batch of {} records...", models.len());
                        super::to_db::save_to_db(&db, models).await?;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse page: {:?}", e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::prelude::*;
    use scraper::Html;
    #[tokio::test]
    async fn end() -> anyhow::Result<()> {
        EndToDB::end().await
    }

    #[tokio::test]
    async fn test_1() {
        let urls = XCrawl::all_url().unwrap();
        // 随机获取一个url
        let mut rng = rand::rng();
        let url = urls.choose(&mut rng).unwrap().clone();

        let parse = ParseHtml::new(url);
        let end = parse.parse_html().await.unwrap();
        for astroturfer in end {
            println!("{:?}\n", astroturfer);
        }
    }
    #[test]
    fn parse_html() {
        let html = r#"
    <article class="user-card">
        <div class="user-avatar-wrap">
            <a href="https://twitter.com/ynhu434128" target="_blank" rel="noopener noreferrer">
                <img src="https://pbs.twimg.com/profile_images/1963991803566186496/m8T6UVyR_normal.jpg" alt="烟火（互fo带你看真实的中国） avatar" loading="lazy" class="user-avatar">
            </a>
        </div>
        <div class="user-content">
            <div class="user-title-row">
                <h2 class="user-name" title="烟火（互fo带你看真实的中国）">
                    烟火（互fo带你看真实的中国）
                </h2>
            </div>
            <div class="user-handle">
                @ynhu434128
            </div>
            <div class="user-meta"><span>注册：2024-09-02</span> · <span>地区：China</span> · <span>来源：Web</span> · <span>改名次数：0</span></div>
            <div class="user-id">ID: 1830540823630675969</div>
        </div>
    </article>
    "#;

        let doc = Html::parse_document(html);

        // 头像链接
        let avatar_selector =
            Selector::parse("img.user-avatar").unwrap();
        for img in doc.select(&avatar_selector) {
            if let Some(src) = img.value().attr("src") {
                println!("avatar src: {}", src);
            }
            if let Some(alt) = img.value().attr("alt") {
                println!("avatar alt: {}", alt);
            }
        }

        // 用户名
        let name_selector =
            Selector::parse("h2.user-name").unwrap();
        for name in doc.select(&name_selector) {
            let text: String = name.text().collect();
            println!("user name: {}", text.trim());
        }

        // handle (@xxx)
        let handle_selector =
            Selector::parse("div.user-handle").unwrap();
        for handle in doc.select(&handle_selector) {
            let text: String = handle.text().collect();
            println!("handle: {}", text.trim());
        }

        // meta 信息（注册时间、地区、来源、改名次数）
        let meta_selector =
            Selector::parse("div.user-meta span").unwrap();
        for span in doc.select(&meta_selector) {
            let text: String = span.text().collect();
            println!("meta: {}", text.trim());
        }

        // 用户 ID
        let id_selector =
            Selector::parse("div.user-id").unwrap();
        for id in doc.select(&id_selector) {
            let text: String = id.text().collect();
            println!("user id: {}", text.trim());
        }
    }
    #[test]
    fn all_urls() {
        let urls = XCrawl::all_url().unwrap();
        for url in urls {
            println!("{}", url)
        }
    }

    #[test]
    fn url() {
        let url = XCrawl::url_1(1).unwrap();
        assert_eq!(
            url,
            "https://pluto0x0.github.io/X_based_china/"
        );

        let url = XCrawl::url_1(3).unwrap();
        assert_eq!(
            url,
            "https://pluto0x0.github.io/X_based_china/page3.html"
        );
    }
}
