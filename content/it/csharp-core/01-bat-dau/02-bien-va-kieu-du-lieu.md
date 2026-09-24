---
title: Biến và kiểu dữ liệu
minutes: 5
---

Chương trình bán hàng cần nhớ tên sản phẩm, giá, số lượng trong kho. Để lưu
những giá trị đó, bạn dùng biến. Mỗi biến có một kiểu, và chọn đúng kiểu là
việc đầu tiên phải làm.

## Khái niệm

📦 **Biến (variable)**: một cái tên dùng để lưu và đọc lại một giá trị trong chương trình.

🏷️ **Kiểu dữ liệu (data type)**: loại giá trị mà biến được phép chứa, ví dụ số nguyên hay chuỗi.

C# kiểm tra kiểu ngay lúc build. Biến đã khai báo kiểu `int` thì không gán
chuỗi vào được.

## Ví dụ

```csharp
string name = "Bút bi";
int quantity = 120;
decimal price = 5000.50m;
bool inStock = true;

Console.WriteLine(name);
Console.WriteLine(price);
```

Khai báo biến theo mẫu `kiểu tên = giá trị;`. Năm kiểu dùng nhiều nhất:

| Kiểu | Chứa gì | Ví dụ |
|---|---|---|
| `int` | số nguyên | `120` |
| `double` | số thực, tính nhanh nhưng có sai số nhỏ | `1.75` |
| `decimal` | số thực chính xác, dùng cho tiền | `5000.50m` |
| `bool` | đúng hoặc sai | `true`, `false` |
| `string` | chuỗi chữ | `"Bút bi"` |

- Số `decimal` phải có chữ `m` ở cuối: `5000.50m`.
- Tên biến viết kiểu camelCase: chữ đầu thường, các từ sau viết hoa
  (`inStock`, `unitPrice`).

## Từ khoá var

Khi giá trị bên phải đã rõ kiểu, bạn có thể viết `var` thay cho tên kiểu:

```csharp
var quantity = 120;      // int
var price = 5000.50m;    // decimal
var name = "Bút bi";     // string
```

`var` không phải kiểu "gì cũng được". Compiler tự suy ra kiểu từ giá trị, và
kiểu đó cố định từ đó về sau.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
double a = 0.1;
double b = 0.2;
Console.WriteLine(a + b);

decimal c = 0.1m;
decimal d = 0.2m;
Console.WriteLine(c + d);
```

**Đoán trước khi chạy:** hai dòng in ra có giống nhau không?

<details>
<summary>Xem kết quả</summary>

```text
0.30000000000000004
0.3
```

Không giống. `double` lưu số theo hệ nhị phân nên có sai số nhỏ. `decimal`
lưu chính xác số thập phân, vì vậy tiền luôn dùng `decimal`.

</details>

## Lỗi hay gặp

**Gán sai kiểu.** `"5"` là chuỗi, không phải số.

```csharp
// SAI — lỗi compile: không gán chuỗi cho int
int quantity = "5";
```

```csharp
// ĐÚNG
int quantity = 5;
```

**Quên chữ `m` với `decimal`.** Số có dấu chấm mặc định là `double`.

```csharp
// SAI — lỗi compile: 9.99 là double
decimal price = 9.99;
```

```csharp
// ĐÚNG
decimal price = 9.99m;
```

## Tóm tắt

- Khai báo biến: `kiểu tên = giá trị;`.
- `int` cho số nguyên, `decimal` cho tiền, `bool` cho đúng/sai, `string` cho
  chữ.
- `var` cho compiler tự suy ra kiểu, nhưng kiểu vẫn cố định.
- Tiền dùng `decimal`, không dùng `double`.

```quiz
[
  {
    "prompt": "Bạn cần lưu tổng tiền của một hoá đơn. Kiểu nào phù hợp nhất?",
    "options": [
      "int",
      "double",
      "string",
      "decimal"
    ],
    "answer": 4,
    "explain": "decimal lưu số thập phân chính xác, không có sai số như double. int không chứa được phần lẻ."
  },
  {
    "prompt": "Sau dòng var count = 10; thì dòng count = \"mười\"; có chạy được không?",
    "options": [
      "Được, vì var chứa được mọi kiểu",
      "Không, count đã là int nên không gán chuỗi được",
      "Được, nhưng count thành null",
      "Không, vì phải viết var count = \"mười\""
    ],
    "answer": 2,
    "explain": "var chỉ để compiler tự suy ra kiểu. count được suy ra là int và giữ kiểu int mãi mãi."
  },
  {
    "prompt": "Biến lưu trạng thái \"đơn hàng đã thanh toán hay chưa\" nên dùng kiểu gì?",
    "options": [
      "bool",
      "string",
      "int",
      "decimal"
    ],
    "answer": 1,
    "explain": "Chỉ có hai trạng thái: rồi hoặc chưa. bool với true/false diễn tả đúng điều đó."
  }
]
```
