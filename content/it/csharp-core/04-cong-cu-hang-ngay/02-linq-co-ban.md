---
title: LINQ cơ bản
minutes: 5
---

Lọc sản phẩm còn hàng, sắp xếp theo giá, lấy tên, tính tổng tiền. Viết bằng
`foreach` thì mỗi việc tốn năm sáu dòng. LINQ làm từng việc đó trong một dòng,
dùng lambda của bài trước.

## Khái niệm

🔎 **LINQ**: bộ method có sẵn để lọc, sắp xếp, biến đổi và tính toán trên collection, nhận lambda làm điều kiện.

## Ví dụ

```csharp
var products = new List<Product>
{
    new Product("Bút bi", 5000m, 120),
    new Product("Vở", 12000m, 0),
    new Product("Balo", 350000m, 8),
};

List<string> names = products
    .Where(p => p.Stock > 0)
    .OrderBy(p => p.Price)
    .Select(p => p.Name)
    .ToList();

Console.WriteLine(string.Join(", ", names));
// Bút bi, Balo

class Product
{
    public string Name { get; }
    public decimal Price { get; }
    public int Stock { get; }

    public Product(
        string name, decimal price, int stock)
    {
        Name = name;
        Price = price;
        Stock = stock;
    }
}
```

- Các method nối nhau bằng dấu chấm, đọc từ trên xuống như một câu: lọc còn
  hàng, xếp theo giá, lấy tên.
- `ToList()` gom kết quả thành `List`.
- `string.Join(", ", names)` ghép các phần tử thành một chuỗi.

Những method dùng nhiều nhất:

| Method | Làm gì |
|---|---|
| `Where(p => ...)` | lọc theo điều kiện |
| `Select(p => ...)` | biến mỗi phần tử thành giá trị khác |
| `OrderBy`, `OrderByDescending` | sắp xếp tăng, giảm |
| `First`, `FirstOrDefault` | lấy phần tử đầu tiên thoả điều kiện |
| `Any(p => ...)` | có phần tử nào thoả không (`bool`) |
| `Count`, `Sum`, `Max`, `Min` | đếm, cộng, lớn nhất, nhỏ nhất |

## Thử ngay

Chép ví dụ trên vào `Program.cs`, thay đoạn từ `List<string> names` tới dòng
comment bằng:

```csharp
int inStock = products.Count(p => p.Stock > 0);
decimal stockValue = products
    .Sum(p => p.Price * p.Stock);
bool hasCheap = products.Any(p => p.Price < 1000m);

Console.WriteLine(inStock);
Console.WriteLine(stockValue);
Console.WriteLine(hasCheap);
```

**Đoán trước khi chạy:** `inStock` là 2 hay 3?

<details>
<summary>Xem kết quả</summary>

```text
2
3400000
False
```

Là 2, vì "Vở" có `Stock` bằng 0. Tổng giá trị kho là
`5000 × 120 + 12000 × 0 + 350000 × 8`.

</details>

## Lỗi hay gặp

**Tưởng `Where` sửa list gốc.** LINQ không đổi collection ban đầu mà trả về
kết quả mới.

```csharp
// SAI — products vẫn đủ 3 món
products.Where(p => p.Stock > 0);
Console.WriteLine(products.Count);
```

```csharp
// ĐÚNG — lưu kết quả vào biến mới
var available = products
    .Where(p => p.Stock > 0)
    .ToList();
Console.WriteLine(available.Count);
```

**`First` khi không có phần tử nào khớp.** Chương trình dừng với
`InvalidOperationException`.

```csharp
// SAI — lỗi khi chạy: không có món nào trên 1 triệu
var item = products.First(p => p.Price > 1000000m);
```

```csharp
// ĐÚNG — không có thì nhận null
var item = products
    .FirstOrDefault(p => p.Price > 1000000m);
Console.WriteLine(item?.Name ?? "Không có");
```

## Tóm tắt

- LINQ gồm các method như `Where`, `Select`, `OrderBy`, `Sum`, nhận lambda.
- Nối nhiều method bằng dấu chấm, `ToList()` để lấy kết quả thành list.
- LINQ không sửa collection gốc, phải lưu kết quả vào biến.
- Có thể không tìm thấy thì dùng `FirstOrDefault`, rồi kiểm tra `null`.

```quiz
[
  {
    "prompt": "var nums = new List<int> { 5, 2, 8 }; var r = nums.OrderBy(n => n).First(); Giá trị của r là gì?",
    "options": [
      "5",
      "8",
      "2",
      "Lỗi khi chạy"
    ],
    "answer": 3,
    "explain": "OrderBy xếp tăng dần thành 2, 5, 8. First lấy phần tử đầu là 2."
  },
  {
    "prompt": "Bạn cần danh sách email của mọi khách hàng trong List<Customer> customers. Dùng method nào?",
    "options": [
      "customers.Select(c => c.Email).ToList()",
      "customers.Where(c => c.Email).ToList()",
      "customers.Any(c => c.Email)",
      "customers.Count(c => c.Email)"
    ],
    "answer": 1,
    "explain": "Select biến mỗi khách hàng thành email của họ. Where dùng để lọc theo điều kiện bool."
  },
  {
    "prompt": "Chỉ cần biết giỏ hàng có món nào hết hàng hay không. Cách nào hợp lý nhất?",
    "options": [
      "cart.Where(i => i.Stock == 0).ToList()",
      "cart.First(i => i.Stock == 0)",
      "cart.Sum(i => i.Stock)",
      "cart.Any(i => i.Stock == 0)"
    ],
    "answer": 4,
    "explain": "Any trả về true/false và dừng ngay khi gặp phần tử khớp. First sẽ lỗi nếu không có món nào hết hàng."
  }
]
```
