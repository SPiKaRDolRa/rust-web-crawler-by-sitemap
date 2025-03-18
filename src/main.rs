extern crate reqwest;
extern crate scraper;
extern crate spider;

use reqwest::Error;
use scraper::{Html, Selector};
use spider::tokio;
use spider::website::Website;
use std::fs::{create_dir_all, write};
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("🚀 Starting Web Crawler...");
    let start_time = Instant::now();

    let sitemap_urls = vec![
        "https://www.heygoody.com/sitemap.xml",
        "https://www.heygoody.com/th/sitemap.xml",
        "https://www.heygoody.com/th/sitemap_index.xml",
        "https://www.heygoody.com/th/post-sitemap.xml",
        "https://www.heygoody.com/th/page-sitemap.xml",
    ];

    let mut all_urls = Vec::new();
    let mut spa_urls = Vec::new();
    let mut ssr_urls = Vec::new();

    // ✅ 1. ดึง URLs ทั้งหมดจาก Sitemap
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

    // ✅ 2. แยก URLs เป็น SPA และ SSR
    for url in all_urls.iter() {
        if is_spa(url).await {
            spa_urls.push(url.clone());
        } else {
            ssr_urls.push(url.clone());
        }
    }

    // ✅ 3. จัดเก็บข้อมูล Markdown เป็นกลุ่มใน `/all-markdown/{category}/`
    for url in all_urls.iter() {
        let category = categorize_url(url);
        let dir_path = format!("all-markdown/{}", category);
        create_dir_all(&dir_path).expect("Failed to create directory");

        let filename = format!("{}/{}.md", dir_path, sanitize_filename(url));

        if spa_urls.contains(url) {
            fetch_with_chrome(url, &filename).await;
        } else {
            fetch_with_http(url, &filename).await;
        }
    }

    // ✅ 4. สร้างไฟล์ `summary.txt`
    let elapsed_time = start_time.elapsed();
    let summary_content = format!(
        "🌐 Total URLs: {}\nSPA URLs: {}\nSSR URLs: {}\n⏳ Total Crawl Time: {:.2?}",
        all_urls.len(),
        spa_urls.len(),
        ssr_urls.len(),
        elapsed_time
    );

    write("all-markdown/summary.txt", summary_content).expect("Failed to write summary file");

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
async fn fetch_with_http(url: &str, filename: &str) {
    let response = reqwest::get(url).await;
    if let Ok(resp) = response {
        let html = resp.text().await.unwrap_or_default();
        let markdown = html_to_markdown(&html);
        write(filename, markdown).expect("Failed to write file");
    }
}

// ✅ Fetch ข้อมูลจาก SPA (Chrome Headless)
async fn fetch_with_chrome(url: &str, filename: &str) {
    let mut website: Website = Website::new(url).with_caching(true).build().unwrap();
    website.crawl().await;

    if let Some(pages) = website.get_pages() {
        if let Some(page) = pages.iter().find(|p| p.get_url() == url) {
            let html = page.get_html();
            let markdown = html_to_markdown(&html);
            write(filename, markdown).expect("Failed to write file");
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

// ✅ จัดกลุ่ม URL เป็น Directory (ตามประเภทของ URL)
fn categorize_url(url: &str) -> String {
    if url.contains("/blog") {
        "blogs".to_string()
    } else if url.contains("/product") {
        "products".to_string()
    } else if url.contains("/news") {
        "news".to_string()
    } else {
        "others".to_string()
    }
}
