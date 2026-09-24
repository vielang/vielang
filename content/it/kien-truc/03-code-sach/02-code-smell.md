---
title: Code smell
minutes: 5
---

Code chạy đúng chưa chắc đã dễ sửa. Có những dấu hiệu báo trước lần sửa sau
sẽ vất vả: cùng một con số chép ở ba nơi, method dài hai trăm dòng. Nhận ra
chúng sớm thì sửa rẻ hơn nhiều so với lúc chúng đã gây lỗi.

## Khái niệm

👃 **Code smell**: dấu hiệu trong code cho thấy code khó hiểu, khó sửa, dù chưa chắc đã là lỗi.

| Smell | Dấu hiệu | Cách sửa thường dùng |
|---|---|---|
| Trùng lặp | cùng một công thức chép ở nhiều nơi | tách thành một method |
| Method dài | phải cuộn mới đọc hết một method | tách thành nhiều method nhỏ có tên |
| Số trần (magic number) | `0.1m`, `500000` nằm giữa code, không có tên | đặt thành hằng có tên |
| Danh sách tham số dài | method nhận năm, sáu tham số | gom thành một class |

## Ví dụ

Tổng tiền có VAT 10% được tính ở hai nơi:

```csharp
Console.WriteLine(CartTotal(100000m));
Console.WriteLine(InvoiceTotal(100000m));

decimal CartTotal(decimal subtotal)
{
    return subtotal * 1.1m;
}

decimal InvoiceTotal(decimal subtotal)
{
    return subtotal * 1.1m;
}
```

Có hai smell cùng lúc: `1.1m` là số trần, không cho biết nó là gì, và công
thức bị trùng ở hai method. Sửa bằng một hằng và một method dùng chung:

```csharp
const decimal VatRate = 0.1m;

decimal WithVat(decimal subtotal)
{
    return subtotal * (1 + VatRate);
}

decimal CartTotal(decimal subtotal)
{
    return WithVat(subtotal);
}

decimal InvoiceTotal(decimal subtotal)
{
    return WithVat(subtotal);
}
```

- `const` khai báo hằng số: gán một lần khi viết code, không đổi được lúc chạy.
- Tên `VatRate` nói rõ con số là gì. VAT đổi thì sửa đúng một dòng.
- Hai method gọi chung `WithVat`, công thức chỉ còn một chỗ.

## Thử ngay

Nhà nước giảm VAT còn 8%. Với bản **chưa sửa**, lập trình viên chỉ nhớ đổi
`CartTotal` thành `subtotal * 1.08m`, quên `InvoiceTotal`. Chạy lại.

**Đoán trước khi chạy:** số tiền ở giỏ hàng và trên hoá đơn có khớp nhau
không?

<details>
<summary>Xem kết quả</summary>

```text
108000.00
110000.0
```

Không khớp: khách thấy 108.000 trong giỏ nhưng hoá đơn ghi 110.000. Công thức
bị chép ở hai nơi thì sớm muộn sẽ có lúc sửa sót một nơi. Với bản đã sửa, chỉ
cần đổi `VatRate` là cả hai cùng đúng.

</details>

## Lỗi hay gặp

**Sửa mọi smell cùng một lúc.** Viết lại cả file trong một lần thì khó biết
thay đổi nào làm hỏng gì. Mỗi lần sửa một smell, chạy test, rồi commit.

```text
# SAI — một commit viết lại cả OrderService
git commit -am "Dọn code"
```

```text
# ĐÚNG — mỗi commit một bước nhỏ, test xanh sau mỗi bước
git commit -am "Đặt hằng VatRate"
git commit -am "Tách method WithVat"
```

## Tóm tắt

- Code smell là dấu hiệu code khó sửa, chưa chắc là lỗi.
- Bốn smell hay gặp: trùng lặp, method dài, số trần, tham số dài.
- Số trần thay bằng `const` có tên. Công thức trùng gom về một method.
- Sửa từng smell một, test và commit sau mỗi bước.

```quiz
[
  {
    "prompt": "if (total > 500000) xuất hiện ở năm file khác nhau. Smell nào, sửa thế nào?",
    "options": [
      "Số trần, trùng lặp: đặt hằng dùng chung",
      "Method dài: tách thành method nhỏ",
      "Không có smell vì mỗi file chỉ một dòng",
      "Tham số dài: gom thành class"
    ],
    "answer": 1,
    "explain": "500000 không nói nó là gì và bị chép ở năm nơi. Một hằng có tên như FreeShippingThreshold giải quyết cả hai."
  },
  {
    "prompt": "Method CreateOrder(name, phone, email, street, city, district) có smell gì?",
    "options": [
      "Tên method không rõ ý định",
      "Danh sách tham số dài",
      "Trùng lặp với method khác",
      "Không có smell, tên đều rõ"
    ],
    "answer": 2,
    "explain": "Sáu tham số là quá dài. street, city, district luôn đi cùng nhau, gom thành class Address cho gọn và rõ nghĩa."
  },
  {
    "prompt": "const decimal VatRate = 0.1m; Có đổi được VatRate lúc chương trình đang chạy không?",
    "options": [
      "Có, gán lại như biến thường",
      "Có, nếu khai báo thêm static",
      "Không đổi được",
      "Chỉ đổi được trong constructor"
    ],
    "answer": 3,
    "explain": "Hằng const cố định từ lúc viết code. Muốn đổi thì sửa code và build lại."
  }
]
```
