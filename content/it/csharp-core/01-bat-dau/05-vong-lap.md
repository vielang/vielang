---
title: Vòng lặp
minutes: 5
---

In bảng giá cho 1 đến 10 sản phẩm, hay nhắc khách nhập lại cho tới khi nhập
đúng: đó là những việc lặp đi lặp lại. Thay vì viết một câu lệnh mười lần, bạn
dùng vòng lặp.

## Khái niệm

🔁 **Vòng lặp (loop)**: khối code được chạy lặp lại nhiều lần, chừng nào điều kiện lặp còn đúng.

🔢 **for**: vòng lặp dùng khi biết trước số lần lặp, có sẵn biến đếm.

⏳ **while**: vòng lặp dùng khi chỉ biết điều kiện dừng, không biết trước số lần lặp.

## Ví dụ

```csharp
decimal price = 5000m;

for (int i = 1; i <= 3; i++)
{
    Console.WriteLine(i + " cái: " + price * i);
}
```

```text
1 cái: 5000
2 cái: 10000
3 cái: 15000
```

Dòng `for` có ba phần, cách nhau bằng dấu `;`:

- `int i = 1`: khởi tạo biến đếm, chạy một lần lúc bắt đầu.
- `i <= 3`: điều kiện, còn đúng thì còn lặp.
- `i++`: tăng `i` thêm 1 sau mỗi vòng (`i--` thì giảm 1).

## while và break

`while` chỉ có điều kiện. Vòng lặp chạy chừng nào điều kiện còn đúng:

```csharp
int stock = 10;
int orders = 0;

while (stock >= 3)
{
    stock = stock - 3;
    orders++;
}

Console.WriteLine(orders);   // 3
Console.WriteLine(stock);    // 1
```

`break` thoát khỏi vòng lặp ngay lập tức, kể cả khi điều kiện vẫn đúng.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
for (int i = 1; i <= 10; i++)
{
    if (i == 4)
    {
        break;
    }
    Console.WriteLine("Đơn số " + i);
}
```

**Đoán trước khi chạy:** màn hình in ra bao nhiêu dòng?

<details>
<summary>Xem kết quả</summary>

```text
Đơn số 1
Đơn số 2
Đơn số 3
```

Ba dòng. Khi `i` bằng 4, `break` thoát vòng lặp trước khi kịp in.

</details>

## Lỗi hay gặp

**Vòng lặp vô tận.** Bạn quên thay đổi biến dùng trong điều kiện, nên điều
kiện đúng mãi và chương trình treo.

```csharp
// SAI — stock không bao giờ giảm
int stock = 10;
while (stock > 0)
{
    Console.WriteLine("Bán 1 cái");
}
```

```csharp
// ĐÚNG
int stock = 10;
while (stock > 0)
{
    Console.WriteLine("Bán 1 cái");
    stock--;
}
```

**Lệch một vòng.** `<` và `<=` khác nhau đúng một lần lặp.

```csharp
// SAI — muốn in 1 đến 5 nhưng chỉ ra 1 đến 4
for (int i = 1; i < 5; i++)
{
    Console.WriteLine(i);
}
```

```csharp
// ĐÚNG
for (int i = 1; i <= 5; i++)
{
    Console.WriteLine(i);
}
```

## Tóm tắt

- Biết trước số lần lặp thì dùng `for`, chỉ biết điều kiện dừng thì dùng
  `while`.
- `for (khởi tạo; điều kiện; bước tăng)`.
- `break` thoát vòng lặp ngay.
- Luôn kiểm tra: biến trong điều kiện có thay đổi không, và dùng `<` hay `<=`.

```quiz
[
  {
    "prompt": "for (int i = 0; i < 3; i++) Console.WriteLine(i); In ra những số nào?",
    "options": [
      "1 2 3",
      "0 1 2 3",
      "0 1 2",
      "0 1"
    ],
    "answer": 3,
    "explain": "i bắt đầu từ 0 và dừng khi i < 3 thành sai, tức là lúc i bằng 3. Nên in 0, 1, 2."
  },
  {
    "prompt": "Chương trình cần hỏi mật khẩu cho tới khi người dùng nhập đúng. Vòng lặp nào phù hợp?",
    "options": [
      "while, vì không biết trước người dùng nhập sai mấy lần",
      "for, vì luôn biết số lần lặp",
      "switch",
      "Không cần vòng lặp"
    ],
    "answer": 1,
    "explain": "Số lần nhập không biết trước, chỉ biết điều kiện dừng là nhập đúng. Đó là việc của while."
  },
  {
    "prompt": "int n = 5; while (n > 0) { Console.WriteLine(n); } Chuyện gì xảy ra?",
    "options": [
      "In 5 4 3 2 1",
      "Lỗi compile",
      "In 5 một lần rồi dừng",
      "In 5 mãi không dừng"
    ],
    "answer": 4,
    "explain": "n không bao giờ giảm nên n > 0 đúng mãi. Cần thêm n-- trong thân vòng lặp."
  }
]
```
