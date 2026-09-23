---
title: enum và hằng số
minutes: 6
---

Chuỗi "paid", "PAID", "Paid" rải khắp code là cách nhanh nhất để sinh bug. `enum`
biến tập giá trị cố định thành thứ compiler kiểm tra được.

## enum

```csharp
public enum OrderStatus
{
    New = 0,
    Paid = 1,
    Shipped = 2,
    Cancelled = 3,
}

var status = OrderStatus.Paid;
Console.WriteLine(status);         // Paid
Console.WriteLine((int)status);    // 1
```

Ghi rõ số cho từng thành viên khi enum được lưu xuống database hoặc gửi qua API:
thêm một giá trị vào giữa mà không ghi số sẽ làm mọi bản ghi cũ đổi nghĩa — loại
bug lặng lẽ và rất khó truy.

```csharp
Enum.TryParse<OrderStatus>("Paid", out var parsed);   // từ chuỗi, không ném lỗi
var all = Enum.GetValues<OrderStatus>();              // duyệt mọi giá trị
```

Cẩn thận: `(OrderStatus)99` **không** ném lỗi. Giá trị từ bên ngoài phải kiểm tra
bằng `Enum.IsDefined` trước khi tin.

## enum dạng cờ

```csharp
[Flags]
public enum Permission
{
    None   = 0,
    Read   = 1,
    Write  = 2,
    Delete = 4,
}

var p = Permission.Read | Permission.Write;   // gộp
bool canWrite = p.HasFlag(Permission.Write);  // True
```

Giá trị phải là luỹ thừa của 2 thì phép bit mới đúng. Dùng cho quyền, cho tuỳ
chọn bật/tắt nhiều thứ cùng lúc.

## const và readonly

```csharp
public class Config
{
    public const int MaxRetry = 3;                  // cố định lúc compile
    public static readonly TimeSpan Timeout         // tính lúc chạy
        = TimeSpan.FromSeconds(30);

    private readonly string _connectionString;      // gán một lần trong constructor

    public Config(string connectionString) => _connectionString = connectionString;
}
```

`const` bị **nhúng thẳng** vào nơi gọi lúc compile. Nếu nó là `public` và nằm ở
thư viện khác, người dùng thư viện phải build lại mới thấy giá trị mới — vì vậy
với hằng số công khai có khả năng đổi, hãy dùng `static readonly`.

## Đừng dùng magic number

```csharp
if (order.Status == 2) { }                      // 2 là gì?
if (order.Status == OrderStatus.Shipped) { }    // rõ ràng

if (retry > 3) { }                              // rải rác khắp nơi
if (retry > Config.MaxRetry) { }                // một chỗ để sửa
```

## Ghi nhớ

- Trạng thái, loại, quyền → `enum`, đừng dùng `string` hay `int` trần.
- Ghi số tường minh cho enum được lưu trữ hoặc truyền đi.
- `switch expression` trên enum sẽ được compiler nhắc khi bạn quên nhánh mới thêm.
