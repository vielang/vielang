
# Data Fetching và API Integration

> **Mô tả ngắn gọn**: Tìm hiểu các cách fetch dữ liệu trong Next.js App Router: Server-side, Client-side, Route Handlers và SWR.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu các cách fetch dữ liệu trong Next.js App Router
- [ ] Sử dụng `fetch()` trong Server Components
- [ ] Tạo Route Handlers (API Routes) trong App Router
- [ ] Sử dụng SWR để fetch data từ client
- [ ] Xử lý loading state, error state và caching
- [ ] Biết lựa chọn cách fetch phù hợp cho từng use case

### Kiến Thức Yêu Cầu

- Bài 1-8: Next.js App Router, Server/Client Components, State Management
- Async/await và Promises
- REST API cơ bản

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về Data Fetching | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành tạo API và fetch data | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Tổng Quan Về Data Fetching

> **💡 Định nghĩa**: Data Fetching là quá trình lấy dữ liệu từ nguồn bên ngoài (API, database) để hiển thị lên giao diện.

**Các cách fetch trong Next.js:**

| Cách | Nơi chạy | Use case |
|------|----------|----------|
| Server Component fetch | Server | SEO, initial data, sensitive data |
| Client Component fetch | Browser | Real-time, user-specific data |
| Route Handlers | Server | API endpoints, webhooks |

### 1.2. Data Fetching Trong Server Components

```tsx
// app/posts/page.tsx
interface Post {
  id: number;
  title: string;
  body: string;
}

async function getPosts(): Promise<Post[]> {
  const res = await fetch("https://jsonplaceholder.typicode.com/posts", {
    next: { revalidate: 60 }, // ISR: Revalidate mỗi 60 giây
  });

  if (!res.ok) {
    throw new Error("Failed to fetch posts");
  }

  return res.json();
}

export default async function PostsPage() {
  const posts = await getPosts();

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Bài viết</h1>
      {posts.slice(0, 10).map((post) => (
        <article key={post.id} className="p-4 border rounded">
          <h2 className="font-semibold">{post.title}</h2>
          <p className="text-gray-600 line-clamp-2">{post.body}</p>
        </article>
      ))}
    </div>
  );
}
```

**Ưu điểm Server-side fetch:**

- Tải trang nhanh hơn (không cần fetch lại trên client)
- Tối ưu SEO (content có sẵn trong HTML)
- Có thể truy cập trực tiếp database
- Bảo mật API keys

### 1.3. Caching và Revalidation

```tsx
// Static Data - Cache vĩnh viễn
fetch(url); // Mặc định cache

// Revalidate theo thời gian (ISR)
fetch(url, { next: { revalidate: 60 } }); // Refresh mỗi 60s

// Revalidate theo tag
fetch(url, { next: { tags: ["posts"] } });

// Không cache (Dynamic)
fetch(url, { cache: "no-store" });
```

**Revalidate on-demand:**

```tsx
// app/api/revalidate/route.ts
import { revalidateTag, revalidatePath } from "next/cache";
import { NextRequest, NextResponse } from "next/server";

export async function POST(request: NextRequest) {
  const { tag, path } = await request.json();

  if (tag) {
    revalidateTag(tag);
  }

  if (path) {
    revalidatePath(path);
  }

  return NextResponse.json({ revalidated: true });
}
```

### 1.4. Route Handlers (API Routes)

> **💡 Định nghĩa**: Route Handlers là API endpoints trong App Router, nằm trong `/app/api`.

#### GET Request

```tsx
// app/api/posts/route.ts
import { NextResponse } from "next/server";

const posts = [
  { id: 1, title: "Next.js là gì?", author: "John" },
  { id: 2, title: "App Router chuyên sâu", author: "Jane" },
];

export async function GET() {
  return NextResponse.json(posts);
}
```

#### POST Request

```tsx
// app/api/posts/route.ts
import { NextRequest, NextResponse } from "next/server";

export async function POST(request: NextRequest) {
  const body = await request.json();

  // Validate
  if (!body.title) {
    return NextResponse.json(
      { error: "Title is required" },
      { status: 400 }
    );
  }

  // Tạo post mới (giả lập)
  const newPost = {
    id: Date.now(),
    title: body.title,
    author: body.author || "Anonymous",
  };

  return NextResponse.json(newPost, { status: 201 });
}
```

#### Dynamic Route Handler

```tsx
// app/api/posts/[id]/route.ts
import { NextRequest, NextResponse } from "next/server";

export async function GET(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const { id } = params;

  // Fetch post by id
  const post = await getPostById(id);

  if (!post) {
    return NextResponse.json(
      { error: "Post not found" },
      { status: 404 }
    );
  }

  return NextResponse.json(post);
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  const { id } = params;

  // Delete logic
  await deletePost(id);

  return NextResponse.json({ success: true });
}
```

### 1.5. Client-Side Fetching Với SWR

#### Cài đặt

```bash
npm install swr
```

#### Sử dụng cơ bản

```tsx
// components/PostList.tsx
"use client";

import useSWR from "swr";

interface Post {
  id: number;
  title: string;
}

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export default function PostList() {
  const { data, error, isLoading } = useSWR<Post[]>("/api/posts", fetcher);

  if (isLoading) {
    return <div className="animate-pulse">Đang tải...</div>;
  }

  if (error) {
    return <div className="text-red-500">Lỗi khi tải dữ liệu</div>;
  }

  return (
    <ul className="space-y-2">
      {data?.map((post) => (
        <li key={post.id} className="p-4 border rounded">
          {post.title}
        </li>
      ))}
    </ul>
  );
}
```

#### SWR với Mutation

```tsx
"use client";

import useSWR, { mutate } from "swr";
import { useState } from "react";

export default function PostManager() {
  const { data: posts } = useSWR("/api/posts", fetcher);
  const [title, setTitle] = useState("");

  const handleCreate = async () => {
    // Optimistic update
    const optimisticPost = { id: Date.now(), title };

    await mutate(
      "/api/posts",
      async () => {
        const res = await fetch("/api/posts", {
          method: "POST",
          body: JSON.stringify({ title }),
        });
        const newPost = await res.json();
        return [...(posts || []), newPost];
      },
      {
        optimisticData: [...(posts || []), optimisticPost],
        rollbackOnError: true,
      }
    );

    setTitle("");
  };

  return (
    <div>
      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        placeholder="Tiêu đề"
      />
      <button onClick={handleCreate}>Tạo</button>
    </div>
  );
}
```

### 1.6. Xử Lý Loading và Error

```tsx
// Với Suspense (Server Component)
import { Suspense } from "react";

export default function Page() {
  return (
    <Suspense fallback={<PostsSkeleton />}>
      <PostList />
    </Suspense>
  );
}

// Loading Skeleton
function PostsSkeleton() {
  return (
    <div className="space-y-4">
      {[1, 2, 3].map((i) => (
        <div key={i} className="p-4 border rounded animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
          <div className="h-3 bg-gray-200 rounded w-1/2"></div>
        </div>
      ))}
    </div>
  );
}
```

```tsx
// Error Boundary (error.tsx)
// app/posts/error.tsx
"use client";

export default function Error({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  return (
    <div className="p-4 border border-red-200 rounded bg-red-50">
      <h2 className="text-red-700 font-semibold">Đã xảy ra lỗi!</h2>
      <p className="text-red-600">{error.message}</p>
      <button
        onClick={reset}
        className="mt-2 px-4 py-2 bg-red-600 text-white rounded"
      >
        Thử lại
      </button>
    </div>
  );
}
```

### 1.7. So Sánh Các Cách Fetch

| Tiêu chí | Server fetch | SWR | Route Handler |
|----------|--------------|-----|---------------|
| Nơi chạy | Server | Client | Server |
| SEO | Tốt | Cần SSR | N/A |
| Real-time | Không | Có | Không |
| Caching | Next.js cache | SWR cache | Tùy chỉnh |
| Use case | Initial data | Dynamic data | API endpoint |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn cần xây dựng trang danh sách sản phẩm:
- Hiển thị danh sách sản phẩm từ API
- Có thể search và filter
- Pagination
- Real-time inventory update

**Yêu cầu**:

- SEO tốt cho product list
- Filter không reload trang
- Inventory cập nhật real-time

**🤔 Câu hỏi suy ngẫm:**

1. Nên fetch initial data ở đâu?
2. Search/filter nên dùng Server hay Client fetch?
3. Inventory update nên implement như thế nào?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. **Initial data**: Server Component với fetch() - SEO tốt
2. **Search/filter**: Kết hợp:
   - URL search params để SEO
   - Server fetch khi params thay đổi
3. **Real-time inventory**: SWR với `refreshInterval` hoặc WebSocket

```tsx
// Search với URL params
const searchParams = useSearchParams();
const { data } = useSWR(
  `/api/products?q=${searchParams.get("q")}`,
  fetcher
);
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Không fetch cùng một dữ liệu ở cả Server và Client.

#### ✅ Nên Làm

```tsx
// Server Component fetch initial data
// app/products/page.tsx
async function getProducts() {
  const res = await fetch(`${API_URL}/products`, {
    next: { tags: ["products"] },
  });
  return res.json();
}

export default async function ProductsPage() {
  const products = await getProducts();

  return (
    <div>
      <ProductList initialProducts={products} />
    </div>
  );
}

// Client Component nhận initial data và handle updates
// components/ProductList.tsx
"use client";

export default function ProductList({ initialProducts }) {
  const { data: products } = useSWR("/api/products", fetcher, {
    fallbackData: initialProducts, // Sử dụng data từ server
  });

  return (/* render products */);
}
```

**Tại sao tốt:**

- Initial load nhanh (server-rendered)
- SEO tốt
- Client có thể refresh data khi cần

#### ❌ Không Nên Làm

```tsx
// ❌ Fetch 2 lần cùng data
// Server fetch
const products = await getProducts();

// Client lại fetch
const { data } = useSWR("/api/products"); // Fetch lại từ đầu
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Fetch failed in production | Dùng localhost URL | Dùng absolute URL hoặc internal fetch |
| Data stale | Không revalidate | Set revalidate hoặc dùng SWR |
| CORS error | Cross-origin request | Tạo Route Handler làm proxy |
| Double fetch | Fetch ở cả Server và Client | Dùng fallbackData trong SWR |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo trang /books với API và data fetching

**Yêu cầu kỹ thuật:**

- Route Handler `/api/books`
- Server Component fetch và display
- Loading và Error handling

#### Bước 1: Tạo Route Handler

```tsx
// app/api/books/route.ts
import { NextResponse } from "next/server";

interface Book {
  id: number;
  title: string;
  author: string;
  year: number;
}

const books: Book[] = [
  { id: 1, title: "Clean Code", author: "Robert C. Martin", year: 2008 },
  { id: 2, title: "The Pragmatic Programmer", author: "David Thomas", year: 1999 },
  { id: 3, title: "Design Patterns", author: "Gang of Four", year: 1994 },
];

export async function GET() {
  // Simulate delay
  await new Promise((resolve) => setTimeout(resolve, 1000));

  return NextResponse.json(books);
}

export async function POST(request: Request) {
  const body = await request.json();

  const newBook: Book = {
    id: books.length + 1,
    title: body.title,
    author: body.author,
    year: body.year || new Date().getFullYear(),
  };

  books.push(newBook);

  return NextResponse.json(newBook, { status: 201 });
}
```

#### Bước 2: Tạo Books Page (Server Component)

```tsx
// app/books/page.tsx
import { Suspense } from "react";
import BookList from "./BookList";
import BooksSkeleton from "./BooksSkeleton";

export const metadata = {
  title: "Danh sách sách",
  description: "Khám phá bộ sưu tập sách của chúng tôi",
};

export default function BooksPage() {
  return (
    <main className="container mx-auto p-6">
      <h1 className="text-2xl font-bold mb-6">Danh sách sách</h1>

      <Suspense fallback={<BooksSkeleton />}>
        <BookList />
      </Suspense>
    </main>
  );
}
```

#### Bước 3: Tạo BookList Component

```tsx
// app/books/BookList.tsx
interface Book {
  id: number;
  title: string;
  author: string;
  year: number;
}

async function getBooks(): Promise<Book[]> {
  const res = await fetch("http://localhost:3000/api/books", {
    next: { revalidate: 30 },
  });

  if (!res.ok) {
    throw new Error("Failed to fetch books");
  }

  return res.json();
}

export default async function BookList() {
  const books = await getBooks();

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {books.map((book) => (
        <article
          key={book.id}
          className="p-4 border rounded-lg hover:shadow-md transition-shadow"
        >
          <h2 className="font-semibold text-lg">{book.title}</h2>
          <p className="text-gray-600">{book.author}</p>
          <p className="text-sm text-gray-500 mt-2">Năm: {book.year}</p>
        </article>
      ))}
    </div>
  );
}
```

#### Bước 4: Tạo Loading Skeleton

```tsx
// app/books/BooksSkeleton.tsx
export default function BooksSkeleton() {
  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {[1, 2, 3, 4, 5, 6].map((i) => (
        <div key={i} className="p-4 border rounded-lg animate-pulse">
          <div className="h-5 bg-gray-200 rounded w-3/4 mb-2"></div>
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-2"></div>
          <div className="h-3 bg-gray-200 rounded w-1/4"></div>
        </div>
      ))}
    </div>
  );
}
```

#### Bước 5: Tạo Error Boundary

```tsx
// app/books/error.tsx
"use client";

import { Button } from "@/components/ui/button";

export default function Error({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  return (
    <div className="text-center py-10">
      <h2 className="text-xl font-semibold text-red-600 mb-2">
        Không thể tải danh sách sách
      </h2>
      <p className="text-gray-600 mb-4">{error.message}</p>
      <Button onClick={reset}>Thử lại</Button>
    </div>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo trang /users với SWR client-side fetch

<details>
<summary>💡 Gợi ý</summary>

- Tạo `/api/users` route handler
- Sử dụng useSWR trong Client Component
- Hiển thị loading và error state

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// app/api/users/route.ts
import { NextResponse } from "next/server";

const users = [
  { id: 1, name: "Nguyen Van A", email: "a@example.com" },
  { id: 2, name: "Tran Van B", email: "b@example.com" },
];

export async function GET() {
  await new Promise((r) => setTimeout(r, 500));
  return NextResponse.json(users);
}

// app/users/page.tsx
"use client";

import useSWR from "swr";

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export default function UsersPage() {
  const { data: users, error, isLoading } = useSWR("/api/users", fetcher);

  if (isLoading) return <div>Đang tải...</div>;
  if (error) return <div>Lỗi: {error.message}</div>;

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Người dùng</h1>
      <ul className="space-y-2">
        {users?.map((user) => (
          <li key={user.id} className="p-4 border rounded">
            <p className="font-semibold">{user.name}</p>
            <p className="text-gray-500">{user.email}</p>
          </li>
        ))}
      </ul>
    </div>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo CRUD API cho products

**Mở rộng**:

- GET /api/products - Danh sách
- GET /api/products/[id] - Chi tiết
- POST /api/products - Tạo mới
- PUT /api/products/[id] - Cập nhật
- DELETE /api/products/[id] - Xóa

### 3.3. Mini Project

**Dự án**: Blog với Search và Pagination

**Mô tả**: Xây dựng blog với đầy đủ tính năng fetch data

**Yêu cầu chức năng:**

1. Danh sách bài viết với pagination
2. Search theo title
3. Filter theo category
4. Chi tiết bài viết với related posts

**Technical Stack:**

- Next.js 14+ với App Router
- SWR cho client-side updates
- Route Handlers cho API

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Fetch data trong Server Component
- [ ] Tạo Route Handlers
- [ ] Sử dụng SWR cho client-side fetch
- [ ] Xử lý loading và error
- [ ] (Tùy chọn) Hoàn thành mini project Blog

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Sự khác nhau giữa Server fetch và Client fetch?
2. **Ứng dụng**: Khi nào dùng Route Handler?
3. **Phân tích**: So sánh SWR với useEffect + useState?
4. **Thực hành**: Demo API với CRUD operations?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Các cách fetch data trong Next.js
- Demo Route Handler và SWR
- Chia sẻ caching strategies
- Error handling best practices

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Route Handlers trong App Router được đặt ở đâu?

- A. `/pages/api/`
- B. `/app/api/`
- C. `/routes/api/`
- D. `/api/`

**Câu 2**: Option nào để fetch data mà không cache trong Next.js?

- A. `{ cache: "no-cache" }`
- B. `{ cache: "no-store" }`
- C. `{ revalidate: 0 }`
- D. `{ static: false }`

**Câu 3**: SWR là viết tắt của?

- A. State While Rendering
- B. Stale While Revalidate
- C. Store With React
- D. Sync With Remote

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Khi nào dùng Server fetch, khi nào dùng SWR?</strong></summary>

**Server fetch:**
- Initial page load data
- SEO-critical content
- Data không thay đổi thường xuyên
- Cần truy cập database trực tiếp

**SWR:**
- Data cần update real-time
- User-specific data (sau login)
- Paginated data với infinite scroll
- Data thay đổi bởi user actions

</details>

<details>
<summary><strong>Q2: Làm sao handle CORS trong Route Handler?</strong></summary>

```tsx
// app/api/data/route.ts
import { NextResponse } from "next/server";

export async function GET() {
  const data = { message: "Hello" };

  return NextResponse.json(data, {
    headers: {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE",
      "Access-Control-Allow-Headers": "Content-Type",
    },
  });
}

// OPTIONS for preflight
export async function OPTIONS() {
  return NextResponse.json({}, {
    headers: {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE",
      "Access-Control-Allow-Headers": "Content-Type",
    },
  });
}
```

</details>

<details>
<summary><strong>Q3: Làm sao để type-safe với API response?</strong></summary>

```tsx
// types/api.ts
export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export interface Post {
  id: number;
  title: string;
}

// Fetch với type
async function getPosts(): Promise<Post[]> {
  const res = await fetch("/api/posts");
  const data: Post[] = await res.json();
  return data;
}

// SWR với type
const { data } = useSWR<Post[]>("/api/posts", fetcher);
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 9

</footer>
