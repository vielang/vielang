---
title: LINQ thường dùng
minutes: 10
---

**LINQ** (Language Integrated Query) là bộ phép toán trên tập dữ liệu, viết ngay
trong C#. Một truy vấn LINQ thay được mười dòng vòng lặp, và quan trọng hơn là
đọc ra ngay ý định.

## Lọc, chiếu, sắp xếp

```csharp
var ketQua = orders
    .Where(o => o.Total > 100_000)                       // lọc
    .OrderByDescending(o => o.CreatedAt)                 // sắp xếp giảm dần
    .ThenBy(o => o.CustomerName)                         // cùng ngày thì theo tên
    .Select(o => new { o.Id, o.CustomerName, o.Total })  // chỉ lấy vài field
    .ToList();
```

`Select` gọi là **projection**: biến mỗi phần tử thành hình dạng khác. Lấy đúng
thứ cần thay vì bê cả object là thói quen tốt, nhất là khi dữ liệu đến từ
database.

## Lấy một phần tử

```csharp
var a = orders.First(o => o.Id == id);             // không có → ném exception
var b = orders.FirstOrDefault(o => o.Id == id);    // không có → null
var c = orders.Single(o => o.Id == id);            // có 2 cái trở lên → ném exception
var d = orders.SingleOrDefault(o => o.Id == id);
```

Chọn theo ý định: `Single` nói "chắc chắn chỉ có một, có hai là dữ liệu hỏng, hãy
báo lỗi". `First` nói "lấy cái đầu, còn lại kệ". Dùng `First` cho khoá chính là
giấu mất lỗi trùng dữ liệu.

## Kiểm tra và đếm

```csharp
bool coDonHuy = orders.Any(o => o.IsCancelled);
bool tatCaDaTra = orders.All(o => o.IsPaid);
int soDon      = orders.Count(o => o.IsPaid);

if (orders.Any()) { }        // đúng: dừng ngay khi thấy phần tử đầu
if (orders.Count() > 0) { }  // phải duyệt hết mới biết
```

## Tính tổng

```csharp
decimal doanhThu = orders.Sum(o => o.Total);
decimal trungBinh = orders.Average(o => o.Total);
decimal lonNhat  = orders.Max(o => o.Total);
var donLonNhat   = orders.MaxBy(o => o.Total);     // trả về CẢ đơn hàng
```

`Max` trả về giá trị lớn nhất, `MaxBy` trả về phần tử có giá trị đó — hay nhầm.
Trên danh sách rỗng, `Sum` ra 0 nhưng `Average` và `Max` thì ném exception.

## Nhóm

```csharp
var theoKhach = orders
    .GroupBy(o => o.CustomerId)
    .Select(g => new
    {
        CustomerId = g.Key,
        SoDon = g.Count(),
        TongTien = g.Sum(o => o.Total),
    })
    .OrderByDescending(x => x.TongTien)
    .ToList();
```

`GroupBy` trả về các nhóm, mỗi nhóm có `Key` và chính nó là một tập phần tử. Đây
là bản LINQ của `GROUP BY` trong SQL.

## Ghép hai tập

```csharp
var chiTiet = orders.Join(
    customers,
    o => o.CustomerId,      // khoá bên trái
    c => c.Id,              // khoá bên phải
    (o, c) => new { o.Id, c.Name, o.Total });
```

Ghép trong bộ nhớ chỉ hợp khi cả hai tập đã nằm sẵn ở đó. Dữ liệu ở database thì
để database ghép — xem bài sau.

## Làm phẳng và loại trùng

```csharp
var moiMatHang = orders.SelectMany(o => o.Items);        // gộp mọi item lại
var maKhach    = orders.Select(o => o.CustomerId).Distinct();
var theoTen    = orders.DistinctBy(o => o.CustomerName); // .NET 6+
var trang2     = orders.Skip(20).Take(20);               // phân trang
```

`SelectMany` làm phẳng danh sách lồng danh sách — không có nó thì phải hai vòng
lặp lồng nhau.

## Hai cách viết

```csharp
// Method syntax — phổ biến hơn, dùng được mọi phép
var a = orders.Where(o => o.IsPaid).Select(o => o.Total);

// Query syntax — giống SQL, dễ đọc khi có join và group
var b = from o in orders
        where o.IsPaid
        select o.Total;
```

Hai cách cho ra cùng một thứ. Code .NET ngày nay dùng **method syntax** là chính.

## Chuyển sang cấu trúc khác

```csharp
var list = orders.ToList();
var map  = orders.ToDictionary(o => o.Id);            // trùng khoá → ném exception
var look = orders.ToLookup(o => o.CustomerId);        // cho phép trùng, mỗi khoá một nhóm
```

## Ghi nhớ

- `Any()` thay cho `Count() > 0`.
- `FirstOrDefault` trả `null` — nhớ kiểm tra trước khi dùng, nhất là khi đã bật nullable.
- Nối nhiều phép rồi `ToList()` **một lần** ở cuối, đừng `ToList()` giữa chuỗi.
