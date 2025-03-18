# 🚀 Web Crawler for defined domain (Rust + Spider-rs)

## 1️⃣ โจทย์ของโปรเจค

โปรเจคนี้เป็น **Web Crawler** ที่ใช้ `Rust` และ `spider-rs` ในการดึงข้อมูลจาก **[เว็บไซต์ตัวอย่าง](https://www.heygoody.com/)** โดยมีข้อกำหนดหลักดังนี้:

✅ **Crawler จะไม่ต้องกำหนด Sitemap เอง** แต่จะ **เข้าไปอ่าน `robots.txt` และดึง Sitemap อัตโนมัติ**  
✅ ใช้ **`spider-rs`** เป็นตัวหลักสำหรับการ Crawl  
✅ รองรับ **ทั้ง SPA และ SSR**  
✅ แปลง HTML เป็น **Markdown**  
✅ จัดกลุ่ม Markdown ตาม **ประเภทของ URL**  
✅ สร้าง **Summary** ของข้อมูลที่ดึงมา  

---

## 2️⃣ แนวคิด & การแก้ปัญหา

### 🔹 **การค้นหา Sitemap**
Crawler จะใช้ **หลายวิธีในการหา Sitemap** คล้าย Googlebot เพื่อรองรับเว็บไซต์ที่ไม่ได้ประกาศ Sitemap ไว้อย่างชัดเจน  

1️⃣ **ตรวจสอบ `robots.txt`**  
   - โหลดไฟล์ `robots.txt` แล้วดึง `Sitemap:` ออกมา  
   
2️⃣ **ลองเดา URL ของ Sitemap**  
   - ทดลองโหลด:  
     - `/sitemap.xml`  
     - `/sitemap_index.xml`  
     - `/sitemaps.xml`  
   
3️⃣ **ค้นหาภายใน `<head>` ของเว็บไซต์**  
   - ดู `<link rel="sitemap" href="...">` ว่ามี Sitemap URL หรือไม่  

4️⃣ **ตรวจสอบ `<a href>` ที่ลิงก์ไปยัง Sitemap**  
   - ค้นหา `<a>` ที่มี `href` ซึ่งอาจเป็น Sitemap เช่น `href="/sitemap.xml"`  

---

### 🔹 **การแยก URLs เป็น SPA และ SSR**
- **SSR (Server-Side Rendered)** → ใช้ HTTP Request (`reqwest`) ดึง HTML โดยตรง  
- **SPA (Single Page Application)** → ใช้ Chrome Headless (`spider-rs`) เพื่อเรนเดอร์หน้าเว็บ  

---

### 🔹 **การแปลง HTML เป็น Markdown และบันทึกข้อมูล**
- **แปลง HTML เป็น Markdown** แล้วบันทึกไฟล์ `.md`  
- **จัดเก็บ Markdown ตามประเภทของ URL** ใน `/all-markdown/{category}/`  
- **สร้างไฟล์ `summary.txt`** เพื่อสรุปผลลัพธ์  

---

## 3️⃣ วิธีใช้งานโปรเจค

### 🔹 **1. Fork โปรเจคนี้**
กดปุ่ม **Fork** ที่มุมขวาบนของ GitHub  

### 🔹 **2. Clone โปรเจค**
```sh
git clone https://github.com/SPiKaRDolRa/rust-web-crawler-by-sitemap.git
cd rust-web-crawler-by-sitemap
```

---

## 4️⃣ อธิบายการทำงาน และผลลัพธ์ที่คาดหวัง
📌 กระบวนการทำงาน
1️⃣ เข้าไปโหลด robots.txt เพื่อดึง URL ของ Sitemap อัตโนมัติ
2️⃣ โหลด Sitemap และตรวจสอบว่าเป็น SPA หรือ SSR
3️⃣ ดึง HTML และแปลงเป็น Markdown
4️⃣ บันทึก Markdown ใน /all-markdown/{category}/
5️⃣ สร้างไฟล์ summary.txt ที่สรุปจำนวน URL ที่ถูกดึงมา

📂 โครงสร้างของไฟล์ผลลัพธ์
```sh
all-markdown/
│── blogs/
│   ├── blog1.md
│   ├── blog2.md
│── products/
│   ├── product1.md
│── news/
│   ├── news1.md
│── others/
│   ├── misc1.md
│── summary.txt  <-- ✅ สรุปข้อมูลการดึงข้อมูลทั้งหมด
```

---

## 🎯 ผลลัพธ์ที่คาดหวัง
✅ Markdown ของแต่ละหน้าเว็บใน /all-markdown/
✅ จัดกลุ่ม Markdown ตามประเภทของ URL
✅ ไฟล์ summary.txt ที่สรุปผลการทำงาน
✅ Console Output ควรขึ้นแบบนี้:
```sh
🚀 Starting Web Crawler...
🌐 Found ??? URLs to process.
🎉 Web Crawling Completed!
```