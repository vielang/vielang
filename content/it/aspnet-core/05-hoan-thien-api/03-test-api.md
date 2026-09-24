---
title: Viết test cho API
minutes: 5
---

Bạn sửa công thức giảm giá, gọi thử bằng `curl` thấy đúng. Ba tuần sau, một
người khác sửa chỗ khác làm hỏng công thức đó, không ai phát hiện. Test tự
động chạy lại mọi lần kiểm tra chỉ bằng một lệnh.

## Khái niệm

🧪 **Unit test**: code tự động kiểm tra một phần nhỏ như một method hay một class có chạy đúng không.

✔️ **xUnit**: thư viện test phổ biến của .NET. Mỗi method test đánh dấu `[Fact]`, kiểm tra kết quả bằng `Assert`.

Mỗi test gồm ba bước:

| Bước | Làm gì |
|---|---|
| Arrange | chuẩn bị object và dữ liệu |
| Act | gọi method cần test |
| Assert | kiểm tra kết quả đúng như mong đợi |

## Ví dụ

Class cần test nằm trong project `ShopApi`:

```csharp
public class PriceCalculator
{
    public decimal Total(decimal price, int quantity)
    {
        if (quantity <= 0)
        {
            throw new ArgumentException(
                "Số lượng phải > 0");
        }

        decimal total = price * quantity;
        if (total >= 500000)
        {
            total = total * 90 / 100;
        }
        return total;
    }
}
```

Tạo project test và chạy:

```bash
dotnet new xunit -o ShopApi.Tests
dotnet add ShopApi.Tests reference ShopApi
dotnet test ShopApi.Tests
```

Test trong `ShopApi.Tests`:

```csharp
using Xunit;

public class PriceCalculatorTests
{
    [Fact]
    public void Total_Under500k_NoDiscount()
    {
        var calculator = new PriceCalculator();

        decimal total = calculator.Total(100000m, 2);

        Assert.Equal(200000m, total);
    }

    [Fact]
    public void Total_ZeroQuantity_Throws()
    {
        var calculator = new PriceCalculator();

        Assert.Throws<ArgumentException>(
            () => calculator.Total(100000m, 0));
    }
}
```

- `dotnet add ... reference` cho project test dùng được class của `ShopApi`.
- Mỗi test là một method `[Fact]`, tên nói rõ tình huống và kết quả mong
  đợi.
- `Assert.Equal(mong đợi, thực tế)` sai là test đỏ.
- `Assert.Throws` kiểm tra method có ném đúng loại exception không.

## Thử ngay

Thêm class test này vào project `ShopApi.Tests` rồi chạy `dotnet test`:

```csharp
using Xunit;

public class BoundaryTests
{
    [Fact]
    public void Total_Exactly500k()
    {
        var calculator = new PriceCalculator();

        decimal total = calculator.Total(250000m, 2);

        Assert.Equal(500000m, total);
    }
}
```

**Đoán trước khi chạy:** test mới qua hay đỏ?

<details>
<summary>Xem kết quả</summary>

```text
Assert.Equal() Failure: Values differ
Expected: 500000
Actual:   450000

Failed!  - Failed: 1, Passed: 2, Total: 3
```

Đỏ. Tổng đúng bằng 500000 thì điều kiện `>= 500000` vẫn giảm 10%, còn
450000. Test ở mốc ranh giới như thế này bắt được những hiểu nhầm mà thử bằng
tay dễ bỏ qua.

</details>

## Lỗi hay gặp

**Test phụ thuộc database hay API thật.** Test chạy chậm, lúc qua lúc đỏ tuỳ
mạng. Class cần database thì nhận interface qua constructor, lúc test truyền
vào một bản giả, như bài DIP ở khoá OOP.

```csharp
// SAI — test phụ thuộc file thật trên máy
using Xunit;

public class ReportTests
{
    [Fact]
    public void Read_Report()
    {
        string text =
            File.ReadAllText("C:/data/report.txt");
        Assert.NotEmpty(text);
    }
}
```

**Một test kiểm tra quá nhiều thứ.** Test đỏ mà không biết chỗ nào hỏng. Mỗi
test nên kiểm tra một tình huống.

## Tóm tắt

- Unit test kiểm tra tự động một phần nhỏ của code.
- Viết theo ba bước Arrange, Act, Assert, đánh dấu `[Fact]`.
- `dotnet test` chạy toàn bộ test chỉ bằng một lệnh.
- Test cả mốc ranh giới, và thay database hay API bằng bản giả.

```quiz
[
  {
    "prompt": "Trong test, dòng var result = calculator.Total(100m, 3); thuộc bước nào?",
    "options": [
      "Arrange",
      "Assert",
      "Act",
      "Không thuộc bước nào"
    ],
    "answer": 3,
    "explain": "Act là bước gọi method cần test. Arrange chuẩn bị, Assert kiểm tra kết quả."
  },
  {
    "prompt": "Phí ship miễn phí khi đơn từ 300000đ. Nên có test cho giá trị nào để bắt lỗi ranh giới?",
    "options": [
      "Đúng 300000, và ngay dưới là 299999",
      "Chỉ 1000000",
      "Chỉ 0",
      "Không cần, thử bằng tay là đủ"
    ],
    "answer": 1,
    "explain": "Lỗi hay nằm ở mốc ranh giới như >= và >. Test ngay tại mốc và ngay dưới mốc sẽ bắt được."
  },
  {
    "prompt": "OrderService nhận IPaymentGateway qua constructor. Muốn test OrderService mà không gọi ngân hàng thật thì làm gì?",
    "options": [
      "Gọi ngân hàng thật với số tiền nhỏ",
      "Không test được",
      "Sửa OrderService để bỏ qua thanh toán khi test",
      "Truyền vào một class giả implement IPaymentGateway"
    ],
    "answer": 4,
    "explain": "Nhờ nhận interface qua constructor, lúc test chỉ cần truyền bản giả trả kết quả định sẵn."
  }
]
```
