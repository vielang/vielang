
# Cấu Trúc Thư Mục App Router

> **Mô tả ngắn gọn**: Tìm hiểu cấu trúc thư mục `app/`, các file đặc biệt, route groups, dynamic routes và metadata trong Next.js.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu rõ vai trò và chức năng của thư mục `app/`
- [ ] Nắm được cách hoạt động của các file đặc biệt: `page.tsx`, `layout.tsx`, `loading.tsx`
- [ ] Biết cách tạo route cơ bản và tổ chức layout dùng lại
- [ ] Làm quen với route groups, dynamic routes và metadata

### Kiến Thức Yêu Cầu

- Bài 1: Giới thiệu Next.js App Router
- React cơ bản (component, props)
- TypeScript cơ bản

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về cấu trúc thư mục | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành tạo routes | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Thư Mục `app/`

> **💡 Định nghĩa**: Thư mục `app/` là trung tâm của cấu trúc routing trong Next.js 13+. Mỗi thư mục con bên trong `app/` đại diện cho một route.

**Cấu trúc cơ bản:**

```
app/
├── layout.tsx     # Layout gốc cho toàn app
├── page.tsx       # Trang chủ (route /)
├── about/
│   └── page.tsx   # Route /about
└── contact/
    └── page.tsx   # Route /contact
```

**Tại sao cấu trúc này quan trọng?**

- File-based routing: Không cần cấu hình router thủ công
- Tự động code-splitting theo route
- Layout và loading state được tích hợp sẵn

### 1.2. Các File Đặc Biệt

#### `page.tsx` - Entry Point

- Là entry point của mỗi route
- Mỗi folder có file `page.tsx` sẽ tạo ra một route tương ứng

```tsx
// app/about/page.tsx → route /about
export default function AboutPage() {
  return <h1>Giới thiệu</h1>
}
```

#### `layout.tsx` - Layout Component

- Xác định layout cho toàn bộ hoặc một phần cụ thể của ứng dụng
- Layout được dùng lại khi chuyển route → tránh render lại các phần không đổi

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

**📝 Đặc điểm:**

- `children` là nội dung của page hoặc layout con
- Layout gốc phải trả về cấu trúc `<html><body>{children}</body></html>`

#### `loading.tsx` - Loading State

- Tự động hiển thị khi đang chờ load component bên trong
- Giúp nâng cao trải nghiệm người dùng

```tsx
// app/about/loading.tsx
export default function Loading() {
  return <p>Đang tải trang Giới thiệu...</p>
}
```

### 1.3. Route Groups và Dynamic Routes

#### Route Groups `(group-name)`

- Nhóm các route mà không ảnh hưởng đến URL
- Dùng để tổ chức code, áp dụng layout chung

```
app/
├── (public)/
│   ├── about/page.tsx     # Route /about
│   └── contact/page.tsx   # Route /contact
└── (admin)/
    └── dashboard/page.tsx # Route /dashboard
```

> ⚠️ URL không chứa `(public)` hoặc `(admin)`, chỉ để tổ chức file.

#### Dynamic Routes `[param]`

- Cho phép route động theo tham số

```
app/products/[id]/page.tsx → "/products/123"
```

```tsx
// app/products/[id]/page.tsx
export default function ProductPage({ params }) {
  return <p>ID sản phẩm: {params.id}</p>
}
```

### 1.4. Metadata và SEO

> **💡 Định nghĩa**: Metadata là thông tin giúp cải thiện SEO, chia sẻ mạng xã hội, hiển thị title...

```tsx
// app/about/page.tsx
export const metadata = {
  title: "Trang Giới Thiệu",
  description: "Thông tin về công ty",
}

export default function AboutPage() {
  return <h1>Giới thiệu</h1>
}
```

**📝 Đặc điểm:**

- Metadata được render ở `<head>`
- Hỗ trợ tự động cập nhật theo route
- Có thể đặt ở `page.tsx` hoặc `layout.tsx`

### 1.5. So Sánh & Đối Chiếu

| File | Chức năng | Phạm vi |
|------|-----------|---------|
| `page.tsx` | Entry point của route | Một route cụ thể |
| `layout.tsx` | Layout bọc ngoài content | Route và các route con |
| `loading.tsx` | Hiển thị khi đang load | Route hiện tại |
| `error.tsx` | Xử lý lỗi | Route hiện tại |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn cần xây dựng một website với:
- Các trang public: Home, About, Contact
- Các trang admin: Dashboard, Users, Settings
- Layout riêng cho mỗi nhóm trang

**Yêu cầu**:

- Navbar chung cho trang public
- Sidebar cho trang admin
- Loading state cho mỗi trang

**🤔 Câu hỏi suy ngẫm:**

1. Nên tổ chức cấu trúc thư mục như thế nào?
2. Làm sao để có layout riêng cho public và admin?
3. Cách triển khai loading state hiệu quả?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. Sử dụng route groups: `(public)` và `(admin)`
2. Mỗi group có `layout.tsx` riêng với UI phù hợp
3. Đặt `loading.tsx` trong mỗi route hoặc layout

```
app/
├── (public)/
│   ├── layout.tsx      # Navbar
│   ├── page.tsx
│   ├── about/page.tsx
│   └── contact/page.tsx
└── (admin)/
    ├── layout.tsx      # Sidebar
    ├── dashboard/page.tsx
    └── users/page.tsx
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Mỗi route cần có file `page.tsx` để có thể truy cập được.

#### ✅ Nên Làm

```tsx
// Tổ chức layout rõ ràng
app/
├── layout.tsx          # Root layout
├── (marketing)/
│   ├── layout.tsx      # Marketing layout
│   ├── page.tsx
│   └── about/page.tsx
```

**Tại sao tốt:**

- Tách biệt rõ ràng các phần của ứng dụng
- Dễ maintain và scale
- Layout được tái sử dụng hiệu quả

#### ❌ Không Nên Làm

```tsx
// Không tách layout, mọi thứ ở root
app/
├── layout.tsx
├── page.tsx
├── about/page.tsx
├── dashboard/page.tsx    # Admin page lẫn với public
├── users/page.tsx
```

**Tại sao không tốt:**

- Khó quản lý khi dự án lớn
- Không thể có layout riêng cho admin

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Route không hoạt động | Thiếu `page.tsx` | Đảm bảo mỗi route có `page.tsx` |
| Layout không hiển thị | Thiếu `{children}` | Luôn return `{children}` trong layout |
| Metadata không cập nhật | Đặt sai vị trí | Export `metadata` ở đầu file |
| Route group xuất hiện trong URL | Thiếu dấu ngoặc | Dùng `(group-name)` với dấu ngoặc |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo website với các route `/`, `/about`, `/contact` với layout và loading state

**Yêu cầu kỹ thuật:**

- Layout chung cho toàn bộ trang
- Trang loading riêng cho `/about`
- Metadata cho từng trang

#### Bước 1: Tạo cấu trúc thư mục

```
app/
├── layout.tsx
├── page.tsx
├── about/
│   ├── page.tsx
│   └── loading.tsx
├── contact/
│   └── page.tsx
```

#### Bước 2: Tạo Root Layout

```tsx
// app/layout.tsx
export default function RootLayout({ children }) {
  return (
    <html lang="vi">
      <body>
        <header className="bg-gray-100 p-4">
          <nav className="flex gap-4">
            <a href="/">Trang chủ</a>
            <a href="/about">Giới thiệu</a>
            <a href="/contact">Liên hệ</a>
          </nav>
        </header>
        <main className="p-4">{children}</main>
      </body>
    </html>
  )
}
```

#### Bước 3: Tạo các trang với metadata

```tsx
// app/page.tsx
export const metadata = {
  title: "Trang chủ",
  description: "Chào mừng đến với website",
}

export default function HomePage() {
  return <h1>Chào mừng!</h1>
}
```

```tsx
// app/about/page.tsx
export const metadata = {
  title: "Giới thiệu",
  description: "Thông tin về chúng tôi",
}

export default function AboutPage() {
  return (
    <div>
      <h1>Giới thiệu</h1>
      <p>Đây là trang giới thiệu.</p>
    </div>
  )
}
```

#### Bước 4: Tạo Loading State

```tsx
// app/about/loading.tsx
export default function Loading() {
  return (
    <div className="animate-pulse">
      <div className="h-8 bg-gray-200 rounded w-1/4 mb-4"></div>
      <div className="h-4 bg-gray-200 rounded w-3/4"></div>
    </div>
  )
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo trang `/services` để hiển thị danh sách dịch vụ

<details>
<summary>💡 Gợi ý</summary>

- Tạo thư mục `app/services/`
- Tạo file `page.tsx` với metadata
- Hiển thị danh sách dịch vụ dạng cards

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// app/services/page.tsx
export const metadata = {
  title: "Dịch vụ",
  description: "Danh sách dịch vụ của chúng tôi",
}

export default function ServicesPage() {
  const services = [
    { name: "Web Development", desc: "Xây dựng website" },
    { name: "Mobile App", desc: "Phát triển ứng dụng di động" },
    { name: "Consulting", desc: "Tư vấn công nghệ" },
  ]

  return (
    <div>
      <h1>Dịch vụ của chúng tôi</h1>
      <div className="grid gap-4">
        {services.map((service) => (
          <div key={service.name} className="p-4 border rounded">
            <h2>{service.name}</h2>
            <p>{service.desc}</p>
          </div>
        ))}
      </div>
    </div>
  )
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Thêm layout riêng cho `/services` với sidebar

**Mở rộng**:

- Tạo `layout.tsx` trong thư mục `services/`
- Sidebar hiển thị danh mục dịch vụ
- Sử dụng dynamic route cho chi tiết dịch vụ: `/services/[id]`

### 3.3. Mini Project

**Dự án**: Blog với nhiều category

**Mô tả**: Xây dựng blog với cấu trúc route động cho bài viết

**Yêu cầu chức năng:**

1. Trang chủ hiển thị danh sách bài viết
2. Route động `/posts/[slug]` cho chi tiết bài viết
3. Route group cho admin: `/dashboard`, `/posts/new`

**Technical Stack:**

- Next.js 14+ với App Router
- TypeScript

**Hướng dẫn triển khai:**

```
app/
├── layout.tsx
├── page.tsx
├── posts/
│   ├── page.tsx
│   └── [slug]/
│       └── page.tsx
└── (admin)/
    ├── layout.tsx
    └── dashboard/
        └── page.tsx
```

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu rõ cấu trúc thư mục `app/`
- [ ] Nắm được các file đặc biệt (`page.tsx`, `layout.tsx`, `loading.tsx`)
- [ ] Tạo được route cơ bản với layout
- [ ] Sử dụng được route groups và dynamic routes
- [ ] (Tùy chọn) Hoàn thành mini project blog

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Giải thích sự khác nhau giữa `page.tsx` và `layout.tsx`?
2. **Ứng dụng**: Khi nào nên sử dụng route groups?
3. **Phân tích**: So sánh cách tổ chức code với và không có route groups?
4. **Thực hành**: Demo website với layout và loading state?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Tóm tắt cấu trúc thư mục App Router
- Demo tổ chức project thực tế
- Chia sẻ best practices về tổ chức code
- Tips khi làm việc với dynamic routes

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: File nào định nghĩa layout chung cho các route con?

- A. `page.tsx`
- B. `layout.tsx`
- C. `template.tsx`
- D. `root.tsx`

**Câu 2**: Route groups sử dụng cú pháp nào?

- A. `[group-name]`
- B. `(group-name)`
- C. `{group-name}`
- D. `_group-name`

**Câu 3**: Để tạo dynamic route với param `id`, bạn đặt tên thư mục như thế nào?

- A. `:id`
- B. `{id}`
- C. `[id]`
- D. `$id`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Route groups có ảnh hưởng đến URL không?</strong></summary>

Không! Route groups (dùng dấu ngoặc đơn như `(marketing)`) chỉ để tổ chức code và không xuất hiện trong URL. Ví dụ: `app/(marketing)/about/page.tsx` sẽ có URL là `/about`, không phải `/(marketing)/about`.

</details>

<details>
<summary><strong>Q2: Có thể có nhiều layout lồng nhau không?</strong></summary>

Có! Layout có thể lồng nhau. Layout cha sẽ bọc layout con, và layout con sẽ bọc page. Điều này cho phép bạn có layout chung cho toàn app và layout riêng cho từng phần.

```
app/
├── layout.tsx           # Root layout (navbar)
└── dashboard/
    ├── layout.tsx       # Dashboard layout (sidebar)
    └── page.tsx         # Dashboard page
```

</details>

<details>
<summary><strong>Q3: loading.tsx hoạt động như thế nào?</strong></summary>

`loading.tsx` sử dụng React Suspense bên dưới. Khi một route đang load (fetch data, lazy load component), Next.js sẽ tự động hiển thị nội dung của `loading.tsx`. Khi load xong, nội dung sẽ được thay thế bằng `page.tsx`.

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 2

</footer>
