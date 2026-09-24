---
title: Strategy
minutes: 5
---

Cửa hàng có ba cách giao: tiêu chuẩn, hoả tốc và khách tự đến lấy. Mỗi cách
tính phí một kiểu. Nếu viết hết vào một method, dùng `if` theo tên cách giao,
thì mỗi lần thêm cách giao lại phải sửa method đó. Strategy tách mỗi cách
tính thành một class riêng.

## Khái niệm

📘 **Design pattern**: cách giải có tên gọi chung cho một vấn đề thiết kế hay gặp.

♟️ **Strategy**: mỗi cách làm một việc nằm trong một class riêng, các class cùng implement một interface, và nơi dùng nhận cách làm qua interface đó.

Strategy là một cách áp dụng OCP của khoá OOP: thêm cách giao là thêm class,
không sửa code cũ. `IDiscount` ở bài OCP và `IShipping` ở bài Composition
đều là Strategy.

`StandardShipping` dưới đây tính đơn giản hơn `ShippingCalculator` của bài
Refactor an toàn: từ 500.000 thì miễn phí, còn lại 20.000 cho mọi nơi.

## Ví dụ

```csharp
var checkout = new Checkout(new StandardShipping());
Console.WriteLine(checkout.Total(200000m));   // 220000

public interface IShippingStrategy
{
    decimal Fee(decimal orderTotal);
}

public class StandardShipping : IShippingStrategy
{
    public decimal Fee(decimal orderTotal)
    {
        if (orderTotal >= 500000)
        {
            return 0;
        }
        return 20000;
    }
}

public class ExpressShipping : IShippingStrategy
{
    public decimal Fee(decimal orderTotal)
    {
        return 50000;
    }
}

public class Checkout
{
    private readonly IShippingStrategy _shipping;

    public Checkout(IShippingStrategy shipping)
    {
        _shipping = shipping;
    }

    public decimal Total(decimal subtotal)
    {
        return subtotal + _shipping.Fee(subtotal);
    }
}
```

- `IShippingStrategy` đại diện cho việc tính phí giao hàng, mỗi class là một
  cách tính.
- `Checkout` chỉ biết interface và nhận cách tính qua constructor, như bài
  DIP và dependency injection.
- Đổi cách giao là đổi object truyền vào, `Checkout` không đổi dòng nào.

```mermaid Checkout chỉ biết interface, mỗi cách giao là một class
classDiagram
    class IShippingStrategy {
        <<interface>>
        Fee(orderTotal)
    }
    IShippingStrategy <|.. StandardShipping
    IShippingStrategy <|.. ExpressShipping
    Checkout --> IShippingStrategy
```

## Thử ngay

Thêm cách "khách tự đến lấy", miễn phí, rồi dùng thử:

```csharp
Console.WriteLine(
    new Checkout(new ExpressShipping()).Total(200000m));
Console.WriteLine(
    new Checkout(new PickupShipping()).Total(200000m));

public class PickupShipping : IShippingStrategy
{
    public decimal Fee(decimal orderTotal)
    {
        return 0;
    }
}
```

**Đoán trước khi chạy:** hai dòng in ra bao nhiêu? Phải sửa `Checkout` không?

<details>
<summary>Xem kết quả</summary>

```text
250000
200000
```

Hoả tốc cộng 50.000, tự lấy cộng 0. Thêm `PickupShipping` là thêm một class
mới, `Checkout` và hai cách giao cũ giữ nguyên.

</details>

## Lỗi hay gặp

**Rải `if` theo tên cách giao khắp code.** Thêm cách giao mới phải tìm và sửa
mọi chỗ có các `if` này, sót một chỗ là tính sai.

```csharp
// SAI — thêm cách giao mới phải sửa method này
decimal Fee(string method, decimal orderTotal)
{
    if (method == "express")
    {
        return 50000;
    }
    if (orderTotal >= 500000)
    {
        return 0;
    }
    return 20000;
}
```

```csharp
// ĐÚNG — nơi dùng chỉ nhận interface
public class Checkout
{
    private readonly IShippingStrategy _shipping;

    public Checkout(IShippingStrategy shipping)
    {
        _shipping = shipping;
    }
}
```

## Tóm tắt

- Design pattern là lời giải có tên cho vấn đề thiết kế hay gặp.
- Strategy: mỗi cách làm một class, chung một interface.
- Nơi dùng nhận interface qua constructor, đổi cách làm không phải sửa nó.
- Thêm cách mới là thêm class, đúng tinh thần OCP.

```quiz
[
  {
    "prompt": "Cửa hàng thêm cách giao \"giao trong 2 giờ\". Với Strategy, cần làm gì?",
    "options": [
      "Sửa Checkout thêm một nhánh if",
      "Sửa StandardShipping cho cả hai",
      "Thêm class TwoHourShipping",
      "Thêm method vào IShippingStrategy"
    ],
    "answer": 3,
    "explain": "Thêm cách làm là thêm class mới implement IShippingStrategy. Code cũ giữ nguyên, đúng OCP."
  },
  {
    "prompt": "Checkout biết gì về cách tính phí giao hàng?",
    "options": [
      "Biết cả ba class cụ thể",
      "Biết tên cách giao dạng chuỗi",
      "Biết class nào đang được dùng",
      "Chỉ biết IShippingStrategy"
    ],
    "answer": 4,
    "explain": "Checkout chỉ gọi Fee qua interface, nên cách tính nào truyền vào cũng dùng được."
  },
  {
    "prompt": "Nguyên lý SOLID nào đứng sau Strategy rõ nhất?",
    "options": [
      "OCP",
      "LSP",
      "ISP",
      "SRP"
    ],
    "answer": 1,
    "explain": "Mở rộng bằng class mới mà không sửa code cũ chính là OCP."
  }
]
```
