---
title: String
minutes: 5
---

Tên sản phẩm, email khách, nội dung hoá đơn đều là chữ. Chương trình nào cũng
phải ghép, cắt, tìm và so sánh chữ. Bài này giới thiệu kiểu `string` và những
thao tác dùng hằng ngày.

## Khái niệm

🔤 **String**: chuỗi ký tự đặt trong dấu nháy kép, ví dụ `"Bút bi"`.

💬 **String interpolation**: cách chèn giá trị vào chuỗi bằng dấu `$` đứng trước và cặp ngoặc `{ }` bên trong.

## Ví dụ

```csharp
string name = "  Bút bi Thiên Long  ";
decimal price = 5000m;

string clean = name.Trim();
Console.WriteLine(clean);          // Bút bi Thiên Long
Console.WriteLine(clean.Length);   // 17
Console.WriteLine(clean.ToUpper());
// BÚT BI THIÊN LONG

Console.WriteLine($"{clean} giá {price}đ");
```

- Gọi thao tác trên chuỗi bằng dấu chấm: `name.Trim()`.
- `$"{clean} giá {price}đ"` thay `{clean}` và `{price}` bằng giá trị của
  biến. Viết vậy gọn hơn nối bằng `+`.

Những thao tác hay dùng:

| Thao tác | Kết quả |
|---|---|
| `s.Length` | số ký tự |
| `s.Trim()` | bỏ khoảng trắng hai đầu |
| `s.ToUpper()`, `s.ToLower()` | viết hoa, viết thường |
| `s.Contains("bi")` | có chứa chuỗi con không (`bool`) |
| `s.Replace("a", "b")` | thay chuỗi con |
| `s.Substring(0, 3)` | cắt từ vị trí 0, lấy 3 ký tự |

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
string code = "pen-01";
code.ToUpper();
Console.WriteLine(code);

string upper = code.ToUpper();
Console.WriteLine(upper);
```

**Đoán trước khi chạy:** dòng in đầu tiên là `pen-01` hay `PEN-01`?

<details>
<summary>Xem kết quả</summary>

```text
pen-01
PEN-01
```

String không sửa được sau khi tạo. `ToUpper()` không đổi `code` mà trả về một
chuỗi **mới**. Muốn dùng kết quả thì phải gán nó vào một biến.

</details>

## Lỗi hay gặp

**Quên dấu `$`.** Thiếu `$` thì `{name}` được in ra nguyên văn.

```csharp
// SAI — in ra: Xin chào {name}
string name = "An";
Console.WriteLine("Xin chào {name}");
```

```csharp
// ĐÚNG — in ra: Xin chào An
string name = "An";
Console.WriteLine($"Xin chào {name}");
```

**So sánh mà quên chuyện hoa thường.** `==` coi `"Admin"` và `"admin"` là
khác nhau.

```csharp
// SAI — false dù người dùng gõ đúng chữ
string role = "Admin";
Console.WriteLine(role == "admin");
```

```csharp
// ĐÚNG — so sánh không phân biệt hoa thường
string role = "Admin";
Console.WriteLine(string.Equals(
    role, "admin", StringComparison.OrdinalIgnoreCase));
```

## Tóm tắt

- String đặt trong nháy kép, thao tác qua dấu chấm: `s.Trim()`,
  `s.Length`.
- `$"...{bien}..."` chèn giá trị vào chuỗi.
- String không sửa được. `ToUpper()`, `Trim()`, `Replace()` đều trả về chuỗi
  mới, phải gán lại mới dùng được.
- `==` phân biệt hoa thường.

```quiz
[
  {
    "prompt": "string email = \" an@shop.vn \"; email.Trim(); Console.WriteLine(email.Length); In ra gì?",
    "options": [
      "10",
      "12",
      "0",
      "Lỗi compile"
    ],
    "answer": 2,
    "explain": "Trim() trả về chuỗi mới nhưng không được gán lại, nên email vẫn còn hai khoảng trắng và dài 12 ký tự."
  },
  {
    "prompt": "int qty = 3; Dòng nào in ra đúng \"Còn 3 cái\"?",
    "options": [
      "Console.WriteLine(\"Còn {qty} cái\");",
      "Console.WriteLine(\"Còn qty cái\");",
      "Console.WriteLine($\"Còn {qty} cái\");",
      "Console.WriteLine($\"Còn qty cái\");"
    ],
    "answer": 3,
    "explain": "Cần cả dấu $ ở đầu chuỗi lẫn tên biến trong { } thì giá trị mới được chèn vào."
  },
  {
    "prompt": "Bạn cần kiểm tra tên sản phẩm có chứa chữ \"sale\" hay không. Dùng thao tác nào?",
    "options": [
      "name.Contains(\"sale\")",
      "name.Replace(\"sale\", \"\")",
      "name.Length",
      "name.Substring(0, 4)"
    ],
    "answer": 1,
    "explain": "Contains trả về true hoặc false tuỳ chuỗi có chứa chuỗi con hay không."
  }
]
```
