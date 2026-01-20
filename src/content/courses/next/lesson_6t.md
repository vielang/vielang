
# Layout và Navigation

> **Mô tả ngắn gọn**: Tìm hiểu cách xây dựng layout lồng nhau, navigation menu với next/link và usePathname, metadata động cho SEO.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu và áp dụng layout lồng nhau (nested layouts) trong App Router
- [ ] Tạo và quản lý layout đặc biệt cho từng nhóm route (auth, admin, dashboard)
- [ ] Xây dựng navigation menu với `next/link` và `usePathname`
- [ ] Tạo metadata động để hỗ trợ SEO
- [ ] Thiết kế layout responsive với TailwindCSS và ShadcnUI

### Kiến Thức Yêu Cầu

- Bài 1-5: Next.js App Router, TypeScript, TailwindCSS, ShadcnUI
- Hiểu về cấu trúc thư mục App Router
- React components cơ bản

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về Layout và Navigation | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành xây dựng layout | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Layout Trong Next.js App Router

> **💡 Định nghĩa**: `layout.tsx` định nghĩa khung giao diện dùng chung cho các trang con bên trong nó. Nó cho phép tái sử dụng các phần như Header, Sidebar, Footer.

**Ví dụ thực tế:**

- Layout chính cho toàn bộ app chứa Navigation bar
- Layout của trang xác thực chỉ là một card nhỏ ở giữa màn hình
- Layout của dashboard có sidebar bên trái

**Cấu trúc cơ bản:**

```tsx
// app/layout.tsx
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="vi">
      <body>
        <header>Logo & Navigation</header>
        <main>{children}</main>
        <footer>© 2025</footer>
      </body>
    </html>
  );
}
```

### 1.2. Layout Lồng Nhau (Nested Layouts)

> **💡 Định nghĩa**: Next.js cho phép tạo layout bên trong layout. Layout ở thư mục cha bao bọc layout của thư mục con.

**Cấu trúc thư mục:**

```
app/
├── layout.tsx           # Layout chính (navbar, footer)
├── page.tsx
├── auth/
│   ├── layout.tsx       # Layout riêng cho auth (centered card)
│   ├── login/page.tsx
│   └── register/page.tsx
└── dashboard/
    ├── layout.tsx       # Layout riêng (sidebar)
    └── page.tsx
```

**Layout Auth (centered):**

```tsx
// app/auth/layout.tsx
export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100">
      <div className="w-full max-w-md bg-white p-6 rounded-lg shadow">
        {children}
      </div>
    </div>
  );
}
```

**Layout Dashboard (sidebar):**

```tsx
// app/dashboard/layout.tsx
export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen">
      <aside className="w-64 bg-gray-800 text-white p-4">
        <nav>Sidebar Menu</nav>
      </aside>
      <main className="flex-1 p-6">{children}</main>
    </div>
  );
}
```

### 1.3. Navigation Với `next/link`

> **💡 Định nghĩa**: `next/link` giúp tạo link chuyển trang trong SPA mà không reload lại toàn bộ trang.

**Sử dụng cơ bản:**

```tsx
import Link from "next/link";

export default function Navbar() {
  return (
    <nav className="flex gap-4">
      <Link href="/">Trang chủ</Link>
      <Link href="/about">Giới thiệu</Link>
      <Link href="/dashboard">Dashboard</Link>
    </nav>
  );
}
```

**Với `usePathname` để highlight menu:**

```tsx
"use client"

import Link from "next/link";
import { usePathname } from "next/navigation";

export default function Navbar() {
  const pathname = usePathname();

  const links = [
    { name: "Trang chủ", href: "/" },
    { name: "Giới thiệu", href: "/about" },
    { name: "Dashboard", href: "/dashboard" },
  ];

  return (
    <nav className="flex gap-4">
      {links.map((link) => (
        <Link
          key={link.href}
          href={link.href}
          className={`px-3 py-2 rounded ${
            pathname === link.href
              ? "bg-blue-600 text-white"
              : "text-gray-700 hover:bg-gray-100"
          }`}
        >
          {link.name}
        </Link>
      ))}
    </nav>
  );
}
```

**📝 Lưu ý:** `usePathname` là hook nên cần `"use client"` directive.

### 1.4. Metadata Động và SEO

```tsx
// app/about/page.tsx
export const metadata = {
  title: "Giới thiệu - Vievlog",
  description: "Trang giới thiệu về ứng dụng Vievlog.",
  openGraph: {
    title: "Giới thiệu - Vievlog",
    description: "Trang giới thiệu về ứng dụng Vievlog.",
  },
};

export default function AboutPage() {
  return <h1>Giới thiệu</h1>;
}
```

**Dynamic metadata:**

```tsx
// app/posts/[slug]/page.tsx
export async function generateMetadata({ params }) {
  const post = await getPost(params.slug);

  return {
    title: post.title,
    description: post.excerpt,
  };
}
```

### 1.5. Responsive Navigation

```tsx
"use client"

import { useState } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Menu, X } from "lucide-react";

export default function ResponsiveNav() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <header className="border-b">
      {/* Desktop Nav */}
      <nav className="hidden md:flex items-center justify-between p-4">
        <span className="font-bold text-xl">Logo</span>
        <div className="flex gap-4">
          <Link href="/">Trang chủ</Link>
          <Link href="/about">Giới thiệu</Link>
        </div>
      </nav>

      {/* Mobile Nav */}
      <div className="md:hidden flex items-center justify-between p-4">
        <span className="font-bold text-xl">Logo</span>
        <Button variant="ghost" size="icon" onClick={() => setIsOpen(!isOpen)}>
          {isOpen ? <X /> : <Menu />}
        </Button>
      </div>

      {/* Mobile Menu */}
      {isOpen && (
        <div className="md:hidden p-4 border-t">
          <Link href="/" className="block py-2">Trang chủ</Link>
          <Link href="/about" className="block py-2">Giới thiệu</Link>
        </div>
      )}
    </header>
  );
}
```

### 1.6. So Sánh & Đối Chiếu

| Khái niệm | Mô tả | Phạm vi |
|-----------|-------|---------|
| `layout.tsx` | Bọc các page bên trong cùng cấp thư mục | Tự động áp dụng cho tất cả route con |
| `template.tsx` | Giống layout nhưng re-mount mỗi khi navigate | Dùng khi cần reset state |
| `next/link` | Navigation không reload trang | Thay thế thẻ `<a>` |
| `usePathname` | Lấy URL path hiện tại | Dùng để highlight menu |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn cần xây dựng ứng dụng với:
- Layout chung (navbar, footer) cho trang public
- Layout riêng cho auth (login, register) - không có navbar
- Layout dashboard với sidebar

**Yêu cầu**:

- Navigation highlight trang hiện tại
- Responsive trên mobile
- SEO tốt với metadata

**🤔 Câu hỏi suy ngẫm:**

1. Nên tổ chức cấu trúc thư mục như thế nào?
2. Làm sao để auth layout không kế thừa navbar từ root layout?
3. Cách tối ưu để tái sử dụng Navbar component?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. **Cấu trúc thư mục:**
```
app/
├── layout.tsx           # Root layout minimal
├── (public)/
│   ├── layout.tsx       # Navbar + Footer
│   ├── page.tsx
│   └── about/page.tsx
├── (auth)/
│   ├── layout.tsx       # Centered card only
│   ├── login/page.tsx
│   └── register/page.tsx
└── dashboard/
    ├── layout.tsx       # Sidebar layout
    └── page.tsx
```

2. **Route groups** `(public)` và `(auth)` cho phép có layout riêng mà không ảnh hưởng URL

3. Tạo component `Navbar.tsx` và import vào các layout cần

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Layout được giữ lại khi navigate giữa các trang con. State trong layout không bị reset.

#### ✅ Nên Làm

```tsx
// Tổ chức components rõ ràng
app/
├── components/
│   ├── Navbar.tsx
│   ├── Sidebar.tsx
│   └── Footer.tsx
├── (public)/
│   └── layout.tsx  // Import Navbar, Footer
└── dashboard/
    └── layout.tsx  // Import Sidebar
```

**Tại sao tốt:**

- Components có thể tái sử dụng
- Dễ maintain và test
- Rõ ràng về cấu trúc

#### ❌ Không Nên Làm

```tsx
// Inline tất cả trong layout
export default function Layout({ children }) {
  return (
    <html>
      <body>
        <nav>
          {/* 100 dòng code navbar */}
        </nav>
        {children}
        <footer>
          {/* 50 dòng code footer */}
        </footer>
      </body>
    </html>
  );
}
```

**Tại sao không tốt:**

- File quá dài, khó đọc
- Không thể tái sử dụng components
- Khó test và maintain

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| `usePathname` không hoạt động | Thiếu `"use client"` | Thêm directive ở đầu file |
| Auth layout vẫn có navbar | Kế thừa từ root layout | Dùng route groups với layout riêng |
| Menu không highlight | So sánh path sai | Dùng `pathname.startsWith()` cho nested routes |
| Layout bị re-mount | Dùng `template.tsx` thay vì `layout.tsx` | Kiểm tra tên file |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo hệ thống layout với navigation highlight

**Yêu cầu kỹ thuật:**

- Root layout với Navbar
- Auth layout riêng (không có Navbar)
- Navigation highlight trang hiện tại

#### Bước 1: Tạo Navbar Component

```tsx
// app/components/Navbar.tsx
"use client"

import Link from "next/link";
import { usePathname } from "next/navigation";

const links = [
  { name: "Trang chủ", href: "/" },
  { name: "Giới thiệu", href: "/about" },
  { name: "Dashboard", href: "/dashboard" },
];

export default function Navbar() {
  const pathname = usePathname();

  return (
    <header className="border-b bg-white">
      <nav className="container mx-auto flex items-center justify-between p-4">
        <Link href="/" className="font-bold text-xl">
          Vievlog
        </Link>
        <div className="flex gap-2">
          {links.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={`px-4 py-2 rounded-md transition-colors ${
                pathname === link.href
                  ? "bg-blue-600 text-white"
                  : "text-gray-700 hover:bg-gray-100"
              }`}
            >
              {link.name}
            </Link>
          ))}
        </div>
      </nav>
    </header>
  );
}
```

#### Bước 2: Root Layout

```tsx
// app/layout.tsx
import "./globals.css";
import Navbar from "./components/Navbar";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="vi">
      <body className="min-h-screen bg-gray-50">
        <Navbar />
        <main>{children}</main>
        <footer className="p-4 text-center text-sm text-gray-500 border-t">
          © 2025 Vievlog
        </footer>
      </body>
    </html>
  );
}
```

#### Bước 3: Auth Layout (không có Navbar)

```tsx
// app/(auth)/layout.tsx
export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="w-full max-w-md">
        {children}
      </div>
    </div>
  );
}
```

#### Bước 4: Tạo Login Page

```tsx
// app/(auth)/login/page.tsx
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export const metadata = {
  title: "Đăng nhập",
};

export default function LoginPage() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Đăng nhập</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Input placeholder="Email" type="email" />
        <Input placeholder="Mật khẩu" type="password" />
        <Button className="w-full">Đăng nhập</Button>
      </CardContent>
    </Card>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo Dashboard layout với sidebar

<details>
<summary>💡 Gợi ý</summary>

- Sidebar cố định bên trái với width `w-64`
- Main content chiếm phần còn lại với `flex-1`
- Highlight menu hiện tại với `usePathname`

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// app/dashboard/layout.tsx
"use client"

import Link from "next/link";
import { usePathname } from "next/navigation";

const menuItems = [
  { name: "Tổng quan", href: "/dashboard" },
  { name: "Cài đặt", href: "/dashboard/settings" },
  { name: "Người dùng", href: "/dashboard/users" },
];

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();

  return (
    <div className="flex min-h-screen">
      <aside className="w-64 bg-gray-900 text-white">
        <div className="p-4 font-bold text-xl border-b border-gray-700">
          Dashboard
        </div>
        <nav className="p-4 space-y-2">
          {menuItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={`block px-4 py-2 rounded ${
                pathname === item.href
                  ? "bg-blue-600"
                  : "hover:bg-gray-800"
              }`}
            >
              {item.name}
            </Link>
          ))}
        </nav>
      </aside>
      <main className="flex-1 p-6 bg-gray-100">
        {children}
      </main>
    </div>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo Responsive Navbar với mobile drawer

**Mở rộng**:

- Desktop: Hiển thị menu ngang
- Mobile: Hamburger menu với slide-in drawer
- Animation khi mở/đóng menu

### 3.3. Mini Project

**Dự án**: Admin Panel Layout

**Mô tả**: Xây dựng layout hoàn chỉnh cho admin panel

**Yêu cầu chức năng:**

1. Header với logo, user menu dropdown
2. Sidebar collapsible (có thể thu gọn)
3. Breadcrumbs hiển thị đường dẫn hiện tại
4. Responsive trên mobile (sidebar thành drawer)

**Technical Stack:**

- Next.js 14+ với App Router
- TailwindCSS + ShadcnUI
- Lucide icons

**Hướng dẫn triển khai:**

```
app/
├── admin/
│   ├── layout.tsx      # Admin layout chính
│   ├── components/
│   │   ├── Header.tsx
│   │   ├── Sidebar.tsx
│   │   └── Breadcrumbs.tsx
│   ├── page.tsx        # Dashboard
│   ├── users/page.tsx
│   └── settings/page.tsx
```

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu cách hoạt động của layout trong App Router
- [ ] Tạo được nested layouts
- [ ] Sử dụng `next/link` và `usePathname`
- [ ] Tạo metadata cho SEO
- [ ] (Tùy chọn) Hoàn thành mini project Admin Panel

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Sự khác nhau giữa `layout.tsx` và `template.tsx`?
2. **Ứng dụng**: Làm sao để có layout riêng cho auth mà không kế thừa navbar?
3. **Phân tích**: Khi nào nên dùng route groups?
4. **Thực hành**: Demo navigation với highlight menu?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Các pattern tổ chức layout trong Next.js
- Demo hệ thống layout đã tạo
- Chia sẻ best practices về responsive navigation
- Tips khi làm việc với usePathname

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Hook nào dùng để lấy URL path hiện tại trong Next.js?

- A. `useRouter`
- B. `usePathname`
- C. `useLocation`
- D. `usePath`

**Câu 2**: Route groups trong App Router sử dụng cú pháp nào?

- A. `[group-name]`
- B. `(group-name)`
- C. `_group-name`
- D. `@group-name`

**Câu 3**: Layout nào sẽ re-mount component mỗi khi navigate?

- A. `layout.tsx`
- B. `page.tsx`
- C. `template.tsx`
- D. `route.tsx`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Làm sao để auth layout không có navbar?</strong></summary>

Sử dụng route groups:

```
app/
├── (main)/
│   ├── layout.tsx    # Có navbar
│   └── page.tsx
└── (auth)/
    ├── layout.tsx    # Không có navbar
    └── login/page.tsx
```

Route groups `(main)` và `(auth)` cho phép có layout riêng biệt mà không ảnh hưởng đến URL.

</details>

<details>
<summary><strong>Q2: Sự khác nhau giữa layout và template?</strong></summary>

- **layout.tsx**: Persist state giữa các navigation. Component không bị re-mount khi chuyển giữa các trang con.
- **template.tsx**: Re-mount mỗi khi navigate. Dùng khi cần reset state hoặc chạy effect mỗi lần.

Hầu hết trường hợp nên dùng `layout.tsx`.

</details>

<details>
<summary><strong>Q3: Làm sao highlight nested routes?</strong></summary>

Dùng `startsWith` thay vì so sánh exact:

```tsx
const isActive = pathname === href || pathname.startsWith(`${href}/`);
```

Ví dụ: Khi ở `/dashboard/users`, cả menu "Dashboard" và "Users" đều được highlight.

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 6

</footer>
