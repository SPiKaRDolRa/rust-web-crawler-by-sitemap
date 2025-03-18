# 🚀 Web Crawler for HeyGoody.com (Rust + Spider-rs)

## 1️⃣ โจทย์ของโปรเจค

โปรเจคนี้เป็น **Web Crawler** ที่ใช้ `Rust` และ `spider-rs` ในการดึงข้อมูลจากเว็บไซต์ **[HeyGoody.com](https://www.heygoody.com/)** โดยมีข้อกำหนดหลักดังนี้:

✅ **Crawler จะไม่ต้องกำหนด Sitemap เอง** แต่จะ **เข้าไปอ่าน `robots.txt` และดึง Sitemap อัตโนมัติ**  
✅ ใช้ **`spider-rs`** เป็นตัวหลักสำหรับการ Crawl  
✅ รองรับ **ทั้ง SPA และ SSR**  
✅ แปลง HTML เป็น **Markdown**  
✅ จัดกลุ่ม Markdown ตาม **ประเภทของ URL**  
✅ สร้าง **Summary** ของข้อมูลที่ดึงมา  

---

## 2️⃣ แนวคิด & การแก้ปัญหา

### 🔹 **วิธีการ Crawl**
1. **ดึง Sitemap จาก `robots.txt` ของเว็บไซต์**  
   - ระบบจะเข้าไปโหลด `robots.txt` จากโดเมนที่ระบุ  
   - ค้นหาทุก `Sitemap:` URL ภายใน `robots.txt`  
   - ใช้ Sitemap ที่พบเป็นจุดเริ่มต้นของ Crawler  

2. **แยก URLs เป็น SPA และ SSR**
   - ใช้ HTTP Request โหลด HTML และตรวจหาคีย์เวิร์ดที่บ่งบอกว่าเป็น **SPA** (`window.__NUXT__` หรือ `data-reactroot`)  

3. **ดึง HTML และแปลงเป็น Markdown**
   - **SSR** → ใช้ `reqwest` ดึง HTML โดยตรง  
   - **SPA** → ใช้ `spider-rs` (Chrome Headless) เพื่อเรนเดอร์หน้าเว็บ  

4. **บันทึก Markdown และสร้าง Summary**
   - บันทึก Markdown ไว้ใน `/all-markdown/{category}/`
   - สร้าง `summary.txt` เพื่อสรุปผลการทำงาน  

---

## 3️⃣ วิธีใช้งานโปรเจค

### 🔹 **1. Fork โปรเจคนี้**
กดปุ่ม **Fork** ที่มุมขวาบนของ GitHub  

### 🔹 **2. Clone โปรเจค**
```sh
git clone https://github.com/{your-username}/web-crawler-rust.git
cd web-crawler-rust
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

🎯 ผลลัพธ์ที่คาดหวัง
✅ Markdown ของแต่ละหน้าเว็บใน /all-markdown/
✅ จัดกลุ่ม Markdown ตามประเภทของ URL
✅ ไฟล์ summary.txt ที่สรุปผลการทำงาน
✅ Console Output ควรขึ้นแบบนี้:

arduino
คัดลอก
🚀 Starting Web Crawler...
🌐 Found 50 URLs to process.
🎉 Web Crawling Completed!