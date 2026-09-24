---
title: Master-detail
minutes: 5
---

Màn hình đơn hàng cần hai bảng: trên là danh sách đơn, dưới là các dòng hàng
của đơn đang chọn. Chọn đơn khác thì bảng dưới đổi theo. Bài này ghép quan hệ
một-nhiều của EF Core với hai `BindingSource` để làm việc đó.

## Khái niệm

🗂️ **Master-detail**: màn hình hai phần. Chọn một dòng ở bảng chính (master) thì bảng phụ (detail) hiện các dòng con của nó.

🧷 **DataMember**: tên property danh sách con. `BindingSource` phụ dùng nó để lấy danh sách con từ dòng đang chọn của `BindingSource` chính.

Đây vẫn là quan hệ một-nhiều giữa `ORDERS` và `ORDER_LINES` của khoá SQL,
và `Order.Lines` của khoá ASP.NET Core. Chỉ khác là giờ hiện nó lên màn hình.

## Ví dụ

Thêm `Order`, `OrderLine` và hai `DbSet` vào code, chép nguyên từ bài Quan hệ
một-nhiều của khoá ASP.NET Core. Đơn mẫu đã tạo bằng
`POST /api/orders/sample` ở bài đó, muốn thêm đơn thì gọi lại lệnh này.

```csharp
using Microsoft.EntityFrameworkCore;

class MainForm : Form
{
    private readonly ShopDbContext _db;
    private readonly BindingSource _orders =
        new BindingSource();
    private readonly BindingSource _lines =
        new BindingSource();

    public MainForm(ShopDbContext db)
    {
        _db = db;
        Text = "Đơn hàng";
        Width = 600;
        Height = 450;

        var ordersGrid = new DataGridView
        {
            Dock = DockStyle.Top,
            Height = 150,
            DataSource = _orders,
            ReadOnly = true,
            AllowUserToAddRows = false
        };
        var linesGrid = new DataGridView
        {
            Dock = DockStyle.Fill,
            DataSource = _lines,
            ReadOnly = true,
            AllowUserToAddRows = false
        };
        Controls.Add(linesGrid);
        Controls.Add(ordersGrid);

        Load += async (sender, e) =>
        {
            _orders.DataSource = await _db.Orders
                .Include(o => o.Lines)
                .ToListAsync();
            _lines.DataSource = _orders;
            _lines.DataMember = "Lines";
        };
    }
}
```

- `Include(o => o.Lines)` đọc đơn kèm các dòng hàng, như ở khoá ASP.NET Core.
- `_lines.DataSource = _orders` và `DataMember = "Lines"`: danh sách dưới là
  `Lines` của đơn đang chọn ở trên.
- Lưới trên chỉ có cột `Id`, `CustomerId`. Property danh sách `Lines` không
  thành cột.

```mermaid Chọn đơn ở trên, lưới dưới hiện Lines của đơn đó
flowchart LR
    O[BindingSource đơn] --> G1[Lưới đơn]
    O -- "DataMember = Lines" --> L[BindingSource dòng]
    L --> G2[Lưới dòng hàng]
```

## Thử ngay

Xoá dòng `.Include(o => o.Lines)` rồi chạy lại.

**Đoán trước khi chạy:** lưới dưới hiện các dòng hàng, báo lỗi, hay trống?

<details>
<summary>Xem kết quả</summary>

```text
Lưới trên vẫn đủ các đơn.
Lưới dưới trống, dù chọn đơn nào.
```

Không có `Include`, EF Core chỉ đọc bảng `ORDERS`. `Lines` của mỗi đơn giữ
list rỗng lúc khởi tạo, giống hệt Thử ngay ở bài Quan hệ một-nhiều của khoá
ASP.NET Core.

</details>

## Lỗi hay gặp

**Đặt `DataMember` trước khi có dữ liệu.** Lúc đó `_orders` chưa có
`DataSource`, nên `BindingSource` không biết `Lines` là gì. App dừng ngay khi
mở với `ArgumentException`: không tìm thấy `DataMember` tên `Lines`.

```csharp
// SAI — đặt trong constructor, _orders còn rỗng
_lines.DataSource = _orders;
_lines.DataMember = "Lines";
```

```csharp
// ĐÚNG — đặt sau khi _orders đã có danh sách đơn
_orders.DataSource = await _db.Orders
    .Include(o => o.Lines)
    .ToListAsync();
_lines.DataSource = _orders;
_lines.DataMember = "Lines";
```

## Tóm tắt

- Master-detail: chọn dòng ở bảng chính, bảng phụ hiện các dòng con.
- `BindingSource` phụ lấy `DataSource` là `BindingSource` chính, `DataMember`
  là tên property danh sách con.
- Phải `Include` danh sách con, nếu không bảng phụ luôn trống.
- Đặt `DataMember` sau khi `BindingSource` chính đã có dữ liệu.

```quiz
[
  {
    "prompt": "Màn hình khách hàng: trên là khách, dưới là đơn của khách đó (Customer.Orders). BindingSource _customerOrders cần gì?",
    "options": [
      "DataSource = _customers, DataMember = \"Orders\"",
      "DataSource = _db.Orders",
      "DataMember = \"Customer\"",
      "DataSource = _customers, DataMember = \"Customers\""
    ],
    "answer": 1,
    "explain": "BindingSource phụ lấy nguồn là BindingSource chính, DataMember là tên property danh sách con Orders."
  },
  {
    "prompt": "Lưới đơn hiện đủ, nhưng lưới dòng hàng luôn trống dù database có dữ liệu. Nguyên nhân hay gặp nhất?",
    "options": [
      "Thiếu AllowUserToAddRows = false",
      "Quên .Include(o => o.Lines) khi đọc đơn",
      "Lưới dòng hàng phải ReadOnly",
      "Oracle chưa COMMIT"
    ],
    "answer": 2,
    "explain": "Không Include thì EF Core không đọc ORDER_LINES, Lines của mỗi đơn là list rỗng."
  },
  {
    "prompt": "Class Order có Id, CustomerId, Lines. Gắn List<Order> vào DataGridView thì lưới có những cột nào?",
    "options": [
      "Id, CustomerId, Lines",
      "Chỉ Lines",
      "Id, CustomerId",
      "Không có cột nào"
    ],
    "answer": 3,
    "explain": "Property kiểu danh sách không thành cột. Muốn xem Lines thì dùng một lưới phụ với DataMember."
  }
]
```
