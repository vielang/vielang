---
title: Factory
minutes: 5
---

Khách chọn cách giao trên trang đặt hàng, API nhận về một chuỗi như
`"express"`. Phải có một chỗ đổi chuỗi đó thành đúng object `ExpressShipping`.
Nếu mỗi controller tự làm việc này bằng `if` riêng, thêm cách giao mới lại
phải sửa khắp nơi. Factory gom việc tạo object về một chỗ.

## Khái niệm

🏭 **Factory**: class hoặc method chuyên tạo object, nhận vào một lựa chọn và trả về object phù hợp dưới dạng interface, để nơi dùng không phải biết class cụ thể.

Strategy trả lời "có những cách làm nào". Factory trả lời "với lựa chọn này
thì dùng cách nào". Giống `Main` ở khoá WinForms, factory là chỗ duy nhất gọi
`new` cho các class cụ thể.

## Ví dụ

Dùng lại `IShippingStrategy` và ba class của bài Strategy:

```csharp
IShippingStrategy shipping =
    ShippingFactory.Create("express");
Console.WriteLine(shipping.Fee(200000m));   // 50000

public static class ShippingFactory
{
    public static IShippingStrategy Create(
        string method)
    {
        switch (method)
        {
            case "standard":
                return new StandardShipping();
            case "express":
                return new ExpressShipping();
            case "pickup":
                return new PickupShipping();
            default:
                throw new ArgumentException(
                    "Không có cách giao: " + method);
        }
    }
}
```

- `Create` là `static`, gọi thẳng qua tên class như bài static ở khoá C#
  Core.
- Kiểu trả về là interface, nơi gọi không cần biết `ExpressShipping`.
- `switch` chỉ nằm ở đây. Thêm cách giao mới: thêm class và thêm một `case`.
- Chuỗi lạ thì ném `ArgumentException` ngay, như bài Exception, thay vì lặng
  lẽ dùng một cách mặc định.

Trong controller, lựa chọn của khách đi qua factory rồi vào `Checkout`:

```csharp
var checkout = new Checkout(
    ShippingFactory.Create(request.ShippingMethod));
```

## Thử ngay

Khách gửi lên một cách giao không có trong danh sách:

```csharp
try
{
    ShippingFactory.Create("bay");
}
catch (ArgumentException e)
{
    Console.WriteLine(e.Message);
}
```

**Đoán trước khi chạy:** in ra gì?

<details>
<summary>Xem kết quả</summary>

```text
Không có cách giao: bay
```

`default` của `switch` bắt mọi chuỗi không khớp. Ném lỗi rõ ràng như vậy thì
API trả về lỗi ngay, không tính nhầm phí cho một cách giao không tồn tại.

</details>

## Lỗi hay gặp

**Gọi `new` class cụ thể rải rác trong controller.** Mỗi controller tự quyết
định class nào ứng với chuỗi nào, thêm cách giao mới phải sửa từng nơi.

```csharp
// SAI — controller tự chọn class cụ thể
IShippingStrategy shipping;
if (method == "express")
{
    shipping = new ExpressShipping();
}
else
{
    shipping = new StandardShipping();
}
```

```csharp
// ĐÚNG — hỏi factory, không tự chọn class
IShippingStrategy shipping =
    ShippingFactory.Create(method);
```

## Tóm tắt

- Factory gom việc tạo object về một chỗ, trả về interface.
- Nơi dùng chỉ đưa lựa chọn, không gọi `new` class cụ thể.
- Lựa chọn không hợp lệ thì ném exception rõ ràng.
- Hay đi cùng Strategy: Strategy định nghĩa các cách làm, Factory chọn cách.

```quiz
[
  {
    "prompt": "ShippingFactory.Create(\"pickup\") trả về kiểu gì ở chỗ gọi?",
    "options": [
      "IShippingStrategy",
      "PickupShipping, và nơi gọi phải biết class đó",
      "string",
      "ShippingFactory"
    ],
    "answer": 1,
    "explain": "Factory trả về interface, nơi gọi dùng Fee mà không cần biết class cụ thể."
  },
  {
    "prompt": "Thêm cách giao \"cod\" với cặp Strategy và Factory. Cần sửa những đâu?",
    "options": [
      "Mọi controller",
      "Checkout",
      "Interface IShippingStrategy",
      "Thêm class CodShipping và một case trong factory"
    ],
    "answer": 4,
    "explain": "Chỉ factory biết ánh xạ từ chuỗi sang class, nên chỉ cần thêm class mới và một case."
  },
  {
    "prompt": "Khi chuỗi không khớp case nào, vì sao nên ném exception thay vì trả StandardShipping?",
    "options": [
      "Vì switch bắt buộc phải throw",
      "Vì StandardShipping bị lỗi",
      "Để lỗi dữ liệu lộ ra ngay, không lặng lẽ tính sai phí",
      "Để chương trình chạy nhanh hơn"
    ],
    "answer": 3,
    "explain": "Trả mặc định che mất lỗi phía client. Ném exception giúp phát hiện sai sót sớm."
  }
]
```
