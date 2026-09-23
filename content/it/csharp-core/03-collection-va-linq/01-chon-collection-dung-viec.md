---
title: Chọn collection đúng việc
minutes: 9
---

Chọn sai collection không làm code sai, nhưng làm nó chậm dần theo lượng dữ
liệu — đúng loại bug chỉ lộ ra trên production. Điều cần nắm là mỗi loại nhanh ở
thao tác nào.

## Array

```csharp
int[] diem = new int[3];          // 3 phần tử, mặc định là 0
string[] ten = { "Huy", "Nam" };  // khởi tạo sẵn

Console.WriteLine(ten.Length);    // 2 — mảng dùng Length
```

Kích thước **cố định** sau khi tạo. Dùng khi biết trước số phần tử và không thêm
bớt. Ngoài trường hợp đó thì `List<T>` tiện hơn hẳn.

## List&lt;T&gt; — mặc định cho danh sách

```csharp
var orders = new List<Order>();
orders.Add(order);                // thêm cuối
orders.Insert(0, order);          // chèn đầu — phải dịch mọi phần tử, chậm
orders.RemoveAt(0);
orders.Contains(order);           // duyệt tuần tự để tìm

Console.WriteLine(orders.Count);  // List dùng Count, không phải Length
```

Bên trong `List<T>` vẫn là một mảng, tự cấp phát lại khi đầy. Truy cập theo chỉ
số `list[i]` là tức thì; tìm theo giá trị thì phải duyệt hết.

## Dictionary — tra theo khoá

```csharp
var giaTheoMa = new Dictionary<string, decimal>
{
    ["SP01"] = 150_000,
    ["SP02"] = 90_000,
};

if (giaTheoMa.TryGetValue("SP01", out var gia))
    Console.WriteLine(gia);

giaTheoMa["SP03"] = 120_000;          // thêm hoặc ghi đè
var thieu = giaTheoMa["SP99"];        // ném KeyNotFoundException
```

Tra theo khoá gần như tức thì dù có một triệu phần tử — đây là lý do tồn tại của
`Dictionary`. Luôn dùng `TryGetValue` thay vì kiểm tra `ContainsKey` rồi mới lấy:
một lần tra thay vì hai.

## HashSet — tập hợp không trùng

```csharp
var daXuLy = new HashSet<int>();

if (!daXuLy.Add(orderId))             // Add trả false nếu đã có
    return;                            // bỏ qua đơn đã xử lý

var chung = setA.Intersect(setB);     // giao
var gop = setA.Union(setB);            // hợp
```

Cần hỏi "phần tử này đã có chưa" thật nhiều lần thì `HashSet` nhanh hơn `List`
rất nhiều — `List.Contains` phải duyệt tuần tự.

## Queue và Stack

```csharp
var hangCho = new Queue<Job>();       // FIFO — vào trước ra trước
hangCho.Enqueue(job);
var tiepTheo = hangCho.Dequeue();

var lichSu = new Stack<string>();     // LIFO — vào sau ra trước
lichSu.Push("/trang-chu");
var quayLai = lichSu.Pop();
```

## Bảng chọn nhanh

| Việc cần làm | Dùng | Chi phí |
|---|---|---|
| Danh sách có thứ tự, thêm cuối | `List<T>` | thêm/đọc theo chỉ số: tức thì |
| Tra theo khoá | `Dictionary<K,V>` | tra: tức thì |
| Kiểm tra đã tồn tại chưa | `HashSet<T>` | kiểm tra: tức thì |
| Xếp hàng xử lý | `Queue<T>` / `Stack<T>` | vào/ra: tức thì |
| Tìm theo giá trị trong danh sách | `List<T>` + duyệt | tỉ lệ với số phần tử |

Nói "tức thì" ở đây là **O(1)**, còn "tỉ lệ với số phần tử" là **O(n)** — ký hiệu
Big-O sẽ nói kỹ ở khoá DSA.

## Lộ ra ngoài thì dùng interface

```csharp
public IReadOnlyList<Order> Items => _items;        // chỉ đọc
public IEnumerable<Order> GetPending() => _items.Where(o => o.IsPending);

private readonly List<Order> _items = new();        // chi tiết bên trong
```

Trả về `List<T>` là cho người gọi quyền sửa danh sách bên trong object của bạn.
Trả `IReadOnlyList<T>` hoặc `IEnumerable<T>` thì họ chỉ đọc được, mà bạn vẫn tự
do đổi cấu trúc lưu trữ sau này.

## Ghi nhớ

- Mặc định: `List<T>`. Tra theo khoá: `Dictionary`. Hỏi tồn tại: `HashSet`.
- `Length` cho array, `Count` cho collection — nhầm chỗ này là lỗi compile quen thuộc.
- Các collection này **không thread-safe**; nhiều luồng cùng ghi thì dùng `ConcurrentDictionary` hoặc khoá lại.
