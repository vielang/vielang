---
title: enum và hằng số
minutes: 11
---

Sprint này thêm trạng thái "Đang đóng gói". Lập trình viên chèn nó vào giữa
enum cho đúng thứ tự nghiệp vụ.

Deploy xong, mọi đơn hàng cũ trong database đổi nghĩa. Đơn "đã giao" hôm qua
giờ hiện là "đang đóng gói", còn đơn đã huỷ thì thành đã giao. Không ai sửa dữ
liệu cả — chỉ là những con số phía sau enum đã trượt đi một nấc.

> **Học xong bài này bạn sẽ:** dùng `enum` thay cho chuỗi trạng thái rải rác;
> biết vì sao phải ghi số tường minh; phân biệt `const` với `readonly` và chọn
> đúng cái.
>
> **Cần biết trước:** kiểu dữ liệu, `switch expression`.

## enum bên dưới chỉ là số nguyên

```csharp
public enum OrderStatus
{
    New = 0,
    Paid = 1,
    Shipped = 2,
    Cancelled = 3,
}

var status = OrderStatus.Paid;
Console.WriteLine(status);        // Paid
Console.WriteLine((int)status);   // 1
```

Cái tên chỉ tồn tại trong code. Xuống database hay qua API, nó là con số. Đây
là chi tiết quyết định cả bài này.

Vì vậy phải **ghi rõ số** cho từng thành viên, ngay khi enum được lưu trữ hay
truyền đi:

```csharp
// SAI — thêm vào giữa là mọi số phía sau trượt
public enum OrderStatus
{
    New, Paid, Packing, Shipped, Cancelled
}
```

```csharp
// ĐÚNG — số cố định, thêm mới thì lấy số còn trống
public enum OrderStatus
{
    New = 0,
    Paid = 1,
    Shipped = 2,
    Cancelled = 3,
    Packing = 4,
}
```

Bản ghi cũ lưu số `2`, mà `2` vừa đổi chủ. Đó là toàn bộ sự cố ở đầu bài.

## Thử ngay: enum không kiểm tra giá trị

```csharp
var status = (OrderStatus)99;

Console.WriteLine(status);
Console.WriteLine(Enum.IsDefined(status));
```

**Đoán trước khi chạy:** ép số 99 sang enum. Chương trình ném lỗi, hay in ra
cái gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
99
False
```

Ép kiểu sang enum **không kiểm tra gì cả**. Bạn nhận một giá trị không có
trong danh sách, và nó lặng lẽ đi tiếp vào hệ thống.

Giá trị đến từ bên ngoài — query string, JSON, cột số trong database cũ — thì
phải kiểm bằng `Enum.IsDefined` trước khi tin.

</details>

```csharp
Enum.TryParse<OrderStatus>("Paid", out var parsed);
var all = Enum.GetValues<OrderStatus>();
```

## enum dạng cờ để gộp nhiều lựa chọn

```csharp
[Flags]
public enum Permission
{
    None = 0,
    Read = 1,
    Write = 2,
    Delete = 4,
}

var p = Permission.Read | Permission.Write;
bool canWrite = p.HasFlag(Permission.Write);   // True
```

| Giá trị | Nhị phân | Gộp được |
|---|---|---|
| `Read = 1` | `001` | ✔ |
| `Write = 2` | `010` | ✔ |
| `Delete = 4` | `100` | ✔ |
| `Delete = 3` | `011` | ✘ trùng `Read\|Write` |

Mỗi quyền phải chiếm một bit riêng. Nghĩa là giá trị phải là luỹ thừa của 2.

Đặt 1, 2, 3 thì `Read | Write` bằng đúng `3`, mà `3` lại chính là quyền thứ ba.

Cấp cho ai đó quyền đọc và ghi, xong `p.HasFlag(Permission.Delete)` trả về
`True`. Họ xoá được dữ liệu mà chưa ai cấp quyền xoá.

## const, static readonly hay readonly

| Khai báo | Chốt lúc nào | Dùng cho |
|---|---|---|
| `const` | compile | số, chuỗi, bool không bao giờ đổi |
| `static readonly` | chạy | hằng số công khai có thể đổi |
| `readonly` field | constructor | phụ thuộc của một object |

```csharp
public class Config
{
    public const int MaxRetry = 3;

    public static readonly TimeSpan Timeout
        = TimeSpan.FromSeconds(30);

    private readonly string _connectionString;

    public Config(string cs) => _connectionString = cs;
}
```

`const` bị **nhúng thẳng** vào nơi gọi lúc compile. Sửa giá trị trong thư
viện là chưa đủ. Bên dùng phải build lại mới thấy.

Nên với hằng số công khai có khả năng đổi, hãy dùng `static readonly`.

## Đừng để magic number nằm rải trong code

```csharp
// SAI — 2 là gì? 3 là gì?
if (order.Status == 2) { }
if (retryCount > 3) { }
```

```csharp
// ĐÚNG — đọc là hiểu, sửa một chỗ
if (order.Status == OrderStatus.Shipped) { }
if (retryCount > Config.MaxRetry) { }
```

Con số trần không nói gì. Cần đổi thì phải đi tìm hết mọi chỗ đã gõ nó, và
chỉ sót một chỗ là dữ liệu lệch.

## Dấu hiệu trong code của bạn

- Chuỗi trạng thái so sánh bằng `==` ("paid", "PAID") → đổi sang `enum`, compiler bắt lỗi gõ sai giúp bạn.
- `enum` lưu xuống database mà không ghi số tường minh → quả bom hẹn giờ.
- Ép `(MyEnum)soNguyen` từ dữ liệu ngoài mà không `Enum.IsDefined` → giá trị rác đi thẳng vào nghiệp vụ.
- Số lạ nằm rải trong điều kiện (`> 3`, `== 2`) → đặt tên cho chúng.

## Ghi nhớ

- Trạng thái, loại, quyền → `enum`, đừng dùng `string` hay `int` trần.
- Ghi số tường minh; thêm giá trị mới thì lấy số còn trống.
- Ép kiểu sang enum không kiểm tra gì — dùng `Enum.IsDefined` cho dữ liệu ngoài.
- `const` nhúng lúc compile; hằng số công khai hay đổi thì `static readonly`.

## Bước tiếp theo

Có `enum` rồi, câu hỏi kế là xử lý nó thế nào cho gọn.

Bài sau, **Pattern matching**, thay những chuỗi `if` ép kiểu dài dòng bằng một
biểu thức. Đây là thứ bạn sẽ gặp khắp nơi trong code C# hiện đại.

```quiz
[
  {
    "prompt": "Enum lưu xuống database dạng số. Ai đó thêm một giá trị vào GIỮA danh sách. Chuyện gì xảy ra với dữ liệu cũ?",
    "options": [
      "Không sao, enum lưu theo tên",
      "Mọi bản ghi có số lớn hơn vị trí chèn đều đổi nghĩa",
      "Database tự cập nhật lại",
      "Ứng dụng ném exception khi đọc bản ghi cũ"
    ],
    "answer": 2,
    "explain": "Enum bên dưới là số tự đánh từ 0. Chèn vào giữa làm mọi giá trị phía sau trượt một nấc, còn dữ liệu cũ thì vẫn giữ số cũ."
  },
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "var status = (OrderStatus)99;\nConsole.WriteLine(Enum.IsDefined(status));",
    "options": ["True", "False", "Ném InvalidCastException", "Lỗi compile"],
    "answer": 2,
    "explain": "Ép kiểu sang enum không kiểm tra gì, nên status mang giá trị 99 không có trong danh sách. IsDefined chính là cách phát hiện."
  },
  {
    "prompt": "Enum [Flags] dùng cho quyền Read, Write, Delete. Giá trị phải đặt thế nào?",
    "options": [
      "1, 2, 3 cho dễ nhớ",
      "1, 2, 4 — luỹ thừa của 2",
      "0, 1, 2 như enum thường",
      "Bao nhiêu cũng được, HasFlag tự xử lý"
    ],
    "answer": 2,
    "explain": "Mỗi quyền phải chiếm một bit riêng thì phép | và HasFlag mới đúng. Với 1, 2, 3 thì Read|Write bằng 3, trùng luôn với Delete."
  },
  {
    "prompt": "Hằng số public MaxRetry nằm trong một thư viện dùng chung, thỉnh thoảng phải chỉnh. Khai báo thế nào?",
    "options": [
      "public const int — nhanh nhất",
      "public static readonly int — đổi giá trị không cần build lại bên dùng",
      "private const int",
      "Biến thường, gán trong constructor"
    ],
    "answer": 2,
    "explain": "const bị nhúng thẳng vào nơi gọi lúc compile, nên bên dùng vẫn giữ giá trị cũ cho tới khi build lại. static readonly đọc lúc chạy."
  }
]
```
