---
title: Kiểu dữ liệu và biến
minutes: 8
---

C# là ngôn ngữ **statically typed**: kiểu của biến được xác định lúc compile, sai
kiểu là báo lỗi ngay chứ không đợi tới lúc chạy.

## Value type và reference type

```csharp
int a = 5;          // value type  — giá trị nằm ngay trong biến
string s = "abc";   // reference type — biến giữ tham chiếu tới vùng nhớ
```

- **Value type**: `int`, `double`, `bool`, `decimal`, `DateTime`, `struct`, `enum`. Gán là **chép giá trị**.
- **Reference type**: `string`, `class`, `array`, `record class`. Gán là **chép tham chiếu**, hai biến cùng trỏ một đối tượng.

Trong nghiệp vụ tiền bạc, dùng `decimal` chứ không dùng `double`: `double` là số
thực nhị phân nên `0.1 + 0.2` không ra đúng `0.3`.

## var và kiểu tường minh

```csharp
var total = 10;             // compiler suy ra int
var name = "Huy";           // string
List<string> tags = new();  // target-typed new
```

`var` **không phải** kiểu động. Nó chỉ nói "compiler tự suy ra", còn kiểu vẫn cố
định. Dùng `var` khi kiểu đã hiện rõ ở vế phải.

## Nullable reference types

Từ C# 8, project bật `<Nullable>enable</Nullable>` thì compiler cảnh báo khi bạn
có thể chạm vào `null`:

```csharp
string? middleName = null;   // cho phép null
string firstName = "Huy";    // không được null

int length = middleName.Length;      // cảnh báo CS8602
int safe = middleName?.Length ?? 0;  // an toàn
```

`?.` là **null-conditional operator**, `??` là **null-coalescing operator**. Hai
toán tử này là cách ngắn gọn thay cho `if (x != null)`.

## Ghi nhớ

- Đổi kiểu: `int.Parse` ném **exception** khi chuỗi sai; `int.TryParse` trả `bool` — dùng `TryParse` cho dữ liệu từ người dùng.
- `const` cố định lúc compile, `readonly` gán một lần lúc chạy (thường trong **constructor**).
