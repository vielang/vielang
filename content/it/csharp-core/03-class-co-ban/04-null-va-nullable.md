---
title: null và nullable
minutes: 5
---

Bạn tìm khách hàng theo số điện thoại, nhưng chưa có khách nào dùng số đó. Kết
quả là không có khách nào. C# dùng `null` để diễn tả "không có gì", và `null`
cũng là nguồn gây lỗi phổ biến nhất khi chương trình chạy.

## Khái niệm

🕳️ **null**: giá trị cho biết biến không trỏ tới object nào.

❓ **Nullable**: kiểu có dấu `?` phía sau, như `int?` hay `Customer?`, cho biết biến được phép mang giá trị `null`.

Reference type mặc định là `null` khi chưa trỏ tới object nào. Value type như
`int` thì không nhận `null`, trừ khi viết thành `int?`.

## Ví dụ

```csharp
Customer? found = FindByPhone("0900000000");

if (found != null)
{
    Console.WriteLine(found.Name);
}
else
{
    Console.WriteLine("Không tìm thấy");
}

Customer? FindByPhone(string phone)
{
    return null;   // giả sử không có ai
}

class Customer
{
    public string Name { get; set; } = "";
}
```

- `Customer?` báo cho người đọc và cho compiler: kết quả có thể là `null`.
- Luôn kiểm tra `!= null` trước khi dùng một biến nullable.
- Project .NET mới bật sẵn chế độ kiểm tra null. Dùng biến có thể `null` mà
  chưa kiểm tra thì compiler hiện cảnh báo.

## Hai toán tử ?. và ??

Hai toán tử giúp viết phần kiểm tra null ngắn hơn:

```csharp
string? note = null;
int? discount = null;

int length = note?.Length ?? 0;
int percent = discount ?? 0;

Console.WriteLine(length);    // 0
Console.WriteLine(percent);   // 0
```

- `note?.Length`: nếu `note` là `null` thì cả biểu thức là `null`, không gọi
  `.Length`.
- `a ?? b`: nếu `a` là `null` thì lấy `b`.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
string? coupon = null;
Console.WriteLine(coupon?.ToUpper() ?? "Không có mã");

coupon = "sale10";
Console.WriteLine(coupon?.ToUpper() ?? "Không có mã");
```

**Đoán trước khi chạy:** lần in đầu tiên có bị lỗi vì gọi `ToUpper()` trên
`null` không?

<details>
<summary>Xem kết quả</summary>

```text
Không có mã
SALE10
```

Không lỗi. `?.` thấy `coupon` là `null` nên bỏ qua `ToUpper()`, rồi `??` thay
bằng "Không có mã".

</details>

## Lỗi hay gặp

**Dùng biến `null` mà không kiểm tra.** Chương trình dừng với
`NullReferenceException`.

```csharp
// SAI — lỗi khi chạy: note đang là null
string? note = null;
Console.WriteLine(note.Length);
```

```csharp
// ĐÚNG
string? note = null;
Console.WriteLine(note?.Length ?? 0);
```

**Gán `null` cho value type.** `int` luôn phải có một con số.

```csharp
// SAI — lỗi compile: int không nhận null
int stock = null;
```

```csharp
// ĐÚNG — int? cho phép "chưa biết"
int? stock = null;
```

## Tóm tắt

- `null` nghĩa là không trỏ tới object nào.
- Thêm `?` sau kiểu (`int?`, `string?`) để cho phép `null`.
- Kiểm tra `!= null` trước khi dùng, hoặc dùng `?.` và `??`.
- Gọi thành viên trên `null` gây `NullReferenceException`.

```quiz
[
  {
    "prompt": "int? points = null; int total = points ?? 10; Giá trị của total là gì?",
    "options": [
      "null",
      "0",
      "10",
      "Lỗi compile"
    ],
    "answer": 3,
    "explain": "points là null nên ?? lấy vế phải là 10."
  },
  {
    "prompt": "Chương trình dừng với NullReferenceException ở dòng customer.Name.ToUpper(). Nguyên nhân nhiều khả năng nhất là gì?",
    "options": [
      "customer hoặc customer.Name đang là null",
      "Name là string nên không gọi ToUpper được",
      "Thiếu dấu ; ở cuối dòng",
      "ToUpper chỉ dùng được với chữ tiếng Anh"
    ],
    "answer": 1,
    "explain": "NullReferenceException xảy ra khi gọi thành viên trên một giá trị null. Cần kiểm tra null hoặc dùng ?."
  },
  {
    "prompt": "Khai báo nào hợp lệ?",
    "options": [
      "int quantity = null;",
      "decimal price = null;",
      "bool paid = null;",
      "int? quantity = null;"
    ],
    "answer": 4,
    "explain": "Value type như int, decimal, bool không nhận null. Thêm ? thành int? thì mới được."
  }
]
```
