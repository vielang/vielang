---
title: LINQ thường dùng
minutes: 11
---

Sếp hỏi: "Tháng này khách nào mua nhiều nhất?"

Bạn có `List<Order>` trong tay. Viết bằng vòng lặp thì phải dựng một
`Dictionary` tạm rồi tự cộng dồn.

Viết bằng LINQ là bốn phép nối nhau, đọc ra thành câu: nhóm theo khách, cộng
tiền, xếp giảm dần, lấy cái đầu.

> **Học xong bài này bạn sẽ:** dùng thạo bộ phép LINQ hay gặp nhất; chọn đúng
> giữa `First` và `Single`, `Any` và `Count`; nhóm dữ liệu rồi tính tổng theo
> nhóm mà không cần `Dictionary` tạm.
>
> **Cần biết trước:** `List<T>`, lambda, và hoãn thực thi (bài trước).

## Bộ phép LINQ theo nhóm việc

Trả lời câu hỏi của sếp chỉ cần bốn phép trong bảng này. Bảng đặt theo việc cần
làm, vì lúc viết code bạn biết mình muốn gì trước khi biết tên phép.

| Việc | Phép | Ghi chú |
|---|---|---|
| Lọc | `Where` | giữ lại phần tử thoả điều kiện |
| Đổi hình dạng | `Select` | gọi là **projection** |
| Sắp xếp | `OrderBy`, `ThenBy` | thêm `Descending` để đảo |
| Lấy một | `First`, `Single`, `FirstOrDefault` | khác nhau ở chỗ không có thì sao |
| Hỏi có không | `Any`, `All` | dừng sớm, không duyệt hết |
| Tính | `Count`, `Sum`, `Max`, `MaxBy` | `Max` trả giá trị, `MaxBy` trả phần tử |
| Nhóm | `GroupBy` | bản LINQ của `GROUP BY` |
| Làm phẳng | `SelectMany` | danh sách lồng danh sách |
| Ghép hai danh sách | `Join` | theo một khoá chung |
| Bỏ trùng | `Distinct`, `DistinctBy` | |
| Phân trang | `Skip`, `Take` | cắt n đầu, lấy n tiếp |
| Chốt kết quả | `ToList`, `ToDictionary`, `ToLookup` | dừng hoãn thực thi |

## Thử ngay: câu hỏi của sếp

Nhìn cột giữa là thấy quy luật: mỗi phép làm đúng một việc, nối chúng lại mới
thành câu trả lời.

```csharp
var orders = new List<Order>
{
    new("Huy", 300), new("Nam", 120),
    new("Huy", 250), new("Lan", 900),
};

var top = orders
    .GroupBy(o => o.Customer)
    .Select(g => new {
        g.Key,
        Total = g.Sum(o => o.Total),
    })
    .OrderByDescending(x => x.Total)
    .First();

Console.WriteLine($"{top.Key}: {top.Total}");

record Order(string Customer, decimal Total);
```

**Đoán trước khi chạy:** ai đứng đầu — Huy với hai đơn, hay Lan với một đơn?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Lan: 900
```

Huy mua **nhiều lần hơn**: hai đơn, cộng lại 550. Nhưng câu truy vấn này xếp
theo **tổng tiền**, mà Lan một đơn đã 900.

Đổi `Total = g.Sum(o => o.Total)` thành `Count = g.Count()`, và đổi luôn
`OrderByDescending(x => x.Total)` thành `OrderByDescending(x => x.Count)`. Lúc
đó đáp án là Huy.

Bài học nhỏ: "mua nhiều nhất" là câu hỏi mơ hồ. Nhiều tiền hay nhiều lần? LINQ
buộc bạn trả lời rõ, còn vòng lặp thì giấu câu hỏi ấy trong một `Dictionary`
tạm.

</details>

`GroupBy` trả về các nhóm. Mỗi nhóm có `Key`, và bản thân nó cũng là một tập
phần tử để bạn `Sum` hay `Count` tiếp.

## Lọc, chiếu và sắp xếp nối được thành chuỗi

Ba phép hay đi cùng nhau nhất, và thứ tự viết ra đúng thứ tự bạn nghĩ.

```csharp
var result = orders
    .Where(o => o.Total > 100)
    .OrderByDescending(o => o.Total)
    .ThenBy(o => o.Customer)
    .Select(o => new { o.Customer, o.Total })
    .ToList();
```

`Select` gọi là **projection**: biến mỗi phần tử thành hình dạng khác. Lấy
đúng thứ cần thay vì bê cả object là thói quen tốt, nhất là khi dữ liệu đến từ
database.

## First lấy cái đầu, Single đòi phải đúng một

Lấy một phần tử ra thì có bốn phép, và chúng khác nhau ở chỗ "không có thì
sao".

| Phép | Không có phần tử nào | Có từ hai trở lên |
|---|---|---|
| `First` | ném exception | lấy cái đầu |
| `FirstOrDefault` | trả `default` | lấy cái đầu |
| `Single` | ném exception | **ném exception** |
| `SingleOrDefault` | trả `default` | ném exception |

Để ý chữ `default`, đừng đọc thành `null`. Với `List<Order>` thì đúng là `null`,
nhưng với `List<decimal>` nó trả `0`.

Nghĩa là "không có đơn nào" và "có một đơn 0 đồng" cho ra cùng một kết quả. Chỗ
đó phải `Any()` kiểm tra trước.

Chọn theo ý định của bạn. `Single` nói "chắc chắn chỉ có một, có hai là dữ
liệu hỏng". `First` nói "lấy cái đầu, còn lại kệ".

Tra theo khoá chính mà dùng `First` thì bạn **giấu mất** lỗi trùng dữ liệu.

## Any dừng sớm, còn Count phải đếm hết

Hỏi "có phần tử nào không" và đếm xem có bao nhiêu là hai việc khác nhau, dù
kết quả nhiều khi giống nhau.

```csharp
bool hasBig = orders.Any(o => o.Total > 500);
bool allPaid = orders.All(o => o.Total > 0);
int bigCount = orders.Count(o => o.Total > 100);

if (orders.Any()) { }        // dừng ở phần tử đầu
if (orders.Count() > 0) { }  // duyệt hết mới biết
```

Hai dòng cuối cho cùng kết quả. Nhưng `Any()` dừng ngay khi thấy phần tử đầu
tiên, còn `Count()` phải đếm hết.

## Max trả giá trị, MaxBy trả phần tử

Nhóm phép tính toán có một cặp rất hay bị nhầm.

```csharp
decimal revenue = orders.Sum(o => o.Total);
decimal max = orders.Max(o => o.Total);
var biggest = orders.MaxBy(o => o.Total);

var detail = orders.Join(customers,
    o => o.Customer.Name,   // khoá bên trái
    c => c.Name,            // khoá bên phải
    (o, c) => new { o.Total, c.Phone });

var allLines = orders.SelectMany(o => o.Lines);
var page2 = orders.Skip(20).Take(20);
```

`Max` trả về **giá trị** lớn nhất, `MaxBy` trả về **phần tử** mang giá trị đó.
Hai cái này hay bị nhầm.

Trên danh sách rỗng, `Sum` ra 0 nhưng `Average` và `Max` thì ném exception.

## Method syntax và query syntax cho cùng một kết quả

LINQ có hai cú pháp. Bạn sẽ đọc code của cả hai, nên cần nhận ra cả hai.

```csharp
// Method syntax — phổ biến hơn
var a = orders.Where(o => o.Total > 100)
              .Select(o => o.Total);

// Query syntax — giống SQL
var b = from o in orders
        where o.Total > 100
        select o.Total;

// trùng khoá là ném lỗi
var map = orders.ToDictionary(o => o.Customer);
// cho phép trùng, mỗi khoá một nhóm
var lookup = orders.ToLookup(o => o.Customer);
```

Hai cách cho ra cùng một thứ. Code .NET ngày nay dùng **method syntax** là
chính.

## Dấu hiệu trong code của bạn

- `Count() > 0` → đổi sang `Any()`.
- `First()` dùng để lấy bản ghi theo id → nên là `Single()`, để trùng dữ liệu lộ ra thay vì bị giấu.
- `FirstOrDefault()` rồi dùng luôn kết quả → thiếu kiểm tra null.
- `Dictionary` tạm dựng bằng vòng lặp để cộng dồn theo nhóm → `GroupBy` làm đúng việc đó.
- `ToList()` đứng giữa chuỗi phép → chốt sớm. Nếu dữ liệu đến từ database thì đây là lỗi nặng; bài sau nói vì sao.

## Ghi nhớ

- `Any()` thay cho `Count() > 0`.
- `Single` khi dữ liệu lẽ ra chỉ có một; `First` khi lấy cái đầu là đủ.
- `Max` trả giá trị, `MaxBy` trả phần tử.
- `GroupBy` cộng `Select` thay cho `Dictionary` tạm.
- Nối nhiều phép rồi `ToList()` **một lần** ở cuối.

## Bước tiếp theo

Những phép này chạy trên `List` thì vô hại.

Bài sau, **LINQ khi chạm database**, cho thấy cũng câu ấy chạy trên EF Core có
thể kéo cả bảng về máy chủ ứng dụng.

Nó mở bằng bốn dòng code trông rất sạch sẽ, mà bên dưới là hai mươi mốt câu
SQL.

```quiz
[
  {
    "prompt": "Bạn cần lấy đơn hàng theo id — id là khoá chính nên lẽ ra chỉ có một. Dùng gì?",
    "options": [
      "First(o => o.Id == id)",
      "Where(o => o.Id == id).ToList()",
      "Single(o => o.Id == id)",
      "FirstOrDefault(o => o.Id == id)!"
    ],
    "answer": 3,
    "explain": "Single ném lỗi nếu có từ hai bản ghi trở lên — dữ liệu trùng khoá chính là chuyện phải biết ngay, không nên bị First giấu đi."
  },
  {
    "prompt": "Cách nào nhanh hơn khi chỉ cần biết danh sách có phần tử nào không?",
    "options": [
      "list.Any()",
      "list.Count() > 0",
      "list.Length > 0",
      "list.ToList().Count > 0"
    ],
    "answer": 1,
    "explain": "Any() dừng ngay khi gặp phần tử đầu tiên; Count() phải duyệt hết mới trả về được con số."
  },
  {
    "prompt": "Bạn muốn lấy ĐƠN HÀNG có giá trị lớn nhất, không phải con số lớn nhất. Dùng gì?",
    "options": [
      "orders.Max(o => o.Total)",
      "orders.Select(o => o.Total).Max()",
      "orders.OrderBy(o => o.Total).First()",
      "orders.MaxBy(o => o.Total)"
    ],
    "answer": 4,
    "explain": "Max trả về giá trị lớn nhất, MaxBy trả về chính phần tử mang giá trị đó. OrderBy(...).First() cũng ra một phần tử, nhưng là phần tử NHỎ nhất, vì OrderBy sắp tăng dần."
  },
  {
    "prompt": "Đoạn vòng lặp này làm gì, và LINQ viết lại thế nào?",
    "code": "var totals = new Dictionary<string, decimal>();\nforeach (var o in orders)\n{\n    totals.TryGetValue(o.Customer.Name, out var t);\n    totals[o.Customer.Name] = t + o.Total;\n}",
    "options": [
      "Lọc theo khách — dùng Where",
      "Cộng tiền theo từng khách — dùng GroupBy rồi Sum",
      "Loại trùng khách — dùng Distinct",
      "Ghép hai danh sách — dùng Join"
    ],
    "answer": 2,
    "explain": "Đây đúng là mẫu gom nhóm rồi cộng dồn: orders.GroupBy(o => o.Customer).Select(g => new { g.Key, Total = g.Sum(o => o.Total) })."
  }
]
```
