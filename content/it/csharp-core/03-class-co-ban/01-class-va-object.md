---
title: Class và object
minutes: 4
---

Tới giờ bạn mới dùng các kiểu có sẵn như `int`, `string`, `List<T>`. Nhưng
một sản phẩm trong cửa hàng có cả tên, giá và cách tính tiền, không kiểu có
sẵn nào chứa hết được. Bài này hướng dẫn bạn tự tạo kiểu riêng bằng class.

## Khái niệm

📐 **Class**: kiểu dữ liệu do bạn tự định nghĩa, gom dữ liệu và hành vi liên quan vào một chỗ.

📦 **Object**: một giá trị cụ thể được tạo ra từ class bằng từ khoá `new`.

🏷️ **Field**: biến khai báo bên trong class, mỗi object giữ một bản riêng.

Class chỉ là phần mô tả. Muốn dùng thì phải tạo object từ nó, và từ một
class tạo bao nhiêu object cũng được.

## Ví dụ

```csharp
var pen = new Product();
pen.Name = "Bút bi";
pen.Price = 5000;

Console.WriteLine(pen.TotalFor(2)); // 10000

class Product
{
    public string Name = "";   // mặc định là chuỗi rỗng
    public decimal Price;

    public decimal TotalFor(int quantity)
    {
        return Price * quantity;
    }
}
```

- `class Product` khai báo hai field (`Name`, `Price`) và một method
  (`TotalFor`).
- `public` cho phép code bên ngoài class dùng field và method đó. Bài Đóng
  gói của khoá OOP sẽ nói kỹ.
- `new Product()` tạo một object, biến `pen` giữ object đó.
- Dùng dấu chấm để truy cập field và method của object: `pen.Price`,
  `pen.TotalFor(2)`. Bên trong `TotalFor`, `Price` chính là giá của `pen`.

## Thử ngay

Chép ví dụ trên vào `Program.cs`, thay bốn dòng đầu bằng đoạn sau rồi chạy
`dotnet run`:

```csharp
var pen = new Product();
pen.Name = "Bút bi";
pen.Price = 5000;

var notebook = new Product();
notebook.Name = "Vở";
notebook.Price = 12000;

pen.Price = 6000;

Console.WriteLine($"{pen.Name}: {pen.Price}");
Console.WriteLine($"{notebook.Name}: {notebook.Price}");
```

**Đoán trước khi chạy:** đổi giá `pen` thành 6000 thì giá `notebook` có đổi
theo không?

<details>
<summary>Xem kết quả</summary>

```text
Bút bi: 6000
Vở: 12000
```

Không đổi. `pen` và `notebook` là hai object riêng, mỗi object giữ `Price` của
mình.

</details>

## Lỗi hay gặp

**Khai báo biến nhưng quên `new`.** Biến chưa trỏ tới object nào nên không
dùng được.

```csharp
// SAI — lỗi compile: biến chưa được gán
Product pen;
pen.Price = 5000;
```

```csharp
// ĐÚNG — tạo object trước rồi mới dùng
var pen = new Product();
pen.Price = 5000;
```

## Tóm tắt

- Class mô tả dữ liệu (field) và hành vi (method).
- `new` tạo object từ class, mỗi object giữ field riêng.
- Dùng dấu chấm để truy cập field và method: `pen.Price`, `pen.TotalFor(2)`.

```quiz
[
  {
    "prompt": "Class Customer có field public string Name = \"\". Chạy: var a = new Customer(); var b = new Customer(); a.Name = \"An\"; Console.WriteLine(b.Name); In ra gì?",
    "options": [
      "An",
      "Chuỗi rỗng",
      "null",
      "Lỗi compile"
    ],
    "answer": 2,
    "explain": "a và b là hai object riêng. Gán Name cho a không đụng tới b, nên b.Name vẫn là giá trị khởi tạo \"\"."
  },
  {
    "prompt": "Trong dòng var cart = new Cart(); thì Cart và cart lần lượt là gì?",
    "options": [
      "Cả hai đều là class",
      "Cart là object, cart là class",
      "Cart là class, cart là biến giữ object vừa tạo",
      "Cả hai đều là object"
    ],
    "answer": 3,
    "explain": "Cart là tên class. new Cart() tạo một object, và biến cart giữ object đó."
  },
  {
    "prompt": "Field khác biến khai báo bên trong method ở điểm nào?",
    "options": [
      "Field thuộc về object, mỗi object giữ một bản riêng",
      "Field chỉ chứa được số",
      "Field không cần khai báo kiểu",
      "Field dùng chung cho mọi object của class"
    ],
    "answer": 1,
    "explain": "Field nằm trong object và tồn tại cùng object. Biến trong method chỉ sống trong lúc method chạy."
  }
]
```
