---
title: Dictionary
minutes: 5
---

Kho có hàng nghìn sản phẩm, và bạn cần biết ngay mã `PEN-01` còn bao nhiêu
cái. Duyệt cả list để tìm thì chậm và dài dòng. Dictionary cho phép tra thẳng
bằng mã.

## Khái niệm

📖 **Dictionary**: tập hợp các cặp khoá–giá trị, tra giá trị theo khoá, viết là `Dictionary<kiểu khoá, kiểu giá trị>`.

🔑 **Khoá (key)**: giá trị dùng để tra cứu, mỗi khoá chỉ xuất hiện một lần trong dictionary.

## Ví dụ

```csharp
Dictionary<string, int> stock =
    new Dictionary<string, int>();

stock["PEN-01"] = 120;
stock["BOOK-02"] = 35;

Console.WriteLine(stock["PEN-01"]);          // 120
Console.WriteLine(stock.Count);              // 2
Console.WriteLine(stock.ContainsKey("X"));   // False

foreach (var item in stock)
{
    Console.WriteLine($"{item.Key}: {item.Value}");
}
```

- `Dictionary<string, int>`: khoá là mã sản phẩm (`string`), giá trị là số
  lượng (`int`).
- `stock["PEN-01"] = 120` thêm cặp mới, hoặc ghi đè nếu khoá đã có.
- `stock["PEN-01"]` đọc giá trị theo khoá.
- `ContainsKey` kiểm tra khoá có tồn tại không.
- `foreach` trả về từng cặp, đọc bằng `.Key` và `.Value`.

## Thử ngay

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
Dictionary<string, int> stock =
    new Dictionary<string, int>();

stock["PEN-01"] = 120;
stock["BOOK-02"] = 35;
stock["PEN-01"] = 80;

Console.WriteLine(stock.Count);
Console.WriteLine(stock["PEN-01"]);
```

**Đoán trước khi chạy:** gán `PEN-01` hai lần thì dictionary có 2 hay 3
cặp?

<details>
<summary>Xem kết quả</summary>

```text
2
80
```

Vẫn 2 cặp. Khoá là duy nhất, nên lần gán thứ hai chỉ ghi đè giá trị cũ của
`PEN-01` thành 80.

</details>

## Lỗi hay gặp

**Đọc khoá không tồn tại.** Chương trình dừng với lỗi `KeyNotFoundException`.

```csharp
// SAI — lỗi khi chạy: không có khoá "RULER"
Dictionary<string, int> stock =
    new Dictionary<string, int>();
stock["PEN-01"] = 120;

Console.WriteLine(stock["RULER"]);
```

```csharp
// ĐÚNG — kiểm tra trước khi đọc
Dictionary<string, int> stock =
    new Dictionary<string, int>();
stock["PEN-01"] = 120;

if (stock.ContainsKey("RULER"))
{
    Console.WriteLine(stock["RULER"]);
}
else
{
    Console.WriteLine("Không có mã này");
}
```

**Dùng `Add` với khoá đã có.** `Add` không ghi đè mà báo lỗi
`ArgumentException`.

```csharp
// SAI — lỗi khi chạy: khoá PEN-01 đã tồn tại
Dictionary<string, int> stock =
    new Dictionary<string, int>();
stock.Add("PEN-01", 120);
stock.Add("PEN-01", 80);
```

```csharp
// ĐÚNG — gán qua [ ] để thêm mới hoặc ghi đè
Dictionary<string, int> stock =
    new Dictionary<string, int>();
stock["PEN-01"] = 120;
stock["PEN-01"] = 80;
```

## Tóm tắt

- Dictionary lưu cặp khoá–giá trị và tra giá trị thẳng theo khoá.
- Mỗi khoá chỉ có một. Gán `d[key] = value` sẽ thêm mới hoặc ghi đè.
- Đọc khoá không tồn tại là lỗi, nên kiểm tra bằng `ContainsKey` trước.
- `foreach` trả về từng cặp với `.Key` và `.Value`.

```quiz
[
  {
    "prompt": "Bạn cần tra nhanh tên khách hàng theo số điện thoại. Kiểu nào phù hợp nhất?",
    "options": [
      "List<string>",
      "string[]",
      "Dictionary<string, string>",
      "string"
    ],
    "answer": 3,
    "explain": "Số điện thoại là khoá duy nhất, tên là giá trị. Dictionary tra thẳng theo khoá, không phải duyệt cả danh sách."
  },
  {
    "prompt": "var prices = new Dictionary<string, decimal>(); prices[\"A\"] = 10m; prices[\"B\"] = 20m; prices[\"A\"] = 15m; Tổng các giá trị là bao nhiêu?",
    "options": [
      "45",
      "30",
      "25",
      "35"
    ],
    "answer": 4,
    "explain": "Gán \"A\" lần hai ghi đè 10 thành 15. Dictionary còn A = 15 và B = 20, tổng là 35."
  },
  {
    "prompt": "stock[\"PEN\"] làm chương trình dừng với KeyNotFoundException. Nguyên nhân?",
    "options": [
      "Dictionary chưa có khoá \"PEN\"",
      "Giá trị của \"PEN\" bằng 0",
      "Giá trị kiểu int nên không đọc bằng [ ] được",
      "Phải viết stock.PEN"
    ],
    "answer": 1,
    "explain": "KeyNotFoundException nghĩa là khoá cần đọc không tồn tại. Kiểm tra bằng ContainsKey trước khi đọc."
  }
]
```
