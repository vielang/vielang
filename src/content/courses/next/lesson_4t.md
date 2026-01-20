
# TailwindCSS và Styling

> **Mô tả ngắn gọn**: Tìm hiểu cách cài đặt TailwindCSS, triết lý utility-first, responsive design và dark mode trong Next.js.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu rõ cách cài đặt và cấu hình TailwindCSS trong Next.js
- [ ] Nắm được triết lý "Utility-First" của TailwindCSS
- [ ] Sử dụng các lớp Tailwind để thiết kế giao diện nhanh chóng
- [ ] Áp dụng Responsive Design và Dark Mode
- [ ] Biết cách tùy chỉnh theme với TailwindCSS

### Kiến Thức Yêu Cầu

- Bài 1-3: Next.js App Router và TypeScript
- HTML, CSS cơ bản
- Hiểu về responsive design

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về TailwindCSS | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành styling | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. TailwindCSS Là Gì?

> **💡 Định nghĩa**: TailwindCSS là một utility-first CSS framework, cung cấp các class ngắn gọn để styling trực tiếp trong HTML/JSX mà không cần viết CSS riêng.

**Ví dụ so sánh:**

CSS truyền thống:

```css
.btn {
  background-color: blue;
  padding: 8px 16px;
  color: white;
  border-radius: 4px;
}
```

TailwindCSS:

```jsx
<button className="bg-blue-500 px-4 py-2 text-white rounded">
  Click
</button>
```

**Tại sao dùng TailwindCSS?**

- Phát triển nhanh, không cần chuyển đổi giữa file HTML và CSS
- Dễ tái sử dụng thông qua components
- Không lo xung đột CSS giữa các components
- Bundle size nhỏ nhờ purge CSS không sử dụng

### 1.2. Cài Đặt TailwindCSS

#### Bước 1: Cài đặt dependencies

```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

#### Bước 2: Cấu hình `tailwind.config.js`

```js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/**/*.{js,ts,jsx,tsx}",
    "./components/**/*.{js,ts,jsx,tsx}"
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

**📝 Giải thích:**

- `content`: Đường dẫn các file sử dụng Tailwind classes
- `theme.extend`: Mở rộng theme mặc định
- `plugins`: Thêm plugins (forms, typography,...)

#### Bước 3: Tạo file CSS globals

```css
/* app/globals.css */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

#### Bước 4: Import vào layout

```tsx
// app/layout.tsx
import "./globals.css";

export default function RootLayout({ children }) {
  return (
    <html lang="vi">
      <body>{children}</body>
    </html>
  );
}
```

### 1.3. Utility-First CSS

> **💡 Định nghĩa**: Sử dụng các class nhỏ, cụ thể để style từng thuộc tính thay vì dùng class tổng hợp.

**Các utility classes phổ biến:**

| Category | Examples |
|----------|----------|
| Spacing | `p-4`, `m-2`, `px-6`, `my-auto` |
| Colors | `bg-blue-500`, `text-gray-700` |
| Typography | `text-lg`, `font-bold`, `text-center` |
| Layout | `flex`, `grid`, `items-center`, `justify-between` |
| Border | `border`, `rounded-lg`, `border-gray-200` |
| Shadow | `shadow`, `shadow-md`, `shadow-lg` |

**Ví dụ Card component:**

```jsx
<div className="bg-white p-6 rounded-lg shadow-md">
  <h2 className="text-xl font-bold text-gray-800 mb-2">
    Tiêu đề
  </h2>
  <p className="text-gray-600">
    Nội dung card
  </p>
</div>
```

### 1.4. Responsive Design

TailwindCSS hỗ trợ responsive bằng breakpoint prefixes:

| Breakpoint | Min-width | CSS |
|------------|-----------|-----|
| `sm:` | 640px | `@media (min-width: 640px)` |
| `md:` | 768px | `@media (min-width: 768px)` |
| `lg:` | 1024px | `@media (min-width: 1024px)` |
| `xl:` | 1280px | `@media (min-width: 1280px)` |
| `2xl:` | 1536px | `@media (min-width: 1536px)` |

**Ví dụ responsive:**

```jsx
<div className="w-full md:w-1/2 lg:w-1/3">
  {/* Full width trên mobile, 1/2 trên tablet, 1/3 trên desktop */}
</div>

<p className="text-sm md:text-base lg:text-lg">
  {/* Text size thay đổi theo breakpoint */}
</p>
```

**Grid responsive:**

```jsx
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  <div>Item 1</div>
  <div>Item 2</div>
  <div>Item 3</div>
</div>
```

### 1.5. Dark Mode

#### Cấu hình Dark Mode

```js
// tailwind.config.js
module.exports = {
  darkMode: 'class',  // hoặc 'media'
  // ...
}
```

**📝 Modes:**

- `class`: Toggle bằng class `.dark` trên `<html>`
- `media`: Theo system preference

#### Sử dụng Dark Mode

```jsx
<div className="bg-white dark:bg-gray-900 text-black dark:text-white">
  Dark Mode Ready!
</div>
```

#### Toggle Dark Mode

```tsx
"use client"

export function ThemeToggle() {
  const toggleDark = () => {
    document.documentElement.classList.toggle('dark');
  };

  return (
    <button onClick={toggleDark} className="p-2 rounded bg-gray-200 dark:bg-gray-700">
      Toggle Theme
    </button>
  );
}
```

### 1.6. Tùy Chỉnh Theme

Mở rộng màu sắc, font, spacing trong `tailwind.config.js`:

```js
module.exports = {
  theme: {
    extend: {
      colors: {
        brand: {
          light: '#60a5fa',
          DEFAULT: '#3b82f6',
          dark: '#1d4ed8',
        }
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
      },
      spacing: {
        '128': '32rem',
      }
    }
  }
}
```

**Sử dụng custom colors:**

```jsx
<h1 className="text-brand">My Brand</h1>
<div className="bg-brand-light">Light variant</div>
<button className="bg-brand-dark">Dark variant</button>
```

### 1.7. So Sánh TailwindCSS vs CSS Modules

| Tiêu chí | TailwindCSS | CSS Modules |
|----------|-------------|-------------|
| Tốc độ phát triển | Nhanh | Trung bình |
| Learning curve | Trung bình | Dễ |
| Bundle size | Tối ưu (purge) | Phụ thuộc code |
| Tùy biến | Cao | Rất cao |
| Component isolation | Qua components | Qua modules |

**Khi nào dùng gì?**

- **TailwindCSS**: Layout, spacing, responsive, prototyping nhanh
- **CSS Modules**: Styles phức tạp, animations, CSS-in-JS alternatives

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng một Profile Card responsive với:
- Avatar tròn
- Tên và email
- Nút Follow
- Hỗ trợ Dark Mode

**Yêu cầu**:

- Mobile: Card full width
- Tablet+: Card có max-width
- Dark mode toggle

**🤔 Câu hỏi suy ngẫm:**

1. Classes nào cần cho avatar tròn?
2. Làm sao để card có shadow và border radius?
3. Cách triển khai dark mode cho từng element?

<details>
<summary>💭 Gợi ý phân tích</summary>

- Avatar: `w-24 h-24 rounded-full`
- Card: `max-w-sm mx-auto bg-white shadow-md rounded-lg p-6`
- Dark mode: Thêm `dark:` prefix cho mỗi màu cần thay đổi

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Luôn import `globals.css` vào layout, nếu không Tailwind sẽ không hoạt động.

#### ✅ Nên Làm

```jsx
// Sử dụng semantic spacing
<div className="p-4 md:p-6 lg:p-8">
  <h1 className="text-xl md:text-2xl font-bold mb-4">
    Title
  </h1>
  <p className="text-gray-600 dark:text-gray-300">
    Content
  </p>
</div>
```

**Tại sao tốt:**

- Responsive rõ ràng với breakpoint prefixes
- Dark mode được xử lý ở từng element
- Spacing nhất quán

#### ❌ Không Nên Làm

```jsx
// Quá nhiều classes, khó đọc
<div className="p-4 m-2 bg-white rounded shadow flex items-center justify-between w-full max-w-lg mx-auto border border-gray-200 hover:shadow-lg transition-shadow duration-200">
```

**Tại sao không tốt:**

- Khó đọc và maintain
- Nên tách thành component hoặc dùng `@apply`

**Cách cải thiện:**

```tsx
// Tách thành component
function Card({ children, className = "" }) {
  return (
    <div className={`
      p-4 bg-white rounded shadow
      max-w-lg mx-auto border border-gray-200
      hover:shadow-lg transition-shadow
      ${className}
    `}>
      {children}
    </div>
  );
}
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Tailwind không hoạt động | Chưa import globals.css | Import vào layout.tsx |
| Màu không hiện | Không có trong content config | Thêm đường dẫn vào content array |
| Class bị ghi đè | Thứ tự class | Class sau ghi đè class trước |
| Dark mode không toggle | darkMode không phải 'class' | Đặt `darkMode: 'class'` |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng Profile Card responsive với dark mode

**Yêu cầu kỹ thuật:**

- Avatar, tên, email, nút Follow
- Responsive (mobile-first)
- Dark mode support

#### Bước 1: Tạo Component

```tsx
// components/ProfileCard.tsx
export default function ProfileCard() {
  return (
    <div className="max-w-sm mx-auto bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 text-center">
      {/* Avatar */}
      <img
        className="w-24 h-24 rounded-full mx-auto mb-4 object-cover"
        src="https://i.pravatar.cc/150?img=3"
        alt="User avatar"
      />

      {/* Info */}
      <h2 className="text-xl font-semibold text-gray-800 dark:text-white">
        Nguyen Van A
      </h2>
      <p className="text-gray-600 dark:text-gray-300 mb-4">
        nguyenvana@example.com
      </p>

      {/* Button */}
      <button className="px-6 py-2 bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors">
        Follow
      </button>
    </div>
  );
}
```

**📝 Giải thích:**

- `max-w-sm mx-auto`: Card có max-width và căn giữa
- `dark:bg-gray-800`: Background tối khi dark mode
- `rounded-full`: Avatar và button bo tròn hoàn toàn
- `transition-colors`: Smooth hover effect

#### Bước 2: Sử dụng trong Page

```tsx
// app/page.tsx
import ProfileCard from "@/components/ProfileCard";

export default function HomePage() {
  return (
    <main className="min-h-screen bg-gray-100 dark:bg-gray-900 py-12">
      <ProfileCard />
    </main>
  );
}
```

#### Bước 3: Thêm Theme Toggle

```tsx
// components/ThemeToggle.tsx
"use client"

import { useEffect, useState } from "react";

export default function ThemeToggle() {
  const [dark, setDark] = useState(false);

  useEffect(() => {
    if (dark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [dark]);

  return (
    <button
      onClick={() => setDark(!dark)}
      className="fixed top-4 right-4 p-2 rounded-lg bg-gray-200 dark:bg-gray-700"
    >
      {dark ? '☀️' : '🌙'}
    </button>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo BlogPostCard với tiêu đề, ảnh, đoạn mô tả, và tag

<details>
<summary>💡 Gợi ý</summary>

- Dùng `aspect-video` cho image ratio
- `line-clamp-2` để giới hạn text
- Tags dùng `inline-flex` với `rounded-full`

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
export default function BlogPostCard() {
  return (
    <div className="max-w-md bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden">
      {/* Image */}
      <img
        className="w-full aspect-video object-cover"
        src="https://picsum.photos/400/225"
        alt="Blog cover"
      />

      <div className="p-4">
        {/* Tags */}
        <div className="flex gap-2 mb-2">
          <span className="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded-full">
            React
          </span>
          <span className="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full">
            Tutorial
          </span>
        </div>

        {/* Title */}
        <h2 className="text-lg font-bold text-gray-800 dark:text-white mb-2">
          Hướng dẫn TailwindCSS
        </h2>

        {/* Description */}
        <p className="text-gray-600 dark:text-gray-300 text-sm line-clamp-2">
          Tìm hiểu cách sử dụng TailwindCSS để xây dựng giao diện nhanh chóng và hiệu quả.
        </p>
      </div>
    </div>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo responsive Navbar với mobile menu

**Mở rộng**:

- Desktop: Hiển thị links ngang
- Mobile: Hamburger menu với slide-in drawer
- Dark mode support

### 3.3. Mini Project

**Dự án**: Landing Page cho sản phẩm

**Mô tả**: Xây dựng landing page responsive với các sections

**Yêu cầu chức năng:**

1. Hero section với tiêu đề lớn và CTA button
2. Features section với 3 cards grid
3. Footer với links và social icons
4. Fully responsive và dark mode

**Technical Stack:**

- Next.js 14+ với App Router
- TailwindCSS
- TypeScript

**Hướng dẫn triển khai:**

```
app/
├── layout.tsx      # Global styles, theme provider
├── page.tsx        # Landing page
└── components/
    ├── Hero.tsx
    ├── Features.tsx
    └── Footer.tsx
```

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Cài đặt và cấu hình TailwindCSS
- [ ] Hiểu triết lý utility-first
- [ ] Sử dụng responsive design với breakpoints
- [ ] Triển khai dark mode
- [ ] (Tùy chọn) Hoàn thành mini project landing page

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Utility-first CSS là gì?
2. **Ứng dụng**: Làm sao để một element có màu nền khác nhau ở dark mode?
3. **Phân tích**: So sánh TailwindCSS với CSS truyền thống?
4. **Thực hành**: Demo Profile Card responsive với dark mode?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Ưu điểm của TailwindCSS
- Demo component đã tạo
- Chia sẻ tips responsive design
- Best practices khi dùng Tailwind

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Để set darkMode bằng class toggle, cấu hình nào đúng trong tailwind.config.js?

- A. `darkMode: 'media'`
- B. `darkMode: 'class'`
- C. `darkMode: true`
- D. `darkMode: 'toggle'`

**Câu 2**: Breakpoint `md:` trong TailwindCSS tương đương min-width bao nhiêu?

- A. 640px
- B. 768px
- C. 1024px
- D. 1280px

**Câu 3**: Để tạo avatar tròn với Tailwind, class nào phù hợp nhất?

- A. `rounded`
- B. `rounded-lg`
- C. `rounded-full`
- D. `circle`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Tại sao styles không hoạt động?</strong></summary>

Kiểm tra các điều sau:

1. Đã import `globals.css` vào `layout.tsx` chưa?
2. `tailwind.config.js` có đúng đường dẫn trong `content` không?
3. Đã chạy `npm run dev` lại sau khi cấu hình?

```js
// Đảm bảo content bao gồm tất cả files
content: [
  "./app/**/*.{js,ts,jsx,tsx,mdx}",
  "./components/**/*.{js,ts,jsx,tsx,mdx}",
]
```

</details>

<details>
<summary><strong>Q2: Làm sao để dùng custom colors?</strong></summary>

Thêm vào `theme.extend.colors` trong `tailwind.config.js`:

```js
theme: {
  extend: {
    colors: {
      primary: '#3b82f6',
      secondary: '#10b981',
    }
  }
}
```

Sau đó sử dụng: `bg-primary`, `text-secondary`

</details>

<details>
<summary><strong>Q3: Có nên dùng @apply không?</strong></summary>

`@apply` cho phép tái sử dụng Tailwind classes trong CSS:

```css
.btn-primary {
  @apply px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600;
}
```

**Khi nào dùng:**
- Styles lặp lại nhiều nơi không thể tách component
- Base styles cho third-party components

**Khi nào không nên:**
- Có thể tách thành React component
- Chỉ dùng ở 1-2 nơi

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 4

</footer>
