
# TypeScript Trong Next.js App Router

> **Mô tả ngắn gọn**: Tìm hiểu cách cài đặt và cấu hình TypeScript, các type definitions cơ bản và áp dụng vào React components.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu được TypeScript là gì và tại sao nên sử dụng
- [ ] Biết cách cài đặt TypeScript trong dự án Next.js
- [ ] Nắm rõ cấu trúc file `tsconfig.json`
- [ ] Sử dụng các type definitions cơ bản
- [ ] Áp dụng TypeScript để viết React components có kiểu dữ liệu rõ ràng

### Kiến Thức Yêu Cầu

- Bài 1-2: Next.js App Router cơ bản
- JavaScript ES6+ (arrow functions, destructuring)
- React components và props

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức về TypeScript | 15 phút |
| 2 | Phân tích & Tư duy | 10 phút |
| 3 | Thực hành viết components | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. TypeScript Là Gì?

> **💡 Định nghĩa**: TypeScript là ngôn ngữ lập trình dựa trên JavaScript, mở rộng thêm tính năng gõ kiểu tĩnh (static typing).

**Tại sao nên dùng TypeScript?**

- Phát hiện lỗi sớm ngay khi viết code
- IDE hỗ trợ autocomplete và intellisense tốt hơn
- Code dễ đọc, dễ bảo trì
- Refactoring an toàn hơn

**Ví dụ đơn giản:**

```typescript
// JavaScript - không có kiểu
let age = 30;
age = "thirty"; // Không báo lỗi, nhưng có thể gây bug

// TypeScript - có kiểu
let age: number = 30;
age = "thirty"; // Lỗi ngay khi viết code!
```

### 1.2. Cài Đặt TypeScript

#### Cách 1: Tạo dự án mới với TypeScript

```bash
npx create-next-app@latest my-app
# Chọn "Yes" khi được hỏi "Would you like to use TypeScript?"
cd my-app
```

#### Cách 2: Thêm vào dự án hiện tại

```bash
# Cài đặt dependencies
npm install --save-dev typescript @types/react @types/node

# Tạo tsconfig.json tự động
npx next dev
```

**📝 Giải thích:**

- `typescript`: Compiler TypeScript
- `@types/react`: Type definitions cho React
- `@types/node`: Type definitions cho Node.js

### 1.3. File `tsconfig.json`

> **💡 Định nghĩa**: File cấu hình quyết định cách TypeScript hoạt động trong dự án.

```json
{
  "compilerOptions": {
    "target": "esnext",
    "module": "esnext",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "noEmit": true,
    "esModuleInterop": true,
    "moduleResolution": "node",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve"
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules"]
}
```

**Các option quan trọng:**

| Option | Mô tả |
|--------|-------|
| `strict` | Bật chế độ kiểm tra nghiêm ngặt |
| `allowJs` | Cho phép mix JS và TS |
| `jsx` | Hỗ trợ JSX syntax |
| `include` | Files được TypeScript xử lý |

### 1.4. Type Definitions Cơ Bản

#### Kiểu dữ liệu nguyên thủy

```typescript
let name: string = "Nguyen Van A";
let age: number = 25;
let isStudent: boolean = true;
let anything: any = "có thể là bất kỳ kiểu gì";
```

**📝 Lưu ý:** Tránh dùng `any` vì mất đi lợi ích của TypeScript.

#### Arrays và Objects

```typescript
// Array
let numbers: number[] = [1, 2, 3];
let names: string[] = ["An", "Bình", "Chi"];

// Object với inline type
let user: { name: string; age: number } = {
  name: "An",
  age: 25
};
```

#### Interface

```typescript
interface User {
  id: number;
  name: string;
  email?: string;  // ? = optional
}

const user: User = {
  id: 1,
  name: "Nguyen Van A"
  // email không bắt buộc
};
```

**📝 Đặc điểm Interface:**

- Mô tả "hình dạng" của object
- `?` đánh dấu property không bắt buộc
- Có thể extend từ interface khác

#### Type vs Interface

```typescript
// Type - linh hoạt hơn
type ID = string | number;
type UserType = {
  id: ID;
  name: string;
};

// Interface - tốt cho objects, có thể extend
interface UserInterface {
  id: number;
  name: string;
}

interface AdminUser extends UserInterface {
  role: string;
}
```

### 1.5. TypeScript Trong React Components

#### Props với Interface

```tsx
// app/components/UserCard.tsx
interface UserCardProps {
  name: string;
  age: number;
  isOnline?: boolean;
}

export default function UserCard({
  name,
  age,
  isOnline = false
}: UserCardProps) {
  return (
    <div className="p-4 border rounded">
      <h2 className="font-bold">{name}</h2>
      <p>Tuổi: {age}</p>
      <p>Trạng thái: {isOnline ? "Online" : "Offline"}</p>
    </div>
  );
}
```

**📝 Giải thích:**

- `UserCardProps` định nghĩa kiểu cho props
- `isOnline = false` là giá trị mặc định cho prop optional

#### Sử dụng Component

```tsx
// app/page.tsx
import UserCard from "./components/UserCard";

export default function HomePage() {
  return (
    <div>
      <UserCard name="An" age={25} />
      <UserCard name="Bình" age={30} isOnline />
    </div>
  );
}
```

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Bạn đang xây dựng một ứng dụng quản lý sản phẩm. Cần tạo component ProductCard hiển thị thông tin sản phẩm với các trường: tên, giá, mô tả (optional), số lượng tồn kho.

**Yêu cầu**:

- Type-safe cho props
- Xử lý trường hợp mô tả không có
- Format giá tiền

**🤔 Câu hỏi suy ngẫm:**

1. Interface cho ProductCard nên có những trường nào?
2. Làm sao để đảm bảo giá luôn là số dương?
3. Cách xử lý khi description không được truyền?

<details>
<summary>💭 Gợi ý phân tích</summary>

```typescript
interface ProductCardProps {
  name: string;
  price: number;
  description?: string;
  stock: number;
}
```

- `description?`: Optional với dấu `?`
- Kiểm tra `price > 0` bằng logic trong component
- Dùng conditional rendering cho description

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Luôn khai báo kiểu cho props trong component để tránh lỗi runtime.

#### ✅ Nên Làm

```tsx
// Định nghĩa rõ ràng interface
interface ButtonProps {
  label: string;
  onClick: () => void;
  variant?: "primary" | "secondary";
}

export function Button({
  label,
  onClick,
  variant = "primary"
}: ButtonProps) {
  return (
    <button
      className={variant === "primary" ? "bg-blue-500" : "bg-gray-500"}
      onClick={onClick}
    >
      {label}
    </button>
  );
}
```

**Tại sao tốt:**

- IDE hiển thị gợi ý khi sử dụng component
- Lỗi được phát hiện ngay khi truyền sai props
- Code tự document qua interface

#### ❌ Không Nên Làm

```tsx
// Không có type, dùng any
export function Button(props: any) {
  return <button onClick={props.onClick}>{props.label}</button>;
}
```

**Tại sao không tốt:**

- Mất hết lợi ích của TypeScript
- Không có autocomplete
- Dễ gây lỗi runtime

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| `Object is possibly 'undefined'` | Truy cập optional property | Dùng optional chaining `?.` hoặc kiểm tra null |
| `Type 'string' is not assignable to type 'number'` | Sai kiểu dữ liệu | Kiểm tra lại kiểu khi gán giá trị |
| Props không được nhận | Thiếu destructuring hoặc sai tên | Đảm bảo tên props khớp với interface |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Tạo component ProfileCard với TypeScript

**Yêu cầu kỹ thuật:**

- `username`: chuỗi (bắt buộc)
- `email`: chuỗi (không bắt buộc)
- `age`: số (bắt buộc)

#### Bước 1: Định nghĩa Interface

```tsx
// app/components/ProfileCard.tsx
interface ProfileCardProps {
  username: string;
  email?: string;
  age: number;
}
```

#### Bước 2: Tạo Component

```tsx
export default function ProfileCard({
  username,
  email,
  age
}: ProfileCardProps) {
  return (
    <div className="max-w-sm p-6 bg-white border rounded-lg shadow">
      <h2 className="text-xl font-bold text-gray-800">{username}</h2>
      <p className="text-gray-600">Tuổi: {age}</p>
      <p className="text-gray-500">
        {email ? email : "Email chưa cập nhật"}
      </p>
    </div>
  );
}
```

**📝 Giải thích:**

- Conditional rendering cho email: nếu có thì hiển thị, không thì hiển thị text mặc định
- Tailwind CSS cho styling đơn giản

#### Bước 3: Sử dụng Component

```tsx
// app/page.tsx
import ProfileCard from "./components/ProfileCard";

export default function HomePage() {
  return (
    <div className="p-8 space-y-4">
      <ProfileCard username="NguyenVanA" age={25} email="a@email.com" />
      <ProfileCard username="TranVanB" age={30} />
    </div>
  );
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo component `TodoItem`

Props:
- `title`: string (bắt buộc)
- `completed`: boolean (mặc định false)
- `dueDate`: string (không bắt buộc)

<details>
<summary>💡 Gợi ý</summary>

- Dùng `interface` để định nghĩa props
- Dùng giá trị mặc định cho `completed`
- Conditional rendering cho `dueDate`

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```tsx
interface TodoItemProps {
  title: string;
  completed?: boolean;
  dueDate?: string;
}

export default function TodoItem({
  title,
  completed = false,
  dueDate
}: TodoItemProps) {
  return (
    <div className={`p-4 border rounded ${completed ? "bg-green-50" : ""}`}>
      <h3 className={completed ? "line-through" : ""}>{title}</h3>
      {dueDate && (
        <p className="text-sm text-gray-500">Hạn: {dueDate}</p>
      )}
      <span className="text-xs">
        {completed ? "Hoàn thành" : "Chưa hoàn thành"}
      </span>
    </div>
  );
}
```

**Giải thích:**

- `completed = false`: Giá trị mặc định
- `{dueDate && ...}`: Chỉ render khi có dueDate
- Dynamic className dựa trên completed status

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Tạo component `ProductList` với array của products

**Mở rộng**:

- Định nghĩa interface `Product`
- Props nhận array `products: Product[]`
- Thêm hàm callback `onProductClick: (id: number) => void`

### 3.3. Mini Project

**Dự án**: Contact List App

**Mô tả**: Xây dựng ứng dụng hiển thị danh sách liên hệ với TypeScript

**Yêu cầu chức năng:**

1. Interface `Contact` với: id, name, email, phone (optional), avatar (optional)
2. Component `ContactCard` hiển thị thông tin contact
3. Component `ContactList` nhận array contacts và render danh sách

**Technical Stack:**

- Next.js 14+ với App Router
- TypeScript strict mode
- Tailwind CSS

**Hướng dẫn triển khai:**

```tsx
// types/contact.ts
export interface Contact {
  id: number;
  name: string;
  email: string;
  phone?: string;
  avatar?: string;
}

// components/ContactCard.tsx
interface ContactCardProps {
  contact: Contact;
  onClick?: (id: number) => void;
}

// components/ContactList.tsx
interface ContactListProps {
  contacts: Contact[];
  onContactClick?: (id: number) => void;
}
```

---

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu TypeScript và lý do sử dụng
- [ ] Cài đặt được TypeScript trong Next.js
- [ ] Nắm được các type cơ bản
- [ ] Viết được component với typed props
- [ ] (Tùy chọn) Hoàn thành mini project Contact List

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: TypeScript khác gì JavaScript?
2. **Ứng dụng**: Khi nào dùng `interface`, khi nào dùng `type`?
3. **Phân tích**: Tại sao nên tránh dùng `any`?
4. **Thực hành**: Demo component với typed props?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Lợi ích của TypeScript trong dự án thực tế
- Demo component với interface
- Chia sẻ lỗi thường gặp và cách fix
- Tips khi làm việc với TypeScript

---

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Ký hiệu nào đánh dấu một property là optional trong TypeScript?

- A. `!`
- B. `?`
- C. `*`
- D. `&`

**Câu 2**: File nào chứa cấu hình TypeScript?

- A. `typescript.json`
- B. `ts.config.js`
- C. `tsconfig.json`
- D. `config.ts`

**Câu 3**: Kiểu dữ liệu nào nên tránh sử dụng trong TypeScript?

- A. `string`
- B. `number`
- C. `any`
- D. `boolean`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Có cần cài TypeScript riêng cho Next.js không?</strong></summary>

Khi tạo dự án mới bằng `create-next-app`, bạn có thể chọn TypeScript ngay từ đầu. Next.js sẽ tự động cấu hình mọi thứ. Nếu thêm vào dự án có sẵn, cần cài `typescript`, `@types/react`, `@types/node` và chạy `next dev` để tự động tạo `tsconfig.json`.

</details>

<details>
<summary><strong>Q2: Nên dùng interface hay type cho React props?</strong></summary>

Cả hai đều hoạt động tốt cho props. Quy ước phổ biến:
- **Interface**: Dùng cho object shapes, đặc biệt là props và state
- **Type**: Dùng cho union types, intersection types, hoặc kiểu phức tạp

```tsx
// Thường dùng interface cho props
interface ButtonProps {
  label: string;
}

// Type cho union
type ButtonVariant = "primary" | "secondary" | "danger";
```

</details>

<details>
<summary><strong>Q3: Làm sao để TypeScript không báo lỗi với thư viện JS?</strong></summary>

Cài `@types/{package-name}` cho thư viện đó. Ví dụ: `npm install @types/lodash`. Nếu không có type definitions, có thể tạo file `.d.ts` để khai báo hoặc dùng `// @ts-ignore` (không khuyến khích).

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Next.js App Router | **Lesson**: 3

</footer>
