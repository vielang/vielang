---
title: LINQ thường dùng
minutes: 11
---

Sếp hỏi: "Tháng này khách nào mua nhiều nhất?"

Bạn có `List<Order>` trong tay. Viết bằng vòng lặp mất mười lăm dòng và một
`Dictionary` tạm. Viết bằng LINQ mất bốn dòng. Và quan trọng hơn, người đọc
code hiểu ngay bạn định làm gì.

> **Học xong bài này bạn sẽ:** dùng thạo bộ phép LINQ hay gặp nhất; chọn đúng
> giữa `First` và `Single`, `Any` và `Count`; nhóm dữ liệu rồi tính tổng theo
> nhóm mà không cần `Dictionary` tạm.
>
> **Cần biết trước:** `List<T>`, lambda, và hoãn thực thi (bài trước).

## Bộ phép LINQ theo nhóm việc

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
| Phân trang | `Skip`, `Take` | |
| Chốt kết quả | `ToList`, `ToDictionary`, `ToLookup` | |

## Thử ngay: câu hỏi của sếp

```csharp
record Order(string Customer, decimal Total);

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
```

**Đoán trước khi chạy:** ai đứng đầu — Huy với hai đơn, hay Lan với một đơn?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Lan: 900
```

Huy mua **nhiều lần hơn**: hai đơn, cộng lại 550. Nhưng câu truy vấn này xếp
theo **tổng tiền**, mà Lan một đơn đã 900.

Đổi `Total = g.Sum(...)` thành `Count = g.Count()` là đáp án đổi sang Huy.

Bài học nhỏ: "mua nhiều nhất" là câu hỏi mơ hồ. Nhiều tiền hay nhiều lần? LINQ
buộc bạn trả lời rõ, còn vòng lặp thì giấu câu hỏi ấy giữa mười lăm dòng.

</details>

`GroupBy` trả về các nhóm. Mỗi nhóm có `Key`, và bản thân nó cũng là một tập
phần tử để bạn `Sum` hay `Count` tiếp.

## Lọc, chiếu và sắp xếp nối được thành chuỗi

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

## First, Single và bạn bè

| Phép | Không có phần tử nào | Có từ hai trở lên |
|---|---|---|
| `First` | ném exception | lấy cái đầu |
| `FirstOrDefault` | trả `null` | lấy cái đầu |
| `Single` | ném exception | **ném exception** |
| `SingleOrDefault` | trả `null` | ném exception |

Chọn theo ý định của bạn. `Single` nói "chắc chắn chỉ có một, có hai là dữ
liệu hỏng". `First` nói "lấy cái đầu, còn lại kệ".

Dùng `First` để tra theo khoá chính là **giấu mất** lỗi trùng dữ liệu.

## Any dừng sớm, còn Count phải đếm hết

```csharp
bool hasBig = orders.Any(o => o.Total > 500);
bool allPaid = orders.All(o => o.Total > 0);
int bigCount = orders.Count(o => o.Total > 100);

if (orders.Any()) { }        // dừng ở phần tử đầu
if (orders.Count() > 0) { }  // duyệt hết mới biết
```

Hai dòng cuối cho cùng kết quả. Nhưng `Any()` dừng ngay khi thấy phần tử đầu
tiên, còn `Count()` phải đếm hết.

## Tính tổng, ghép, làm phẳng

```csharp
decimal revenue = orders.Sum(o => o.Total);
decimal max = orders.Max(o => o.Total);
var biggest = orders.MaxBy(o => o.Total);

var detail = orders.Join(customers,
    o => o.Customer,    // khoá bên trái
    c => c.Name,        // khoá bên phải
    (o, c) => new { o.Total, c.Phone });

var allItems = orders.SelectMany(o => o.Items);
var page2 = orders.Skip(20).Take(20);
```

`Max` trả về **giá trị** lớn nhất, `MaxBy` trả về **phần tử** mang giá trị đó.
Hai cái này hay bị nhầm.

Trên danh sách rỗng, `Sum` ra 0 nhưng `Average` và `Max` thì ném exception.

## Hai cách viết, và cách chốt kết quả

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
- `ToList()` đứng giữa chuỗi phép → chốt sớm, xem bài sau khi dữ liệu ở database.

## Ghi nhớ

- `Any()` thay cho `Count() > 0`.
- `Single` khi dữ liệu lẽ ra chỉ có một; `First` khi lấy cái đầu là đủ.
- `Max` trả giá trị, `MaxBy` trả phần tử.
- `GroupBy` cộng `Select` thay cho `Dictionary` tạm.
- Nối nhiều phép rồi `ToList()` **một lần** ở cuối.

## Bước tiếp theo

Những phép này chạy trên `List` thì vô hại.

Bài sau, **LINQ khi chạm database**, cho thấy cũng câu ấy chạy trên EF Core có
thể kéo cả bảng về máy chủ ứng dụng. Mở bằng một API sinh ra 21 câu SQL cho
một vòng `foreach`.

```quiz
[
  {
    "prompt": "Bạn cần lấy đơn hàng theo id — id là khoá chính nên lẽ ra chỉ có một. Dùng gì?",
    "options": [
      "First(o => o.Id == id)",
      "Single(o => o.Id == id)",
      "Where(o => o.Id == id).ToList()",
      "FirstOrDefault(o => o.Id == id)!"
    ],
    "answer": 2,
    "explain": "Single ném lỗi nếu có từ hai bản ghi trở lên — dữ liệu trùng khoá chính là chuyện phải biết ngay, không nên bị First giấu đi."
  },
  {
    "prompt": "Cách nào nhanh hơn khi chỉ cần biết danh sách có phần tử nào không?",
    "options": [
      "list.Count() > 0",
      "list.Any()",
      "list.Length > 0",
      "list.ToList().Count > 0"
    ],
    "answer": 2,
    "explain": "Any() dừng ngay khi gặp phần tử đầu tiên; Count() phải duyệt hết mới trả về được con số."
  },
  {
    "prompt": "Bạn muốn lấy ĐƠN HÀNG có giá trị lớn nhất, không phải con số lớn nhất. Dùng gì?",
    "options": [
      "orders.Max(o => o.Total)",
      "orders.MaxBy(o => o.Total)",
      "orders.OrderBy(o => o.Total).First()",
      "orders.Select(o => o.Total).Max()"
    ],
    "answer": 2,
    "explain": "Max trả về giá trị lớn nhất, MaxBy trả về chính phần tử mang giá trị đó. Phương án C cũng ra phần tử nhưng là nhỏ nhất, vì OrderBy sắp tăng dần."
  },
  {
    "prompt": "Đoạn vòng lặp này làm gì, và LINQ viết lại thế nào?",
    "code": "var totals = new Dictionary<string, decimal>();\nforeach (var o in orders)\n{\n    totals.TryGetValue(o.Customer, out var t);\n    totals[o.Customer] = t + o.Total;\n}",
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
