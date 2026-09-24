---
title: Viết test cho API
minutes: 5
---

Bạn sửa công thức giảm giá, gọi thử bằng `curl` thấy đúng. Ba tuần sau, một
người khác sửa code chỗ khác và làm hỏng công thức đó, nhưng không ai phát
hiện. Test tự động giúp chạy lại mọi phép kiểm tra chỉ bằng một lệnh.

## Khái niệm

🧪 **Unit test**: code tự động kiểm tra một phần nhỏ như một method hay một class có chạy đúng không.

✔️ **xUnit**: thư viện test phổ biến của .NET, mỗi method test đánh dấu `[Fact]` và kiểm tra kết quả bằng `Assert`.

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

Tạo project test cạnh thư mục `ShopApi` rồi chạy:

```bash
cd ..
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

- `cd ..` ra khỏi thư mục `ShopApi`, để hai project nằm cạnh nhau.
- `dotnet add ... reference` cho project test dùng được class của `ShopApi`.
- Mỗi test là một method `[Fact]`, tên nói rõ tình huống và kết quả mong
  đợi.
- `Assert.Equal(mong đợi, thực tế)`: hai giá trị khác nhau thì test đỏ.
- `Assert.Throws` kiểm tra method có ném đúng loại exception không.
  `() => ...` là lambda không tham số, bọc lời gọi để `Assert.Throws` tự
  chạy và bắt exception.

## Thử ngay

Thêm class test này vào project `ShopApi.Tests` rồi chạy lại
`dotnet test ShopApi.Tests`:

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

Failed!  - Failed: 1, Passed: 3, Total: 4
```

Đỏ. Tổng đúng bằng 500000 thì điều kiện `>= 500000` vẫn giảm 10%, còn
450000. Passed là 3 vì project mới tạo có sẵn một test mẫu trong
`UnitTest1.cs`.

Test ở mốc ranh giới như thế này bắt được những hiểu nhầm mà thử bằng tay dễ
bỏ qua.

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

```csharp
// ĐÚNG — bản giả trả dữ liệu định sẵn, không đọc file
public interface IReportSource
{
    string Read();
}

public class FakeReportSource : IReportSource
{
    public string Read()
    {
        return "Doanh thu: 500000";
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
    "prompt": "Vừa tạo ShopApi.Tests bằng dotnet new xunit, test gọi new ShippingCalculator() của ShopApi thì build báo không tìm thấy tên này. Thiếu bước nào?",
    "options": [
      "Gắn [Fact] lên ShippingCalculator",
      "Thêm using Xunit vào ShopApi",
      "Đổi ShippingCalculator thành static",
      "Thêm reference tới ShopApi"
    ],
    "answer": 4,
    "explain": "Project test chỉ thấy class của ShopApi sau lệnh dotnet add ShopApi.Tests reference ShopApi."
  },
  {
    "prompt": "Test viết Assert.Equal(total, 200000m), trong đó total là kết quả thực tế. Khi test đỏ, thông báo lỗi sẽ thế nào?",
    "options": [
      "Expected và Actual bị đảo chỗ",
      "Test luôn qua dù kết quả sai",
      "Test không biên dịch được",
      "Thông báo in ra y như viết đúng"
    ],
    "answer": 1,
    "explain": "Assert.Equal nhận giá trị mong đợi trước, thực tế sau. Viết ngược thì test vẫn qua hay đỏ đúng như cũ, nhưng dòng Expected lại in kết quả thực tế nên dễ đọc nhầm."
  }
]
```
