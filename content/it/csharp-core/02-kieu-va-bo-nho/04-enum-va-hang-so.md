---
title: enum và hằng số
minutes: 9
---

Sprint này có thêm trạng thái "Đang đóng gói", lập trình viên thêm nó vào giữa
enum cho đúng thứ tự nghiệp vụ. Deploy xong, mọi đơn hàng cũ trong database
đổi nghĩa: đơn "đã giao" hôm qua giờ hiện là "đã huỷ". Không ai sửa dữ liệu cả
— chỉ là những con số phía sau enum đã trượt đi một nấc.

> **Học xong bài này bạn sẽ:** dùng `enum` thay cho chuỗi trạng thái rải rác;
> biết vì sao phải ghi số tường minh; phân biệt `const` với `readonly` và
> chọn đúng cái.
>
> **Cần biết trước:** kiểu dữ liệu, `switch expression`.

## enum

```csharp
public enum OrderStatus
{
    New = 0,
    Paid = 1,
    Shipped = 2,
    Cancelled = 3,
}

var tt = OrderStatus.Paid;
Console.WriteLine(tt);        // Paid
Console.WriteLine((int)tt);   // 1
```

Bên dưới `enum` chỉ là số nguyên. Vì vậy phải **ghi rõ số** cho từng thành viên
khi enum được lưu xuống database hoặc gửi qua API:

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

Đây chính là sự cố ở đầu bài: bản ghi cũ lưu số `2`, mà `2` vừa đổi chủ.

## Thử ngay: enum không kiểm tra giá trị

```csharp
var tt = (OrderStatus)99;

Console.WriteLine(tt);
Console.WriteLine(Enum.IsDefined(tt));
```

**Đoán trước khi chạy:** ép số 99 sang enum — chương trình ném lỗi, hay in ra
cái gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
99
False
```

Ép kiểu sang enum **không hề kiểm tra**. Bạn nhận một giá trị không tồn tại
trong danh sách, và nó lặng lẽ đi tiếp vào hệ thống. Giá trị đến từ bên ngoài
(query string, JSON, cột số trong database cũ) thì phải kiểm bằng
`Enum.IsDefined` trước khi tin.

</details>

```csharp
Enum.TryParse<OrderStatus>("Paid", out var tt2);
var tatCa = Enum.GetValues<OrderStatus>();
```

## enum dạng cờ

```csharp
[Flags]
public enum Quyen
{
    None = 0,
    Doc = 1,
    Ghi = 2,
    Xoa = 4,
}

var q = Quyen.Doc | Quyen.Ghi;
bool ghiDuoc = q.HasFlag(Quyen.Ghi);   // True
```

Giá trị phải là luỹ thừa của 2 thì phép bit mới đúng. Dùng cho quyền, cho các
tuỳ chọn bật/tắt nhiều thứ cùng lúc.

## const và readonly

```csharp
public class Config
{
    public const int MaxRetry = 3;

    public static readonly TimeSpan Timeout
        = TimeSpan.FromSeconds(30);

    private readonly string _conn;

    public Config(string conn) => _conn = conn;
}
```

- `const` — cố định lúc compile, chỉ dùng được với số, chuỗi, bool.
- `static readonly` — tính lúc chạy, dùng được với mọi kiểu.
- `readonly` field — gán một lần trong constructor rồi khoá.

`const` bị **nhúng thẳng** vào nơi gọi lúc compile. Nếu nó `public` và nằm ở
thư viện khác, người dùng thư viện phải build lại mới thấy giá trị mới — nên
với hằng số công khai có khả năng đổi, hãy dùng `static readonly`.

## Đừng dùng magic number

```csharp
// SAI — 2 là gì? 3 là gì?
if (don.TrangThai == 2) { }
if (soLanThu > 3) { }
```

```csharp
// ĐÚNG — đọc là hiểu, sửa một chỗ
if (don.TrangThai == OrderStatus.Shipped) { }
if (soLanThu > Config.MaxRetry) { }
```

## Dấu hiệu trong code của bạn

- Chuỗi trạng thái so sánh bằng `==` ("paid", "PAID") → đổi sang `enum`, compiler bắt lỗi gõ sai giúp bạn.
- `enum` lưu xuống database mà không ghi số tường minh → quả bom hẹn giờ, chờ người thêm giá trị vào giữa.
- Ép `(MyEnum)soNguyen` từ dữ liệu ngoài mà không `Enum.IsDefined` → giá trị rác đi thẳng vào nghiệp vụ.
- Số lạ nằm rải trong điều kiện (`> 3`, `== 2`) → đặt tên cho chúng.

## Ghi nhớ

- Trạng thái, loại, quyền → `enum`, đừng dùng `string` hay `int` trần.
- Ghi số tường minh cho enum được lưu trữ hoặc truyền đi; thêm giá trị mới thì lấy số còn trống.
- Ép kiểu sang enum không kiểm tra gì — `Enum.IsDefined` cho dữ liệu từ ngoài.
- `const` nhúng lúc compile; hằng số công khai hay đổi thì dùng `static readonly`.

## Bước tiếp theo

Bài sau — **Pattern matching** — cách C# hiện đại hỏi "giá trị này có dạng thế
nào", thay cho chuỗi `if` ép kiểu dài dòng. `enum` vừa học sẽ xuất hiện lại ở
đó trong `switch expression`.

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
    "code": "var tt = (OrderStatus)99;\nConsole.WriteLine(Enum.IsDefined(tt));",
    "options": ["True", "False", "Ném InvalidCastException", "Lỗi compile"],
    "answer": 2,
    "explain": "Ép kiểu sang enum không kiểm tra gì, nên tt mang giá trị 99 không có trong danh sách. IsDefined chính là cách phát hiện."
  },
  {
    "prompt": "Enum [Flags] dùng cho quyền Doc, Ghi, Xoa. Giá trị phải đặt thế nào?",
    "options": [
      "1, 2, 3 cho dễ nhớ",
      "1, 2, 4 — luỹ thừa của 2",
      "0, 1, 2 như enum thường",
      "Bao nhiêu cũng được, HasFlag tự xử lý"
    ],
    "answer": 2,
    "explain": "Mỗi quyền phải chiếm một bit riêng thì phép | và HasFlag mới đúng. Với 1, 2, 3 thì Doc|Ghi bằng 3, trùng luôn với Xoa."
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
