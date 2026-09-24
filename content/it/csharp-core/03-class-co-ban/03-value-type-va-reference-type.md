---
title: Value type và reference type
minutes: 5
---

Bạn chép giỏ hàng mẫu cho khách A, rồi thêm món vào giỏ của A. Mở giỏ mẫu ra,
món đó cũng có mặt. Không dòng nào gán nhầm. Lý do nằm ở cách C# chép hai loại
kiểu khác nhau.

## Khái niệm

🔢 **Value type**: kiểu mà biến chứa trực tiếp giá trị, gán sang biến khác là chép cả giá trị.

🔗 **Reference type**: kiểu mà biến chỉ chứa tham chiếu tới object, gán sang biến khác là chép tham chiếu, và hai biến cùng trỏ một object.

| Loại | Gồm những kiểu |
|---|---|
| Value type | `int`, `double`, `decimal`, `bool` |
| Reference type | mọi `class`, `string`, array, `List`, `Dictionary` |

## Ví dụ

```csharp
int a = 5;
int b = a;
b = 10;
Console.WriteLine(a);        // 5

var p1 = new Product { Price = 5000m };
var p2 = p1;
p2.Price = 9000m;
Console.WriteLine(p1.Price); // 9000

class Product
{
    public decimal Price { get; set; }
}
```

- `int b = a` chép số 5 sang `b`. Đổi `b` không đụng tới `a`.
- `var p2 = p1` chỉ chép tham chiếu. `p1` và `p2` cùng trỏ **một** object,
  sửa qua biến nào thì biến kia cũng thấy.
- `new Product { Price = 5000m }` tạo object rồi gán luôn property trong cặp
  `{ }`.
- `string` là reference type nhưng không sửa được, nên dùng như value type mà
  không gặp vấn đề này.

## Thử ngay

Chép class `Product` ở ví dụ trên, thay các dòng đầu bằng:

```csharp
int price = 5000;
var pen = new Product { Price = 5000m };

DiscountNumber(price);
DiscountProduct(pen);

Console.WriteLine(price);
Console.WriteLine(pen.Price);

void DiscountNumber(int value)
{
    value = value - 1000;
}

void DiscountProduct(Product product)
{
    product.Price = product.Price - 1000m;
}
```

**Đoán trước khi chạy:** hai method làm cùng một việc. Hai dòng in ra có cùng
bị giảm không?

<details>
<summary>Xem kết quả</summary>

```text
5000
4000
```

Không. Tham số `int` nhận bản chép của số, nên `price` bên ngoài giữ nguyên.
Tham số `Product` nhận bản chép của tham chiếu, vẫn trỏ đúng object `pen`, nên
sửa bên trong method thì `pen` đổi theo.

</details>

## Lỗi hay gặp

**Tưởng gán là tạo bản sao.** Hai giỏ hàng thành một.

```csharp
// SAI — cartA và sample là cùng một list
var sample = new List<string> { "Bút bi" };
var cartA = sample;
cartA.Add("Vở");
Console.WriteLine(sample.Count);   // 2
```

```csharp
// ĐÚNG — tạo list mới, chép phần tử sang
var sample = new List<string> { "Bút bi" };
var cartA = new List<string>(sample);
cartA.Add("Vở");
Console.WriteLine(sample.Count);   // 1
```

**So sánh hai object bằng `==`.** Với class, `==` so tham chiếu chứ không so
dữ liệu.

```csharp
// SAI — in False dù hai sản phẩm giống hệt nhau
var x = new Product { Price = 5000m };
var y = new Product { Price = 5000m };
Console.WriteLine(x == y);
```

```csharp
// ĐÚNG — so sánh đúng dữ liệu cần so
var x = new Product { Price = 5000m };
var y = new Product { Price = 5000m };
Console.WriteLine(x.Price == y.Price);
```

## Tóm tắt

- Value type chép giá trị, reference type chép tham chiếu.
- Hai biến cùng trỏ một object thì sửa qua biến này, biến kia thấy ngay.
- Truyền object vào method thì method sửa được chính object đó.
- Muốn bản sao độc lập thì phải tạo object mới.
- `==` giữa hai object class so tham chiếu, không so dữ liệu.

```quiz
[
  {
    "prompt": "var a = new List<int> { 1 }; var b = a; b.Add(2); Console.WriteLine(a.Count); In ra gì?",
    "options": [
      "1",
      "0",
      "2",
      "Lỗi compile"
    ],
    "answer": 3,
    "explain": "List là reference type. b = a làm hai biến cùng trỏ một list, nên thêm qua b thì a cũng có 2 phần tử."
  },
  {
    "prompt": "decimal total = 100m; void AddFee(decimal t) { t = t + 20m; } Gọi AddFee(total); rồi in total. Kết quả?",
    "options": [
      "100",
      "120",
      "20",
      "Lỗi compile"
    ],
    "answer": 1,
    "explain": "decimal là value type. Method nhận bản chép, nên total bên ngoài vẫn là 100."
  },
  {
    "prompt": "Kiểu nào là value type?",
    "options": [
      "string",
      "List<int>",
      "Một class Customer bạn tự viết",
      "bool"
    ],
    "answer": 4,
    "explain": "bool là value type. string, List và mọi class đều là reference type."
  }
]
```
