---
title: Code smell
minutes: 5
---

Code chạy đúng chưa chắc đã dễ sửa. Có những dấu hiệu nhìn là biết lần sửa
sau sẽ khổ: cùng một con số chép ở ba nơi, method dài hai trăm dòng. Nhận ra
chúng sớm thì sửa rẻ hơn nhiều so với lúc chúng đã gây lỗi.

## Khái niệm

👃 **Code smell**: dấu hiệu trên bề mặt code cho thấy thiết kế có thể có vấn đề, chưa chắc là lỗi nhưng làm code khó hiểu, khó sửa.

| Smell | Dấu hiệu | Cách sửa thường dùng |
|---|---|---|
| Trùng lặp | cùng một công thức chép ở nhiều nơi | tách thành một method |
| Method dài | phải cuộn mới đọc hết một method | tách thành nhiều method nhỏ có tên |
| Số trần (magic number) | `0.1m`, `500000` đứng trơ trọi giữa code | đặt thành hằng có tên |
| Danh sách tham số dài | method nhận năm, sáu tham số | gom thành một class |

## Ví dụ

Tiền VAT 10% được tính ở hai nơi:

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

Có hai smell cùng lúc: `1.1m` là số trần không nói nó là gì, và công thức bị
trùng ở hai method. Sửa bằng một hằng và một method dùng chung:

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

Smell cũng không phải luật cứng. Method dài mà chỉ là một danh sách cấu hình
dễ đọc thì không cần tách.

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
      "Method dài, tách method",
      "Không có vấn đề gì",
      "Tham số dài, gom class",
      "Số trần và trùng lặp: đặt hằng FreeShippingThreshold dùng chung"
    ],
    "answer": 4,
    "explain": "500000 không nói nó là gì và bị chép ở năm nơi. Một hằng có tên giải quyết cả hai."
  },
  {
    "prompt": "Method CreateOrder(name, phone, email, street, city, district) có smell gì?",
    "options": [
      "Danh sách tham số dài, nên gom địa chỉ thành class Address",
      "Tên method dở",
      "Số trần",
      "Không có smell"
    ],
    "answer": 1,
    "explain": "Các tham số street, city, district luôn đi cùng nhau, gom lại thành một class cho gọn và rõ nghĩa."
  },
  {
    "prompt": "const decimal VatRate = 0.1m; Có đổi được VatRate lúc chương trình đang chạy không?",
    "options": [
      "Có, gán lại như biến thường",
      "Có, nếu dùng static",
      "Không, const cố định từ lúc viết code",
      "Chỉ đổi được trong constructor"
    ],
    "answer": 3,
    "explain": "Hằng const được gán một lần trong code. Muốn đổi thì sửa code và build lại."
  }
]
```
