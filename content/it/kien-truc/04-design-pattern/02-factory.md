---
title: Factory
minutes: 5
---

Khách chọn cách giao trên trang đặt hàng, API nhận được một chuỗi như
`"express"`. Cần một chỗ đổi chuỗi đó thành object `ExpressShipping`. Nếu mỗi
controller tự làm bằng `if` riêng, thêm cách giao mới lại phải sửa khắp nơi.
Factory gom việc tạo object về một chỗ.

## Khái niệm

🏭 **Factory**: class hoặc method chuyên tạo object, nhận một lựa chọn và trả về object phù hợp dưới kiểu interface.

Strategy trả lời "có những cách làm nào". Factory trả lời "với lựa chọn này
thì dùng cách nào". Nơi dùng chỉ đưa lựa chọn, không phải biết class cụ thể.

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

- `Create` là `static`, gọi thẳng qua tên class như bài static của khoá C#
  Core.
- Kiểu trả về là interface, nơi gọi không cần biết `ExpressShipping`.
- `switch` chỉ nằm ở đây. Thêm cách giao mới: thêm class và thêm một `case`.
- Gặp chuỗi lạ thì ném `ArgumentException` như bài Exception, không lặng lẽ
  dùng một cách giao mặc định.

Trong controller, chuỗi `method` khách gửi lên đi qua factory rồi vào
`Checkout`:

```csharp
var checkout = new Checkout(
    ShippingFactory.Create(method));
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

`default` của `switch` bắt mọi chuỗi không khớp. Nhờ ném lỗi, API báo sai
ngay thay vì tính phí cho một cách giao không tồn tại.

</details>

## Lỗi hay gặp

**Gọi `new` class cụ thể rải rác trong controller.** Mỗi controller tự chọn
class theo chuỗi, nên thêm cách giao mới phải sửa từng nơi.

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
    "prompt": "Controller tính phí cho cách giao khách chọn qua ShippingFactory. Controller cần biết những kiểu nào?",
    "options": [
      "Cả ba class cách giao cụ thể",
      "ShippingFactory và IShippingStrategy",
      "Class cụ thể ứng với lựa chọn",
      "StandardShipping để làm mặc định"
    ],
    "answer": 2,
    "explain": "Factory trả về interface, nên controller gọi Fee mà không cần biết class cụ thể nào."
  },
  {
    "prompt": "Thêm cách giao \"cod\" với cặp Strategy và Factory. Cần sửa những đâu?",
    "options": [
      "Mọi controller có chọn cách giao",
      "Checkout và IShippingStrategy",
      "Class CodShipping và một case",
      "Chỉ cần thêm class CodShipping"
    ],
    "answer": 3,
    "explain": "Chỉ factory biết ánh xạ từ chuỗi sang class, nên cần thêm class mới và một case trong factory. Thiếu case thì \"cod\" rơi vào default."
  },
  {
    "prompt": "Khi chuỗi không khớp case nào, vì sao nên ném exception thay vì trả StandardShipping?",
    "options": [
      "Vì default của switch bắt buộc throw",
      "Vì return ở default là lỗi compile",
      "Để API trả kết quả nhanh hơn",
      "Để dữ liệu sai lộ ra ngay"
    ],
    "answer": 4,
    "explain": "Trả cách giao mặc định sẽ lặng lẽ tính sai phí và che mất dữ liệu sai do client gửi lên. Ném exception giúp phát hiện lỗi sớm."
  }
]
```
