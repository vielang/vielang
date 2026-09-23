---
title: Chọn collection đúng việc
minutes: 11
---

Trang đồng bộ danh sách khách hàng chạy 40 mili giây trên máy dev.

Khách hàng thật có 20.000 bản ghi. Nó chạy gần một phút. Code không sai chỗ
nào — chỉ là bên trong vòng lặp có một câu `list.Contains(x)`.

> **Học xong bài này bạn sẽ:** chọn đúng collection cho từng việc; biết thao
> tác nào tức thì và thao tác nào tốn thời gian theo số phần tử; lộ dữ liệu ra
> ngoài mà không cho người khác sửa ruột object của bạn.
>
> **Cần biết trước:** `List<T>`, vòng lặp `foreach`.

## Mỗi collection sinh ra cho một việc

| Việc cần làm | Dùng | Chi phí |
|---|---|---|
| Danh sách có thứ tự, thêm cuối | `List<T>` | thêm và đọc theo chỉ số: tức thì |
| Tra theo khoá | `Dictionary<K,V>` | tra: tức thì |
| Hỏi "đã có chưa" | `HashSet<T>` | kiểm tra: tức thì |
| Xếp hàng xử lý | `Queue<T>` / `Stack<T>` | vào và ra: tức thì |
| Tìm theo giá trị trong danh sách | `List<T>` + duyệt | tỉ lệ với số phần tử |

"Tức thì" ở đây là **O(1)**, còn "tỉ lệ với số phần tử" là **O(n)**. Ký hiệu
Big-O sẽ nói kỹ ở khoá DSA.

Điều cần nhớ bây giờ chỉ một câu. Một phép O(n) đặt trong vòng lặp O(n) thì
thành O(n²), và đó là lúc 40 mili giây biến thành một phút.

## Thử ngay: cùng một việc, hai cấu trúc

```csharp
using System.Diagnostics;

var source = Enumerable.Range(0, 20_000).ToList();
var list = new List<int>(source);
var set = new HashSet<int>(source);

var sw = Stopwatch.StartNew();
foreach (var x in source)
    if (list.Contains(x)) { }
Console.WriteLine($"List:    {sw.ElapsedMilliseconds}");

sw.Restart();
foreach (var x in source)
    if (set.Contains(x)) { }
Console.WriteLine($"HashSet: {sw.ElapsedMilliseconds}");
```

**Đoán trước khi chạy:** hai con số chênh nhau cỡ nào — gấp đôi, gấp mười, hay
hơn nữa?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
List:    920
HashSet: 1
```

Đơn vị là mili giây. Con số cụ thể tuỳ máy, nhưng tỉ lệ thì luôn cỡ này.

`List.Contains` phải duyệt tuần tự để tìm. Hai mươi nghìn lần tìm trên hai
mươi nghìn phần tử là 400 triệu phép so sánh. `HashSet.Contains` tính thẳng ra
chỗ cần nhìn, không duyệt gì cả.

</details>

Đó chính là trang đồng bộ ở đầu bài. Sửa một dòng khai báo là xong:

```csharp
// SAI — tìm trong List, bên trong vòng lặp
var existing = new List<string>(oldCodes);
foreach (var code in newCodes)
    if (!existing.Contains(code)) Add(code);
```

```csharp
// ĐÚNG — HashSet cho việc "đã có chưa"
var existing = new HashSet<string>(oldCodes);
foreach (var code in newCodes)
    if (!existing.Contains(code)) Add(code);
```

## Array cố định, List&lt;T&gt; co giãn được

```csharp
int[] scores = new int[3];         // cố định 3 phần tử
string[] names = { "Huy", "Nam" };
Console.WriteLine(names.Length);   // mảng dùng Length

var orders = new List<Order>();
orders.Add(order);           // thêm cuối — nhanh
orders.Insert(0, order);     // chèn đầu — dịch cả dãy
Console.WriteLine(orders.Count);   // List dùng Count
```

Bên trong `List<T>` vẫn là một mảng, tự cấp phát lại khi đầy. Đọc theo chỉ số
là tức thì. Tìm theo giá trị thì phải duyệt.

## Dictionary: tra theo khoá

```csharp
var prices = new Dictionary<string, decimal>
{
    ["SP01"] = 150_000,
    ["SP02"] = 90_000,
};

if (prices.TryGetValue("SP01", out var price))
    Console.WriteLine(price);

prices["SP03"] = 120_000;      // thêm hoặc ghi đè
var missing = prices["SP99"];  // KeyNotFoundException
```

Luôn dùng `TryGetValue` thay vì `ContainsKey` rồi mới lấy. Một lần tra thay vì
hai lần, mà code cũng ngắn hơn.

## HashSet hỏi tồn tại, Queue và Stack xếp hàng

```csharp
var processed = new HashSet<int>();

// Add trả false nếu phần tử đã có
if (!processed.Add(orderId)) return;

var queue = new Queue<Job>();   // FIFO
queue.Enqueue(job);
var next = queue.Dequeue();

var history = new Stack<string>();   // LIFO
history.Push("/trang-chu");
var back = history.Pop();
```

`HashSet.Add` trả về `bool`. Nhờ vậy mẫu "đã xử lý rồi thì bỏ qua" chỉ tốn một
dòng, không cần kiểm tra riêng.

`Queue` là hàng đợi: vào trước ra trước, hợp với danh sách việc cần xử lý.
`Stack` thì ngược lại, vào sau ra trước — đúng cách lưu lịch sử để quay lui.

## Lộ ra ngoài thì dùng interface

```csharp
private readonly List<Order> _items = new();

public IReadOnlyList<Order> Items => _items;

public IEnumerable<Order> Pending() =>
    _items.Where(o => o.IsPending);
```

Trả về `List<T>` là cho người gọi quyền `Add` và `Remove` vào ruột object của
bạn. Trả `IReadOnlyList<T>` thì họ chỉ đọc được.

Bạn cũng được lợi. Đổi cấu trúc lưu trữ bên trong sau này mà không phá code
của ai cả.

## Dấu hiệu trong code của bạn

- `.Contains(...)` trên `List` nằm **bên trong** một vòng lặp → đổi sang `HashSet`, đây là O(n²) trá hình.
- `ContainsKey` rồi ngay dòng sau lấy `dict[key]` → gộp lại thành `TryGetValue`.
- `Insert(0, x)` hay `RemoveAt(0)` gọi liên tục → bạn đang cần `Queue<T>`.
- Property `public List<T>` → đổi sang `IReadOnlyList<T>`.
- Collection dùng chung giữa nhiều luồng → `ConcurrentDictionary` hoặc khoá lại; các loại ở trên **không thread-safe**.

## Ghi nhớ

- Mặc định `List<T>`. Tra theo khoá: `Dictionary`. Hỏi tồn tại: `HashSet`.
- `Length` cho array, `Count` cho collection.
- `TryGetValue` thay cho `ContainsKey` rồi lấy.
- Một phép O(n) đặt trong vòng lặp là O(n²).

## Bước tiếp theo

Chọn được chỗ chứa dữ liệu rồi. Giờ tới cách xử lý chúng.

Bài sau, **IEnumerable và hoãn thực thi**, mở bằng một API mất 8 giây. Log cho
thấy câu truy vấn nặng nhất chạy ba lần, dù code chỉ viết nó một lần.

```quiz
[
  {
    "prompt": "Đoạn này chạy rất chậm khi oldCodes có 20.000 phần tử. Sửa thế nào cho nhanh nhất?",
    "code": "var existing = new List<string>(oldCodes);\n\nforeach (var code in newCodes)\n    if (!existing.Contains(code))\n        Add(code);",
    "options": [
      "Đổi foreach thành for",
      "Đổi existing sang HashSet<string>",
      "Sắp xếp existing trước khi duyệt",
      "Gọi existing.ToArray() trước vòng lặp"
    ],
    "answer": 2,
    "explain": "List.Contains duyệt tuần tự nên nằm trong vòng lặp là O(n²). HashSet.Contains tính thẳng ra chỗ cần nhìn, gần như tức thì."
  },
  {
    "prompt": "Bạn cần lấy giá theo mã sản phẩm, nhiều lần, trong một vòng lặp. Cấu trúc nào đúng?",
    "options": [
      "List<(string Code, decimal Price)> rồi FirstOrDefault",
      "Dictionary<string, decimal>",
      "HashSet<string>",
      "Queue<decimal>"
    ],
    "answer": 2,
    "explain": "Dictionary sinh ra đúng cho việc tra theo khoá, chi phí gần như không đổi dù có bao nhiêu phần tử."
  },
  {
    "prompt": "Cách nào tốt hơn, và vì sao?",
    "code": "if (dict.ContainsKey(code))\n    var price = dict[code];",
    "options": [
      "Giữ nguyên, code này rõ ràng nhất",
      "Dùng dict.TryGetValue(code, out var price) — chỉ tra một lần thay vì hai",
      "Dùng dict.Keys.Contains(code) trước",
      "Bọc trong try - catch KeyNotFoundException"
    ],
    "answer": 2,
    "explain": "ContainsKey rồi dict[code] là hai lần tra cùng một khoá. TryGetValue làm gọn trong một lần và cũng an toàn."
  },
  {
    "prompt": "Class có danh sách item bên trong, cần cho bên ngoài đọc nhưng không được sửa. Khai báo property thế nào?",
    "options": [
      "public List<Item> Items { get; set; }",
      "public IReadOnlyList<Item> Items => _items;",
      "public Item[] Items => _items.ToArray();",
      "public IEnumerable<Item> Items { get; set; }"
    ],
    "answer": 2,
    "explain": "IReadOnlyList cho đọc và đếm nhưng không có Add hay Remove, lại không phải chép mảng mới mỗi lần gọi như phương án ToArray."
  }
]
```
