---
title: Đặt tên và method ngắn
minutes: 5
---

Code được đọc nhiều hơn được viết rất nhiều lần: khi sửa lỗi, khi review pull
request, khi thêm tính năng. Tên mơ hồ và `if` lồng nhiều tầng làm người đọc
phải đoán. Bài này nói hai thói quen giúp code tự giải thích.

## Khái niệm

📛 **Tên rõ ý định (intention-revealing name)**: tên cho biết biến, method dùng để làm gì mà không cần đọc code bên trong hay comment.

✍️ **Guard clause (điều kiện chặn đầu)**: kiểm các trường hợp đặc biệt ở đầu method và `return` ngay, để phần chính của method không phải lồng trong `if`.

| Loại | Quy ước | Ví dụ |
|---|---|---|
| Method | động từ, nói việc làm | `CalculateShippingFee` |
| Biến `bool` | bắt đầu bằng `is`, `has`, `can` | `isMember`, `hasDiscount` |
| Biến số | kèm ý nghĩa hoặc đơn vị | `orderTotal`, `deliveryDays` |

## Ví dụ

Tính phí giao hàng: đơn từ 500.000 hoặc khách thành viên thì miễn phí, nội
thành Hà Nội 20.000, nơi khác 35.000.

Bản khó đọc:

```csharp
decimal Calc(decimal t, bool m, string c)
{
    decimal r;
    if (t < 500000)
    {
        if (m)
        {
            r = 0;
        }
        else
        {
            if (c == "Hà Nội")
            {
                r = 20000;
            }
            else
            {
                r = 35000;
            }
        }
    }
    else
    {
        r = 0;
    }
    return r;
}
```

Bản rõ ràng, cùng kết quả:

```csharp
decimal ShippingFee(
    decimal orderTotal, bool isMember, string city)
{
    if (orderTotal >= 500000)
    {
        return 0;
    }
    if (isMember)
    {
        return 0;
    }
    if (city == "Hà Nội")
    {
        return 20000;
    }
    return 35000;
}
```

- `t`, `m`, `c`, `r` bắt người đọc lần ngược xem chúng là gì. `orderTotal`,
  `isMember`, `city` đọc là hiểu.
- Mỗi guard clause xử lý xong một trường hợp rồi `return`. Không còn `if`
  lồng ba tầng.
- Method làm đúng một việc như SRP ở khoá OOP, nên ngắn và dễ đặt tên.

## Thử ngay

Gọi cả hai bản với cùng dữ liệu:

```csharp
Console.WriteLine(ShippingFee(600000, false, "Hà Nội"));
Console.WriteLine(ShippingFee(200000, true, "Đà Nẵng"));
Console.WriteLine(ShippingFee(200000, false, "Hà Nội"));
Console.WriteLine(
    ShippingFee(200000, false, "Đà Nẵng"));
```

**Đoán trước khi chạy:** bốn dòng in ra là gì? Đổi `ShippingFee` thành
`Calc` thì kết quả có khác không?

<details>
<summary>Xem kết quả</summary>

```text
0
0
20000
35000
```

`Calc` cho đúng bốn con số này. Đổi tên và bỏ `if` lồng không làm thay đổi
hành vi, chỉ làm code dễ đọc hơn.

</details>

## Lỗi hay gặp

**Dùng comment để chữa tên dở.** Comment dễ lỗi thời khi code đổi, còn tên
thì đi theo biến tới mọi chỗ nó được dùng.

```csharp
// SAI — cần comment mới hiểu d là gì
int d = 3;   // số ngày giao hàng
```

```csharp
// ĐÚNG — tên đã nói đủ
int deliveryDays = 3;
```

Comment nên dành để giải thích **vì sao** code làm vậy, ví dụ một quy định
của công ty, chứ không để giải thích code làm **gì**.

## Tóm tắt

- Tên nói rõ ý định: method là động từ, `bool` bắt đầu bằng `is`, `has`.
- Không viết tắt khó đoán như `t`, `m`, `r`.
- Guard clause xử lý trường hợp đặc biệt ở đầu và `return` sớm, bớt `if`
  lồng nhau.
- Comment giải thích "vì sao", tên giải thích "là gì".

```quiz
[
  {
    "prompt": "Biến bool cho biết đơn hàng đã thanh toán nên đặt tên thế nào?",
    "options": [
      "paid2",
      "isPaid",
      "flag",
      "p"
    ],
    "answer": 2,
    "explain": "Biến bool bắt đầu bằng is, has, can đọc như một câu hỏi có hoặc không."
  },
  {
    "prompt": "Guard clause giúp gì cho method có nhiều if lồng nhau?",
    "options": [
      "Chạy nhanh hơn",
      "Bớt được tham số",
      "Xử lý trường hợp đặc biệt rồi return ngay, phần còn lại không phải lồng trong if",
      "Không cần kiểm điều kiện nữa"
    ],
    "answer": 3,
    "explain": "Mỗi trường hợp đặc biệt thoát sớm, nên code chính nằm thẳng hàng, dễ đọc."
  },
  {
    "prompt": "Comment nào đáng giữ lại?",
    "options": [
      "// cộng 1 vào i",
      "// biến x là tổng tiền",
      "// lặp qua danh sách",
      "// Theo quy định thuế, hoá đơn dưới 200.000 không cần in VAT"
    ],
    "answer": 4,
    "explain": "Comment tốt giải thích lý do mà code không tự nói được, như một quy định nghiệp vụ."
  }
]
```
