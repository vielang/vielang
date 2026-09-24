---
title: static
minutes: 5
---

Mỗi sản phẩm có tên và giá riêng. Nhưng thuế suất 10% thì chung cho mọi sản
phẩm, không thuộc về riêng cái nào. Những thứ dùng chung như vậy được khai báo
bằng `static`.

## Khái niệm

📌 **static**: từ khoá đánh dấu thành viên thuộc về chính class chứ không thuộc về object nào, gọi qua tên class.

Bạn đã dùng static từ bài đầu tiên mà không để ý. `Console.WriteLine` là một
static method: gọi thẳng qua tên class `Console`, không cần `new Console()`.

## Ví dụ

```csharp
decimal price = Tax.Apply(100000m);
Console.WriteLine(price);      // 110000
Console.WriteLine(Tax.Rate);   // 10

class Tax
{
    public static int Rate = 10;   // phần trăm

    public static decimal Apply(decimal price)
    {
        return price + price * Rate / 100;
    }
}
```

- `Tax.Rate` và `Tax.Apply(...)` gọi qua tên class, không tạo object.
- Static field chỉ có **một** bản, dùng chung cho cả chương trình.
- Static method không dùng trực tiếp được field hay property thường của
  class, vì nó không thuộc object nào.

So sánh nhanh:

| | Thành viên thường | Thành viên static |
|---|---|---|
| Thuộc về | từng object | class |
| Gọi qua | object: `pen.Price` | tên class: `Tax.Rate` |
| Số bản | mỗi object một bản | một bản duy nhất |

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
var a = new Product("Bút bi");
var b = new Product("Vở");
var c = new Product("Thước");

Console.WriteLine(Product.Count);

class Product
{
    public static int Count;
    public string Name { get; }

    public Product(string name)
    {
        Name = name;
        Count++;
    }
}
```

**Đoán trước khi chạy:** mỗi lần tạo object, `Count` tăng thêm 1.
`Product.Count` in ra 1 hay 3?

<details>
<summary>Xem kết quả</summary>

```text
3
```

Là 3. `Count` là static nên cả ba object dùng chung một biến đếm. Nếu bỏ
`static`, mỗi object sẽ có `Count` riêng, cái nào cũng bằng 1.

</details>

## Lỗi hay gặp

**Static method dùng thành viên thường.** Compiler không biết lấy `Price` của
object nào.

```csharp
// SAI — lỗi compile: Price thuộc về từng object
class Item
{
    public decimal Price { get; set; }

    public static decimal WithTax()
    {
        return Price * 1.1m;
    }
}
```

```csharp
// ĐÚNG — truyền giá trị vào qua tham số
class Item
{
    public decimal Price { get; set; }

    public static decimal WithTax(decimal price)
    {
        return price * 1.1m;
    }
}
```

**Gọi thành viên static qua object.** Static thuộc về class, nên phải gọi qua
tên class.

```csharp
// SAI — lỗi compile: Count phải gọi qua Product
var pen = new Product("Bút bi");
Console.WriteLine(pen.Count);
```

```csharp
// ĐÚNG
var pen = new Product("Bút bi");
Console.WriteLine(Product.Count);
```

## Tóm tắt

- `static` gắn thành viên với class, không với object.
- Gọi qua tên class: `Tax.Rate`, `Console.WriteLine`.
- Static field chỉ có một bản dùng chung.
- Static method không dùng trực tiếp được field và property thường.

```quiz
[
  {
    "prompt": "Math.Max(3, 7) được gọi mà không cần new Math(). Điều đó cho biết gì về Max?",
    "options": [
      "Max là property",
      "Max là static method",
      "Math là một biến",
      "Max là constructor"
    ],
    "answer": 2,
    "explain": "Gọi thẳng qua tên class mà không tạo object nghĩa là Max là thành viên static."
  },
  {
    "prompt": "Class Order có public static int NextId = 1; Tạo 5 object Order, mỗi constructor tăng NextId thêm 1. NextId cuối cùng là bao nhiêu?",
    "options": [
      "1",
      "5",
      "2",
      "6"
    ],
    "answer": 4,
    "explain": "NextId dùng chung một bản. Bắt đầu từ 1, tăng 5 lần thành 6."
  },
  {
    "prompt": "Thứ nào nên khai báo static?",
    "options": [
      "Phí vận chuyển cố định áp dụng cho mọi đơn hàng",
      "Tên của từng khách hàng",
      "Số lượng của từng dòng trong đơn",
      "Địa chỉ giao hàng của một đơn"
    ],
    "answer": 1,
    "explain": "Phí cố định dùng chung cho mọi đơn, không thuộc về đơn nào. Ba thứ còn lại khác nhau theo từng object."
  }
]
```
