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

`[Fact]` là một test với dữ liệu cố định. Với `[Theory]`, mỗi dòng
`[InlineData]` là một test riêng, qua hay đỏ độc lập với các dòng khác.

## Ví dụ

Dùng lại `PriceCalculator` của bài Viết test cho API: tổng từ 500.000 trở lên
thì giảm 10%. Thêm method `Total_ReturnsExpected` vào class
`PriceCalculatorTests` có sẵn của bài đó:

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

- Ba dòng `[InlineData]` là ba test. Các giá trị được truyền vào tham số
  theo đúng thứ tự: `price`, `quantity`, `expected`.
- `100000` là số nguyên, xUnit tự đổi sang `decimal` cho tham số `price`.
- Dòng thứ hai rơi đúng mốc 500.000, chỗ dễ sai nhất như bài Viết test cho
  API đã chỉ ra.

Nên chọn dữ liệu theo nhóm: trường hợp bình thường, các mốc ranh giới, và
trường hợp sai như số lượng 0 (kiểm bằng `Assert.Throws` ở một test riêng).

## Thử ngay

Thêm dòng `[InlineData(100000, 5, 500000)]` ngay dưới ba dòng `[InlineData]`
của `Total_ReturnsExpected`, rồi chạy `dotnet test`.

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

Bốn dòng `[InlineData]` là bốn test: ba qua, một đỏ. Test đỏ ghi rõ bộ dữ
liệu hỏng. 100000 × 5 đúng bằng 500.000 nên được giảm 10% còn 450000, vậy
kỳ vọng 500000 là sai.

Số test trên máy bạn còn cộng thêm các test cũ trong project.

</details>

## Lỗi hay gặp

**Viết số `decimal` có hậu tố `m` trong `[InlineData]`.** Tham số của
attribute chỉ nhận hằng số của vài kiểu cơ bản như `int`, `string`, và
`decimal` không thuộc số đó.

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
      "5",
      "1",
      "6",
      "Tuỳ số lần gọi Assert"
    ],
    "answer": 1,
    "explain": "Mỗi dòng InlineData là một test riêng."
  },
  {
    "prompt": "Một [Theory] có 4 dòng [InlineData], dòng thứ hai đỏ. Hai dòng sau nó thì sao?",
    "options": [
      "Không chạy, vì test dừng ở dòng đỏ",
      "Vẫn chạy và qua hay đỏ tuỳ dữ liệu",
      "Cũng bị tính là đỏ theo dòng thứ hai",
      "Chỉ chạy khi sửa xong dòng thứ hai"
    ],
    "answer": 2,
    "explain": "Mỗi dòng InlineData là một test độc lập. Dòng này đỏ không làm các dòng khác dừng hay đỏ theo."
  },
  {
    "prompt": "[InlineData(5000m)] báo lỗi compile. Sửa thế nào?",
    "options": [
      "Đổi [Theory] thành [Fact]",
      "Viết [InlineData((decimal)5000)]",
      "Viết [InlineData(5000)]",
      "Viết [InlineData(\"5000m\")]"
    ],
    "answer": 3,
    "explain": "Attribute không nhận hằng decimal, kể cả khi ép kiểu. Viết số nguyên, xUnit tự đổi sang decimal cho tham số của method."
  }
]
```
