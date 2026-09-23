---
title: Chọn collection đúng việc
minutes: 10
---

Trang đồng bộ danh sách khách hàng chạy 40 mili giây trên máy dev với 100 bản
ghi. Khách hàng thật có 20.000 bản ghi, và nó chạy mất gần một phút. Code
không sai chỗ nào — chỉ là bên trong vòng lặp có một câu `danhSach.Contains(x)`.

> **Học xong bài này bạn sẽ:** chọn đúng collection cho từng việc; biết thao
> tác nào tức thì và thao tác nào tốn thời gian theo số phần tử; lộ dữ liệu ra
> ngoài mà không cho người khác sửa ruột object của bạn.
>
> **Cần biết trước:** `List<T>`, vòng lặp `foreach`.

## Thử ngay: cùng một việc, hai cấu trúc

```csharp
using System.Diagnostics;

var nguon = Enumerable.Range(0, 20_000).ToList();
var ds = new List<int>(nguon);
var tap = new HashSet<int>(nguon);

var dh = Stopwatch.StartNew();
foreach (var x in nguon)
    if (ds.Contains(x)) { }
Console.WriteLine($"List:    {dh.ElapsedMilliseconds}");

dh.Restart();
foreach (var x in nguon)
    if (tap.Contains(x)) { }
Console.WriteLine($"HashSet: {dh.ElapsedMilliseconds}");
```

**Đoán trước khi chạy:** hai dòng in ra con số chênh nhau cỡ nào — gấp đôi,
gấp mười, hay hơn nữa?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
List:    920
HashSet: 1
```

Đơn vị là mili giây. Con số cụ thể tuỳ máy, nhưng tỉ lệ thì luôn cỡ này. `List.Contains` phải duyệt
tuần tự để tìm, nên 20.000 lần tìm trên 20.000 phần tử là 400 triệu phép so
sánh. `HashSet.Contains` tính thẳng ra chỗ cần nhìn, không duyệt gì cả.

</details>

Đây chính là trang đồng bộ ở đầu bài. Đổi một dòng khai báo là xong:

```csharp
// SAI — tìm trong List, bên trong vòng lặp
var daCo = new List<string>(maCu);
foreach (var m in maMoi)
    if (!daCo.Contains(m)) Them(m);
```

```csharp
// ĐÚNG — HashSet cho việc "đã có chưa"
var daCo = new HashSet<string>(maCu);
foreach (var m in maMoi)
    if (!daCo.Contains(m)) Them(m);
```

## Array và List&lt;T&gt;

```csharp
int[] diem = new int[3];           // cố định 3 phần tử
string[] ten = { "Huy", "Nam" };
Console.WriteLine(ten.Length);     // mảng dùng Length

var dons = new List<Order>();
dons.Add(don);            // thêm cuối — nhanh
dons.Insert(0, don);      // chèn đầu — phải dịch cả dãy
dons.RemoveAt(0);
Console.WriteLine(dons.Count);     // List dùng Count
```

Bên trong `List<T>` vẫn là một mảng, tự cấp phát lại khi đầy. Truy cập theo chỉ
số `list[i]` là tức thì; tìm theo giá trị thì phải duyệt.

## Dictionary — tra theo khoá

```csharp
var gia = new Dictionary<string, decimal>
{
    ["SP01"] = 150_000,
    ["SP02"] = 90_000,
};

if (gia.TryGetValue("SP01", out var g))
    Console.WriteLine(g);

gia["SP03"] = 120_000;      // thêm hoặc ghi đè
var thieu = gia["SP99"];    // ném KeyNotFoundException
```

Tra theo khoá gần như tức thì dù có một triệu phần tử. Luôn dùng `TryGetValue`
thay vì `ContainsKey` rồi mới lấy: một lần tra thay vì hai.

## HashSet, Queue và Stack

```csharp
var daXuly = new HashSet<int>();
// Add trả false nếu phần tử đã có
if (!daXuly.Add(donId)) return;

var chung = tapA.Intersect(tapB);
var gop = tapA.Union(tapB);

var hang = new Queue<Job>();      // FIFO
hang.Enqueue(job);
var tiep = hang.Dequeue();

var lichSu = new Stack<string>(); // LIFO
lichSu.Push("/trang-chu");
var quayLai = lichSu.Pop();
```

## Bảng chọn nhanh

| Việc cần làm | Dùng | Chi phí |
|---|---|---|
| Danh sách có thứ tự, thêm cuối | `List<T>` | thêm/đọc theo chỉ số: tức thì |
| Tra theo khoá | `Dictionary<K,V>` | tra: tức thì |
| Hỏi "đã có chưa" | `HashSet<T>` | kiểm tra: tức thì |
| Xếp hàng xử lý | `Queue<T>` / `Stack<T>` | vào/ra: tức thì |
| Tìm theo giá trị trong danh sách | `List<T>` + duyệt | tỉ lệ với số phần tử |

"Tức thì" ở đây là **O(1)**, "tỉ lệ với số phần tử" là **O(n)** — ký hiệu Big-O
sẽ nói kỹ ở khoá DSA. Điều cần nhớ bây giờ: O(n) nằm trong một vòng lặp O(n)
thì thành O(n²), và đó là lúc 40 mili giây biến thành một phút.

## Lộ ra ngoài thì dùng interface

```csharp
private readonly List<Order> _items = new();

public IReadOnlyList<Order> Items => _items;

public IEnumerable<Order> DangCho() =>
    _items.Where(o => o.IsPending);
```

Trả về `List<T>` là cho người gọi quyền `Add`/`Remove` vào ruột object của bạn.
Trả `IReadOnlyList<T>` thì họ chỉ đọc được, mà bạn vẫn tự do đổi cấu trúc lưu
trữ bên trong sau này.

## Dấu hiệu trong code của bạn

- `.Contains(...)` trên `List` nằm **bên trong** một vòng lặp → đổi sang `HashSet`, đây là O(n²) trá hình.
- `ContainsKey` rồi ngay dòng sau lấy `dict[key]` → gộp lại thành `TryGetValue`.
- `Insert(0, x)` hay `RemoveAt(0)` gọi liên tục → bạn đang cần `Queue<T>`.
- Property `public List<T>` → đổi sang `IReadOnlyList<T>` trước khi có người `Add` vào đó.
- Collection dùng chung giữa nhiều luồng → `ConcurrentDictionary` hoặc khoá lại; các loại ở trên **không thread-safe**.

## Ghi nhớ

- Mặc định `List<T>`. Tra theo khoá: `Dictionary`. Hỏi tồn tại: `HashSet`.
- `Length` cho array, `Count` cho collection.
- `TryGetValue` thay cho `ContainsKey` + lấy.
- Một phép O(n) đặt trong vòng lặp là O(n²) — chỗ này quyết định app chạy được với bao nhiêu dữ liệu.

## Bước tiếp theo

Bài sau — **IEnumerable và hoãn thực thi** — giải thích vì sao một truy vấn
LINQ có thể chạy ba lần dù bạn chỉ viết nó một lần.

```quiz
[
  {
    "prompt": "Đoạn này chạy rất chậm khi maCu có 20.000 phần tử. Sửa thế nào cho nhanh nhất?",
    "code": "var daCo = new List<string>(maCu);\n\nforeach (var m in maMoi)\n    if (!daCo.Contains(m))\n        Them(m);",
    "options": [
      "Đổi foreach thành for",
      "Đổi daCo sang HashSet<string>",
      "Sắp xếp daCo trước khi duyệt",
      "Gọi daCo.ToArray() trước vòng lặp"
    ],
    "answer": 2,
    "explain": "List.Contains duyệt tuần tự nên nằm trong vòng lặp là O(n²). HashSet.Contains tính thẳng ra chỗ cần nhìn, gần như tức thì."
  },
  {
    "prompt": "Bạn cần lấy giá theo mã sản phẩm, nhiều lần, trong một vòng lặp. Cấu trúc nào đúng?",
    "options": [
      "List<(string Ma, decimal Gia)> rồi FirstOrDefault",
      "Dictionary<string, decimal>",
      "HashSet<string>",
      "Queue<decimal>"
    ],
    "answer": 2,
    "explain": "Dictionary sinh ra đúng cho việc tra theo khoá, chi phí gần như không đổi dù có bao nhiêu phần tử."
  },
  {
    "prompt": "Cách nào tốt hơn, và vì sao?",
    "code": "if (dict.ContainsKey(ma))\n    var g = dict[ma];",
    "options": [
      "Giữ nguyên, code này rõ ràng nhất",
      "Dùng dict.TryGetValue(ma, out var g) — chỉ tra một lần thay vì hai",
      "Dùng dict.Keys.Contains(ma) trước",
      "Bọc trong try - catch KeyNotFoundException"
    ],
    "answer": 2,
    "explain": "ContainsKey rồi dict[ma] là hai lần tra cùng một khoá. TryGetValue làm gọn trong một lần và cũng an toàn."
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
    "explain": "IReadOnlyList cho đọc và đếm nhưng không có Add/Remove, lại không phải chép mảng mới mỗi lần gọi như phương án ToArray."
  }
]
```
