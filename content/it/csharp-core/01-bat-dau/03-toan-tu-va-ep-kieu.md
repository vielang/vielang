---
title: Toán tử và ép kiểu
minutes: 5
---

Tính tổng tiền, so sánh tồn kho, đổi chuỗi người dùng nhập thành số: việc nào
cũng cần toán tử và ép kiểu. Bài này gom những toán tử dùng hằng ngày, kèm hai
cái bẫy mà người mới hay rơi vào.

## Khái niệm

🧮 **Toán tử (operator)**: ký hiệu thực hiện một phép tính trên giá trị, như `+`, `>`, `&&`.

🔄 **Ép kiểu (casting)**: chuyển một giá trị từ kiểu này sang kiểu khác, như từ `double` sang `int`.

## Ví dụ

```csharp
int quantity = 7;
decimal price = 5000m;

decimal total = price * quantity;      // 35000
bool bigOrder = total > 30000;         // true
bool freeShip = bigOrder && quantity >= 5;

Console.WriteLine(total);
Console.WriteLine(freeShip);
```

Các toán tử hay dùng:

| Nhóm | Toán tử | Ý nghĩa |
|---|---|---|
| Số học | `+` `-` `*` `/` `%` | cộng, trừ, nhân, chia, chia lấy dư |
| So sánh | `==` `!=` `>` `<` `>=` `<=` | kết quả là `bool` |
| Logic | `&&` `\|\|` `!` | và, hoặc, phủ định |

- `%` cho phần dư: `7 % 2` bằng `1`.
- `==` là so sánh, `=` là gán. Hai dấu này khác nhau hoàn toàn.
- `+` với chuỗi là nối chuỗi: `"Tổng: " + total`.

## Ép kiểu

Khi không mất dữ liệu, C# tự chuyển kiểu, ví dụ từ `int` sang `double`. Khi có
thể mất dữ liệu, bạn phải ghi rõ kiểu đích trong ngoặc:

```csharp
double rating = 4.8;
int stars = (int)rating;    // 4, phần lẻ bị cắt

string input = "12";
int quantity = int.Parse(input);   // chuỗi -> số
```

- `(int)` cắt bỏ phần lẻ, không làm tròn.
- Chuỗi thì không ép bằng ngoặc được, phải dùng `int.Parse`,
  `decimal.Parse`.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
Console.WriteLine(7 / 2);
Console.WriteLine(7 / 2.0);
Console.WriteLine(7 % 2);
Console.WriteLine((int)3.9);
```

**Đoán trước khi chạy:** dòng đầu in ra `3.5` hay số khác?

<details>
<summary>Xem kết quả</summary>

```text
3
3.5
1
3
```

`7 / 2` là phép chia hai số nguyên nên kết quả cũng là số nguyên, phần lẻ bị
bỏ. Chỉ cần một vế là số thực (`2.0`) thì kết quả mới là `3.5`.

</details>

## Lỗi hay gặp

**Chia nguyên ngoài ý muốn.** Tính trung bình mà cả hai vế đều là `int`.

```csharp
// SAI — ra 3 chứ không phải 3.5
int totalStars = 7;
int reviews = 2;
double average = totalStars / reviews;
```

```csharp
// ĐÚNG — ép một vế sang double trước khi chia
int totalStars = 7;
int reviews = 2;
double average = (double)totalStars / reviews;
```

**Nối chuỗi với phép cộng.** C# đọc từ trái sang phải. Gặp chuỗi trước thì
mọi dấu `+` phía sau đều thành nối chuỗi.

```csharp
// SAI — in ra "Tổng: 12"
Console.WriteLine("Tổng: " + 1 + 2);
```

```csharp
// ĐÚNG — in ra "Tổng: 3"
Console.WriteLine("Tổng: " + (1 + 2));
```

## Tóm tắt

- `/` giữa hai số nguyên cho số nguyên. Cần phần lẻ thì ép một vế sang
  `double` hoặc `decimal`.
- `==` để so sánh, `=` để gán.
- `(int)` cắt phần lẻ, không làm tròn.
- Chuỗi sang số dùng `int.Parse`, `decimal.Parse`.

```quiz
[
  {
    "prompt": "int boxes = 10; int perBox = 4; Console.WriteLine(boxes / perBox); In ra gì?",
    "options": [
      "2.5",
      "3",
      "Lỗi compile",
      "2"
    ],
    "answer": 4,
    "explain": "Cả hai vế là int nên đây là phép chia nguyên. 10 / 4 được 2, phần lẻ bị bỏ."
  },
  {
    "prompt": "Người dùng nhập \"25\" vào biến string input. Cách nào đổi nó thành số int?",
    "options": [
      "int.Parse(input)",
      "(int)input",
      "int age = input;",
      "input.ToInt()"
    ],
    "answer": 1,
    "explain": "Không ép chuỗi sang số bằng ngoặc được. Chuỗi sang số phải dùng int.Parse."
  },
  {
    "prompt": "Biểu thức nào đúng khi đơn hàng trên 500000 VÀ khách là thành viên?",
    "options": [
      "total > 500000 || isMember",
      "total > 500000 && isMember",
      "total > 500000 & isMember == false",
      "total = 500000 && isMember"
    ],
    "answer": 2,
    "explain": "&& là \"và\": cả hai điều kiện phải đúng. || là \"hoặc\", còn = là phép gán chứ không phải so sánh."
  }
]
```
