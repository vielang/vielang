---
title: Theory và nhiều trường hợp
minutes: 5
---

Bài Viết test cho API của khoá ASP.NET Core đã có `PriceCalculator` và vài
test `[Fact]`. Muốn kiểm thêm năm bộ giá và số lượng thì phải chép ra năm
method gần giống hệt nhau. `[Theory]` cho một method test chạy với nhiều bộ dữ
liệu.

## Khái niệm

🧫 **Theory**: method test nhận tham số, được đánh dấu `[Theory]`, chạy một lần cho mỗi bộ dữ liệu `[InlineData]` gắn kèm.

`[Fact]` là một test cố định. `[Theory]` là một khuôn test, mỗi dòng
`[InlineData]` là một test riêng, qua hay đỏ riêng.

## Ví dụ

Dùng lại `PriceCalculator` của bài Viết test cho API: tổng từ 500.000 trở lên
thì giảm 10%.

```csharp
using Xunit;

public class PriceCalculatorTests
{
    [Theory]
    [InlineData(100000, 2, 200000)]
    [InlineData(250000, 2, 450000)]
    [InlineData(300000, 2, 540000)]
    public void Total_ReturnsExpected(
        decimal price, int quantity, decimal expected)
    {
        var calculator = new PriceCalculator();

        decimal total =
            calculator.Total(price, quantity);

        Assert.Equal(expected, total);
    }
}
```

- Ba dòng `[InlineData]` là ba test. Giá trị truyền vào tham số theo đúng thứ
  tự: `price`, `quantity`, `expected`.
- Viết `100000` là số nguyên, xUnit tự đổi sang `decimal` cho tham số
  `price`.
- Dòng thứ hai là mốc ranh giới 500.000, chỗ dễ sai nhất như bài trước đã
  chỉ ra.

Nên chọn dữ liệu theo nhóm: một trường hợp bình thường, các mốc ranh giới, và
trường hợp sai như số lượng 0 (kiểm bằng `Assert.Throws` ở một test riêng).

## Thử ngay

Thêm dòng `[InlineData(100000, 5, 500000)]` vào `Total_ReturnsExpected`:

```csharp
using Xunit;

public class PriceCalculatorTests
{
    [Theory]
    [InlineData(100000, 2, 200000)]
    [InlineData(250000, 2, 450000)]
    [InlineData(300000, 2, 540000)]
    [InlineData(100000, 5, 500000)]
    public void Total_ReturnsExpected(
        decimal price, int quantity, decimal expected)
    {
        var calculator = new PriceCalculator();

        decimal total =
            calculator.Total(price, quantity);

        Assert.Equal(expected, total);
    }
}
```

Chạy `dotnet test`.

**Đoán trước khi chạy:** `Total_ReturnsExpected` giờ là mấy test, và dòng
dữ liệu nào đỏ?

<details>
<summary>Xem kết quả</summary>

```text
PriceCalculatorTests.Total_ReturnsExpected(price: 100000,
  quantity: 5, expected: 500000) [FAIL]
Assert.Equal() Failure: Values differ
Expected: 500000
Actual:   450000

Failed!  - Failed: 1, ...
```

Bốn dòng `[InlineData]` là bốn test, ba qua và một đỏ. Số Passed, Total trên
máy bạn còn cộng thêm các test cũ trong project. Test đỏ ghi rõ bộ dữ liệu
nào hỏng.
100000 × 5 là đúng 500.000 nên được giảm 10%, kỳ vọng 500000 là sai.

</details>

## Lỗi hay gặp

**Viết số `decimal` có hậu tố `m` trong `[InlineData]`.** Tham số của
attribute chỉ nhận hằng số kiểu cơ bản, mà `decimal` không nằm trong số đó.

```csharp
// SAI — lỗi compile CS0182
using Xunit;

public class DiscountTests
{
    [Theory]
    [InlineData(100000m, 2, 200000m)]
    public void Total(
        decimal price, int qty, decimal expected)
    {
    }
}
```

```csharp
// ĐÚNG — viết số nguyên, xUnit tự đổi sang decimal
using Xunit;

public class DiscountTests
{
    [Theory]
    [InlineData(100000, 2, 200000)]
    public void Total(
        decimal price, int qty, decimal expected)
    {
    }
}
```

## Tóm tắt

- `[Theory]` kèm nhiều `[InlineData]`: một method, nhiều test.
- Mỗi dòng dữ liệu qua hay đỏ riêng, báo lỗi kèm bộ dữ liệu hỏng.
- Chọn dữ liệu: trường hợp bình thường, mốc ranh giới, trường hợp sai.
- Tiền trong `[InlineData]` viết số nguyên, không dùng hậu tố `m`.

```quiz
[
  {
    "prompt": "Một [Theory] có 5 dòng [InlineData]. dotnet test đếm bao nhiêu test?",
    "options": [
      "1",
      "5",
      "6",
      "Tuỳ số lần gọi Assert"
    ],
    "answer": 2,
    "explain": "Mỗi dòng InlineData là một test riêng."
  },
  {
    "prompt": "Giảm giá khi tổng từ 500.000 trở lên. Bộ dữ liệu nào quan trọng nhất phải có?",
    "options": [
      "Tổng 100",
      "Tổng 10 triệu",
      "Tổng đúng 500.000 và ngay dưới 500.000",
      "Chỉ cần một bộ bất kỳ"
    ],
    "answer": 3,
    "explain": "Lỗi hay nằm ở mốc ranh giới, như nhầm giữa lớn hơn và lớn hơn hoặc bằng."
  },
  {
    "prompt": "[InlineData(5000m)] báo lỗi compile. Sửa thế nào?",
    "options": [
      "Đổi [Theory] thành [Fact]",
      "Thêm using System.Decimal",
      "Bỏ tham số của method",
      "Viết [InlineData(5000)], xUnit tự đổi sang decimal"
    ],
    "answer": 4,
    "explain": "Attribute không nhận hằng decimal. Viết số nguyên và để tham số của method là decimal."
  }
]
```
