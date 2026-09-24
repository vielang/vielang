---
title: Lambda
minutes: 5
---

Lọc sản phẩm đắt hơn 100.000đ, sắp xếp theo giá, tìm món đầu tiên còn hàng:
mỗi việc cần truyền một *điều kiện* vào method. Viết hẳn một method có tên cho
mỗi điều kiện thì quá dài. Lambda cho phép viết điều kiện đó ngay tại chỗ.

## Khái niệm

➡️ **Lambda**: một method không tên viết gọn theo dạng `tham số => kết quả`, dùng để truyền vào method khác hoặc gán vào biến.

📮 **Func**: kiểu dùng để lưu lambda có trả về, viết là `Func<kiểu vào, kiểu ra>`.

`p => p > 100000` đọc là: "nhận `p`, trả về `p > 100000`".

## Ví dụ

```csharp
Func<decimal, decimal> addVat = price => price * 1.1m;
Console.WriteLine(addVat(100m));   // 110.0

List<decimal> prices = new List<decimal>
{
    5000m, 150000m, 30000m, 200000m
};

List<decimal> expensive =
    prices.FindAll(p => p > 100000);

foreach (decimal p in expensive)
{
    Console.WriteLine(p);   // 150000, 200000
}
```

- `Func<decimal, decimal>`: nhận `decimal`, trả về `decimal`. Kiểu cuối cùng
  luôn là kiểu trả về.
- `addVat(100m)` gọi lambda giống gọi method.
- `prices.FindAll(p => p > 100000)` truyền điều kiện vào `FindAll`.
  `FindAll` tự chạy lambda với từng phần tử và giữ lại phần tử cho kết quả
  `true`.

## Method viết gọn bằng =>

Method chỉ có một dòng `return` cũng viết được bằng `=>`:

```csharp
decimal Total(decimal price, int quantity) =>
    price * quantity;

Console.WriteLine(Total(5000m, 3));   // 15000
```

Viết như vậy tương đương với `{ return price * quantity; }`. Bạn sẽ gặp cách
viết này rất nhiều trong code thực tế.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
Func<int, int> twice = x => x * 2;
Func<int, bool> isEven = x => x % 2 == 0;

Console.WriteLine(twice(twice(3)));
Console.WriteLine(isEven(7));
```

**Đoán trước khi chạy:** `twice(twice(3))` ra 6 hay số khác?

<details>
<summary>Xem kết quả</summary>

```text
12
False
```

`twice(3)` ra 6, rồi `twice(6)` ra 12. Lambda gọi lồng nhau như method bình
thường.

</details>

## Lỗi hay gặp

**Dùng `var` cho lambda mà không có kiểu tham số.** Compiler không biết `x` là
kiểu gì.

```csharp
// SAI — lỗi compile: không suy ra được kiểu của x
var triple = x => x * 3;
```

```csharp
// ĐÚNG — khai báo rõ kiểu bằng Func
Func<int, int> triple = x => x * 3;
```

**Có ngoặc `{ }` mà quên `return`.** Lambda nhiều dòng phải `return` như
method.

```csharp
// SAI — lỗi compile: thiếu return
Func<int, int> discount = x => { x - 1000; };
```

```csharp
// ĐÚNG
Func<int, int> discount = x => { return x - 1000; };
```

## Tóm tắt

- Lambda là method không tên: `tham số => kết quả`.
- Lưu lambda bằng `Func<vào, ra>`, kiểu cuối là kiểu trả về.
- Lambda thường được truyền vào method khác làm điều kiện, như `FindAll`.
- Method một dòng viết gọn được bằng `=>`.

```quiz
[
  {
    "prompt": "Func<string, int> f = s => s.Length; Console.WriteLine(f(\"shop\")); In ra gì?",
    "options": [
      "shop",
      "4",
      "Lỗi compile",
      "0"
    ],
    "answer": 2,
    "explain": "f nhận chuỗi và trả về độ dài. \"shop\" có 4 ký tự."
  },
  {
    "prompt": "Khai báo nào đúng cho một lambda nhận decimal và trả về bool?",
    "options": [
      "Func<bool, decimal> check = p => p > 0;",
      "var check = p => p > 0;",
      "Func<decimal> check = p => p > 0;",
      "Func<decimal, bool> check = p => p > 0;"
    ],
    "answer": 4,
    "explain": "Func liệt kê kiểu vào trước, kiểu trả về đứng cuối: Func<decimal, bool>."
  },
  {
    "prompt": "names là List<string>. names.FindAll(n => n.StartsWith(\"A\")) trả về gì?",
    "options": [
      "Một list mới gồm các tên bắt đầu bằng A",
      "true nếu có tên bắt đầu bằng A",
      "Tên đầu tiên bắt đầu bằng A",
      "names bị xoá các tên không bắt đầu bằng A"
    ],
    "answer": 1,
    "explain": "FindAll chạy lambda với từng phần tử và trả về list mới gồm các phần tử cho kết quả true. List gốc không đổi."
  }
]
```
