---
title: Array và List
minutes: 5
---

Một giỏ hàng có nhiều món, một cửa hàng có nhiều sản phẩm. Khi có hàng trăm món, không
thể tạo từng biến `price1`, `price2`, `price3`. Bài
này giới thiệu cách lưu nhiều giá trị trong một biến.

## Khái niệm

🗃️ **Array**: danh sách các phần tử cùng kiểu, số lượng cố định ngay khi tạo.

📋 **List**: danh sách các phần tử cùng kiểu, thêm và bớt phần tử được, viết là `List<kiểu>`.

🔁 **foreach**: vòng lặp đi qua lần lượt từng phần tử của một danh sách.

Phần tử được đánh số từ **0**. Phần tử đầu là `[0]`, phần tử cuối là
`[số lượng - 1]`.

## Ví dụ

```csharp
decimal[] prices = { 5000m, 12000m, 30000m };
Console.WriteLine(prices[0]);      // 5000
Console.WriteLine(prices.Length);  // 3

List<string> cart = new List<string>();
cart.Add("Bút bi");
cart.Add("Vở");
cart.Add("Thước");
cart.Remove("Vở");

Console.WriteLine(cart.Count);     // 2
Console.WriteLine(cart[1]);        // Thước
```

- `decimal[]` là array chứa `decimal`. Tạo xong thì luôn có đúng 3 phần tử.
- `List<string>` là list chứa `string`. `Add` thêm vào cuối, `Remove` xoá
  phần tử.
- Array đếm bằng `.Length`, list đếm bằng `.Count`.
- Trong thực tế, list được dùng nhiều hơn vì thêm bớt được.

## Duyệt bằng foreach

`foreach` lấy lần lượt từng phần tử, không cần biến đếm. List dưới đây được
điền sẵn phần tử trong cặp `{ }` ngay lúc khai báo:

```csharp
List<decimal> prices = new List<decimal>
{
    5000m, 12000m, 30000m
};

decimal total = 0;
foreach (decimal price in prices)
{
    total = total + price;
}

Console.WriteLine(total);   // 47000
```

Cần biết vị trí của phần tử thì dùng `for` với `prices[i]`. Chỉ cần giá trị
thì dùng `foreach`.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
List<string> cart = new List<string>();
cart.Add("Bút bi");
cart.Add("Vở");
cart.Add("Bút bi");

Console.WriteLine(cart.Count);
Console.WriteLine(cart.Contains("Thước"));

foreach (string item in cart)
{
    Console.WriteLine($"- {item}");
}
```

**Đoán trước khi chạy:** thêm "Bút bi" hai lần thì `cart.Count` là 2 hay 3?

<details>
<summary>Xem kết quả</summary>

```text
3
False
- Bút bi
- Vở
- Bút bi
```

Là 3. List giữ nguyên mọi phần tử được thêm vào, kể cả phần tử trùng, và giữ
đúng thứ tự thêm.

</details>

## Lỗi hay gặp

**Truy cập ngoài phạm vi.** List có 3 phần tử thì vị trí cuối là `[2]`, không
phải `[3]`.

```csharp
// SAI — lỗi khi chạy: không có phần tử [3]
List<string> cart = new List<string>
{
    "Bút bi", "Vở", "Thước"
};
Console.WriteLine(cart[3]);
```

```csharp
// ĐÚNG — phần tử cuối là [Count - 1]
List<string> cart = new List<string>
{
    "Bút bi", "Vở", "Thước"
};
Console.WriteLine(cart[cart.Count - 1]);
```

**Thêm phần tử vào array.** Array cố định số lượng, không có `Add`.

```csharp
// SAI — lỗi compile: array không có Add
string[] tags = { "sale", "new" };
tags.Add("hot");
```

```csharp
// ĐÚNG — cần thêm bớt thì dùng List
List<string> tags = new List<string> { "sale", "new" };
tags.Add("hot");
```

## Tóm tắt

- Array cố định số lượng, list thêm bớt được. Thường dùng list.
- Vị trí đánh số từ 0, phần tử cuối là `[Count - 1]`.
- `Add`, `Remove`, `Contains`, `Count` là những thao tác hay dùng của list.
- `foreach` duyệt từng phần tử khi không cần biết vị trí.

```quiz
[
  {
    "prompt": "int[] scores = { 8, 9, 10 }; Console.WriteLine(scores[1]); In ra gì?",
    "options": [
      "8",
      "10",
      "Lỗi khi chạy",
      "9"
    ],
    "answer": 4,
    "explain": "Vị trí đánh số từ 0: scores[0] là 8, scores[1] là 9."
  },
  {
    "prompt": "Bạn cần lưu danh sách món khách thêm vào giỏ, số món thay đổi liên tục. Nên dùng gì?",
    "options": [
      "Array, vì nhanh hơn",
      "List, vì thêm bớt phần tử được",
      "Nhiều biến riêng item1, item2, item3",
      "string, nối các món bằng dấu phẩy"
    ],
    "answer": 2,
    "explain": "Số món thay đổi nên cần thêm bớt được. Array cố định số lượng ngay khi tạo."
  },
  {
    "prompt": "List<int> ids có 5 phần tử. Vòng lặp for nào duyệt đúng hết, không lỗi?",
    "options": [
      "for (int i = 0; i < ids.Count; i++)",
      "for (int i = 1; i <= ids.Count; i++)",
      "for (int i = 0; i <= ids.Count; i++)",
      "for (int i = 1; i < ids.Count; i++)"
    ],
    "answer": 1,
    "explain": "Vị trí hợp lệ là 0 đến Count - 1. Bắt đầu từ 0 và lặp khi i < Count là duyệt vừa đủ."
  }
]
```
