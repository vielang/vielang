---
title: class, struct hay record
minutes: 9
---

C# có ba cách khai báo một kiểu dữ liệu của riêng bạn. Chọn sai không làm code
chạy sai, nhưng làm nó khó đọc và dễ sinh bug so sánh.

## class — mặc định của mọi thứ

```csharp
public class Order
{
    public int Id { get; set; }
    public string CustomerName { get; set; } = "";
    public List<OrderItem> Items { get; set; } = new();

    public decimal Total => Items.Sum(i => i.Price * i.Quantity);
}
```

Reference type, có **identity**: hai object khác nhau dù mọi field giống hệt thì
`==` vẫn là `false`. Đúng cho những thứ có vòng đời và trạng thái: đơn hàng,
người dùng, service.

## record — dữ liệu, so sánh theo giá trị

```csharp
public record Money(decimal Amount, string Currency);

var a = new Money(100, "VND");
var b = new Money(100, "VND");

Console.WriteLine(a == b);        // True — record so sánh theo GIÁ TRỊ
Console.WriteLine(a);             // Money { Amount = 100, Currency = VND }

var c = a with { Amount = 200 };  // tạo bản sao, đổi một field
```

`record` vẫn là reference type (trừ `record struct`), nhưng compiler tự sinh
`Equals`, `GetHashCode`, `ToString` và toán tử `with`. Mặc định các property khai
báo kiểu rút gọn như trên là **init-only**: gán lúc khởi tạo rồi thôi.

Dùng `record` cho: DTO của API, message giữa các service, **value object** trong
domain (tiền, địa chỉ, khoảng thời gian), kết quả truy vấn.

## struct — value type, nhỏ và bất biến

```csharp
public readonly record struct Point(int X, int Y);
```

`struct` là value type: chép giá trị mỗi lần gán hay truyền đi. Chỉ dùng khi thoả
đồng thời: kích thước nhỏ (khoảng 16 byte trở xuống), **immutable**, và được tạo
ra rất nhiều. Ngoài các trường hợp đó, `class` hoặc `record` luôn là lựa chọn an
toàn hơn — struct lớn còn chậm hơn class vì chép liên tục.

## Property, field và init

```csharp
public class Product
{
    private readonly List<string> _tags = new();   // field: chi tiết bên trong

    public required string Name { get; init; }     // bắt buộc truyền khi khởi tạo
    public decimal Price { get; private set; }     // đọc công khai, sửa nội bộ
    public IReadOnlyList<string> Tags => _tags;    // lộ ra dạng chỉ đọc

    public void ApplyDiscount(decimal percent) => Price -= Price * percent / 100;
}

var p = new Product { Name = "Bàn phím" };   // thiếu Name là lỗi compile
```

Public thì dùng **property**, không dùng field. Property cho phép thêm kiểm tra,
đổi cách tính, hay khoá quyền ghi mà không phá code gọi.

## Chọn cái nào

| Tình huống | Chọn |
|---|---|
| Thực thể có trạng thái, có id (Order, User, Service) | `class` |
| Dữ liệu chỉ để chở đi, so sánh theo giá trị (DTO, value object) | `record` |
| Giá trị nhỏ, bất biến, tạo rất nhiều (Point, Rgb) | `readonly record struct` |

## Ghi nhớ

- `class` so sánh theo tham chiếu, `record` so sánh theo giá trị — đây là khác biệt hay bị hỏi nhất.
- `with` chỉ chép **nông** (shallow copy): list bên trong vẫn dùng chung.
- Mặc định cứ `class` hoặc `record`; chỉ chuyển sang `struct` khi đo đạc cho thấy cần.
