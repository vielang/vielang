
# Giới Thiệu Next.js App Router

> **Mô tả ngắn gọn**: Tìm hiểu Next.js là gì, sự khác biệt giữa App Router và Pages Router, cùng cách xây dựng ứng dụng đầu tiên.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu được Next.js là gì và lý do nên sử dụng
- [ ] Phân biệt được App Router và Pages Router
- [ ] Nắm rõ cấu trúc thư mục với `app/`
- [ ] Phân biệt được Server Component và Client Component

### Kiến Thức Yêu Cầu

- HTML, CSS cơ bản
- JavaScript ES6+
- React cơ bản (component, props, state)

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Next.js | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành tạo dự án | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Next.js Là Gì?

> **💡 Định nghĩa**: Next.js là một React framework phát triển bởi Vercel, giúp xây dựng ứng dụng web hiệu suất cao, có khả năng SEO tốt, hỗ trợ cả SSR (server-side rendering) và SSG (static site generation).

**Tại sao nên sử dụng Next.js?**

- Hỗ trợ Server-Side Rendering (SSR) và Static Site Generation (SSG)
- Tối ưu hóa hiệu suất tự động (code splitting, image optimization)
- Routing dựa trên file system, không cần cấu hình phức tạp
- Hỗ trợ TypeScript out-of-the-box

### 1.2. App Router vs Pages Router

#### App Router (`app/`)

```
app/
├── layout.tsx
├── page.tsx
└── about/
    └── page.tsx
```

**📝 Đặc điểm:**

- Hỗ trợ Server & Client Component
- Có layout lồng nhau (`layout.tsx`)
- Hiện đại, mạnh mẽ, tối ưu performance

#### Pages Router (`pages/`)

```
pages/
├── index.tsx
└── about.tsx
```

**📝 Đặc điểm:**

- Chỉ có Client Component
- Không có Layout gốc
- Đơn giản, quen thuộc

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | Pages Router | App Router |
|----------|-------------|------------|
| Cách routing | Theo file trong `pages/` | Theo file trong `app/` |
| Component type | Chỉ có Client Component | Hỗ trợ Server & Client Component |
| Layout | Không có Layout gốc | Có layout lồng nhau |
| Ưu điểm | Đơn giản, quen thuộc | Hiện đại, tối ưu performance |

### 1.4. Server Component vs Client Component

#### Server Component

- Mặc định trong App Router
- Được render trên server, không gửi JavaScript không cần thiết về client
- Không dùng `useState`, `useEffect`, `onClick`,...

```tsx
// Server Component (mặc định)
export default function HomePage() {
  return <h1>Hello from Server</h1>
}
```

#### Client Component

- Dùng khi cần interactivity (nút nhấn, hiệu ứng, state)
- Phải khai báo `"use client"` ở đầu file

```tsx
"use client"

import { useState } from "react"

export default function Counter() {
  const [count, setCount] = useState(0)
  return <button onClick={() => setCount(count + 1)}>{count}</button>
}
```

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn cần xây dựng một website giới thiệu công ty với các trang: Trang chủ, Giới thiệu, Liên hệ. Website cần load nhanh và SEO tốt.

**Yêu cầu**:

- Hiển thị nội dung tĩnh
- SEO tốt cho các công cụ tìm kiếm
- Navbar chung cho tất cả trang

**🤔 Câu hỏi suy ngẫm:**

1. Nên chọn App Router hay Pages Router?
2. Các trang nên là Server Component hay Client Component?
3. Làm thế nào để tạo layout chung cho navbar?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. **App Router** là lựa chọn tốt hơn vì hỗ trợ layout lồng nhau và Server Component
2. **Server Component** phù hợp vì nội dung tĩnh, không cần state hay event handlers
3. Sử dụng `layout.tsx` để đặt Navbar, tự động áp dụng cho tất cả trang con

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Luôn bắt đầu với Server Component, chỉ chuyển sang Client Component khi thực sự cần thiết.

#### ✅ Nên Làm

```tsx
// app/layout.tsx
export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        <nav>Navbar</nav>
        <main>{children}</main>
      </body>
    </html>
  )
}
```

**Tại sao tốt:**

- Layout được dùng lại, không re-render khi chuyển trang
- Navbar luôn hiển thị nhất quán

#### ❌ Không Nên Làm

```tsx
// Không nên: Thêm "use client" khi không cần thiết
"use client"

export default function AboutPage() {
  return <p>Về chúng tôi</p>
}
```

**Tại sao không tốt:**

- Gửi JavaScript không cần thiết về client
- Giảm hiệu suất và tăng thời gian tải

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Không thể dùng hooks trong Server Component | Server Component không hỗ trợ React hooks | Thêm `"use client"` nếu cần hooks |
| Layout không hiển thị | Thiếu `{children}` trong layout | Đảm bảo return `{children}` trong layout |
| Route không hoạt động | Thiếu file `page.tsx` | Mỗi route cần có `page.tsx` |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo dự án Next.js đầu tiên với App Router

**Yêu cầu kỹ thuật:**

- Sử dụng App Router
- TypeScript
- Tạo route `/about`

#### Bước 1: Khởi tạo dự án

```bash
npx create-next-app@latest my-app
cd my-app
npm run dev
```

**Giải thích:**

- `--app`: Chọn App Router
- `--typescript`: Tạo project với TypeScript
- Truy cập `http://localhost:3000` để xem kết quả

#### Bước 2: Tạo trang About

Tạo file `app/about/page.tsx`:

```tsx
export default function AboutPage() {
  return (
    <div>
      <h1>Giới thiệu</h1>
      <p>Đây là trang giới thiệu về Next.js App Router.</p>
    </div>
  )
}
```

**Giải thích:**

- Thư mục `about/` tạo route `/about`
- File `page.tsx` là entry point của route

#### Bước 3: Kiểm tra kết quả

Truy cập `http://localhost:3000/about` để xem trang About.

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo trang Contact tại `/contact`

<details>
<summary>💡 Gợi ý</summary>

- Tạo thư mục `app/contact/`
- Tạo file `page.tsx` bên trong
- Hiển thị tiêu đề và thông tin liên hệ

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// app/contact/page.tsx
export default function ContactPage() {
  return (
    <div>
      <h1>Liên hệ với chúng tôi</h1>
      <p>Email: contact@myapp.com</p>
    </div>
  )
}
```

**Giải thích chi tiết:**

- Route `/contact` được tạo tự động từ thư mục `contact/`
- Server Component phù hợp vì chỉ hiển thị nội dung tĩnh

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo layout với Navbar dùng chung cho tất cả trang

**Mở rộng**:

- Thêm styling với Tailwind CSS
- Tạo component Navbar riêng
- Highlight trang hiện tại trong Navbar

### 3.3. Mini Project

**Dự án**: Website giới thiệu cá nhân

**Mô tả**: Xây dựng portfolio website đơn giản với 3 trang: Home, About, Projects

**Yêu cầu chức năng:**

1. Trang Home hiển thị lời chào và giới thiệu ngắn
2. Trang About hiển thị thông tin chi tiết về bản thân
3. Trang Projects hiển thị danh sách dự án

**Technical Stack:**

- Next.js 14+ với App Router
- TypeScript
- Tailwind CSS (tùy chọn)

**Hướng dẫn triển khai:**

1. Khởi tạo dự án với `create-next-app`
2. Tạo cấu trúc thư mục cho các route
3. Xây dựng layout chung với Navbar
4. Tạo nội dung cho từng trang

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu rõ Next.js và lý do sử dụng
- [ ] Phân biệt được App Router và Pages Router
- [ ] Hoàn thành tạo dự án Next.js
- [ ] Tạo được route mới với page.tsx
- [ ] (Tùy chọn) Hoàn thành mini project portfolio

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Next.js là gì và khác gì với React thuần?
2. **Ứng dụng**: Khi nào nên dùng Server Component, khi nào dùng Client Component?
3. **Phân tích**: So sánh App Router và Pages Router, khi nào nên dùng cái nào?
4. **Thực hành**: Demo dự án Next.js với route `/about` và `/contact`?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Tóm tắt kiến thức về Next.js App Router
- Demo dự án đã tạo
- Chia sẻ khó khăn gặp phải và cách giải quyết
- Best practices rút ra được

**Format:**

- Live coding demo hoặc
- Slides (3-5 slides)

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Next.js App Router sử dụng thư mục nào làm gốc cho routing?

- A. `pages/`
- B. `app/`
- C. `src/`
- D. `routes/`

**Câu 2**: Để khai báo một Client Component trong App Router, bạn cần thêm gì ở đầu file?

- A. `"use server"`
- B. `"use client"`
- C. `export const dynamic = "force-dynamic"`
- D. `import { Client } from "next"`

**Câu 3**: File nào là entry point của mỗi route trong App Router?

- A. `index.tsx`
- B. `route.tsx`
- C. `page.tsx`
- D. `layout.tsx`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Tại sao nên chọn App Router thay vì Pages Router?</strong></summary>

App Router là routing mới của Next.js (từ version 13+), mang lại nhiều lợi ích:
- Hỗ trợ Server Components giúp giảm JavaScript gửi về client
- Layout lồng nhau dễ dàng quản lý
- Streaming và Suspense được tích hợp sẵn
- Cải thiện hiệu suất tổng thể

Pages Router vẫn được hỗ trợ, nhưng App Router là hướng đi tương lai của Next.js.

</details>

<details>
<summary><strong>Q2: Server Component có thể gọi API không?</strong></summary>

Có! Server Component có thể gọi API trực tiếp bằng `fetch` hoặc truy vấn database. Đây là một trong những lợi thế lớn của Server Components - bạn có thể fetch data ngay trong component mà không cần tạo API endpoint riêng.

```tsx
// Server Component có thể fetch data trực tiếp
export default async function ProductsPage() {
  const products = await fetch('https://api.example.com/products')
  return <div>{/* render products */}</div>
}
```

</details>

<details>
<summary><strong>Q3: Làm sao biết khi nào dùng Server vs Client Component?</strong></summary>

**Dùng Server Component khi:**
- Hiển thị nội dung tĩnh
- Fetch data từ API hoặc database
- Không cần state hoặc event handlers

**Dùng Client Component khi:**
- Cần React hooks (useState, useEffect,...)
- Cần xử lý sự kiện người dùng (onClick, onChange,...)
- Cần truy cập browser APIs

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 1

</footer>
