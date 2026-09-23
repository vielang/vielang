---
title: Làm việc với null
minutes: 8
---

`NullReferenceException` là exception phổ biến nhất trong lịch sử .NET. C# hiện
đại có đủ công cụ để phần lớn lỗi đó bị bắt **lúc compile** — nếu bạn bật và tôn
trọng cảnh báo.

## Bật nullable reference types

```xml
<PropertyGroup>
  <Nullable>enable</Nullable>
  <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
</PropertyGroup>
```

Bật `Nullable` rồi thì kiểu `string` nghĩa là **không bao giờ null**, còn muốn
cho phép null phải viết `string?`. Project mới của .NET đã bật sẵn.

```csharp
string name = null;      // cảnh báo CS8600
string? middle = null;   // hợp lệ

int len = middle.Length;       // CS8602: có thể null
int ok  = middle?.Length ?? 0; // an toàn
```

Đây chỉ là **cảnh báo lúc compile**, runtime không chặn gì cả — dữ liệu từ JSON
hay database vẫn có thể lọt null vào một biến khai báo không null. Vì vậy vẫn
phải kiểm tra ở biên hệ thống.

## Nullable value type

```csharp
int? soLuong = null;          // Nullable<int>
if (soLuong.HasValue) { }
int thuc = soLuong ?? 0;
int chac = soLuong!.Value;    // ném InvalidOperationException nếu đang null
```

Với value type, `?` không chỉ là chú thích cho compiler mà đổi hẳn kiểu thành
`Nullable<T>` — có thật trong runtime, chiếm thêm bộ nhớ cho cờ `HasValue`.

## Các toán tử null

```csharp
var city = user?.Address?.City;            // null-conditional: gặp null thì dừng, trả null
var display = city ?? "Chưa cập nhật";     // null-coalescing
cache ??= new Dictionary<string, string>(); // gán khi đang null
var count = list?.Count ?? 0;
```

## Dấu `!` và khi nào được dùng

```csharp
var order = await db.Orders.FindAsync(id);
var name = order!.CustomerName;   // "tôi cam đoan không null"
```

`!` là **null-forgiving operator**: nó tắt cảnh báo chứ **không** kiểm tra gì.
Mỗi lần viết `!` là bạn nhận trách nhiệm; nếu sai thì lại đúng cái
`NullReferenceException` mà ta đang muốn tránh. Ưu tiên kiểm tra thật:

```csharp
var order = await db.Orders.FindAsync(id);
if (order is null)
    return NotFound();

var name = order.CustomerName;   // từ đây compiler biết chắc không null
```

## Canh ở biên

```csharp
public class Invoice
{
    private readonly Customer _customer;
    private readonly decimal _amount;

    public Invoice(Customer customer, decimal amount)
    {
        ArgumentNullException.ThrowIfNull(customer);
        if (amount <= 0)
            throw new ArgumentOutOfRangeException(nameof(amount), "Số tiền phải lớn hơn 0");

        _customer = customer;
        _amount = amount;
    }
}
```

Chặn dữ liệu sai ngay lúc vào — **fail fast**. Ném `ArgumentNullException` ở
constructor dễ sửa hơn nhiều so với `NullReferenceException` nổ ra sau đó ba tầng
gọi hàm, khi chẳng còn manh mối nào về nơi giá trị null lọt vào.

## Ghi nhớ

- `is null` / `is not null` là cách kiểm tra chuẩn — không bị ảnh hưởng nếu kiểu đó nạp chồng toán tử `==`.
- Đừng trả về `null` cho một danh sách: trả `Array.Empty<T>()` hoặc list rỗng, người gọi khỏi phải kiểm tra.
- Cảnh báo nullable là bạn, đừng dập bằng `!` cho nhanh.
