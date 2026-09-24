---
title: Test đáng tin
minutes: 5
---

Một test hôm qua qua, hôm nay đỏ, mà không ai sửa code. Chạy lại thì qua. Test
như vậy làm cả nhóm mất lòng tin: thấy đỏ cũng không biết là lỗi thật hay test
"dở chứng". Bài này chỉ ra các nguyên nhân hay gặp nhất và cách tránh.

## Khái niệm

🕰️ **Test ổn định (deterministic test)**: test cho cùng một kết quả mỗi lần chạy, bất kể chạy ngày nào, trên máy nào, trước hay sau test khác.

Ba nguồn làm test mất ổn định hay gặp nhất: phụ thuộc giờ hệ thống như
`DateTime.Now`, phụ thuộc thứ tự chạy của các test, và phụ thuộc database hay
mạng (bài trước đã thay bằng fake).

## Ví dụ

Giảm 10% vào thứ bảy, chủ nhật. Method nhận ngày qua tham số thay vì tự đọc
giờ hệ thống:

```csharp
using Xunit;

public class Promo
{
    public decimal DiscountPercent(DateTime day)
    {
        if (day.DayOfWeek == DayOfWeek.Saturday
            || day.DayOfWeek == DayOfWeek.Sunday)
        {
            return 10;
        }
        return 0;
    }
}

public class PromoTests
{
    [Theory]
    [InlineData(2026, 9, 26, 10)]   // thứ bảy
    [InlineData(2026, 9, 28, 0)]    // thứ hai
    public void DiscountPercent_ByDay(
        int year, int month, int day, int expected)
    {
        var promo = new Promo();

        decimal percent = promo.DiscountPercent(
            new DateTime(year, month, day));

        Assert.Equal(expected, percent);
    }
}
```

- Test tự chọn ngày cố định, nên chạy hôm nào cũng cho cùng kết quả.
- `DateTime` không viết được trong `[InlineData]`, nên truyền năm, tháng,
  ngày rồi tạo `DateTime` trong test.
- Tên test theo mẫu `Method_TìnhHuống` hoặc `Method_TìnhHuống_KếtQuả`, như bài
  Viết test cho API. Đọc tên là biết test kiểm gì.
- Mỗi test chỉ kiểm một hành vi. Đỏ thì biết ngay hành vi nào hỏng.

## Thử ngay

xUnit chạy các test theo thứ tự không cố định. Thêm class này rồi chạy
`dotnet test`:

```csharp
using Xunit;

public class CounterTests
{
    private int _count = 0;

    [Fact]
    public void First()
    {
        _count++;
        Assert.Equal(1, _count);
    }

    [Fact]
    public void Second()
    {
        _count++;
        Assert.Equal(1, _count);
    }
}
```

**Đoán trước khi chạy:** hai test cùng tăng `_count`. Test chạy sau sẽ thấy
`_count` bằng 2 và đỏ, hay cả hai đều qua?

<details>
<summary>Xem kết quả</summary>

```text
Passed!  - Failed: 0, ...
```

Cả hai qua. xUnit tạo một object `CounterTests` mới cho mỗi test, nên mỗi test
có `_count` riêng, bắt đầu từ 0. Nhờ vậy các test không dính nhau qua field.
Đổi `_count` thành `static` thì hai test dùng chung một biến, và test chạy sau
sẽ đỏ.

</details>

## Lỗi hay gặp

**Đọc `DateTime.Now` bên trong method cần test.** Test chỉ qua vào cuối tuần,
ngày thường thì đỏ, dù code không đổi.

```csharp
// SAI — kết quả phụ thuộc hôm nay là thứ mấy
public class Promo
{
    public decimal DiscountPercent()
    {
        DayOfWeek today = DateTime.Now.DayOfWeek;
        if (today == DayOfWeek.Saturday
            || today == DayOfWeek.Sunday)
        {
            return 10;
        }
        return 0;
    }
}
```

```csharp
// ĐÚNG — nhận ngày qua tham số
public class Promo
{
    public decimal DiscountPercent(DateTime day)
    {
        if (day.DayOfWeek == DayOfWeek.Saturday
            || day.DayOfWeek == DayOfWeek.Sunday)
        {
            return 10;
        }
        return 0;
    }
}
```

## Tóm tắt

- Test đáng tin cho cùng kết quả mỗi lần chạy.
- Không đọc `DateTime.Now` trong code cần test, truyền ngày vào qua tham số.
- xUnit tạo object test mới cho mỗi test. Tránh `static` dùng chung giữa các
  test.
- Tên test nói rõ tình huống, mỗi test kiểm một hành vi.

```quiz
[
  {
    "prompt": "Test tính phí ship đêm khuya qua lúc 23 giờ nhưng đỏ lúc 9 giờ sáng. Nguyên nhân nhiều khả năng nhất?",
    "options": [
      "Máy chạy test yếu",
      "Code đọc giờ hệ thống bên trong method cần test",
      "xUnit bị lỗi",
      "Test thiếu [Fact]"
    ],
    "answer": 2,
    "explain": "Kết quả phụ thuộc giờ chạy test. Truyền giờ vào qua tham số để test tự chọn giờ cố định."
  },
  {
    "prompt": "Hai test trong cùng class cùng thêm phần tử vào một field List (không static). Test thứ hai có thấy phần tử của test thứ nhất không?",
    "options": [
      "Có, vì cùng một class",
      "Tuỳ thứ tự chạy",
      "Có, nếu chạy trên cùng máy",
      "Không, vì xUnit tạo object mới cho mỗi test"
    ],
    "answer": 4,
    "explain": "Mỗi test có một object riêng nên field không dùng chung. Field static thì khác."
  },
  {
    "prompt": "Một test kiểm cùng lúc giá, tồn kho, email và log, rồi đỏ. Vấn đề là gì?",
    "options": [
      "Không có vấn đề",
      "Test quá ngắn",
      "Khó biết ngay hành vi nào hỏng, nên mỗi test chỉ kiểm một hành vi",
      "Phải dùng [Theory]"
    ],
    "answer": 3,
    "explain": "Test nhỏ, mỗi test một ý thì tên test đỏ đã cho biết chỗ hỏng."
  }
]
```
