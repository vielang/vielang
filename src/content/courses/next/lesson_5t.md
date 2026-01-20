
# ShadcnUI - Thư Viện Component UI

> **Mô tả ngắn gọn**: Tìm hiểu ShadcnUI, cách cài đặt, sử dụng các component phổ biến và xây dựng form với Zod validation.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu ShadcnUI là gì và vì sao nên sử dụng
- [ ] Biết cách cài đặt và cấu hình ShadcnUI trong Next.js
- [ ] Sử dụng thành thạo các component phổ biến: Button, Input, Card, Dialog
- [ ] Tích hợp form validation với Zod và React Hook Form
- [ ] Tùy chỉnh theme và tích hợp với TailwindCSS

### Kiến Thức Yêu Cầu

- Bài 1-4: Next.js, TypeScript, TailwindCSS
- React Hooks cơ bản (useState, useForm)
- Hiểu về form handling

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về ShadcnUI | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành xây dựng form | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. ShadcnUI Là Gì?

> **💡 Định nghĩa**: ShadcnUI là thư viện component UI mã nguồn mở được xây dựng bằng React, TailwindCSS và Radix UI. Điểm đặc biệt là bạn tự sở hữu code - các component được thêm trực tiếp vào dự án.

**Điểm nổi bật:**

- **TailwindCSS thuần**: Dễ kiểm soát và tùy chỉnh styling
- **Radix UI**: Accessibility cao, keyboard support tốt
- **TypeScript-ready**: Typing chính xác
- **Copy-paste code**: Bạn sở hữu và kiểm soát hoàn toàn code

**So sánh với các thư viện khác:**

| Thư viện | Tùy chỉnh | Tailwind | Code ownership | Accessibility |
|----------|-----------|----------|----------------|---------------|
| ShadcnUI | Cao | Tuyệt vời | Có | Tốt |
| MUI | Thấp | Không | Không | Tốt |
| Chakra UI | Trung bình | Không | Không | Tốt |
| Tailwind UI | Cao | Tốt | Giới hạn | Thủ công |

### 1.2. Cài Đặt ShadcnUI

#### Bước 1: Chạy CLI

```bash
npx shadcn@latest init
```

**Các lựa chọn cấu hình:**

- Style: `Default` hoặc `New York`
- Base color: Chọn màu chủ đạo
- CSS variables: `Yes` (khuyến khích)

#### Bước 2: Cấu trúc sau cài đặt

```
my-app/
├── components/
│   └── ui/            # ShadcnUI components
├── lib/
│   └── utils.ts       # Utility functions (cn)
├── tailwind.config.ts # Updated config
└── components.json    # Shadcn config
```

#### Bước 3: Thêm components

```bash
# Thêm từng component
npx shadcn@latest add button
npx shadcn@latest add input
npx shadcn@latest add card
npx shadcn@latest add dialog
npx shadcn@latest add form
```

### 1.3. Các Component Phổ Biến

#### Button

```tsx
import { Button } from "@/components/ui/button";

export default function ButtonDemo() {
  return (
    <div className="flex gap-2">
      <Button>Default</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="destructive">Delete</Button>
      <Button variant="ghost">Ghost</Button>
      <Button size="sm">Small</Button>
      <Button size="lg">Large</Button>
    </div>
  );
}
```

**Props chính:**

| Prop | Values | Default |
|------|--------|---------|
| `variant` | `default`, `outline`, `secondary`, `destructive`, `ghost`, `link` | `default` |
| `size` | `default`, `sm`, `lg`, `icon` | `default` |
| `disabled` | `boolean` | `false` |

#### Input

```tsx
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function InputDemo() {
  return (
    <div className="grid gap-2">
      <Label htmlFor="email">Email</Label>
      <Input
        id="email"
        type="email"
        placeholder="name@example.com"
      />
    </div>
  );
}
```

#### Card

```tsx
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function CardDemo() {
  return (
    <Card className="w-[350px]">
      <CardHeader>
        <CardTitle>Tạo dự án</CardTitle>
        <CardDescription>
          Tạo dự án mới trong vài click.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p>Nội dung card ở đây</p>
      </CardContent>
      <CardFooter>
        <Button>Tạo mới</Button>
      </CardFooter>
    </Card>
  );
}
```

#### Dialog (Modal)

```tsx
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

export default function DialogDemo() {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button>Mở Modal</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Xác nhận</DialogTitle>
          <DialogDescription>
            Bạn có chắc chắn muốn thực hiện hành động này?
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline">Hủy</Button>
          <Button>Xác nhận</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

### 1.4. Form với Zod Validation

#### Cài đặt dependencies

```bash
npm install react-hook-form zod @hookform/resolvers
npx shadcn@latest add form
```

#### Định nghĩa Schema

```tsx
import { z } from "zod";

const loginSchema = z.object({
  email: z.string().email("Email không hợp lệ"),
  password: z.string().min(6, "Mật khẩu tối thiểu 6 ký tự"),
});

type LoginFormValues = z.infer<typeof loginSchema>;
```

#### Tạo Form Component

```tsx
"use client"

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";

const formSchema = z.object({
  email: z.string().email("Email không hợp lệ"),
  password: z.string().min(6, "Mật khẩu tối thiểu 6 ký tự"),
});

export default function LoginForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  function onSubmit(values: z.infer<typeof formSchema>) {
    console.log(values);
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input placeholder="name@example.com" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Mật khẩu</FormLabel>
              <FormControl>
                <Input type="password" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <Button type="submit" className="w-full">
          Đăng nhập
        </Button>
      </form>
    </Form>
  );
}
```

**📝 Giải thích:**

- `zodResolver`: Kết nối Zod schema với React Hook Form
- `FormField`: Wrapper cho mỗi field, tự động handle validation
- `FormMessage`: Hiển thị lỗi validation
- `FormControl`: Wrapper cho input element

### 1.5. Tùy Chỉnh Theme

ShadcnUI sử dụng CSS variables để theme:

```css
/* globals.css */
@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    /* ... */
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    /* ... */
  }
}
```

**Thay đổi màu primary:**

```css
:root {
  --primary: 221.2 83.2% 53.3%;  /* Blue */
}
```

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng form đăng nhập với:
- Email và password validation
- Loading state khi submit
- Error messages rõ ràng
- Remember me checkbox

**Yêu cầu**:

- Validate email format
- Password tối thiểu 6 ký tự
- Disable button khi đang submit
- Hiển thị lỗi inline

**🤔 Câu hỏi suy ngẫm:**

1. Schema Zod cần những validation rules nào?
2. Làm sao xử lý loading state?
3. Cách hiển thị error messages đẹp mắt?

<details>
<summary>💭 Gợi ý phân tích</summary>

```typescript
// Zod schema
const loginSchema = z.object({
  email: z.string().email(),
  password: z.string().min(6),
  rememberMe: z.boolean().optional(),
});

// Loading state
const [isLoading, setIsLoading] = useState(false);

// Submit handler
async function onSubmit(values) {
  setIsLoading(true);
  try {
    await login(values);
  } finally {
    setIsLoading(false);
  }
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: ShadcnUI không phải là package npm - các component được copy vào project. Bạn hoàn toàn có thể modify code.

#### ✅ Nên Làm

```tsx
// Sử dụng Form components đúng cách
<FormField
  control={form.control}
  name="email"
  render={({ field }) => (
    <FormItem>
      <FormLabel>Email</FormLabel>
      <FormControl>
        <Input {...field} />
      </FormControl>
      <FormDescription>
        Email sẽ được dùng để đăng nhập
      </FormDescription>
      <FormMessage />
    </FormItem>
  )}
/>
```

**Tại sao tốt:**

- Tự động handle validation errors
- Accessible với proper labels
- Consistent styling

#### ❌ Không Nên Làm

```tsx
// Không sử dụng FormField wrapper
<div>
  <label>Email</label>
  <Input
    value={form.watch("email")}
    onChange={(e) => form.setValue("email", e.target.value)}
  />
  {form.formState.errors.email && (
    <span>{form.formState.errors.email.message}</span>
  )}
</div>
```

**Tại sao không tốt:**

- Manual error handling, dễ bỏ sót
- Không accessible
- Thiếu nhất quán với các form khác

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Form không submit | Thiếu `"use client"` | Thêm directive ở đầu file |
| Validation không chạy | Thiếu `zodResolver` | Đảm bảo setup đúng resolver |
| Component không hiển thị | Chưa add component | Chạy `npx shadcn add [component]` |
| Theme không đổi | CSS variables không load | Kiểm tra import globals.css |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng form đăng nhập hoàn chỉnh với ShadcnUI

**Yêu cầu kỹ thuật:**

- Email và password fields
- Zod validation
- Loading state
- Error messages

#### Bước 1: Setup Schema và Form

```tsx
// components/LoginForm.tsx
"use client"

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";

const loginSchema = z.object({
  email: z.string().email("Email không hợp lệ"),
  password: z.string().min(6, "Mật khẩu tối thiểu 6 ký tự"),
});

type LoginValues = z.infer<typeof loginSchema>;
```

#### Bước 2: Tạo Form UI

```tsx
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";

export default function LoginForm() {
  const [isLoading, setIsLoading] = useState(false);

  const form = useForm<LoginValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  async function onSubmit(values: LoginValues) {
    setIsLoading(true);
    try {
      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1000));
      console.log("Login:", values);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <Card className="w-[400px]">
      <CardHeader>
        <CardTitle>Đăng nhập</CardTitle>
        <CardDescription>
          Nhập thông tin để đăng nhập vào tài khoản
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="name@example.com"
                      disabled={isLoading}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Mật khẩu</FormLabel>
                  <FormControl>
                    <Input
                      type="password"
                      disabled={isLoading}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? "Đang đăng nhập..." : "Đăng nhập"}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}
```

**📝 Giải thích:**

- `isLoading`: Disable form và đổi text button khi submitting
- `disabled={isLoading}`: Prevent multiple submissions
- Card wrapper cho visual container

#### Bước 3: Sử dụng trong Page

```tsx
// app/login/page.tsx
import LoginForm from "@/components/LoginForm";

export default function LoginPage() {
  return (
    <main className="min-h-screen flex items-center justify-center bg-gray-50">
      <LoginForm />
    </main>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo form đăng ký với email, username, password, confirm password

<details>
<summary>💡 Gợi ý</summary>

- Dùng `.refine()` để validate confirm password match
- Thêm username với min length 3

```typescript
const registerSchema = z.object({
  email: z.string().email(),
  username: z.string().min(3),
  password: z.string().min(6),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Mật khẩu không khớp",
  path: ["confirmPassword"],
});
```

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
"use client"

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";

const registerSchema = z.object({
  email: z.string().email("Email không hợp lệ"),
  username: z.string().min(3, "Username tối thiểu 3 ký tự"),
  password: z.string().min(6, "Mật khẩu tối thiểu 6 ký tự"),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Mật khẩu không khớp",
  path: ["confirmPassword"],
});

export default function RegisterForm() {
  const form = useForm<z.infer<typeof registerSchema>>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      email: "",
      username: "",
      password: "",
      confirmPassword: "",
    },
  });

  function onSubmit(values: z.infer<typeof registerSchema>) {
    console.log("Register:", values);
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 w-[350px]">
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Mật khẩu</FormLabel>
              <FormControl>
                <Input type="password" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="confirmPassword"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Xác nhận mật khẩu</FormLabel>
              <FormControl>
                <Input type="password" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <Button type="submit" className="w-full">Đăng ký</Button>
      </form>
    </Form>
  );
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo form đăng ký trong Dialog modal

**Mở rộng**:

- Nút mở modal ở ngoài
- Form đăng ký bên trong Dialog
- Đóng modal sau khi submit thành công
- Toast notification báo thành công

### 3.3. Mini Project

**Dự án**: Contact Form với Modal Confirmation

**Mô tả**: Xây dựng contact form có modal xác nhận trước khi gửi

**Yêu cầu chức năng:**

1. Form với: name, email, subject, message
2. Validation với Zod
3. Click Submit -> Mở Dialog xác nhận
4. Confirm -> Gửi và hiển thị success message

**Technical Stack:**

- Next.js 14+ với App Router
- ShadcnUI (Form, Dialog, Button, Input, Textarea)
- Zod + React Hook Form

**Hướng dẫn triển khai:**

```tsx
// State cho dialog
const [showConfirm, setShowConfirm] = useState(false);
const [formValues, setFormValues] = useState<FormValues | null>(null);

// Submit handler
function onSubmit(values: FormValues) {
  setFormValues(values);
  setShowConfirm(true);
}

// Confirm handler
function handleConfirm() {
  if (formValues) {
    // Send to API
    console.log("Sending:", formValues);
  }
  setShowConfirm(false);
  form.reset();
}
```

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu ShadcnUI và cách hoạt động
- [ ] Cài đặt và add components
- [ ] Sử dụng Button, Input, Card, Dialog
- [ ] Tạo form với Zod validation
- [ ] (Tùy chọn) Hoàn thành mini project Contact Form

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: ShadcnUI khác gì với MUI hay Chakra UI?
2. **Ứng dụng**: Làm sao để validate form với Zod?
3. **Phân tích**: Ưu điểm của việc "own" code component là gì?
4. **Thực hành**: Demo form đăng nhập với validation?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- So sánh ShadcnUI với các UI libraries khác
- Demo form đã tạo
- Chia sẻ cách customize components
- Tips khi làm việc với Zod

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: ShadcnUI components được lưu ở đâu trong project?

- A. `node_modules/@shadcn/ui`
- B. `components/ui/`
- C. `lib/shadcn/`
- D. `public/components/`

**Câu 2**: Để kết nối Zod schema với React Hook Form, bạn dùng gì?

- A. `zodAdapter`
- B. `zodResolver`
- C. `zodValidator`
- D. `zodConnect`

**Câu 3**: Component nào dùng để bọc form field trong ShadcnUI Form?

- A. `FormWrapper`
- B. `FormInput`
- C. `FormField`
- D. `FieldGroup`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: ShadcnUI có phải là npm package không?</strong></summary>

Không! ShadcnUI không phải là npm package truyền thống. Khi bạn chạy `npx shadcn add button`, CLI sẽ copy source code của component vào project của bạn (thường là `components/ui/`). Điều này có nghĩa:

- Bạn có toàn quyền sửa đổi code
- Không bị phụ thuộc vào phiên bản package
- Có thể customize theo ý muốn

</details>

<details>
<summary><strong>Q2: Làm sao để customize màu sắc của components?</strong></summary>

ShadcnUI sử dụng CSS variables. Bạn có thể thay đổi trong `globals.css`:

```css
@layer base {
  :root {
    --primary: 221.2 83.2% 53.3%;      /* Đổi màu primary */
    --primary-foreground: 210 40% 98%;
  }
}
```

Hoặc trực tiếp override trong component:

```tsx
<Button className="bg-green-500 hover:bg-green-600">
  Custom Color
</Button>
```

</details>

<details>
<summary><strong>Q3: Tại sao form cần "use client"?</strong></summary>

Forms trong ShadcnUI sử dụng React Hook Form, cần hooks như `useForm`. Hooks chỉ hoạt động trong Client Components. Do đó, file chứa form phải có `"use client"` directive ở đầu.

```tsx
"use client"  // Bắt buộc cho form components

import { useForm } from "react-hook-form";
// ...
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 5

</footer>
