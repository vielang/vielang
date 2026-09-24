---
title: DataGridView
minutes: 5
---

Mỗi dòng của `ListBox` chỉ hiện được một thông tin. Màn hình kho cần thấy
cùng lúc mã, tên, giá của mọi sản phẩm, giống kết quả một câu `SELECT` ở khoá
SQL. `DataGridView` hiện dữ liệu thành bảng như vậy.

## Khái niệm

🧮 **DataGridView**: control hiện danh sách object thành bảng, mỗi object một dòng, mỗi property một cột.

Khi `DataSource` là một `List<Product>`, lưới tự tạo cột `Id`, `Name`,
`Price` từ các property public của `Product`. Không cần khai báo cột bằng tay.

## Ví dụ

```csharp
class MainForm : Form
{
    private readonly List<Product> _products =
        new List<Product>
        {
            new Product
            {
                Id = 1, Name = "Bút bi", Price = 5000m
            },
            new Product
            {
                Id = 2, Name = "Vở", Price = 12000m
            }
        };

    public MainForm()
    {
        Text = "Kho hàng";
        Width = 500;

        var grid = new DataGridView
        {
            Dock = DockStyle.Fill,
            DataSource = _products,
            ReadOnly = true,
            AutoSizeColumnsMode =
                DataGridViewAutoSizeColumnsMode.Fill
        };
        grid.DataBindingComplete += (sender, e) =>
        {
            grid.Columns[1].HeaderText = "Tên";
            grid.Columns[2].HeaderText = "Giá";
            grid.Columns[2].DefaultCellStyle.Format =
                "N0";
        };

        Controls.Add(grid);
    }
}

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
}
```

- `ReadOnly = true` chỉ cho xem, không cho sửa ô. Bài CRUD sẽ mở phần sửa.
- `AutoSizeColumnsMode.Fill` giãn các cột cho kín chiều ngang.
- Cột chỉ có sau khi lưới gắn xong dữ liệu, nên đổi tiêu đề trong event
  `DataBindingComplete`. Cột đánh số từ 0 như `List`, theo thứ tự khai báo
  property: `Columns[2]` là cột `Price`. Lambda viết trong constructor nên
  dùng được biến cục bộ `grid`.
- `Format = "N0"` hiện số có dấu phân cách hàng nghìn, không có phần lẻ.

## Thử ngay

Thêm một nút "Thêm thước" vào form. Handler của nút thêm sản phẩm vào
`_products`:

```csharp
// Trong constructor, ngay sau Controls.Add(grid);
var addButton = new Button
{
    Text = "Thêm thước",
    Dock = DockStyle.Top
};
addButton.Click += (sender, e) =>
{
    _products.Add(new Product
    {
        Id = 3, Name = "Thước", Price = 7000m
    });
};
Controls.Add(addButton);
```

**Đoán trước khi chạy:** bấm nút, lưới có hiện thêm dòng "Thước" không?

<details>
<summary>Xem kết quả</summary>

```text
Lưới vẫn chỉ có 2 dòng: Bút bi, Vở.
```

`_products` đã có 3 phần tử, nhưng `List<T>` không báo cho ai khi nó thay
đổi, nên lưới không biết mà vẽ lại. Bài BindingSource sẽ giải quyết chuyện
này.

</details>

## Lỗi hay gặp

**Dùng field thay cho property.** Lưới chỉ tạo cột từ property, nên class
chỉ có field thì lưới trống trơn, không có cột nào.

```csharp
// SAI — lưới không có cột nào
public class Product
{
    public int Id;
    public string Name = "";
}
```

```csharp
// ĐÚNG — có { get; set; }
public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
}
```

## Tóm tắt

- `DataGridView` hiện danh sách object thành bảng.
- Cột được tạo tự động từ property public, không tạo từ field.
- Đổi tiêu đề và định dạng cột trong `DataBindingComplete`.
- Thêm phần tử vào `List<T>` sau khi gắn thì lưới không tự cập nhật.

```quiz
[
  {
    "prompt": "Class Customer có property Id, Name, Email. Gắn List<Customer> vào DataGridView thì lưới có mấy cột?",
    "options": [
      "1 cột",
      "3 cột",
      "0 cột, phải tự khai báo",
      "Tuỳ số khách hàng"
    ],
    "answer": 2,
    "explain": "Mỗi property public thành một cột, nên có 3 cột Id, Name, Email."
  },
  {
    "prompt": "Muốn cột Price hiện số có dấu phân cách hàng nghìn, không có phần lẻ, đặt gì?",
    "options": [
      "HeaderText = \"N0\"",
      "ReadOnly = true",
      "AutoSizeColumnsMode = Fill",
      "DefaultCellStyle.Format = \"N0\""
    ],
    "answer": 4,
    "explain": "Format N0 định dạng số có dấu phân cách hàng nghìn và không có phần lẻ."
  },
  {
    "prompt": "Gắn List<Order> vào lưới, sau đó gọi list.Add(...) thêm một đơn mới. Trên lưới thì sao?",
    "options": [
      "Dòng mới hiện ngay",
      "Chương trình báo lỗi",
      "Lưới không hiện dòng mới, vì List<T> không báo thay đổi",
      "Lưới hiện dòng mới sau 1 giây"
    ],
    "answer": 3,
    "explain": "List<T> không phát thông báo khi thay đổi, nên lưới không biết mà vẽ lại."
  }
]
```
