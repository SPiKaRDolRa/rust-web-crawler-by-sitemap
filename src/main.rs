extern crate reqwest;
extern crate scraper;
extern crate spider;

use reqwest::Error;
use scraper::{Html, Selector};
use spider::tokio;
use spider::website::Website;
use std::fs::write;

#[tokio::main]
async fn main() {
    println!("🚀 Starting Web Crawler...");

    let sitemap_urls = vec![
        "https://www.heygoody.com/sitemap.xml",
        "https://www.heygoody.com/th/sitemap.xml",
        "https://www.heygoody.com/th/sitemap_index.xml",
        "https://www.heygoody.com/th/post-sitemap.xml",
        "https://www.heygoody.com/th/page-sitemap.xml",
    ];

    let mut all_urls = Vec::new();

    for sitemap in &sitemap_urls {
        if let Ok(content) = fetch_sitemap_raw(sitemap).await {
            if sitemap.contains("sitemap_index") {
                let sub_sitemaps = extract_sitemap_links(&content);
                for sub_sitemap in sub_sitemaps {
                    if let Ok(sub_content) = fetch_sitemap_raw(&sub_sitemap).await {
                        let urls = extract_urls_from_sitemap(&sub_content);
                        all_urls.extend(urls);
                    }
                }
            } else {
                let urls = extract_urls_from_sitemap(&content);
                all_urls.extend(urls);
            }
        }
    }

    if all_urls.is_empty() {
        println!("⚠️ No URLs found. Check the sitemap availability.");
        return;
    }

    println!("🌐 Found {} URLs to process.", all_urls.len());

    for url in all_urls {
        if is_spa(&url).await {
            fetch_with_chrome(&url).await;
        } else {
            fetch_with_http(&url).await;
        }
    }

    println!("🎉 Web Crawling Completed!");
}

// ✅ โหลด Sitemap ดิบ
async fn fetch_sitemap_raw(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}

// ✅ ดึง Sitemap ย่อยจาก `sitemap index.xml`
fn extract_sitemap_links(xml: &str) -> Vec<String> {
    let document = Html::parse_document(xml);
    let selector = Selector::parse("loc").unwrap();
    document
        .select(&selector)
        .map(|node| node.text().collect::<String>())
        .collect()
}

// ✅ ดึง URLs จาก Sitemap
fn extract_urls_from_sitemap(xml: &str) -> Vec<String> {
    let document = Html::parse_document(xml);
    let selector = Selector::parse("url loc").unwrap();
    document
        .select(&selector)
        .map(|node| node.text().collect::<String>())
        .collect()
}

// ✅ เช็คว่าเป็น SPA หรือไม่
async fn is_spa(url: &str) -> bool {
    let response = reqwest::get(url).await;
    match response {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            body.contains("window.__NUXT__") || body.contains("data-reactroot")
        }
        Err(_) => false,
    }
}

// ✅ Fetch ข้อมูลจาก SSR (HTTP Request)
async fn fetch_with_http(url: &str) {
    let response = reqwest::get(url).await;
    if let Ok(resp) = response {
        let html = resp.text().await.unwrap_or_default();
        let markdown = html_to_markdown(&html);
        let filename = format!("output_{}.md", sanitize_filename(url));
        write(&filename, markdown).expect("Failed to write file");
    }
}

// ✅ Fetch ข้อมูลจาก SPA (Chrome Headless)
async fn fetch_with_chrome(url: &str) {
    let mut website: Website = Website::new(url).with_caching(true).build().unwrap();
    website.crawl().await;

    if let Some(pages) = website.get_pages() {
        if let Some(page) = pages.iter().find(|p| p.get_url() == url) {
            let html = page.get_html();
            let markdown = html_to_markdown(&html);
            let filename = format!("output_{}.md", sanitize_filename(url));
            write(&filename, markdown).expect("Failed to write file");
        }
    }
}

// ✅ แปลง HTML → Markdown
fn html_to_markdown(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("body").unwrap();
    document
        .select(&selector)
        .next()
        .map(|body| body.text().collect::<Vec<_>>().join("\n"))
        .unwrap_or_else(|| "⚠️ No content extracted".to_string())
}

// ✅ ทำให้ชื่อไฟล์ปลอดภัย
fn sanitize_filename(url: &str) -> String {
    url.replace("https://", "")
        .replace("http://", "")
        .replace("/", "_")
        .replace(":", "_")
}
