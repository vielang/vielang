
# Xác Thực và Ủy Quyền (Authentication)

> **Mô tả ngắn gọn**: Tìm hiểu cách triển khai authentication trong Next.js với JWT, Cookie, Middleware và Role-based Access Control.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu khái niệm xác thực (authentication) và ủy quyền (authorization)
- [ ] Triển khai middleware bảo vệ route
- [ ] Hiểu cách hoạt động của JWT và Cookie-based Auth
- [ ] Tạo trang đăng nhập, đăng ký và kiểm tra trạng thái đăng nhập
- [ ] Áp dụng phân quyền dựa trên vai trò (Role-based Access Control)

### Kiến Thức Yêu Cầu

- Bài 1-9: Next.js App Router, State Management, API
- HTTP Cookies cơ bản
- JWT (JSON Web Token) cơ bản

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về Authentication | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành xây dựng auth system | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Khái Niệm Cơ Bản

#### Authentication (Xác thực)

> **💡 Định nghĩa**: Xác thực là quá trình kiểm tra "bạn là ai" - ví dụ: đăng nhập bằng email và mật khẩu để xác nhận danh tính.

**Ví dụ thực tế:**
- Đăng nhập vào Gmail bằng tài khoản Google
- Đăng nhập Facebook bằng số điện thoại

#### Authorization (Ủy quyền)

> **💡 Định nghĩa**: Ủy quyền là quá trình kiểm tra "bạn được phép làm gì" - ví dụ: sau khi đăng nhập, bạn có được truy cập trang admin không?

**Ví dụ thực tế:**
- User bình thường không thể truy cập admin panel
- Chỉ editor mới được phép chỉnh sửa bài viết

### 1.2. Các Hình Thức Xác Thực

| Loại | Đặc điểm | Phù hợp với |
|------|----------|-------------|
| JWT | Token lưu trên client (cookie/localStorage) | SPA, API-based apps |
| Session | Session lưu trên server | SSR apps, legacy systems |
| OAuth | Xác thực qua Google, Facebook... | Social login |

> Trong bài này, sử dụng **JWT + Cookie-based auth** - phù hợp với Next.js App Router.

### 1.3. Kiến Trúc Authentication

```
app/
├── (auth)/
│   ├── login/page.tsx
│   └── register/page.tsx
├── (protected)/
│   ├── dashboard/page.tsx
│   └── profile/page.tsx
├── api/
│   ├── auth/
│   │   ├── login/route.ts
│   │   ├── logout/route.ts
│   │   └── me/route.ts
├── middleware.ts
├── lib/
│   └── auth.ts
└── context/
    └── auth-context.tsx
```

### 1.4. Trang Đăng Nhập

```tsx
// app/(auth)/login/page.tsx
"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError("");

    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error || "Đăng nhập thất bại");
      }

      router.push("/dashboard");
      router.refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Đã xảy ra lỗi");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Card className="w-[400px]">
      <CardHeader>
        <CardTitle>Đăng nhập</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleLogin} className="space-y-4">
          {error && (
            <div className="p-3 text-sm text-red-500 bg-red-50 rounded">
              {error}
            </div>
          )}

          <Input
            type="email"
            placeholder="Email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            required
          />

          <Input
            type="password"
            placeholder="Mật khẩu"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
          />

          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? "Đang đăng nhập..." : "Đăng nhập"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
```

### 1.5. API Route Login

```tsx
// app/api/auth/login/route.ts
import { NextRequest, NextResponse } from "next/server";
import { cookies } from "next/headers";
import { sign } from "jsonwebtoken";

const JWT_SECRET = process.env.JWT_SECRET || "your-secret-key";

// Giả lập database users
const users = [
  { id: 1, email: "admin@example.com", password: "123456", role: "admin", name: "Admin" },
  { id: 2, email: "user@example.com", password: "123456", role: "user", name: "User" },
];

export async function POST(request: NextRequest) {
  try {
    const { email, password } = await request.json();

    // Validate
    if (!email || !password) {
      return NextResponse.json(
        { error: "Email và mật khẩu là bắt buộc" },
        { status: 400 }
      );
    }

    // Find user
    const user = users.find(
      (u) => u.email === email && u.password === password
    );

    if (!user) {
      return NextResponse.json(
        { error: "Email hoặc mật khẩu không đúng" },
        { status: 401 }
      );
    }

    // Create JWT
    const token = sign(
      { id: user.id, email: user.email, role: user.role },
      JWT_SECRET,
      { expiresIn: "7d" }
    );

    // Set cookie
    cookies().set("access_token", token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      maxAge: 60 * 60 * 24 * 7, // 7 days
    });

    return NextResponse.json({
      user: { id: user.id, email: user.email, name: user.name, role: user.role },
    });
  } catch (error) {
    return NextResponse.json(
      { error: "Đã xảy ra lỗi server" },
      { status: 500 }
    );
  }
}
```

### 1.6. Middleware Bảo Vệ Route

```tsx
// middleware.ts
import { NextRequest, NextResponse } from "next/server";
import { jwtVerify } from "jose";

const JWT_SECRET = new TextEncoder().encode(
  process.env.JWT_SECRET || "your-secret-key"
);

// Routes cần bảo vệ
const protectedRoutes = ["/dashboard", "/profile", "/settings"];
const authRoutes = ["/login", "/register"];

export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const token = request.cookies.get("access_token")?.value;

  // Kiểm tra token
  let isValidToken = false;
  let user = null;

  if (token) {
    try {
      const { payload } = await jwtVerify(token, JWT_SECRET);
      isValidToken = true;
      user = payload;
    } catch {
      isValidToken = false;
    }
  }

  // Protected routes - chưa login thì redirect về login
  if (protectedRoutes.some((route) => pathname.startsWith(route))) {
    if (!isValidToken) {
      const loginUrl = new URL("/login", request.url);
      loginUrl.searchParams.set("from", pathname);
      return NextResponse.redirect(loginUrl);
    }
  }

  // Auth routes - đã login thì redirect về dashboard
  if (authRoutes.some((route) => pathname.startsWith(route))) {
    if (isValidToken) {
      return NextResponse.redirect(new URL("/dashboard", request.url));
    }
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    "/dashboard/:path*",
    "/profile/:path*",
    "/settings/:path*",
    "/login",
    "/register",
  ],
};
```

### 1.7. Auth Context

```tsx
// context/auth-context.tsx
"use client";

import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { useRouter } from "next/navigation";

interface User {
  id: number;
  email: string;
  name: string;
  role: string;
}

interface AuthContextType {
  user: User | null;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();

  // Check auth status on mount
  useEffect(() => {
    checkAuth();
  }, []);

  const checkAuth = async () => {
    try {
      const res = await fetch("/api/auth/me");
      if (res.ok) {
        const data = await res.json();
        setUser(data.user);
      }
    } catch {
      setUser(null);
    } finally {
      setIsLoading(false);
    }
  };

  const login = async (email: string, password: string) => {
    const res = await fetch("/api/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    if (!res.ok) {
      const data = await res.json();
      throw new Error(data.error);
    }

    const data = await res.json();
    setUser(data.user);
    router.push("/dashboard");
    router.refresh();
  };

  const logout = async () => {
    await fetch("/api/auth/logout", { method: "POST" });
    setUser(null);
    router.push("/login");
    router.refresh();
  };

  return (
    <AuthContext.Provider value={{ user, isLoading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used inside AuthProvider");
  }
  return context;
}
```

### 1.8. Role-Based Access Control (RBAC)

```tsx
// lib/auth.ts
export function hasRole(user: { role: string } | null, allowedRoles: string[]) {
  if (!user) return false;
  return allowedRoles.includes(user.role);
}

export function isAdmin(user: { role: string } | null) {
  return hasRole(user, ["admin"]);
}
```

```tsx
// components/RoleGuard.tsx
"use client";

import { useAuth } from "@/context/auth-context";
import { hasRole } from "@/lib/auth";
import { useRouter } from "next/navigation";
import { useEffect } from "react";

interface RoleGuardProps {
  children: React.ReactNode;
  allowedRoles: string[];
  fallback?: React.ReactNode;
}

export function RoleGuard({ children, allowedRoles, fallback }: RoleGuardProps) {
  const { user, isLoading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && !hasRole(user, allowedRoles)) {
      router.push("/dashboard");
    }
  }, [user, isLoading, allowedRoles, router]);

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (!hasRole(user, allowedRoles)) {
    return fallback || null;
  }

  return <>{children}</>;
}
```

```tsx
// app/admin/page.tsx
import { RoleGuard } from "@/components/RoleGuard";

export default function AdminPage() {
  return (
    <RoleGuard allowedRoles={["admin"]}>
      <div>
        <h1>Admin Panel</h1>
        <p>Chỉ admin mới thấy trang này</p>
      </div>
    </RoleGuard>
  );
}
```

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng hệ thống auth cho ứng dụng với:
- Đăng nhập/đăng ký
- Phân quyền: admin, editor, user
- Protected routes
- Remember me functionality

**Yêu cầu**:

- Bảo mật token
- Session persist khi refresh
- Role-based access control

**🤔 Câu hỏi suy ngẫm:**

1. Token nên lưu ở đâu để bảo mật?
2. Làm sao implement "Remember me"?
3. Cách handle expired token?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. **Token storage:**
   - **httpOnly cookie** (khuyến nghị) - tránh XSS
   - **Không dùng** localStorage cho sensitive tokens

2. **Remember me:**
   - Thay đổi `maxAge` của cookie
   - Checked: 30 ngày, Unchecked: session

3. **Expired token:**
   - Middleware check và redirect
   - Refresh token pattern
   - Client-side check với API `/api/auth/me`

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Middleware không thể truy cập localStorage, chỉ dùng cookie.

#### ✅ Nên Làm

```tsx
// Lưu token trong httpOnly cookie
cookies().set("access_token", token, {
  httpOnly: true,        // Không thể truy cập từ JavaScript
  secure: true,          // Chỉ gửi qua HTTPS
  sameSite: "lax",       // Chống CSRF
  maxAge: 60 * 60 * 24,  // 1 ngày
});

// Verify token trong middleware
const { payload } = await jwtVerify(token, secret);
```

**Tại sao tốt:**

- httpOnly chống XSS attacks
- secure đảm bảo chỉ gửi qua HTTPS
- sameSite chống CSRF attacks

#### ❌ Không Nên Làm

```tsx
// ❌ Lưu token trong localStorage
localStorage.setItem("token", token);

// ❌ Lưu password trong state/store
const useAuthStore = create((set) => ({
  password: "", // KHÔNG BAO GIỜ làm điều này
}));

// ❌ Gửi password trong URL
router.push(`/verify?password=${password}`);
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "cookies() can only be called from Server Component" | Gọi trong Client Component | Dùng Route Handler hoặc Server Action |
| Token không được gửi | Cookie không có credentials | Thêm `credentials: 'include'` trong fetch |
| CORS error | Cookie bị block | Cấu hình CORS đúng cách |
| Middleware loop | Redirect trong protected route | Kiểm tra điều kiện redirect chính xác |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng hệ thống auth hoàn chỉnh

**Yêu cầu kỹ thuật:**

- Login/Logout với JWT
- Middleware bảo vệ routes
- Auth Context chia sẻ state

#### Bước 1: Setup API Routes

```tsx
// app/api/auth/me/route.ts
import { NextRequest, NextResponse } from "next/server";
import { cookies } from "next/headers";
import { jwtVerify } from "jose";

const JWT_SECRET = new TextEncoder().encode(
  process.env.JWT_SECRET || "your-secret-key"
);

export async function GET(request: NextRequest) {
  const token = cookies().get("access_token")?.value;

  if (!token) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }

  try {
    const { payload } = await jwtVerify(token, JWT_SECRET);

    return NextResponse.json({
      user: {
        id: payload.id,
        email: payload.email,
        role: payload.role,
      },
    });
  } catch {
    return NextResponse.json({ error: "Invalid token" }, { status: 401 });
  }
}
```

```tsx
// app/api/auth/logout/route.ts
import { NextResponse } from "next/server";
import { cookies } from "next/headers";

export async function POST() {
  cookies().delete("access_token");

  return NextResponse.json({ success: true });
}
```

#### Bước 2: Setup Auth Layout

```tsx
// app/(auth)/layout.tsx
export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100">
      {children}
    </div>
  );
}
```

#### Bước 3: Dashboard với User Info

```tsx
// app/(protected)/dashboard/page.tsx
"use client";

import { useAuth } from "@/context/auth-context";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function DashboardPage() {
  const { user, logout, isLoading } = useAuth();

  if (isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <div className="container mx-auto p-6">
      <Card>
        <CardHeader>
          <CardTitle>Dashboard</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <p className="text-lg">Xin chào, <strong>{user?.name}</strong>!</p>
            <p className="text-gray-500">Email: {user?.email}</p>
            <p className="text-gray-500">Role: {user?.role}</p>
          </div>

          {user?.role === "admin" && (
            <div className="p-4 bg-yellow-50 border border-yellow-200 rounded">
              <p className="font-semibold">Admin Panel</p>
              <p className="text-sm">Bạn có quyền admin</p>
            </div>
          )}

          <Button onClick={logout} variant="outline">
            Đăng xuất
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
```

#### Bước 4: Wrap App với AuthProvider

```tsx
// app/layout.tsx
import { AuthProvider } from "@/context/auth-context";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="vi">
      <body>
        <AuthProvider>{children}</AuthProvider>
      </body>
    </html>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo trang Register

<details>
<summary>💡 Gợi ý</summary>

- Form: email, password, confirm password
- Validate password match
- API route `/api/auth/register`

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// app/(auth)/register/page.tsx
"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import Link from "next/link";

export default function RegisterPage() {
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    password: "",
    confirmPassword: "",
  });
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (formData.password !== formData.confirmPassword) {
      setError("Mật khẩu không khớp");
      return;
    }

    setIsLoading(true);

    try {
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: formData.name,
          email: formData.email,
          password: formData.password,
        }),
      });

      if (!res.ok) {
        const data = await res.json();
        throw new Error(data.error);
      }

      router.push("/login?registered=true");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Đã xảy ra lỗi");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Card className="w-[400px]">
      <CardHeader>
        <CardTitle>Đăng ký</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <div className="p-3 text-sm text-red-500 bg-red-50 rounded">
              {error}
            </div>
          )}

          <Input
            placeholder="Họ tên"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            required
          />

          <Input
            type="email"
            placeholder="Email"
            value={formData.email}
            onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            required
          />

          <Input
            type="password"
            placeholder="Mật khẩu"
            value={formData.password}
            onChange={(e) => setFormData({ ...formData, password: e.target.value })}
            required
          />

          <Input
            type="password"
            placeholder="Xác nhận mật khẩu"
            value={formData.confirmPassword}
            onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
            required
          />

          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? "Đang đăng ký..." : "Đăng ký"}
          </Button>

          <p className="text-center text-sm text-gray-500">
            Đã có tài khoản?{" "}
            <Link href="/login" className="text-blue-600 hover:underline">
              Đăng nhập
            </Link>
          </p>
        </form>
      </CardContent>
    </Card>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo Admin Page với RBAC

**Mở rộng**:

- Chỉ admin mới truy cập được `/admin`
- Hiển thị danh sách users
- Có thể thay đổi role của user

### 3.3. Mini Project

**Dự án**: User Management System

**Mô tả**: Xây dựng hệ thống quản lý người dùng hoàn chỉnh

**Yêu cầu chức năng:**

1. Authentication: Login, Register, Logout
2. Profile: View và Edit profile
3. Admin: Quản lý users (chỉ admin)
4. Role-based UI: Hiển thị UI khác nhau theo role

**Technical Stack:**

- Next.js 14+ với App Router
- JWT Authentication
- Middleware protection
- ShadcnUI components

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu authentication vs authorization
- [ ] Tạo được Login/Register pages
- [ ] Implement middleware protection
- [ ] Sử dụng Auth Context
- [ ] (Tùy chọn) Hoàn thành mini project

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Sự khác nhau giữa authentication và authorization?
2. **Ứng dụng**: Tại sao dùng httpOnly cookie thay vì localStorage?
3. **Phân tích**: So sánh JWT vs Session-based auth?
4. **Thực hành**: Demo hệ thống auth với role-based access?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Các phương pháp authentication
- Demo login flow
- Security best practices
- Chia sẻ về RBAC implementation

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Middleware trong Next.js có thể truy cập gì?

- A. localStorage
- B. sessionStorage
- C. cookies
- D. window object

**Câu 2**: httpOnly cookie có đặc điểm gì?

- A. Có thể truy cập từ JavaScript
- B. Không thể truy cập từ JavaScript
- C. Tự động expire sau 1 giờ
- D. Chỉ hoạt động trên localhost

**Câu 3**: JWT được verify ở đâu trong Next.js App Router?

- A. Chỉ ở Client
- B. Chỉ ở Server
- C. Cả Client và Server
- D. Chỉ trong middleware

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Tại sao không dùng localStorage cho token?</strong></summary>

**Lý do bảo mật:**

1. **XSS Attack**: JavaScript có thể đọc localStorage. Nếu attacker inject script vào trang, họ có thể đánh cắp token.

2. **httpOnly cookie** không thể truy cập từ JavaScript, an toàn hơn.

```tsx
// ❌ Dễ bị tấn công XSS
localStorage.setItem("token", token);

// ✅ An toàn hơn
cookies().set("token", token, { httpOnly: true });
```

</details>

<details>
<summary><strong>Q2: Làm sao handle token expiration?</strong></summary>

**Option 1: Middleware check**
```tsx
// middleware.ts
try {
  await jwtVerify(token, secret);
} catch (error) {
  if (error.code === "ERR_JWT_EXPIRED") {
    // Redirect to login
    return NextResponse.redirect(new URL("/login", request.url));
  }
}
```

**Option 2: Refresh token pattern**
- Lưu refresh_token (longer expiry)
- Khi access_token expire, dùng refresh_token để lấy token mới
- Nếu refresh_token expire, yêu cầu login lại

</details>

<details>
<summary><strong>Q3: Có nên dùng NextAuth.js không?</strong></summary>

**NextAuth.js (Auth.js)** là thư viện authentication phổ biến cho Next.js:

**Ưu điểm:**
- Hỗ trợ nhiều providers (Google, GitHub, etc.)
- Session management có sẵn
- Type-safe
- Active community

**Nhược điểm:**
- Thêm dependency
- Learning curve
- Customization có thể phức tạp

**Recommendation:**
- Dùng NextAuth cho: OAuth/social login, multiple providers
- Tự implement cho: Custom auth flow đơn giản, full control

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 10

</footer>
