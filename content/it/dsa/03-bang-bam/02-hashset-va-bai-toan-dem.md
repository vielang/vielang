---
title: HashSet và bài toán đếm
minutes: 6
---

Rất nhiều bài toán hằng ngày chỉ là đếm và kiểm tra trùng: mỗi sản phẩm bán
được bao nhiêu cái, có bao nhiêu khách khác nhau, có hai món nào vừa đúng ngân
sách không. Làm bằng hai vòng lặp lồng nhau là O(n²). Hash table đưa chúng về
O(n).

## Khái niệm

🎯 **HashSet**: tập hợp các phần tử không trùng nhau, bên trong là hash table nên `Add`, `Contains`, `Remove` trung bình là O(1).

`HashSet<T>` giống `Dictionary` chỉ có key mà không có value. `Add` trả về
`false` nếu phần tử đã có sẵn, nên kiểm trùng và thêm vào làm cùng một lúc.

## Ví dụ

Đếm số lần mỗi sản phẩm được bán, dùng `Dictionary` từ tên sang số lượng:

```csharp
List<string> sold = new List<string>
{
    "Bút bi", "Vở", "Bút bi", "Thước", "Bút bi", "Vở"
};

var counts = new Dictionary<string, int>();
foreach (string name in sold)
{
    if (counts.ContainsKey(name))
    {
        counts[name] = counts[name] + 1;
    }
    else
    {
        counts[name] = 1;
    }
}

foreach (var item in counts)
{
    Console.WriteLine($"{item.Key}: {item.Value}");
}
```

- Duyệt list đúng một lần, mỗi lần tra và cập nhật dictionary là O(1). Tổng
  cộng O(n).
- Đây là việc câu `GROUP BY name` kèm `COUNT(*)` làm ở bài GROUP BY và
  HAVING của khoá SQL.

## Tìm hai món vừa đủ ngân sách

Khách có 19.000đ, muốn mua đúng hai món tiêu hết số tiền đó. Với mỗi giá,
món còn thiếu phải có giá `budget - price`. Hỏi `HashSet` xem đã gặp giá đó
chưa, thay vì so với mọi giá khác.

```csharp
List<decimal> prices = new List<decimal>
{
    5000m, 12000m, 350000m, 7000m
};
decimal budget = 19000m;

var seen = new HashSet<decimal>();
foreach (decimal price in prices)
{
    decimal need = budget - price;
    if (seen.Contains(need))
    {
        Console.WriteLine($"{need} + {price}");
    }
    seen.Add(price);
}
```

- Gặp 7000, cần thêm 12000, mà 12000 đã có trong `seen`. In ra
  `12000 + 7000`.
- Một vòng lặp, mỗi bước O(1), nên O(n). Hai vòng lồng nhau so từng cặp sẽ là
  O(n²).

## Thử ngay

Đếm số khách khác nhau đã đặt hàng hôm nay:

```csharp
var customers = new HashSet<string>();
Console.WriteLine(customers.Add("An"));
Console.WriteLine(customers.Add("Bình"));
Console.WriteLine(customers.Add("An"));
Console.WriteLine(customers.Count);
```

**Đoán trước khi chạy:** bốn dòng in ra là gì?

<details>
<summary>Xem kết quả</summary>

```text
True
True
False
2
```

"An" và "Bình" là lần đầu nên `Add` trả `True`. "An" lần hai đã có sẵn, `Add`
trả `False` và không thêm. Tập chỉ có 2 khách.

</details>

## Lỗi hay gặp

**Lấy phần tử của `HashSet` theo vị trí.** Hash table xếp phần tử theo ô của
hàm băm, không theo thứ tự thêm vào, nên không có `[i]`.

```csharp
// SAI — lỗi compile: HashSet không có [i]
var customers = new HashSet<string>();
customers.Add("An");
Console.WriteLine(customers[0]);
```

```csharp
// ĐÚNG — cần hỏi "có hay không" thì dùng Contains
var customers = new HashSet<string>();
customers.Add("An");
Console.WriteLine(customers.Contains("An"));
```

## Tóm tắt

- `HashSet<T>` giữ phần tử không trùng, `Add` trả `false` nếu đã có.
- Đếm số lần xuất hiện bằng `Dictionary` từ phần tử sang số đếm.
- Tìm cặp có tổng cho trước: với mỗi phần tử, hỏi `HashSet` phần còn thiếu.
- Các bài này từ O(n²) xuống O(n) nhờ tra hash table là O(1).

```quiz
[
  {
    "prompt": "Cần biết có bao nhiêu mã giảm giá khác nhau trong 10.000 đơn hàng. Cách nào hợp nhất?",
    "options": [
      "Thêm từng mã vào HashSet rồi đọc Count",
      "Hai vòng for so từng cặp mã",
      "Thêm vào List rồi đọc Count",
      "Thêm vào Stack rồi Pop hết"
    ],
    "answer": 1,
    "explain": "HashSet bỏ qua mã trùng, Count chính là số mã khác nhau. Tổng cộng O(n)."
  },
  {
    "prompt": "set đã có \"PEN\". Gọi set.Add(\"PEN\") trả về gì?",
    "options": [
      "True",
      "Ném exception",
      "False, và set không đổi",
      "True, và set có hai \"PEN\""
    ],
    "answer": 3,
    "explain": "HashSet không giữ phần tử trùng. Add báo false để bạn biết phần tử đã có."
  },
  {
    "prompt": "Đếm số lần mỗi từ khoá được tìm kiếm trong một list. Cấu trúc nào hợp nhất?",
    "options": [
      "Queue<string>",
      "HashSet<string>",
      "List<int>",
      "Dictionary<string, int>"
    ],
    "answer": 4,
    "explain": "Cần gắn mỗi từ khoá với một số đếm, nên dùng Dictionary từ từ khoá sang số lần."
  }
]
```
