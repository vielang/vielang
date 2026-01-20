
# Server Components vs Client Components

> **Mô tả ngắn gọn**: Tìm hiểu sự khác biệt giữa Server và Client Components, khi nào dùng, cách fetch data và tổ chức code hiệu quả.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu rõ sự khác biệt giữa Server Components và Client Components
- [ ] Biết khi nào nên dùng từng loại component để tối ưu hiệu suất
- [ ] Sử dụng thành thạo directive `"use client"`
- [ ] Fetch dữ liệu trong Server Component
- [ ] Xử lý tương tác người dùng trong Client Component
- [ ] Áp dụng Suspense và Streaming

### Kiến Thức Yêu Cầu

- Bài 1-6: Next.js App Router, TypeScript, Layout
- React hooks cơ bản (useState, useEffect)
- Async/await và Promises

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về Server/Client Components | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành data fetching | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Tổng Quan Về Server & Client Components

> **💡 Định nghĩa**:
> - **Server Components**: Component chạy hoàn toàn trên server, không gửi JavaScript xuống client.
> - **Client Components**: Component được bundle và gửi xuống client, hỗ trợ interactivity.

**Tại sao cần phân biệt?**

- Giảm bundle size gửi về client
- Tối ưu performance và SEO
- Bảo mật logic nhạy cảm ở server
- Cho phép fetch data trực tiếp trong component

### 1.2. So Sánh Chi Tiết

| Đặc điểm | Server Components | Client Components |
|----------|-------------------|-------------------|
| Chạy ở đâu? | Trên server | Trên trình duyệt |
| Bundle xuống client? | Không | Có |
| Dùng useState, useEffect? | Không | Có |
| Dùng event handlers? | Không | Có |
| Fetch data? | Trực tiếp với async/await | Dùng useEffect hoặc SWR |
| SEO | Tốt hơn | Cần SSR/hydration |
| Performance | Tốt hơn (ít JS) | Nặng hơn |

### 1.3. Cách Khai Báo

#### Server Component (Mặc định)

```tsx
// app/components/HelloServer.tsx
// Không cần khai báo gì - mặc định là Server Component

export default function HelloServer() {
  console.log("Log này chỉ hiện trên server terminal");

  return <div>Hello from Server!</div>;
}
```

#### Client Component

```tsx
// app/components/HelloClient.tsx
"use client"; // Bắt buộc để kích hoạt client-side logic

import { useState } from "react";

export default function HelloClient() {
  const [count, setCount] = useState(0);

  return (
    <button onClick={() => setCount(count + 1)}>
      Clicked {count} times
    </button>
  );
}
```

**📝 Lưu ý:** `"use client"` phải ở dòng đầu tiên của file.

### 1.4. Khi Nào Sử Dụng?

#### Server Component - Dùng khi:

- Hiển thị dữ liệu tĩnh hoặc dynamic
- Không cần interactivity
- Cần bảo mật (không expose logic lên client)
- Tối ưu SEO
- Fetch data từ database hoặc API

#### Client Component - Dùng khi:

- Có tương tác người dùng (click, input, animation)
- Sử dụng React hooks (`useState`, `useEffect`, `useRef`)
- Sử dụng browser APIs (localStorage, geolocation)
- Sử dụng thư viện JS chỉ hoạt động phía client

### 1.5. Data Fetching Trong Server Component

```tsx
// app/components/UserList.tsx (Server Component)

interface User {
  id: number;
  name: string;
  email: string;
}

async function getUsers(): Promise<User[]> {
  const res = await fetch("https://jsonplaceholder.typicode.com/users", {
    next: { revalidate: 60 }, // Cache 60 giây
  });
  return res.json();
}

export default async function UserList() {
  const users = await getUsers();

  return (
    <ul className="space-y-2">
      {users.map((user) => (
        <li key={user.id} className="p-4 border rounded">
          <p className="font-bold">{user.name}</p>
          <p className="text-gray-500">{user.email}</p>
        </li>
      ))}
    </ul>
  );
}
```

**📝 Lưu ý:** Không cần `useEffect`, không cần `useState`. Server xử lý và trả về HTML sẵn.

### 1.6. Suspense và Streaming

> **💡 Định nghĩa**:
> - **Hydration**: Quá trình React nối kết event handlers vào HTML đã render từ server.
> - **Streaming**: Server render từng phần HTML khi có dữ liệu, không cần đợi tất cả.

```tsx
// app/page.tsx
import { Suspense } from "react";
import UserList from "./components/UserList";

export default function HomePage() {
  return (
    <main className="p-6">
      <h1 className="text-2xl font-bold mb-4">Danh sách người dùng</h1>

      <Suspense fallback={<LoadingSkeleton />}>
        <UserList />
      </Suspense>
    </main>
  );
}

function LoadingSkeleton() {
  return (
    <div className="space-y-2">
      {[1, 2, 3].map((i) => (
        <div key={i} className="p-4 border rounded animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/3 mb-2"></div>
          <div className="h-3 bg-gray-200 rounded w-1/2"></div>
        </div>
      ))}
    </div>
  );
}
```

### 1.7. Kết Hợp Server và Client Components

```tsx
// app/components/UserCard.tsx (Server Component)
import LikeButton from "./LikeButton";

interface User {
  id: number;
  name: string;
}

export default function UserCard({ user }: { user: User }) {
  return (
    <div className="p-4 border rounded flex justify-between items-center">
      <span>{user.name}</span>
      <LikeButton /> {/* Client Component bên trong Server Component */}
    </div>
  );
}
```

```tsx
// app/components/LikeButton.tsx (Client Component)
"use client";

import { useState } from "react";

export default function LikeButton() {
  const [likes, setLikes] = useState(0);

  return (
    <button
      onClick={() => setLikes(likes + 1)}
      className="px-3 py-1 bg-blue-500 text-white rounded"
    >
      👍 {likes}
    </button>
  );
}
```

**📝 Pattern quan trọng:** Server Component có thể render Client Component, nhưng ngược lại thì không được import Server Component vào Client Component.

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn cần xây dựng trang hiển thị danh sách sản phẩm:
- Fetch data từ API
- Mỗi sản phẩm có nút "Thêm vào giỏ hàng"
- Hiển thị loading khi đang fetch

**Yêu cầu**:

- Tối ưu performance
- SEO tốt cho danh sách sản phẩm
- Interactive cho nút thêm giỏ hàng

**🤔 Câu hỏi suy ngẫm:**

1. Component nào nên là Server, component nào nên là Client?
2. Làm sao để fetch data hiệu quả?
3. Cách tổ chức code để tái sử dụng?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. **Server Component**: `ProductList` (fetch data), `ProductCard` (hiển thị thông tin)
2. **Client Component**: `AddToCartButton` (cần state và onClick)
3. **Cấu trúc:**

```
components/
├── ProductList.tsx      # Server - fetch products
├── ProductCard.tsx      # Server - render card
└── AddToCartButton.tsx  # Client - interactive
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Mặc định component là Server trong App Router. Chỉ thêm `"use client"` khi thực sự cần.

#### ✅ Nên Làm

```tsx
// Tách nhỏ Client Component
// ProductCard.tsx (Server)
import AddToCartButton from "./AddToCartButton";

export default function ProductCard({ product }) {
  return (
    <div className="p-4 border rounded">
      <h3>{product.name}</h3>
      <p>{product.price}</p>
      <AddToCartButton productId={product.id} />
    </div>
  );
}

// AddToCartButton.tsx (Client) - chỉ phần cần interactive
"use client";
export default function AddToCartButton({ productId }) {
  const handleAdd = () => {
    // Add to cart logic
  };
  return <button onClick={handleAdd}>Thêm vào giỏ</button>;
}
```

**Tại sao tốt:**

- Chỉ phần nhỏ cần JavaScript được gửi xuống client
- Product data được render trên server (SEO tốt)
- Performance tối ưu

#### ❌ Không Nên Làm

```tsx
// Cả component là Client vì một button
"use client";

export default function ProductCard({ product }) {
  const handleAdd = () => { /* ... */ };

  return (
    <div className="p-4 border rounded">
      <h3>{product.name}</h3>
      <p>{product.price}</p>
      <button onClick={handleAdd}>Thêm vào giỏ</button>
    </div>
  );
}
```

**Tại sao không tốt:**

- Toàn bộ component bị bundle xuống client
- Mất lợi ích của Server Component
- Bundle size lớn hơn không cần thiết

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "useState is not defined" | Dùng hooks trong Server Component | Thêm `"use client"` |
| "async/await in Client Component" | Client Component không hỗ trợ async | Dùng useEffect hoặc SWR |
| Props không serialize được | Truyền function vào Client Component từ Server | Chỉ truyền data serializable |
| Component không re-render | State nằm ở Server Component | Di chuyển state vào Client Component |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo trang hiển thị danh sách người dùng với nút Like

**Yêu cầu kỹ thuật:**

- UserList: Server Component, fetch data
- LikeButton: Client Component, handle click

#### Bước 1: Tạo LikeButton (Client)

```tsx
// app/components/LikeButton.tsx
"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";

export default function LikeButton() {
  const [likes, setLikes] = useState(0);
  const [isLiked, setIsLiked] = useState(false);

  const handleLike = () => {
    setLikes(isLiked ? likes - 1 : likes + 1);
    setIsLiked(!isLiked);
  };

  return (
    <Button
      variant={isLiked ? "default" : "outline"}
      size="sm"
      onClick={handleLike}
    >
      {isLiked ? "❤️" : "🤍"} {likes}
    </Button>
  );
}
```

#### Bước 2: Tạo UserList (Server)

```tsx
// app/components/UserList.tsx
import LikeButton from "./LikeButton";

interface User {
  id: number;
  name: string;
  email: string;
}

async function getUsers(): Promise<User[]> {
  const res = await fetch("https://jsonplaceholder.typicode.com/users", {
    next: { revalidate: 60 },
  });
  return res.json();
}

export default async function UserList() {
  const users = await getUsers();

  return (
    <div className="space-y-4">
      {users.map((user) => (
        <div
          key={user.id}
          className="flex items-center justify-between p-4 border rounded-lg"
        >
          <div>
            <h3 className="font-semibold">{user.name}</h3>
            <p className="text-sm text-gray-500">{user.email}</p>
          </div>
          <LikeButton />
        </div>
      ))}
    </div>
  );
}
```

#### Bước 3: Tạo Page với Suspense

```tsx
// app/users/page.tsx
import { Suspense } from "react";
import UserList from "@/components/UserList";

export const metadata = {
  title: "Danh sách người dùng",
};

export default function UsersPage() {
  return (
    <main className="container mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">Danh sách người dùng</h1>

      <Suspense fallback={<UserListSkeleton />}>
        <UserList />
      </Suspense>
    </main>
  );
}

function UserListSkeleton() {
  return (
    <div className="space-y-4">
      {[1, 2, 3, 4, 5].map((i) => (
        <div key={i} className="p-4 border rounded-lg animate-pulse">
          <div className="flex items-center justify-between">
            <div className="space-y-2">
              <div className="h-4 w-32 bg-gray-200 rounded"></div>
              <div className="h-3 w-48 bg-gray-200 rounded"></div>
            </div>
            <div className="h-8 w-16 bg-gray-200 rounded"></div>
          </div>
        </div>
      ))}
    </div>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo trang Blog với ToggleContent

<details>
<summary>💡 Gợi ý</summary>

- `BlogList`: Server Component, fetch posts
- `ToggleContent`: Client Component, ẩn/hiện nội dung
- API: `https://jsonplaceholder.typicode.com/posts`

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// components/ToggleContent.tsx
"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";

export default function ToggleContent({ content }: { content: string }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div>
      <Button
        variant="link"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
      >
        {isOpen ? "Ẩn nội dung" : "Xem nội dung"}
      </Button>
      {isOpen && (
        <p className="mt-2 text-gray-600">{content}</p>
      )}
    </div>
  );
}
```

```tsx
// components/BlogList.tsx
import ToggleContent from "./ToggleContent";

interface Post {
  id: number;
  title: string;
  body: string;
}

async function getPosts(): Promise<Post[]> {
  const res = await fetch("https://jsonplaceholder.typicode.com/posts?_limit=5");
  return res.json();
}

export default async function BlogList() {
  const posts = await getPosts();

  return (
    <div className="space-y-6">
      {posts.map((post) => (
        <article key={post.id} className="p-4 border rounded-lg">
          <h2 className="text-lg font-semibold mb-2">{post.title}</h2>
          <ToggleContent content={post.body} />
        </article>
      ))}
    </div>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo trang Products với filter và sort (Client-side)

**Mở rộng**:

- Fetch products từ Server Component
- Filter và sort ở Client Component
- Giữ nguyên data gốc, chỉ thay đổi hiển thị

### 3.3. Mini Project

**Dự án**: Dashboard với Real-time Stats

**Mô tả**: Xây dựng dashboard kết hợp Server và Client Components

**Yêu cầu chức năng:**

1. Stats cards: Server Component (fetch một lần)
2. Chart: Client Component (có thể tương tác)
3. Recent activities: Server Component với Suspense
4. Notification bell: Client Component với state

**Technical Stack:**

- Next.js 14+ với App Router
- TailwindCSS
- Recharts hoặc Chart.js cho biểu đồ

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu sự khác biệt Server vs Client Components
- [ ] Biết khi nào dùng từng loại
- [ ] Fetch data trong Server Component
- [ ] Sử dụng Suspense cho loading
- [ ] (Tùy chọn) Hoàn thành mini project Dashboard

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Tại sao Server Component không thể dùng useState?
2. **Ứng dụng**: Khi nào bạn chọn Client Component?
3. **Phân tích**: So sánh bundle size khi dùng Server vs Client Component?
4. **Thực hành**: Demo kết hợp Server và Client Components?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Sự khác biệt Server vs Client Components
- Demo trang fetch data với Suspense
- Chia sẻ pattern tổ chức code
- Performance optimization tips

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Mặc định component trong App Router là gì?

- A. Client Component
- B. Server Component
- C. Hybrid Component
- D. Static Component

**Câu 2**: Directive nào dùng để khai báo Client Component?

- A. `"use server"`
- B. `"use client"`
- C. `"client side"`
- D. `export const dynamic = "client"`

**Câu 3**: Server Component có thể làm gì mà Client Component không thể?

- A. Sử dụng useState
- B. Xử lý onClick
- C. Fetch data với async/await trực tiếp
- D. Sử dụng useEffect

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Có thể import Server Component vào Client Component không?</strong></summary>

Không! Client Component không thể import Server Component trực tiếp. Tuy nhiên, bạn có thể:

1. **Pass as children:**
```tsx
// ClientWrapper.tsx
"use client";
export default function ClientWrapper({ children }) {
  return <div onClick={...}>{children}</div>;
}

// Page.tsx (Server)
<ClientWrapper>
  <ServerComponent /> {/* Được truyền như children */}
</ClientWrapper>
```

2. **Pass as props:**
```tsx
<ClientComponent
  serverContent={<ServerComponent />}
/>
```

</details>

<details>
<summary><strong>Q2: Làm sao để biết component nào là Server/Client?</strong></summary>

- **Server Component**: Không có `"use client"` ở đầu file
- **Client Component**: Có `"use client"` ở đầu file

Một số dấu hiệu cần Client Component:
- Sử dụng hooks (useState, useEffect,...)
- Sử dụng event handlers (onClick, onChange,...)
- Sử dụng browser APIs

</details>

<details>
<summary><strong>Q3: Server Component có được cache không?</strong></summary>

Có! Next.js tự động cache kết quả của Server Components. Bạn có thể control cache behavior:

```tsx
// Revalidate mỗi 60 giây
fetch(url, { next: { revalidate: 60 } });

// Không cache
fetch(url, { cache: 'no-store' });

// Force static
export const dynamic = 'force-static';

// Force dynamic
export const dynamic = 'force-dynamic';
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 7

</footer>
