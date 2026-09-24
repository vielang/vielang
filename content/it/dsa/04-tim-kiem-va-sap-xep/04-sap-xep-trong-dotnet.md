---
title: Sắp xếp trong .NET
minutes: 5
---

Hai bài trước tự viết thuật toán sắp xếp để hiểu bên trong. Khi đi làm, bạn
gần như luôn dùng hàm có sẵn của .NET vì nhanh, đã được kiểm kỹ và gọn. Điều
cần biết là nên chọn hàm nào, và hàm nào đổi luôn list gốc.

## Khái niệm

⚖️ **Sắp xếp ổn định (stable sort)**: các phần tử bằng nhau theo tiêu chí sắp xếp vẫn giữ đúng thứ tự ban đầu của chúng.

| Cách | Đổi list gốc? | Ổn định? |
|---|---|---|
| `list.Sort(...)`, `Array.Sort(...)` | có, sắp xếp tại chỗ | không |
| `OrderBy`, `ThenBy` của LINQ | không, trả về dãy mới | có |

Cả hai đều O(n log n). `OrderBy` đã gặp ở bài LINQ cơ bản của khoá C# Core.
Nó giống `ORDER BY` ở bài Sắp xếp và phân trang của khoá SQL.

## Ví dụ

```csharp
var products = new List<Product>
{
    new Product("Vở", 12000m),
    new Product("Bút bi", 5000m),
    new Product("Thước", 7000m)
};

products.Sort((a, b) => a.Price.CompareTo(b.Price));
foreach (Product p in products)
{
    Console.WriteLine($"{p.Name}: {p.Price}");
}

class Product
{
    public string Name { get; }
    public decimal Price { get; }

    public Product(string name, decimal price)
    {
        Name = name;
        Price = price;
    }
}
```

- `Sort` nhận lambda so hai phần tử `a`, `b`. Kết quả âm nghĩa là `a` đứng
  trước, dương là `b` đứng trước, 0 là bằng nhau.
- `a.Price.CompareTo(b.Price)` trả về đúng số âm, 0 hoặc dương như vậy.
- Muốn giảm dần thì đổi chỗ: `b.Price.CompareTo(a.Price)`.
- `Sort` đổi thứ tự ngay trong `products`, in ra `Bút bi`, `Thước`, `Vở`.

## Thử ngay

Hai món cùng giá 5000. Dùng `OrderBy` và xem món nào đứng trước:

```csharp
var items = new List<Product>
{
    new Product("Vở", 12000m),
    new Product("Bút chì", 5000m),
    new Product("Thước", 7000m),
    new Product("Bút bi", 5000m)
};

var byPrice = items.OrderBy(p => p.Price);
foreach (Product p in byPrice)
{
    Console.WriteLine($"{p.Name}: {p.Price}");
}
```

**Đoán trước khi chạy:** "Bút chì" và "Bút bi" cùng giá. Món nào in ra trước?

<details>
<summary>Xem kết quả</summary>

```text
Bút chì: 5000
Bút bi: 5000
Thước: 7000
Vở: 12000
```

`OrderBy` ổn định: hai món cùng giá giữ thứ tự trong list gốc, nên "Bút chì"
đứng trước "Bút bi". Muốn các món cùng giá xếp theo tên thì thêm
`.ThenBy(p => p.Name)`, giống `ORDER BY price, name` trong SQL.

</details>

## Lỗi hay gặp

**Gọi `OrderBy` rồi tưởng list đã đổi.** `OrderBy` trả về dãy mới, list gốc
giữ nguyên. Không dùng dãy trả về thì coi như chưa sắp xếp.

```csharp
// SAI — items vẫn giữ thứ tự cũ
items.OrderBy(p => p.Price);
Console.WriteLine(items[0].Name);   // Vở
```

```csharp
// ĐÚNG — dùng dãy mà OrderBy trả về
var sorted = items.OrderBy(p => p.Price).ToList();
Console.WriteLine(sorted[0].Name);   // Bút chì
```

## Tóm tắt

- `list.Sort` và `Array.Sort` sắp xếp tại chỗ, O(n log n), không ổn định.
- `OrderBy`, `ThenBy` trả về dãy mới và ổn định.
- Lambda so sánh trả số âm, 0 hoặc dương. `CompareTo` làm sẵn việc đó.
- `Sort` của .NET kết hợp nhiều thuật toán, dùng sắp xếp chèn cho đoạn ngắn.

```quiz
[
  {
    "prompt": "Cần sắp xếp đơn hàng theo ngày, đơn cùng ngày giữ nguyên thứ tự nhận được. Nên dùng gì?",
    "options": [
      "list.Sort với lambda so ngày",
      "OrderBy(o => o.Date), vì OrderBy ổn định",
      "Array.Sort",
      "HashSet"
    ],
    "answer": 2,
    "explain": "OrderBy là sắp xếp ổn định: đơn cùng ngày giữ thứ tự ban đầu. List.Sort không đảm bảo điều này."
  },
  {
    "prompt": "list.Sort((a, b) => b.Price.CompareTo(a.Price)) sắp xếp thế nào?",
    "options": [
      "Tăng dần theo giá",
      "Theo tên",
      "Không đổi gì",
      "Giảm dần theo giá"
    ],
    "answer": 4,
    "explain": "Đổi chỗ a và b khi so sánh là đảo chiều sắp xếp, thành giảm dần."
  },
  {
    "prompt": "Gọi products.OrderBy(p => p.Name); rồi in products[0]. Kết quả?",
    "options": [
      "Sản phẩm có tên đứng đầu theo bảng chữ cái",
      "Lỗi compile",
      "Phần tử đầu của list gốc, vì OrderBy không đổi list",
      "null"
    ],
    "answer": 3,
    "explain": "OrderBy trả về dãy mới. Kết quả không được gán vào đâu nên products giữ nguyên."
  }
]
```
