---
title: LINQ thường dùng
minutes: 11
---

Sếp hỏi: "Tháng này khách nào mua nhiều nhất?". Bạn có `List<Order>` trong tay.
Viết bằng vòng lặp mất mười lăm dòng và một `Dictionary` tạm. Viết bằng LINQ
mất bốn dòng — và quan trọng hơn, người đọc code hiểu ngay bạn định làm gì.

> **Học xong bài này bạn sẽ:** dùng thạo bộ phép LINQ hay gặp nhất; chọn đúng
> giữa `First` và `Single`, `Any` và `Count`; nhóm dữ liệu rồi tính tổng theo
> nhóm mà không cần `Dictionary` tạm.
>
> **Cần biết trước:** `List<T>`, lambda, và hoãn thực thi (bài trước).

## Thử ngay: câu hỏi của sếp

```csharp
record Don(string Khach, decimal Tien);

var dons = new List<Don>
{
    new("Huy", 300), new("Nam", 120),
    new("Huy", 250), new("Lan", 900),
};

var top = dons
    .GroupBy(d => d.Khach)
    .Select(g => new {
        g.Key,
        Tong = g.Sum(d => d.Tien),
    })
    .OrderByDescending(x => x.Tong)
    .First();

Console.WriteLine($"{top.Key}: {top.Tong}");
```

**Đoán trước khi chạy:** ai đứng đầu — Huy (hai đơn) hay Lan (một đơn)?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Lan: 900
```

Huy mua **nhiều lần hơn** (hai đơn, cộng lại 550), nhưng câu truy vấn này xếp
theo **tổng tiền**, mà Lan một đơn đã 900. Đổi `Tong = g.Sum(...)` thành
`SoDon = g.Count()` là đáp án đổi sang Huy ngay.

Bài học nhỏ: "khách nào mua nhiều nhất" là câu hỏi mơ hồ — nhiều tiền hay
nhiều lần? Câu LINQ buộc bạn phải trả lời rõ điều đó, còn vòng lặp thì giấu
câu hỏi ấy đi giữa mười lăm dòng.

</details>

`GroupBy` trả về các nhóm, mỗi nhóm có `Key` và bản thân nó là một tập phần tử
— đúng bản LINQ của `GROUP BY` trong SQL.

## Lọc, chiếu, sắp xếp

```csharp
var ketQua = dons
    .Where(d => d.Tien > 100)
    .OrderByDescending(d => d.Tien)
    .ThenBy(d => d.Khach)
    .Select(d => new { d.Khach, d.Tien })
    .ToList();
```

`Select` gọi là **projection**: biến mỗi phần tử thành hình dạng khác. Lấy đúng
thứ cần thay vì bê cả object là thói quen tốt, nhất là khi dữ liệu đến từ
database.

## Lấy một phần tử

```csharp
var a = dons.First(d => d.Khach == "Huy");
var b = dons.FirstOrDefault(d => d.Khach == "X");
var c = dons.Single(d => d.Khach == "Lan");
var e = dons.SingleOrDefault(d => d.Khach == "X");
```

Chọn theo ý định: `Single` nói "chắc chắn chỉ có một, có hai là dữ liệu hỏng,
hãy báo lỗi". `First` nói "lấy cái đầu, còn lại kệ". Dùng `First` cho khoá
chính là **giấu mất** lỗi trùng dữ liệu.

`FirstOrDefault` trả `null` khi không có — nhớ kiểm tra trước khi dùng.

## Kiểm tra và đếm

```csharp
bool coHuy = dons.Any(d => d.Tien > 500);
bool taTraHet = dons.All(d => d.Tien > 0);
int soDon = dons.Count(d => d.Tien > 100);

if (dons.Any()) { }        // dừng ở phần tử đầu
if (dons.Count() > 0) { }  // phải duyệt hết mới biết
```

## Tính tổng

```csharp
decimal doanhThu = dons.Sum(d => d.Tien);
decimal trungBinh = dons.Average(d => d.Tien);
decimal lonNhat = dons.Max(d => d.Tien);
var donLonNhat = dons.MaxBy(d => d.Tien);
```

`Max` trả về **giá trị** lớn nhất, `MaxBy` trả về **phần tử** có giá trị đó —
hay nhầm. Trên danh sách rỗng, `Sum` ra 0 nhưng `Average` và `Max` thì ném
exception.

## Ghép, làm phẳng, phân trang

```csharp
var chiTiet = dons.Join(khachs,
    d => d.Khach,      // khoá bên trái
    k => k.Ten,        // khoá bên phải
    (d, k) => new { d.Tien, k.DienThoai });

var moiMon = dons.SelectMany(d => d.Items);
var maKhach = dons.Select(d => d.Khach).Distinct();
var theoTen = dons.DistinctBy(d => d.Khach);
var trang2 = dons.Skip(20).Take(20);
```

`SelectMany` làm phẳng danh sách lồng danh sách — không có nó thì phải hai
vòng lặp lồng nhau.

## Hai cách viết, và cách chốt kết quả

```csharp
// Method syntax — phổ biến hơn
var a2 = dons.Where(d => d.Tien > 100)
             .Select(d => d.Tien);

// Query syntax — giống SQL
var b2 = from d in dons
         where d.Tien > 100
         select d.Tien;

var list = dons.ToList();
// trùng khoá là ném lỗi
var map = dons.ToDictionary(d => d.Khach);
// cho phép trùng, mỗi khoá một nhóm
var look = dons.ToLookup(d => d.Khach);
```

## Dấu hiệu trong code của bạn

- `Count() > 0` → đổi sang `Any()`, nó dừng ngay ở phần tử đầu.
- `First()` dùng để lấy bản ghi theo id → nên là `Single()`, để trùng dữ liệu lộ ra thay vì bị giấu.
- `FirstOrDefault()` rồi dùng luôn kết quả → thiếu kiểm tra null.
- `Dictionary` tạm dựng bằng vòng lặp để cộng dồn theo nhóm → `GroupBy` làm đúng việc đó, ngắn hơn và đọc ra ý định.
- `ToList()` đứng giữa chuỗi phép → chốt sớm, xem bài sau khi dữ liệu ở database.

## Ghi nhớ

- `Any()` thay cho `Count() > 0`; `Single` thay cho `First` khi dữ liệu lẽ ra chỉ có một.
- `Max` trả giá trị, `MaxBy` trả phần tử.
- `GroupBy` + `Select` thay cho `Dictionary` tạm.
- Nối nhiều phép rồi `ToList()` **một lần** ở cuối.

## Bước tiếp theo

Bài sau — **LINQ khi chạm database** — cũng những phép này, nhưng chạy trên EF
Core thì một câu viết ẩu có thể kéo cả bảng về máy chủ ứng dụng.

```quiz
[
  {
    "prompt": "Bạn cần lấy đơn hàng theo id — id là khoá chính nên lẽ ra chỉ có một. Dùng gì?",
    "options": [
      "First(d => d.Id == id)",
      "Single(d => d.Id == id)",
      "Where(d => d.Id == id).ToList()",
      "FirstOrDefault(d => d.Id == id)!"
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
      "dons.Max(d => d.Tien)",
      "dons.MaxBy(d => d.Tien)",
      "dons.OrderBy(d => d.Tien).First()",
      "dons.Select(d => d.Tien).Max()"
    ],
    "answer": 2,
    "explain": "Max trả về giá trị lớn nhất, MaxBy trả về chính phần tử mang giá trị đó. Phương án C cũng ra phần tử nhưng là nhỏ nhất, vì OrderBy sắp tăng dần."
  },
  {
    "prompt": "Đoạn vòng lặp này làm gì, và LINQ viết lại thế nào?",
    "code": "var tong = new Dictionary<string, decimal>();\nforeach (var d in dons)\n{\n    tong.TryGetValue(d.Khach, out var t);\n    tong[d.Khach] = t + d.Tien;\n}",
    "options": [
      "Lọc theo khách — dùng Where",
      "Cộng tiền theo từng khách — dùng GroupBy rồi Sum",
      "Loại trùng khách — dùng Distinct",
      "Ghép hai danh sách — dùng Join"
    ],
    "answer": 2,
    "explain": "Đây đúng là mẫu gom nhóm rồi cộng dồn: dons.GroupBy(d => d.Khach).Select(g => new { g.Key, Tong = g.Sum(d => d.Tien) })."
  }
]
```
