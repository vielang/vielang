---
title: Rẽ nhánh và vòng lặp
minutes: 8
---

Cú pháp phần này giống hầu hết ngôn ngữ họ C. Điều đáng học ở C# là các dạng
**hiện đại** — `switch expression`, `foreach`, `range` — vì code backend thực tế
viết theo dạng đó, đọc code cũ mới thấy dạng dài dòng.

## if và toán tử ba ngôi

```csharp
if (score >= 80)
    level = "Giỏi";
else if (score >= 50)
    level = "Khá";
else
    level = "Trung bình";

string status = isActive ? "Đang dùng" : "Đã khoá";   // ternary operator
```

Luôn viết `{ }` cho thân `if` kể cả khi chỉ có một dòng: thêm một dòng nữa vào
sau này mà quên ngoặc là lỗi rất khó nhìn ra khi review.

## switch expression

```csharp
string label = status switch
{
    OrderStatus.New       => "Mới tạo",
    OrderStatus.Paid      => "Đã thanh toán",
    OrderStatus.Cancelled => "Đã huỷ",
    _                     => "Không xác định",
};
```

`switch` dạng **expression** (từ C# 8) trả về một giá trị, không cần `break`.
Dấu `_` là **discard**, đóng vai `default`. Thiếu nhánh nào đó thì compiler cảnh
báo — an toàn hơn hẳn `switch` dạng câu lệnh cũ.

```csharp
// Dạng cũ, vẫn gặp nhiều trong code có sẵn
switch (status)
{
    case OrderStatus.New:
        label = "Mới tạo";
        break;                 // C# bắt buộc break, không "rơi" sang case sau
    default:
        label = "Không xác định";
        break;
}
```

## for, foreach, while

```csharp
for (int i = 0; i < items.Count; i++)      // cần chỉ số
    Console.WriteLine($"{i}: {items[i]}");

foreach (var item in items)                // chỉ cần phần tử — ưu tiên dạng này
    Console.WriteLine(item);

while (queue.Count > 0)                    // chưa biết lặp bao nhiêu lần
    Handle(queue.Dequeue());

do { attempt++; } while (attempt < 3);     // chạy ít nhất một lần
```

Trong `foreach` **không được** thêm/xoá phần tử của collection đang duyệt — sẽ
ném `InvalidOperationException`. Muốn xoá thì duyệt `for` ngược từ cuối, hoặc lọc
ra danh sách mới bằng LINQ.

## break, continue và index/range

```csharp
foreach (var order in orders)
{
    if (order.IsDeleted) continue;   // bỏ qua phần tử này, chạy tiếp
    if (order.IsFinal) break;        // thoát hẳn vòng lặp
    Process(order);
}

var last  = items[^1];       // phần tử cuối  (index from end)
var first3 = items[..3];     // 3 phần tử đầu (range)
```

## Ghi nhớ

- Lồng quá hai tầng `if` là dấu hiệu nên tách method hoặc dùng **guard clause**: kiểm tra điều kiện sai rồi `return` sớm.
- `foreach` chạy được trên mọi thứ cài `IEnumerable<T>` — array, `List<T>`, kết quả LINQ, cả dữ liệu đọc dần từ database.
- Vòng lặp vô hạn `while (true)` phải luôn có đường thoát rõ ràng (`break`, `CancellationToken`), nếu không service sẽ treo.
