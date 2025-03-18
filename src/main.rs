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

    // ✅ ระบุโดเมนหลักที่ต้องการ
    let domain = "https://www.heygoody.com";

    // ✅ ค้นหา Sitemap ด้วยหลายวิธี
    let sitemap_urls = discover_sitemaps(domain).await;

    if sitemap_urls.is_empty() {
        println!("⚠️ No Sitemaps found for {}", domain);
        return;
    }

    let mut all_urls = Vec::new();
    let mut spa_urls = Vec::new();
    let mut ssr_urls = Vec::new();

    // ✅ ดึง URLs ทั้งหมดจาก Sitemap
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
        println!("⚠️ No URLs found from Sitemaps.");
        return;
    }

    println!("🌐 Found {} URLs to process.", all_urls.len());

    // ✅ แยก URLs เป็น SPA และ SSR
    for url in all_urls.iter() {
        if is_spa(url).await {
            spa_urls.push(url.clone());
        } else {
            ssr_urls.push(url.clone());
        }
    }

    // ✅ จัดเก็บข้อมูล Markdown เป็นกลุ่มใน `/all-markdown/{category}/`
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

    // ✅ สร้างไฟล์ `summary.txt`
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

// ✅ ค้นหา Sitemap หลายวิธีเหมือน Googlebot
async fn discover_sitemaps(domain: &str) -> Vec<String> {
    let mut sitemaps = Vec::new();

    // ✅ 1. ดึง Sitemap จาก robots.txt
    let robots_sitemaps = fetch_sitemaps_from_robots(domain).await;
    sitemaps.extend(robots_sitemaps);

    // ✅ 2. ลองเดา URL ที่เป็นไปได้
    let possible_sitemaps = vec![
        format!("{}/sitemap.xml", domain),
        format!("{}/sitemap_index.xml", domain),
        format!("{}/sitemaps.xml", domain),
    ];
    for sitemap in &possible_sitemaps {
        if let Ok(_) = reqwest::get(sitemap).await {
            sitemaps.push(sitemap.clone());
        }
    }

    // ✅ 3. ค้นหา Sitemap จาก `<head>` ของเว็บไซต์
    if let Ok(head_sitemap) = fetch_sitemap_from_html_head(domain).await {
        sitemaps.extend(head_sitemap);
    }

    // ✅ 4. ค้นหา Sitemap จากลิงก์ในหน้าแรก
    if let Ok(link_sitemap) = fetch_sitemap_from_links(domain).await {
        sitemaps.extend(link_sitemap);
    }

    sitemaps
}

// ✅ ดึง Sitemap จาก robots.txt
async fn fetch_sitemaps_from_robots(domain: &str) -> Vec<String> {
    let robots_url = format!("{}/robots.txt", domain);
    let response = reqwest::get(&robots_url).await;
    match response {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            text.lines()
                .filter(|line| line.starts_with("Sitemap:"))
                .map(|line| line.replace("Sitemap: ", "").trim().to_string())
                .collect()
        }
        Err(_) => vec![],
    }
}

// ✅ ค้นหา Sitemap จาก `<head>` ของ HTML
async fn fetch_sitemap_from_html_head(domain: &str) -> Result<Vec<String>, Error> {
    let mut sitemaps = Vec::new();
    let response = reqwest::get(domain).await?;
    let text = response.text().await?;
    let document = Html::parse_document(&text);
    let selector = Selector::parse("link[rel='sitemap']").unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            sitemaps.push(format!("{}/{}", domain, href));
        }
    }

    Ok(sitemaps)
}

// ✅ ค้นหา Sitemap จากลิงก์ `<a href>` ในหน้าแรก
async fn fetch_sitemap_from_links(domain: &str) -> Result<Vec<String>, Error> {
    let mut sitemaps = Vec::new();
    let response = reqwest::get(domain).await?;
    let text = response.text().await?;
    let document = Html::parse_document(&text);
    let selector = Selector::parse("a[href*='sitemap']").unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            sitemaps.push(format!("{}/{}", domain, href));
        }
    }

    Ok(sitemaps)
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
