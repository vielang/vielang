---
title: Big-O
minutes: 6
---

Cửa hàng có 5 sản phẩm thì cách tìm nào cũng nhanh. Lên 5 triệu sản phẩm, có
cách vẫn trả lời ngay, có cách chạy mãi không xong. Big-O giúp bạn biết trước
một cách giải sẽ chậm đi thế nào khi dữ liệu lớn lên, trước khi nó chậm thật.

## Khái niệm

📈 **Big-O**: cách mô tả số bước của một thuật toán tăng thế nào khi số phần tử `n` tăng, bỏ qua các hằng số.

Big-O không đo giây, vì mỗi máy chạy nhanh chậm khác nhau. Nó đếm số bước và
chỉ quan tâm dáng tăng: gấp đôi dữ liệu thì gấp đôi, gấp bốn, hay gần như
không đổi.

| Big-O | Đọc là | Ví dụ | 1.000 phần tử |
|---|---|---|---|
| O(1) | hằng số | `list[2]`, `dict[key]` | 1 bước |
| O(log n) | logarit | tìm nhị phân trên dãy đã sắp xếp | ~10 bước |
| O(n) | tuyến tính | duyệt list tìm một tên | 1.000 bước |
| O(n log n) | n log n | sắp xếp tốt | ~10.000 bước |
| O(n²) | bình phương | hai vòng lặp lồng nhau | 1.000.000 bước |

## Ví dụ

```csharp
List<string> names = new List<string>
{
    "Bút bi", "Vở", "Balo", "Thước", "Máy tính"
};

// O(1): lấy theo vị trí, luôn chỉ một bước
Console.WriteLine(names[2]);   // Balo

// O(n): tìm theo tên, xấu nhất phải xem hết list
int steps = 0;
foreach (string name in names)
{
    steps = steps + 1;
    if (name == "Máy tính")
    {
        break;
    }
}
Console.WriteLine(steps);   // 5
```

- `names[2]` nhảy thẳng tới ô thứ ba, không cần nhìn các ô khác.
- Tìm "Máy tính" phải so từng tên từ đầu. List 5 triệu tên thì xấu nhất là 5
  triệu lần so.
- Big-O tính theo trường hợp xấu nhất: tên cần tìm nằm cuối, hoặc không có.

Đây cũng là chuyện của bài Index ở khoá SQL. Không có index, Oracle quét cả
bảng, tức O(n). Có index, Oracle tra trên dữ liệu đã sắp xếp, nhanh cỡ
O(log n).

## Vòng lặp lồng nhau

Vòng lặp trong nằm trong vòng lặp ngoài, mỗi vòng chạy `n` lần, nên tổng cộng
là `n × n` bước: O(n²).

```csharp
Console.WriteLine(CountSteps(10));   // 100

int CountSteps(int n)
{
    int steps = 0;
    for (int i = 0; i < n; i++)
    {
        for (int j = 0; j < n; j++)
        {
            steps = steps + 1;
        }
    }
    return steps;
}
```

## Thử ngay

Thêm hai dòng sau vào dưới dòng gọi `CountSteps(10)`:

```csharp
Console.WriteLine(CountSteps(100));
Console.WriteLine(CountSteps(1000));
```

**Đoán trước khi chạy:** `n` tăng 10 lần, từ 10 lên 100 rồi lên 1000. Số
bước tăng bao nhiêu lần mỗi lượt?

<details>
<summary>Xem kết quả</summary>

```text
100
10000
1000000
```

Mỗi lượt tăng 100 lần, vì `n` tăng 10 lần thì `n × n` tăng 10 × 10 lần. Với
O(n²), dữ liệu lớn gấp 10 là chờ lâu gấp 100.

</details>

## Lỗi hay gặp

**Gọi `Contains` của `List` trong vòng lặp.** Nhìn chỉ thấy một vòng lặp,
nhưng `Contains` tự duyệt cả list bên trong, nên tổng cộng là O(n²).

```csharp
// SAI — O(n²): Contains duyệt lại list mỗi vòng
List<string> seen = new List<string>();
foreach (string name in names)
{
    if (seen.Contains(name))
    {
        Console.WriteLine("Trùng: " + name);
    }
    seen.Add(name);
}
```

```csharp
// ĐÚNG — O(n): ContainsKey của Dictionary là O(1)
Dictionary<string, bool> seen =
    new Dictionary<string, bool>();
foreach (string name in names)
{
    if (seen.ContainsKey(name))
    {
        Console.WriteLine("Trùng: " + name);
    }
    seen[name] = true;
}
```

Vì sao `ContainsKey` chỉ tốn một bước là chuyện của chương Bảng băm.

## Tóm tắt

- Big-O mô tả số bước tăng thế nào khi dữ liệu tăng, không đo giây.
- Lấy theo vị trí là O(1), duyệt tìm là O(n), hai vòng lồng nhau là O(n²).
- Tính theo trường hợp xấu nhất và bỏ qua hằng số.
- Coi chừng method có vòng lặp ẩn bên trong như `List.Contains`.

```quiz
[
  {
    "prompt": "Method duyệt list đơn hàng một lần để tính tổng tiền. Đơn hàng tăng từ 1.000 lên 2.000 thì số bước thế nào?",
    "options": [
      "Không đổi",
      "Gấp đôi, vì đây là O(n)",
      "Gấp bốn",
      "Tăng thêm đúng 1 bước"
    ],
    "answer": 2,
    "explain": "Duyệt một lần là O(n): số bước tăng cùng tỉ lệ với số phần tử."
  },
  {
    "prompt": "So từng sản phẩm với mọi sản phẩm khác để tìm tên trùng bằng hai vòng for lồng nhau. Độ phức tạp là gì?",
    "options": [
      "O(1)",
      "O(log n)",
      "O(n)",
      "O(n²)"
    ],
    "answer": 4,
    "explain": "Mỗi vòng chạy n lần và lồng vào nhau, nên tổng cộng khoảng n × n bước."
  },
  {
    "prompt": "Vì sao Big-O đếm số bước thay vì đo bằng giây?",
    "options": [
      "Vì đo giây quá khó",
      "Vì C# không đo được thời gian",
      "Vì số giây phụ thuộc từng máy, còn dáng tăng của số bước thì không",
      "Vì số bước luôn bằng số giây"
    ],
    "answer": 3,
    "explain": "Cùng một code chạy trên máy nhanh và máy chậm cho số giây khác nhau, nhưng số bước tăng theo n thì như nhau."
  }
]
```
