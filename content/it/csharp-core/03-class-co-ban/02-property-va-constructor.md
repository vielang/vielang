---
title: Property và constructor
minutes: 5
---

Ở bài trước, tạo một sản phẩm tốn ba dòng: `new`, gán tên, gán giá. Quên một
dòng là có sản phẩm không tên. Bài này dùng constructor để bắt buộc truyền đủ
dữ liệu lúc tạo, và property để quyết định phần nào được sửa về sau.

## Khái niệm

🏷️ **Property**: thành viên của class dùng như field, nhưng quy định được ai đọc và ai ghi qua `get` và `set`.

🏗️ **Constructor**: method đặc biệt trùng tên class, tự chạy khi gọi `new`, dùng để gán giá trị ban đầu.

## Ví dụ

```csharp
var pen = new Product("Bút bi", 5000m);
pen.Stock = 120;

Console.WriteLine($"{pen.Name}: {pen.Price}đ");
Console.WriteLine(pen.Stock);

class Product
{
    public string Name { get; }
    public decimal Price { get; }
    public int Stock { get; set; }

    public Product(string name, decimal price)
    {
        Name = name;
        Price = price;
    }
}
```

- `{ get; set; }`: đọc và ghi được từ bên ngoài, như `Stock`.
- `{ get; }`: bên ngoài chỉ đọc được. Chỉ constructor mới gán được, như
  `Name` và `Price`.
- Constructor không có kiểu trả về, tên trùng với tên class.
- Class có constructor nhận tham số thì `new Product()` không còn dùng được.
  Muốn tạo sản phẩm thì phải truyền đủ tên và giá.

Theo quy ước, property viết hoa chữ cái đầu (`Price`), tham số viết thường
(`price`).

## Thử ngay

Chép ví dụ trên vào `Program.cs`. Thêm dòng in sau vào đầu constructor, rồi
thay các dòng gọi ở đầu file bằng đoạn dưới:

```csharp
Console.WriteLine($"Tạo sản phẩm: {name}");
```

```csharp
Console.WriteLine("Bắt đầu");
var pen = new Product("Bút bi", 5000m);
var book = new Product("Vở", 12000m);
Console.WriteLine("Kết thúc");
```

**Đoán trước khi chạy:** dòng "Tạo sản phẩm" in ra mấy lần, và nằm ở đâu?

<details>
<summary>Xem kết quả</summary>

```text
Bắt đầu
Tạo sản phẩm: Bút bi
Tạo sản phẩm: Vở
Kết thúc
```

Hai lần, mỗi lần gọi `new` in một lần. Constructor chạy đúng lúc object được tạo.

</details>

## Lỗi hay gặp

**Gán property chỉ có `get`.** Compiler chặn ngay.

```csharp
// SAI — lỗi compile: Price chỉ đọc được
var pen = new Product("Bút bi", 5000m);
pen.Price = 4000m;
```

Muốn cho sửa giá thì phải khai báo `{ get; set; }`. Để `{ get; }` nghĩa là
giá đã chốt từ lúc tạo.

**Gán tham số cho chính nó.** Tên tham số trùng tên property, chỉ khác hoa
thường, rất dễ gõ nhầm.

```csharp
// SAI — gán price cho price, Price vẫn là 0
class Item
{
    public decimal Price { get; set; }

    public Item(decimal price)
    {
        price = price;
    }
}
```

```csharp
// ĐÚNG — vế trái là property Price viết hoa
class Item
{
    public decimal Price { get; set; }

    public Item(decimal price)
    {
        Price = price;
    }
}
```

## Tóm tắt

- Property thay cho field `public`: `{ get; set; }` đọc ghi, `{ get; }` chỉ
  đọc.
- Constructor trùng tên class, chạy khi `new`, gán giá trị ban đầu.
- Constructor có tham số bắt người tạo object phải truyền đủ dữ liệu.
- Trong constructor, vế trái là property (`Price`), vế phải là tham số
  (`price`).

```quiz
[
  {
    "prompt": "Class Customer có property public string Email { get; } và constructor gán Email. Dòng customer.Email = \"moi@shop.vn\"; thì sao?",
    "options": [
      "Chạy được, Email được đổi",
      "Lỗi khi chạy",
      "Chạy được nhưng Email không đổi",
      "Lỗi compile vì Email chỉ đọc"
    ],
    "answer": 4,
    "explain": "Property chỉ có get thì bên ngoài không gán được. Chỉ constructor mới gán được giá trị cho nó."
  },
  {
    "prompt": "Class Order chỉ có constructor Order(int id). Dòng var o = new Order(); thì sao?",
    "options": [
      "Lỗi compile vì phải truyền id",
      "Chạy được, id bằng 0",
      "Chạy được, id bằng null",
      "Lỗi khi chạy"
    ],
    "answer": 1,
    "explain": "Khi class đã có constructor nhận tham số, C# không tự tạo constructor không tham số nữa. Phải gọi new Order(5)."
  },
  {
    "prompt": "Property nào nên để { get; set; } thay vì { get; }?",
    "options": [
      "Mã đơn hàng, cấp một lần lúc tạo",
      "Số lượng tồn kho, thay đổi mỗi lần bán",
      "Ngày tạo tài khoản",
      "Mã số thuế của công ty"
    ],
    "answer": 2,
    "explain": "Tồn kho đổi liên tục nên cần set. Ba giá trị còn lại chốt từ lúc tạo, để get là đủ và an toàn hơn."
  }
]
```
