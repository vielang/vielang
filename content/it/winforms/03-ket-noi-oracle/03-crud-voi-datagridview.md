---
title: CRUD với DataGridView
minutes: 6
---

Màn hình kho mới chỉ xem được. Nhân viên cần sửa giá, thêm sản phẩm mới, xoá
sản phẩm ngừng bán ngay trên lưới, rồi bấm một nút để ghi hết xuống Oracle.
EF Core làm được việc này với rất ít code.

## Khái niệm

👀 **Change tracking (theo dõi thay đổi)**: `DbContext` ghi nhớ mọi object nó đã đọc ra hoặc được thêm vào, và biết object nào mới, bị sửa hay bị xoá.

🪝 **Local.ToBindingList()**: danh sách các object của một `DbSet` mà `DbContext` đang theo dõi, gắn được vào `BindingSource`.

Thêm hay xoá dòng trên lưới gắn với danh sách này thì `DbContext` biết ngay.

`SaveChangesAsync` đã gặp ở khoá ASP.NET Core. Nó biến những thay đổi đang
ghi nhớ thành câu `INSERT`, `UPDATE`, `DELETE`, rồi chạy chúng trong một
transaction như bài Transaction của khoá SQL.

## Ví dụ

```csharp
using Microsoft.EntityFrameworkCore;

class MainForm : Form
{
    private readonly ShopDbContext _db;
    private readonly BindingSource _source =
        new BindingSource();
    private readonly Button _saveButton = new Button
    {
        Text = "Lưu",
        Dock = DockStyle.Top
    };

    public MainForm(ShopDbContext db)
    {
        _db = db;
        Text = "Kho hàng";
        Width = 500;

        var grid = new DataGridView
        {
            Dock = DockStyle.Fill,
            DataSource = _source
        };
        grid.DataBindingComplete += (sender, e) =>
        {
            grid.Columns[0].ReadOnly = true;
        };
        Controls.Add(grid);
        Controls.Add(_saveButton);

        Load += async (sender, e) =>
        {
            await _db.Products.LoadAsync();
            _source.DataSource =
                _db.Products.Local.ToBindingList();
        };
        _saveButton.Click += SaveButton_Click;
    }

    private async void SaveButton_Click(
        object? sender, EventArgs e)
    {
        _saveButton.Enabled = false;
        try
        {
            int count = await _db.SaveChangesAsync();
            Text = $"Đã lưu {count} thay đổi";
        }
        finally
        {
            _saveButton.Enabled = true;
        }
    }
}
```

- Lưới bỏ `ReadOnly` và giữ dòng trống cuối. Sửa ngay trong ô, gõ vào dòng
  trống để thêm. Muốn xoá, bấm ô xám đầu dòng để chọn cả dòng rồi bấm
  phím Delete.
- `LoadAsync()` đọc bảng vào `DbContext`. `Local.ToBindingList()` đưa các
  object đó lên lưới.
- `finally` bật lại nút dù lưu thành công hay lỗi, như bài Exception của
  khoá C# Core.
- Cột `Id` (cột 0) chỉ đọc, vì Oracle tự sinh khoá chính.
- `SaveChangesAsync` trả về số dòng đã ghi.

```mermaid Lưới nối với DbContext, chỉ SaveChangesAsync mới ghi xuống Oracle
flowchart LR
    G[Lưới] -- "sửa, thêm, xoá" --> D[DbContext ghi nhớ]
    D -- SaveChangesAsync --> O[(Oracle)]
```

## Thử ngay

Chạy app. Sửa giá một sản phẩm, gõ thêm một sản phẩm mới ở dòng trống cuối,
chọn cả một dòng khác rồi bấm Delete. Sau đó bấm "Lưu".

**Đoán trước khi chạy:** tiêu đề báo đã lưu mấy thay đổi, và đó là những
thay đổi gì?

<details>
<summary>Xem kết quả</summary>

```text
Đã lưu 3 thay đổi
```

Ba thay đổi, thành một `UPDATE`, một `INSERT` và một `DELETE` trong Oracle.
Trước khi bấm "Lưu", mọi thay đổi chỉ nằm trong bộ nhớ của `DbContext`. Đóng
app lúc đó thì Oracle không nhận được gì, giống transaction chưa `COMMIT`.

</details>

## Lỗi hay gặp

**Gắn lưới vào `ToListAsync()`.** Sửa ô vẫn được lưu, vì `DbContext`
đang theo dõi các object đó. Còn thêm hay xoá dòng trên lưới chỉ làm đổi một
list thường, `DbContext` không biết nên `SaveChangesAsync` bỏ qua.

```csharp
// SAI — thêm, xoá trên lưới không được lưu
_source.DataSource =
    await _db.Products.ToListAsync();
```

```csharp
// ĐÚNG — lưới nối thẳng với DbContext
await _db.Products.LoadAsync();
_source.DataSource =
    _db.Products.Local.ToBindingList();
```

**Để trống tên sản phẩm mới.** Oracle coi chuỗi rỗng là `NULL`, như bài NULL
trong SQL của khoá SQL. Cột `NAME` là `NOT NULL`, nên `SaveChangesAsync` ném
`DbUpdateException` với mã `ORA-01400`. Không có `finally` thì nút "Lưu" bị
tắt luôn sau lần lỗi đó.

## Tóm tắt

- `DbContext` theo dõi object mới, bị sửa, bị xoá (change tracking).
- `LoadAsync()` rồi gắn `Local.ToBindingList()` để lưới nối thẳng với
  `DbContext`.
- `SaveChangesAsync` ghi mọi thay đổi trong một transaction, trả về số dòng.
- Chưa `SaveChangesAsync` thì Oracle chưa nhận gì.

```quiz
[
  {
    "prompt": "Lưới gắn với Local.ToBindingList(). Nhân viên sửa giá 2 dòng và xoá 1 dòng, chưa bấm Lưu. Trong bảng PRODUCTS có gì thay đổi?",
    "options": [
      "Giá 2 dòng đã đổi",
      "Chưa có gì thay đổi",
      "Dòng bị xoá đã mất",
      "Đổi hết, vì lưới nối thẳng với Oracle"
    ],
    "answer": 2,
    "explain": "Lưới nối với DbContext chứ không nối thẳng Oracle. Chỉ SaveChangesAsync mới gửi thay đổi xuống."
  },
  {
    "prompt": "Gắn lưới vào await _db.Products.ToListAsync(). Nhân viên thêm một dòng rồi bấm Lưu. Kết quả?",
    "options": [
      "Dòng mới được INSERT",
      "Báo lỗi ngay khi thêm",
      "Dòng mới không được lưu vì DbContext không biết nó",
      "Mọi dòng bị INSERT lại"
    ],
    "answer": 3,
    "explain": "ToListAsync trả về list thường. Thêm vào list đó không báo cho DbContext, nên không có INSERT."
  },
  {
    "prompt": "SaveChangesAsync gửi một UPDATE và một INSERT, nhưng INSERT bị Oracle từ chối. UPDATE thì sao?",
    "options": [
      "Vẫn được lưu",
      "Lưu một nửa",
      "Được lưu nếu chạy trước INSERT",
      "Bị huỷ theo, vì cả hai nằm trong một transaction"
    ],
    "answer": 4,
    "explain": "SaveChangesAsync chạy mọi câu trong một transaction. Một câu lỗi thì cả transaction bị ROLLBACK."
  }
]
```
