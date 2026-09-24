---
title: Đặt tên và method ngắn
minutes: 5
---

Code được đọc nhiều hơn được viết: khi sửa lỗi, khi review pull request, khi
thêm tính năng. Tên mơ hồ và `if` lồng nhiều tầng bắt người đọc phải đoán. Bài
này dạy hai thói quen giúp code dễ đọc mà không cần comment.

## Khái niệm

📛 **Tên rõ ý định (intention-revealing name)**: tên cho biết biến, method dùng để làm gì mà không cần đọc code bên trong hay comment.

✍️ **Guard clause (điều kiện chặn đầu)**: lệnh `if` ở đầu method xử lý một trường hợp đặc biệt rồi `return` ngay.

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
- Mỗi guard clause xử lý xong một trường hợp rồi `return`, nên phần sau
  không phải lồng trong `if`.
- Method chỉ làm một việc, cùng tinh thần với SRP ở khoá OOP, nên ngắn và dễ
  đặt tên.

## Thử ngay

Gọi `ShippingFee` với bốn bộ dữ liệu:

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

`Calc` cho đúng bốn con số này. Đổi tên và bỏ `if` lồng không đổi hành vi,
chỉ làm code dễ đọc hơn.

</details>

## Lỗi hay gặp

**Dùng comment để chữa tên dở.** Code đổi mà quên sửa comment thì comment
thành sai. Tên thì đi theo biến tới mọi chỗ biến được dùng.

```csharp
// SAI — cần comment mới hiểu d là gì
int d = 3;   // số ngày giao hàng
```

```csharp
// ĐÚNG — tên đã nói đủ
int deliveryDays = 3;
```

Comment dùng để giải thích **vì sao** code làm vậy, ví dụ một quy định của
công ty, không phải để giải thích code làm **gì**.

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
      "paidOrder",
      "isPaid",
      "orderFlag",
      "p"
    ],
    "answer": 2,
    "explain": "Biến bool bắt đầu bằng is, has, can đọc như một câu hỏi có hoặc không."
  },
  {
    "prompt": "Guard clause giúp gì cho method có nhiều if lồng nhau?",
    "options": [
      "Method chạy nhanh hơn hẳn",
      "Bớt được tham số của method",
      "Bớt tầng if lồng nhau",
      "Không phải kiểm điều kiện nữa"
    ],
    "answer": 3,
    "explain": "Guard clause xử lý trường hợp đặc biệt rồi return ngay, nên phần còn lại không phải lồng trong if."
  },
  {
    "prompt": "Comment nào đáng giữ lại?",
    "options": [
      "// cộng 1 vào biến đếm i",
      "// biến x là tổng tiền của đơn hàng",
      "// lặp qua danh sách sản phẩm",
      "// hoá đơn dưới 200.000 miễn VAT"
    ],
    "answer": 4,
    "explain": "Comment tốt giải thích lý do mà code không tự nói được, như một quy định thuế. Ba comment kia chỉ nhắc lại code, hoặc chữa cho tên dở."
  }
]
```
