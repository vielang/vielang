---
title: Refactor an toàn
minutes: 6
---

Hai bài trước chỉ ra chỗ nên sửa. Nhưng sửa code đang chạy rất dễ gây lỗi: gõ
nhầm `>=` thành `>` là đơn đúng 500.000 bị tính phí ship. Bài này sửa từng
bước nhỏ, sau mỗi bước chạy test để chắc hành vi không đổi.

## Khái niệm

🪛 **Refactor**: thay đổi cấu trúc bên trong của code để dễ đọc, dễ sửa hơn mà không làm thay đổi hành vi bên ngoài.

Quy trình một bước refactor:

| Bước | Làm gì |
|---|---|
| 1 | Chạy test, bảo đảm tất cả đang xanh |
| 2 | Sửa một chỗ nhỏ: đổi tên, đặt hằng, tách method |
| 3 | Chạy lại test. Đỏ thì hoàn tác bằng `git restore .` |
| 4 | Xanh thì commit, rồi quay lại bước 2 |

Chưa có test thì viết test trước, như chương Test, rồi mới refactor.

## Ví dụ

Đưa `ShippingFee` của bài Đặt tên và method ngắn vào class
`ShippingCalculator` trong `ShopApi`. Test trong `ShopApi.Tests` chốt hành
vi hiện tại, gồm cả mốc 500.000:

```csharp
using Xunit;

public class ShippingCalculator
{
    public decimal Fee(
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
}

public class ShippingCalculatorTests
{
    [Theory]
    [InlineData(500000, false, "Hà Nội", 0)]
    [InlineData(200000, true, "Đà Nẵng", 0)]
    [InlineData(200000, false, "Hà Nội", 20000)]
    [InlineData(200000, false, "Đà Nẵng", 35000)]
    public void Fee_ReturnsExpected(
        decimal orderTotal, bool isMember,
        string city, decimal expected)
    {
        var calculator = new ShippingCalculator();

        decimal fee =
            calculator.Fee(orderTotal, isMember, city);

        Assert.Equal(expected, fee);
    }
}
```

Bước refactor đầu tiên: số trần `500000` thành hằng có tên, như bài Code
smell. Trong class, hằng khai báo bằng `private const`:

```csharp
public class ShippingCalculator
{
    private const decimal FreeShippingThreshold =
        500000;

    public decimal Fee(
        decimal orderTotal, bool isMember, string city)
    {
        if (orderTotal >= FreeShippingThreshold)
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
}
```

- Chạy `dotnet test`: vẫn xanh, nên bước này không đổi hành vi. Commit.
- Mỗi bước tiếp theo, như đặt hằng cho 20000 và 35000, cũng đi đủ vòng: sửa,
  test, commit.

## Thử ngay

Giả sử lúc đặt hằng, bạn gõ nhầm `>=` thành `>`:

```csharp
if (orderTotal > FreeShippingThreshold)
{
    return 0;
}
```

Chạy `dotnet test`.

**Đoán trước khi chạy:** test có bắt được không? Dòng dữ liệu nào đỏ?

<details>
<summary>Xem kết quả</summary>

```text
Failed ShippingCalculatorTests.Fee_ReturnsExpected(
  orderTotal: 500000, isMember: False,
  city: "Hà Nội", expected: 0)
Assert.Equal() Failure: Values differ
Expected: 0
Actual:   20000
```

Bắt được, nhờ dòng dữ liệu ở đúng mốc 500.000. Ba dòng còn lại vẫn xanh,
nên nếu thiếu dòng này thì lỗi lọt qua. Sửa lại `>=` thì cả bốn xanh.

</details>

## Lỗi hay gặp

**Vừa refactor vừa thêm tính năng.** Test đỏ thì không biết do refactor làm
hỏng hay do tính năng mới, và commit trộn hai việc rất khó review.

```text
# SAI — một commit vừa dọn code vừa thêm phí hoả tốc
git commit -am "Refactor phí ship + thêm hoả tốc"
```

```text
# ĐÚNG — refactor xong, test xanh, commit; rồi mới thêm tính năng
git commit -am "Đặt hằng FreeShippingThreshold"
git commit -am "Thêm phí giao hoả tốc"
```

## Tóm tắt

- Refactor đổi cấu trúc, giữ nguyên hành vi.
- Có test xanh trước, sửa một bước nhỏ, chạy test, commit.
- Test phải có dữ liệu ở mốc ranh giới thì mới bắt được lỗi `>=` và `>`.
- Không trộn refactor với thêm tính năng trong cùng một commit.

```quiz
[
  {
    "prompt": "Đang refactor, chạy test thấy đỏ. Nên làm gì trước tiên?",
    "options": [
      "Sửa test cho xanh",
      "Hoàn tác bước vừa làm, vì bước đó đã đổi hành vi",
      "Commit rồi sửa sau",
      "Xoá test đỏ"
    ],
    "answer": 2,
    "explain": "Refactor không được đổi hành vi. Bước nhỏ nên hoàn tác rẻ, rồi làm lại cẩn thận hơn."
  },
  {
    "prompt": "Việc nào KHÔNG phải refactor?",
    "options": [
      "Đổi tên biến t thành orderTotal",
      "Tách một đoạn code thành method riêng",
      "Đặt hằng cho số trần",
      "Thêm phí giao hàng hoả tốc"
    ],
    "answer": 4,
    "explain": "Thêm phí hoả tốc làm thay đổi hành vi, đó là thêm tính năng."
  },
  {
    "prompt": "Code chưa có test mà cần refactor. Nên làm gì trước?",
    "options": [
      "Refactor luôn, cẩn thận là được",
      "Chạy app bằng tay một lần",
      "Viết test chốt hành vi hiện tại, rồi mới refactor",
      "Viết lại từ đầu"
    ],
    "answer": 3,
    "explain": "Test là lưới an toàn. Không có test thì không biết refactor có làm đổi hành vi hay không."
  }
]
```
