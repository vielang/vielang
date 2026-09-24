---
title: Kế thừa
minutes: 5
---

Cửa hàng bán sách giấy và ebook. Cả hai đều có tên, có giá, có cùng cách hiển
thị. Viết hai class riêng thì phần chung bị lặp lại, sửa một chỗ lại phải nhớ
sửa chỗ kia. Kế thừa giải quyết đúng việc này.

## Khái niệm

🧬 **Kế thừa (inheritance)**: class con nhận lại property và method của class cha, rồi thêm phần riêng của mình.

👪 **Class cha, class con (base class, derived class)**: class được kế thừa là class cha, class kế thừa nó là class con, khai báo bằng `class Con : Cha`.

Kế thừa chỉ đúng khi class con **là một** loại của class cha. Sách giấy là một
loại sản phẩm, nên `PhysicalProduct : Product` hợp lý.

## Ví dụ

```csharp
var book = new PhysicalProduct("Sách C#", 150000, 800);
Console.WriteLine(book.Describe());
Console.WriteLine(book.ShippingFee());

class Product
{
    public string Name { get; }
    public decimal Price { get; }

    public Product(string name, decimal price)
    {
        Name = name;
        Price = price;
    }

    public string Describe()
    {
        return $"{Name} - {Price}đ";
    }
}

class PhysicalProduct : Product
{
    public int WeightGram { get; }

    public PhysicalProduct(
        string name, decimal price, int weightGram)
        : base(name, price)
    {
        WeightGram = weightGram;
    }

    public decimal ShippingFee()
    {
        return WeightGram * 10;
    }
}
```

- `book` dùng được `Name`, `Price`, `Describe()` dù `PhysicalProduct` không
  khai báo chúng. Tất cả nhận lại từ `Product`.
- `WeightGram` và `ShippingFee()` là phần riêng, chỉ class con có.
- Constructor không được kế thừa. `: base(name, price)` gọi constructor của
  `Product` để gán `Name` và `Price`, rồi class con gán tiếp `WeightGram`.
- Một class chỉ kế thừa được **một** class cha.

```mermaid PhysicalProduct nhận lại phần của Product, chỉ khai báo thêm phần riêng
classDiagram
    Product <|-- PhysicalProduct
    class Product {
        +string Name
        +decimal Price
        +Describe() string
    }
    class PhysicalProduct {
        +int WeightGram
        +ShippingFee() decimal
    }
```

## Thử ngay

Chép ví dụ trên vào `Program.cs`. Thay ba dòng đầu bằng các lệnh gọi dưới
đây, rồi thêm class `DigitalProduct` vào cuối file:

```csharp
var ebook = new DigitalProduct(
    "Ebook C#", 90000, "https://shop.vn/ebook");

Console.WriteLine(ebook.Describe());
Console.WriteLine(ebook.DownloadUrl);

class DigitalProduct : Product
{
    public string DownloadUrl { get; }

    public DigitalProduct(
        string name, decimal price, string downloadUrl)
        : base(name, price)
    {
        DownloadUrl = downloadUrl;
    }
}
```

**Đoán trước khi chạy:** `DigitalProduct` không hề khai báo `Describe()`. Dòng
`ebook.Describe()` có chạy được không, và in ra gì?

<details>
<summary>Xem kết quả</summary>

```text
Ebook C# - 90000đ
https://shop.vn/ebook
```

Chạy được. `DigitalProduct` kế thừa `Describe()` từ `Product`, nên phần chung
chỉ viết một lần ở class cha.

</details>

## Lỗi hay gặp

**Quên gọi `base(...)`.** Class cha không có constructor rỗng thì compiler
không biết tạo phần cha bằng cách nào.

```csharp
// SAI — lỗi compile: thiếu base(name, price)
class GiftCard : Product
{
    public GiftCard(string name, decimal price)
    {
    }
}
```

```csharp
// ĐÚNG
class GiftCard : Product
{
    public GiftCard(string name, decimal price)
        : base(name, price)
    {
    }
}
```

**Kế thừa chỉ để dùng lại code.** Khách hàng không phải là một địa chỉ, nên
`Customer : Address` là sai, dù làm vậy giúp `Customer` có sẵn `Street`.

```csharp
// SAI — khách hàng không "là một" địa chỉ
class Customer : Address
{
    public string Name { get; set; } = "";
}

class Address
{
    public string Street { get; set; } = "";
}
```

```csharp
// ĐÚNG — khách hàng "có một" địa chỉ
class Customer
{
    public string Name { get; set; } = "";
    public Address Address { get; set; }
        = new Address();
}
```

## Tóm tắt

- `class Con : Cha`: class con nhận lại property và method của class cha, rồi
  thêm phần riêng.
- Constructor không được kế thừa. Class con gọi constructor của cha bằng
  `: base(...)`.
- Một class chỉ có một class cha.
- Chỉ kế thừa khi con **là một** loại của cha.

```quiz
[
  {
    "prompt": "Class Employee có method GetInfo(). Class Manager : Employee không khai báo gì thêm. Gọi new Manager(...).GetInfo() thì sao?",
    "options": [
      "Lỗi compile vì Manager không có GetInfo",
      "Lỗi khi chạy",
      "Chạy được, dùng GetInfo kế thừa từ Employee",
      "Chạy được nhưng trả về rỗng"
    ],
    "answer": 3,
    "explain": "Manager kế thừa mọi method public của Employee, nên GetInfo có sẵn mà không cần viết lại."
  },
  {
    "prompt": "Class Vehicle chỉ có constructor Vehicle(string plate). Class Truck : Vehicle viết constructor Truck(string plate) { } mà không có : base(plate). Chuyện gì xảy ra?",
    "options": [
      "Lỗi compile: Vehicle không có constructor rỗng để gọi",
      "Chạy được, plate tự được truyền lên class cha",
      "Chạy được, plate của Vehicle là null",
      "Lỗi khi chạy lúc new Truck(...)"
    ],
    "answer": 1,
    "explain": "Không ghi base(...) thì compiler tìm constructor rỗng của Vehicle. Không có thì báo lỗi. Phải viết : base(plate)."
  },
  {
    "prompt": "Trường hợp nào dùng kế thừa là hợp lý?",
    "options": [
      "Order : Customer, để đơn hàng có sẵn tên khách",
      "SavingsAccount : BankAccount",
      "Invoice : List<string>, để hoá đơn có sẵn Add",
      "Car : Engine, để xe có sẵn công suất"
    ],
    "answer": 2,
    "explain": "Tài khoản tiết kiệm là một loại tài khoản ngân hàng. Các trường hợp còn lại là quan hệ \"có một\", nên đặt object kia làm property thay vì kế thừa."
  }
]
```
