---
title: CRUD với DataGridView
minutes: 6
---

Màn hình kho mới chỉ xem được. Nhân viên cần sửa giá, thêm sản phẩm mới, xoá
sản phẩm ngừng bán ngay trên lưới, rồi bấm một nút để ghi hết xuống Oracle.
EF Core làm được việc này với rất ít code.

## Khái niệm

👀 **Change tracking**: `DbContext` ghi nhớ mọi object nó đã đọc hoặc được thêm vào, và biết object nào mới, bị sửa hay bị xoá.

🔗 **Local.ToBindingList()**: danh sách các object mà `DbContext` đang theo dõi, gắn được vào `BindingSource`. Thêm, xoá trên lưới là `DbContext` biết ngay.

`SaveChangesAsync` đã gặp ở khoá ASP.NET Core. Nó đổi các thay đổi đang ghi
nhớ thành câu `INSERT`, `UPDATE`, `DELETE`, và chạy chúng trong một
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
        int count = await _db.SaveChangesAsync();
        Text = $"Kho hàng - đã lưu {count} thay đổi";
        _saveButton.Enabled = true;
    }
}
```

- Lưới không còn `ReadOnly`, cũng không tắt dòng trống cuối: sửa ngay trong
  ô, gõ vào dòng trống để thêm, chọn dòng rồi bấm phím Delete để xoá.
- `LoadAsync()` đọc bảng vào `DbContext`. `Local.ToBindingList()` đưa các
  object đó lên lưới.
- Cột `Id` (cột 0) chỉ đọc, vì Oracle tự sinh khoá chính.
- `SaveChangesAsync` trả về số dòng đã ghi.

## Thử ngay

Chạy app. Sửa giá một sản phẩm, gõ thêm một sản phẩm mới ở dòng trống cuối,
chọn một dòng khác rồi bấm Delete. Sau đó bấm "Lưu".

**Đoán trước khi chạy:** tiêu đề báo đã lưu mấy thay đổi, và đó là những
thay đổi gì?

<details>
<summary>Xem kết quả</summary>

```text
Kho hàng - đã lưu 3 thay đổi
```

Ba thay đổi, thành một `UPDATE`, một `INSERT`, một `DELETE` trong Oracle.
Trước khi bấm "Lưu", mọi thay đổi chỉ nằm trong bộ nhớ của `DbContext`. Đóng
app lúc đó thì Oracle không nhận được gì, giống transaction chưa `COMMIT`.

</details>

## Lỗi hay gặp

**Gắn lưới vào `ToListAsync()`.** Sửa ô vẫn được lưu, vì các object đó
`DbContext` đang theo dõi. Nhưng dòng thêm và dòng xoá trên lưới chỉ đổi
list thường, `DbContext` không biết, nên `SaveChangesAsync` bỏ qua.

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
      "Chưa có gì thay đổi",
      "Giá 2 dòng đã đổi",
      "Dòng bị xoá đã mất",
      "Đổi hết, vì lưới nối thẳng với Oracle"
    ],
    "answer": 1,
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
