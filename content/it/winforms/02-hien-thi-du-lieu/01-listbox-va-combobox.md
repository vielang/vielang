---
title: ListBox và ComboBox
minutes: 5
---

Nhân viên cần chọn một sản phẩm trong danh sách để xem giá. Thêm tay từng
dòng vào control thì dài và dễ sai. Bài này đưa cả một `List<Product>` vào
control để nó tự hiện tên từng sản phẩm.

## Khái niệm

📋 **ListBox**: control hiện một danh sách để người dùng chọn.

🔽 **ComboBox**: control thu gọn thành một ô, bấm vào mới xổ danh sách xuống để chọn.

🔗 **DataSource**: property nhận cả một danh sách object để control hiện mỗi object thành một dòng.

`DisplayMember` là tên property của object được đem ra hiện trên dòng đó.

## Ví dụ

`Product` lấy từ chương EF Core của khoá ASP.NET Core, tạm bỏ `Stock` cho
gọn. Chương sau mới đọc sản phẩm từ Oracle, bài này tạo sẵn trong bộ nhớ.

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
    private readonly ListBox _productList =
        new ListBox { Width = 250 };
    private readonly Label _priceLabel =
        new Label { AutoSize = true };

    public MainForm()
    {
        Text = "Sản phẩm";
        _productList.DataSource = _products;
        _productList.DisplayMember = "Name";
        _productList.SelectedIndexChanged +=
            ProductList_SelectedIndexChanged;

        var panel = new FlowLayoutPanel
        {
            Dock = DockStyle.Fill,
            FlowDirection = FlowDirection.TopDown,
            Padding = new Padding(12)
        };
        panel.Controls.Add(_productList);
        panel.Controls.Add(_priceLabel);
        Controls.Add(panel);
    }

    private void ProductList_SelectedIndexChanged(
        object? sender, EventArgs e)
    {
        object? item = _productList.SelectedItem;
        if (item == null)
        {
            return;
        }
        var product = (Product)item;
        _priceLabel.Text = $"Giá: {product.Price}";
    }
}

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
}
```

- `DataSource = _products` đưa cả danh sách vào. `DisplayMember = "Name"`
  cho mỗi dòng hiện tên sản phẩm.
- `SelectedItem` có kiểu `object`, vì `ListBox` chứa được mọi loại object.
  `(Product)item` ép nó về `Product`. Khác `(int)` ở bài Toán tử và ép kiểu,
  phép ép này không đổi object mà chỉ cho compiler biết object thật là
  `Product`. Sai kiểu thì báo `InvalidCastException`.
- `SelectedIndexChanged` chỉ chạy khi lựa chọn thay đổi. Lúc mở form, dòng
  đầu được chọn sẵn nhưng event chưa chạy, nên nhãn giá còn trống cho tới
  khi bạn chọn dòng khác.

## ComboBox

`ComboBox` dùng đúng các property trên. Thêm `DropDownStyle` để người dùng
chỉ được chọn, không gõ chữ tuỳ ý vào ô:

```csharp
var productBox = new ComboBox
{
    Width = 250,
    DropDownStyle = ComboBoxStyle.DropDownList
};
```

## Thử ngay

Trong constructor, xoá dòng `_productList.DisplayMember = "Name";` rồi chạy
lại.

**Đoán trước khi chạy:** danh sách hiện gì, tên sản phẩm hay thứ khác?

<details>
<summary>Xem kết quả</summary>

```text
ShopDesk.Product
ShopDesk.Product
```

Thiếu `DisplayMember`, control gọi `ToString()` của từng object. Mặc định
`ToString()` trả về tên đầy đủ của class, gồm cả namespace.

</details>

## Lỗi hay gặp

**Gắn hai control vào cùng một list.** Hai control đó dùng chung cả vị trí
đang chọn: chọn "Vở" ở `ComboBox` thì `ListBox` cũng nhảy sang "Vở".

```csharp
// SAI — chọn ở ô này, ô kia nhảy theo
_productList.DataSource = _products;
productBox.DataSource = _products;
```

```csharp
// ĐÚNG — mỗi control một bản list riêng
_productList.DataSource = _products;
productBox.DataSource =
    new List<Product>(_products);
```

## Tóm tắt

- `DataSource` nhận cả danh sách, `DisplayMember` chọn property để hiện.
- `SelectedItem` là `object`, ép về kiểu thật bằng `(Product)`.
- Thiếu `DisplayMember` thì mỗi dòng hiện kết quả `ToString()`.
- `ComboBox` với `DropDownList` chỉ cho chọn, không cho gõ.

```quiz
[
  {
    "prompt": "ListBox có DataSource là List<Customer>. Muốn mỗi dòng hiện email của khách, đặt gì?",
    "options": [
      "ValueMember = \"Email\"",
      "DataSource = \"Email\"",
      "SelectedValue = \"Email\"",
      "DisplayMember = \"Email\""
    ],
    "answer": 4,
    "explain": "DisplayMember là tên property được lấy ra để hiển thị cho từng object."
  },
  {
    "prompt": "Vì sao phải viết (Product)item trước khi đọc .Price?",
    "options": [
      "SelectedItem có kiểu object",
      "Để đổi giá sang số nguyên",
      "Vì Product là struct, không phải class",
      "Không cần, item.Price chạy được luôn"
    ],
    "answer": 1,
    "explain": "ListBox chứa mọi loại object nên SelectedItem khai báo là object, compiler không biết nó có Price. Ép về Product mới dùng được property của Product."
  },
  {
    "prompt": "ComboBox chọn đơn vị tính chỉ cho chọn trong danh sách, không cho gõ chữ khác. Đặt property nào?",
    "options": [
      "ReadOnly = true",
      "DropDownStyle = ComboBoxStyle.DropDownList",
      "Enabled = false",
      "DropDownStyle = ComboBoxStyle.Simple"
    ],
    "answer": 2,
    "explain": "DropDownList biến ô thành chỉ chọn. Simple vẫn cho gõ, còn Enabled = false thì không chọn được gì."
  }
]
```
