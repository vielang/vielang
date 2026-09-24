---
title: Đa hình
minutes: 5
---

Giỏ hàng có cả sách giấy lẫn ebook. Sách giấy tính phí ship theo cân nặng,
ebook thì miễn phí. Viết `if` kiểm tra từng loại thì mỗi lần thêm loại hàng
mới lại phải sửa giỏ hàng. Với đa hình, mỗi loại tự biết cách tính của mình.

## Khái niệm

🎭 **Đa hình (polymorphism)**: cùng một lời gọi method, mỗi class con chạy cách làm riêng của nó.

🔧 **virtual và override**: `virtual` ở class cha cho phép class con viết lại method, `override` ở class con là bản viết lại đó.

## Ví dụ

```csharp
var cart = new List<Product>
{
    new PhysicalProduct { Name = "Sách", Grams = 800 },
    new DigitalProduct { Name = "Ebook C#" },
};

foreach (Product item in cart)
{
    Console.WriteLine(
        $"{item.Name}: {item.ShippingFee()}");
}

class Product
{
    public string Name { get; set; } = "";

    public virtual decimal ShippingFee() => 30000m;
}

class PhysicalProduct : Product
{
    public int Grams { get; set; }

    public override decimal ShippingFee() =>
        Grams * 10m;
}

class DigitalProduct : Product
{
    public override decimal ShippingFee() => 0m;
}
```

```text
Sách: 8000
Ebook C#: 0
```

- `List<Product>` chứa được cả `PhysicalProduct` và `DigitalProduct`, vì
  cả hai đều là `Product`.
- Vòng lặp chỉ gọi `item.ShippingFee()`, không cần biết món hàng thuộc loại
  nào.
- C# chạy bản `override` của object thật, không phải bản của `Product`.
- Thêm loại hàng mới chỉ cần viết class mới có `override`. Vòng lặp không đổi.

## Thử ngay

Chép ba class ở ví dụ trên, thay các dòng đầu bằng:

```csharp
Product item = new DigitalProduct { Name = "Ebook" };
Console.WriteLine(item.ShippingFee());

Product other = new Product { Name = "Quà tặng" };
Console.WriteLine(other.ShippingFee());
```

**Đoán trước khi chạy:** biến `item` có kiểu `Product`. Dòng đầu in 30000
hay 0?

<details>
<summary>Xem kết quả</summary>

```text
0
30000
```

In 0. Kiểu của biến là `Product`, nhưng object thật là `DigitalProduct`, và
C# chạy bản `override` của object thật. Còn `other` là `Product` thường nên
chạy bản gốc, in 30000.

</details>

## Lỗi hay gặp

**Quên `virtual` ở class cha.** Method không có `virtual` thì class con không
`override` được.

```csharp
// SAI — lỗi compile: Fee không phải virtual
class Item
{
    public decimal Fee() => 30000m;
}

class Gift : Item
{
    public override decimal Fee() => 0m;
}
```

```csharp
// ĐÚNG
class Item
{
    public virtual decimal Fee() => 30000m;
}

class Gift : Item
{
    public override decimal Fee() => 0m;
}
```

**Quên `override` ở class con.** Code vẫn build, chỉ có cảnh báo. Nhưng gọi
qua biến kiểu cha thì chạy bản của cha.

```csharp
// SAI — thiếu override, in ra 30000
Item gift = new Gift();
Console.WriteLine(gift.Fee());

class Item
{
    public virtual decimal Fee() => 30000m;
}

class Gift : Item
{
    public decimal Fee() => 0m;
}
```

Thêm `override` vào method của `Gift` thì dòng trên in ra 0 như mong muốn.

## Tóm tắt

- Đa hình: cùng một lời gọi, mỗi class con chạy cách làm riêng.
- Class cha đánh dấu `virtual`, class con viết lại bằng `override`.
- Biến kiểu cha chứa được object của class con, và C# chạy bản của object
  thật.
- Thêm loại mới chỉ cần thêm class, không sửa chỗ gọi.

```quiz
[
  {
    "prompt": "class Animal { public virtual string Sound() => \"...\"; } class Dog : Animal { public override string Sound() => \"Gâu\"; } Animal a = new Dog(); Console.WriteLine(a.Sound()); In ra gì?",
    "options": [
      "...",
      "Gâu",
      "Lỗi compile",
      "Lỗi khi chạy"
    ],
    "answer": 2,
    "explain": "Object thật là Dog, và Dog override Sound. C# chạy bản của object thật, không theo kiểu của biến."
  },
  {
    "prompt": "Class cha có method public decimal Discount(). Class con viết public override decimal Discount(). Chuyện gì xảy ra?",
    "options": [
      "Chạy được, class con dùng bản mới",
      "Chạy được nhưng luôn dùng bản của cha",
      "Lỗi khi chạy",
      "Lỗi compile vì method của cha thiếu virtual"
    ],
    "answer": 4,
    "explain": "Chỉ override được method có virtual (hoặc abstract) ở class cha."
  },
  {
    "prompt": "Cửa hàng thêm loại hàng mới là \"voucher\" với phí ship riêng. Nếu đã dùng đa hình, cần làm gì?",
    "options": [
      "Thêm class Voucher : Product với override ShippingFee",
      "Thêm một nhánh if trong vòng lặp giỏ hàng",
      "Sửa method ShippingFee của Product",
      "Tạo một List<Voucher> riêng và vòng lặp riêng"
    ],
    "answer": 1,
    "explain": "Với đa hình, loại mới tự mang cách tính của nó. Vòng lặp giỏ hàng và class Product không phải sửa."
  }
]
```
