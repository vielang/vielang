
# State Management

> **Mô tả ngắn gọn**: Tìm hiểu các cách quản lý state trong Next.js: useState, useReducer, Context API và Zustand.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu được state là gì và vai trò trong React/Next.js
- [ ] Sử dụng `useState`, `useReducer` cho local state
- [ ] Chia sẻ state giữa các component với Context API
- [ ] Tổ chức global state với Zustand
- [ ] Biết cách persist state khi reload trang
- [ ] Phân biệt khi nào dùng local state, context hay global store

### Kiến Thức Yêu Cầu

- Bài 1-7: Next.js App Router, Server/Client Components
- React hooks cơ bản
- TypeScript interfaces

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về State Management | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành với Context và Zustand | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. State Là Gì?

> **💡 Định nghĩa**: State là dữ liệu nội bộ được lưu trong component để phản ánh UI theo thời gian thực. Khi state thay đổi, UI tự động cập nhật.

**Ví dụ thực tế:**

- Nội dung trong input form
- Trạng thái đăng nhập (logged in/out)
- Theme hiện tại (dark/light)
- Số lượng items trong giỏ hàng

### 1.2. Local State Với `useState`

```tsx
"use client";

import { useState } from "react";

export default function Counter() {
  const [count, setCount] = useState(0);

  return (
    <div className="p-4">
      <p>Bạn đã nhấn {count} lần</p>
      <button
        onClick={() => setCount(count + 1)}
        className="px-4 py-2 bg-blue-500 text-white rounded"
      >
        Tăng
      </button>
    </div>
  );
}
```

**Khi nào dùng useState:**

- State đơn giản (số, string, boolean)
- Chỉ dùng trong 1 component
- Không cần chia sẻ với component khác

### 1.3. Local State Nâng Cao Với `useReducer`

> **💡 Định nghĩa**: `useReducer` phù hợp với logic state phức tạp hơn, giống Redux nhẹ.

```tsx
"use client";

import { useReducer } from "react";

type State = { count: number };
type Action = { type: "increment" } | { type: "decrement" } | { type: "reset" };

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "increment":
      return { count: state.count + 1 };
    case "decrement":
      return { count: state.count - 1 };
    case "reset":
      return { count: 0 };
    default:
      return state;
  }
}

export default function Counter() {
  const [state, dispatch] = useReducer(reducer, { count: 0 });

  return (
    <div className="p-4 space-x-2">
      <span>Count: {state.count}</span>
      <button onClick={() => dispatch({ type: "increment" })}>+</button>
      <button onClick={() => dispatch({ type: "decrement" })}>-</button>
      <button onClick={() => dispatch({ type: "reset" })}>Reset</button>
    </div>
  );
}
```

**Khi nào dùng useReducer:**

- State có nhiều điều kiện chuyển đổi
- Logic phức tạp cần tách riêng
- Muốn tổ chức code giống Redux

### 1.4. Chia Sẻ State Với Context API

#### Tạo ThemeContext

```tsx
// context/theme-context.tsx
"use client";

import { createContext, useContext, useState, ReactNode } from "react";

type Theme = "light" | "dark";

interface ThemeContextType {
  theme: Theme;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>("light");

  const toggleTheme = () => {
    setTheme(theme === "light" ? "dark" : "light");
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <div className={theme}>{children}</div>
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used inside ThemeProvider");
  }
  return context;
}
```

#### Wrap Provider Trong Layout

```tsx
// app/layout.tsx
import { ThemeProvider } from "@/context/theme-context";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="vi">
      <body>
        <ThemeProvider>{children}</ThemeProvider>
      </body>
    </html>
  );
}
```

#### Sử Dụng Trong Component

```tsx
// components/ThemeSwitcher.tsx
"use client";

import { useTheme } from "@/context/theme-context";
import { Button } from "@/components/ui/button";

export default function ThemeSwitcher() {
  const { theme, toggleTheme } = useTheme();

  return (
    <Button onClick={toggleTheme} variant="outline">
      {theme === "light" ? "🌙 Dark" : "☀️ Light"}
    </Button>
  );
}
```

### 1.5. Global State Với Zustand

#### Cài Đặt

```bash
npm install zustand
```

#### Tạo Store

```tsx
// store/counter-store.ts
import { create } from "zustand";

interface CounterState {
  count: number;
  increase: () => void;
  decrease: () => void;
  reset: () => void;
}

export const useCounterStore = create<CounterState>((set) => ({
  count: 0,
  increase: () => set((state) => ({ count: state.count + 1 })),
  decrease: () => set((state) => ({ count: state.count - 1 })),
  reset: () => set({ count: 0 }),
}));
```

#### Sử Dụng Trong Component

```tsx
// components/GlobalCounter.tsx
"use client";

import { useCounterStore } from "@/store/counter-store";
import { Button } from "@/components/ui/button";

export default function GlobalCounter() {
  const { count, increase, decrease, reset } = useCounterStore();

  return (
    <div className="p-4 space-x-2">
      <span className="font-bold">Count: {count}</span>
      <Button onClick={increase}>+</Button>
      <Button onClick={decrease}>-</Button>
      <Button variant="outline" onClick={reset}>Reset</Button>
    </div>
  );
}
```

### 1.6. State Persistence

```tsx
// store/theme-store.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface ThemeState {
  theme: "light" | "dark";
  toggle: () => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      theme: "light",
      toggle: () =>
        set((state) => ({
          theme: state.theme === "light" ? "dark" : "light",
        })),
    }),
    {
      name: "theme-storage", // Key trong localStorage
    }
  )
);
```

**📝 Lưu ý:** State sẽ được lưu vào localStorage và khôi phục khi reload trang.

### 1.7. So Sánh Các Giải Pháp

| Giải pháp | Use case | Ưu điểm | Nhược điểm |
|-----------|----------|---------|------------|
| `useState` | Local state đơn giản | Đơn giản, native | Không share được |
| `useReducer` | Local state phức tạp | Tổ chức tốt | Không share được |
| Context API | Share state nhẹ | Native, dễ dùng | Performance khi update nhiều |
| Zustand | Global state | Nhẹ, ít boilerplate | Thêm dependency |
| Redux | App phức tạp | Ecosystem lớn | Boilerplate nhiều |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn cần xây dựng hệ thống authentication:
- Lưu trạng thái đăng nhập (isLoggedIn)
- Thông tin user (name, email, role)
- Có thể truy cập từ bất kỳ component nào

**Yêu cầu**:

- State persist khi refresh
- Type-safe với TypeScript
- Dễ dàng login/logout

**🤔 Câu hỏi suy ngẫm:**

1. Nên dùng Context API hay Zustand?
2. Làm sao để persist login state?
3. Cách xử lý khi user data null?

<details>
<summary>💭 Gợi ý phân tích</summary>

1. **Zustand** tốt hơn vì:
   - Persist middleware có sẵn
   - Không cần Provider wrapper
   - Selector để tránh re-render không cần thiết

2. **Store example:**
```tsx
const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      isLoggedIn: false,
      login: (user) => set({ user, isLoggedIn: true }),
      logout: () => set({ user: null, isLoggedIn: false }),
    }),
    { name: "auth-storage" }
  )
);
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: `useState` và `useReducer` chỉ hoạt động trong Client Component (`"use client"`).

#### ✅ Nên Làm

```tsx
// Tách logic ra hook riêng
// hooks/useAuth.ts
"use client";

import { useAuthStore } from "@/store/auth-store";

export function useAuth() {
  const { user, isLoggedIn, login, logout } = useAuthStore();

  const isAdmin = user?.role === "admin";

  return {
    user,
    isLoggedIn,
    isAdmin,
    login,
    logout,
  };
}

// Component sử dụng
export default function Dashboard() {
  const { user, isAdmin } = useAuth();

  return (
    <div>
      <p>Welcome, {user?.name}</p>
      {isAdmin && <AdminPanel />}
    </div>
  );
}
```

**Tại sao tốt:**

- Logic tập trung, dễ test
- Component clean, chỉ render UI
- Dễ tái sử dụng

#### ❌ Không Nên Làm

```tsx
// Lưu sensitive data vào store
const useAuthStore = create(persist(
  (set) => ({
    password: "", // ❌ Không bao giờ lưu password
    token: "",    // ❌ Nên dùng httpOnly cookie
  }),
  { name: "auth" }
));
```

**Tại sao không tốt:**

- Lưu password trong localStorage là security risk
- Token nên ở httpOnly cookie để tránh XSS

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "useContext must be inside Provider" | Component ngoài Provider | Đảm bảo wrap đúng vị trí |
| Hydration mismatch | Persist state khác server/client | Dùng `skipHydration` option |
| Re-render quá nhiều | Context update gây re-render tất cả | Dùng Zustand với selectors |
| State reset khi navigate | Dùng useState thay vì global | Di chuyển lên Context/Zustand |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo AuthContext cho login/logout

**Yêu cầu kỹ thuật:**

- Trạng thái isLoggedIn
- Hàm login() và logout()
- Form đăng nhập giả lập

#### Bước 1: Tạo Auth Context

```tsx
// context/auth-context.tsx
"use client";

import { createContext, useContext, useState, ReactNode } from "react";

interface User {
  name: string;
  email: string;
}

interface AuthContextType {
  user: User | null;
  isLoggedIn: boolean;
  login: (email: string, password: string) => Promise<boolean>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);

  const login = async (email: string, password: string) => {
    // Giả lập API call
    await new Promise((resolve) => setTimeout(resolve, 1000));

    if (email && password) {
      setUser({ name: "Nguyen Van A", email });
      return true;
    }
    return false;
  };

  const logout = () => {
    setUser(null);
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        isLoggedIn: !!user,
        login,
        logout,
      }}
    >
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

#### Bước 2: Wrap Provider

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

#### Bước 3: Tạo Login Form

```tsx
// components/LoginForm.tsx
"use client";

import { useState } from "react";
import { useAuth } from "@/context/auth-context";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function LoginForm() {
  const { user, isLoggedIn, login, logout } = useAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleLogin = async () => {
    setIsLoading(true);
    await login(email, password);
    setIsLoading(false);
  };

  if (isLoggedIn) {
    return (
      <Card className="w-[350px]">
        <CardHeader>
          <CardTitle>Xin chào!</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p>Bạn đã đăng nhập với email: {user?.email}</p>
          <Button onClick={logout} variant="outline" className="w-full">
            Đăng xuất
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-[350px]">
      <CardHeader>
        <CardTitle>Đăng nhập</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Input
          placeholder="Email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
        />
        <Input
          type="password"
          placeholder="Mật khẩu"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <Button onClick={handleLogin} disabled={isLoading} className="w-full">
          {isLoading ? "Đang đăng nhập..." : "Đăng nhập"}
        </Button>
      </CardContent>
    </Card>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo CounterStore với Zustand + persist

<details>
<summary>💡 Gợi ý</summary>

- Sử dụng `zustand/middleware` cho persist
- Count tăng mỗi lần reload trang
- Hiển thị ở góc trên bên phải

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
// store/visit-store.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface VisitState {
  visits: number;
  increment: () => void;
}

export const useVisitStore = create<VisitState>()(
  persist(
    (set) => ({
      visits: 0,
      increment: () => set((state) => ({ visits: state.visits + 1 })),
    }),
    { name: "visit-storage" }
  )
);

// components/VisitCounter.tsx
"use client";

import { useEffect } from "react";
import { useVisitStore } from "@/store/visit-store";

export default function VisitCounter() {
  const { visits, increment } = useVisitStore();

  useEffect(() => {
    increment();
  }, []);

  return (
    <div className="fixed top-4 right-4 bg-blue-500 text-white px-3 py-1 rounded">
      Lượt truy cập: {visits}
    </div>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo Shopping Cart Store

**Mở rộng**:

- Add/remove items
- Update quantity
- Calculate total price
- Persist cart data

### 3.3. Mini Project

**Dự án**: Todo App với Zustand

**Mô tả**: Xây dựng todo app với state management đầy đủ

**Yêu cầu chức năng:**

1. Add/delete/toggle todos
2. Filter: All, Active, Completed
3. Persist todos khi reload
4. Clear completed todos

**Technical Stack:**

- Next.js 14+ với App Router
- Zustand với persist middleware
- ShadcnUI components

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu useState và useReducer
- [ ] Tạo được Context với Provider
- [ ] Sử dụng Zustand cho global state
- [ ] Implement state persistence
- [ ] (Tùy chọn) Hoàn thành mini project Todo App

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Khi nào dùng Context, khi nào dùng Zustand?
2. **Ứng dụng**: Làm sao persist state với Zustand?
3. **Phân tích**: So sánh Zustand với Redux?
4. **Thực hành**: Demo Auth Context với login/logout?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Các giải pháp state management trong React
- Demo Zustand store đã tạo
- Chia sẻ pattern tổ chức store
- Tips performance optimization

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Hook nào phù hợp cho state với nhiều actions phức tạp?

- A. `useState`
- B. `useReducer`
- C. `useRef`
- D. `useMemo`

**Câu 2**: Zustand persist middleware lưu data ở đâu mặc định?

- A. sessionStorage
- B. localStorage
- C. Cookie
- D. IndexedDB

**Câu 3**: Context API có nhược điểm gì?

- A. Không thể share state
- B. Re-render tất cả consumers khi update
- C. Không hỗ trợ TypeScript
- D. Không hoạt động với SSR

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Zustand có cần Provider không?</strong></summary>

Không! Đây là điểm khác biệt lớn so với Context API và Redux. Zustand store có thể được import và sử dụng trực tiếp trong bất kỳ component nào:

```tsx
// Không cần Provider
import { useCounterStore } from "@/store/counter-store";

export default function MyComponent() {
  const count = useCounterStore((state) => state.count);
  return <div>{count}</div>;
}
```

</details>

<details>
<summary><strong>Q2: Làm sao tránh re-render không cần thiết với Zustand?</strong></summary>

Sử dụng selector để chỉ subscribe những state cần thiết:

```tsx
// ❌ Subscribe toàn bộ store
const { count, user, theme } = useStore();

// ✅ Chỉ subscribe count
const count = useStore((state) => state.count);

// ✅ Multiple values với shallow comparison
import { shallow } from "zustand/shallow";
const { count, user } = useStore(
  (state) => ({ count: state.count, user: state.user }),
  shallow
);
```

</details>

<details>
<summary><strong>Q3: Hydration mismatch với persist là gì?</strong></summary>

Khi SSR, server không có localStorage nên state khác với client. Giải pháp:

```tsx
// Option 1: Skip hydration
const useStore = create(
  persist(
    (set) => ({ count: 0 }),
    {
      name: "storage",
      skipHydration: true,
    }
  )
);

// Trong component
useEffect(() => {
  useStore.persist.rehydrate();
}, []);

// Option 2: Dùng onRehydrateStorage
persist(
  (set) => ({ count: 0 }),
  {
    name: "storage",
    onRehydrateStorage: () => (state) => {
      // Handle after hydration
    },
  }
);
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 8

</footer>
