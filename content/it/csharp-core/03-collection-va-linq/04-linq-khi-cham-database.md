---
title: LINQ khi chạm database
minutes: 11
---

API danh sách đơn hàng mất 8 giây. Bật log SQL lên thì thấy **21 câu truy
vấn** cho một lần gọi: một câu lấy 20 đơn, rồi hai mươi câu nữa, mỗi câu lấy
tên một khách hàng. Trong code chỉ có đúng một vòng `foreach`.

> **Học xong bài này bạn sẽ:** biết câu LINQ của mình chạy ở database hay
> trong bộ nhớ; nhận ra và sửa bẫy N + 1; đọc được SQL mà EF Core sinh ra.
>
> **Cần biết trước:** LINQ cơ bản và hoãn thực thi (hai bài trước).

## IEnumerable hay IQueryable

```csharp
// lọc TRONG BỘ NHỚ — kéo cả bảng về trước
IEnumerable<Order> a = db.Orders
    .AsEnumerable()
    .Where(o => o.Total > 100_000);

// lọc Ở DATABASE — dịch sang SQL
IQueryable<Order> b = db.Orders
    .Where(o => o.Total > 100_000);
```

- `IEnumerable<T>`: các phép chạy trong bộ nhớ, nên dữ liệu phải về máy chủ ứng dụng trước.
- `IQueryable<T>`: điều kiện được **dịch sang SQL** và gửi cho database, chỉ kết quả mới đi về.

Với bảng vài chục dòng thì không ai thấy khác biệt. Với bảng vài triệu dòng
thì một bên trả về trong mili giây, một bên làm sập service.

## Thử ngay: nhìn SQL thật

Bật log SQL trong `DbContext` — một dòng, và nên bật suốt lúc phát triển:

```csharp
protected override void OnConfiguring(
    DbContextOptionsBuilder o)
{
    o.UseSqlServer(conn)
     .LogTo(Console.WriteLine, LogLevel.Information);
}
```

Rồi chạy đoạn này:

```csharp
var dons = await db.Orders.Take(20).ToListAsync();

foreach (var d in dons)
    Console.WriteLine(d.Customer.Name);
```

**Đoán trước khi chạy:** console in ra bao nhiêu câu `SELECT`?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
SELECT TOP(20) ... FROM [Orders] AS [o]
SELECT ... FROM [Customers] WHERE [Id] = @__p_0
SELECT ... FROM [Customers] WHERE [Id] = @__p_0
... (thêm 18 câu nữa)
```

**21 câu**: một câu lấy đơn hàng, rồi mỗi vòng lặp một câu lấy khách hàng. Tên
gọi của nó là **N + 1**, và đó chính là API 8 giây ở đầu bài.

</details>

```mermaid N + 1: một truy vấn lấy danh sách, rồi mỗi phần tử thêm một truy vấn
sequenceDiagram
    participant App
    participant DB as Database
    App->>DB: SELECT * FROM Orders LIMIT 20
    DB-->>App: 20 đơn hàng
    loop mỗi đơn hàng
        App->>DB: SELECT * FROM Customers WHERE Id = ?
        DB-->>App: 1 khách hàng
    end
```

Hai cách sửa:

```csharp
// Lấy sẵn dữ liệu liên quan trong cùng một câu
var dons = await db.Orders
    .Include(o => o.Customer)
    .Take(20)
    .ToListAsync();
```

```csharp
// Tốt hơn: chiếu thẳng sang DTO, chỉ lấy cột cần
var rows = await db.Orders
    .Select(o => new OrderRow(
        o.Id, o.Customer.Name, o.Total))
    .Take(20)
    .ToListAsync();
```

## Thứ tự các phép

```csharp
var top = await db.Orders
    .Where(o => o.IsPaid)          // vào SQL
    .OrderByDescending(o => o.CreatedAt)
    .Take(20)
    .Select(o => new OrderDto(o.Id, o.Total))
    .ToListAsync();                // gửi truy vấn
```

Mọi phép trước `ToListAsync` gộp lại thành **một** câu SQL. Sau `ToList` thì dữ
liệu đã nằm trong bộ nhớ, mọi phép tiếp theo là LINQ thường.

```csharp
// SAI — tải cả bảng rồi mới lọc
var sai = (await db.Orders.ToListAsync())
    .Where(o => o.IsPaid)
    .Take(20);
```

## Không phải hàm nào cũng dịch được

```csharp
var q = db.Orders.Where(o => TinhDiem(o) > 10);
```

EF Core không biết dịch `TinhDiem` sang SQL nên **ném exception lúc chạy**.
Muốn dùng logic C# thì chốt dữ liệu về bộ nhớ trước — và chỉ làm vậy sau khi
đã lọc cho tập đủ nhỏ:

```csharp
var canXet = await db.Orders
    .Where(o => o.IsPaid && o.CreatedAt >= tuNgay)
    .ToListAsync();

var ketQua = canXet.Where(o => TinhDiem(o) > 10);
```

## Truy vấn chỉ để đọc, và đếm

```csharp
var rows = await db.Orders
    .AsNoTracking()
    .Where(o => o.IsPaid)
    .ToListAsync();

bool co = await db.Orders.AnyAsync(o => o.Id == id);
int n = await db.Orders.CountAsync(o => o.IsPaid);
```

Mặc định EF Core ghi nhớ mọi thực thể đã tải để phát hiện thay đổi khi lưu.
Với truy vấn chỉ để hiển thị, `AsNoTracking` bỏ phần ghi nhớ đó — nhanh hơn và
tốn ít bộ nhớ hơn. Và đừng `ToListAsync()` rồi mới `.Count` — đó là kéo cả tập
về chỉ để đếm.

## Dấu hiệu trong code của bạn

- Vòng lặp chạm tới navigation property (`d.Customer.Name`) mà truy vấn không có `Include` hay `Select` → N + 1.
- `.ToList()` hoặc `.AsEnumerable()` đứng **trước** một `Where` → cả bảng đang về bộ nhớ rồi mới lọc.
- Truy vấn chỉ để hiển thị mà thiếu `AsNoTracking()`.
- Gọi một method C# của bạn bên trong `Where` trên `IQueryable` → sẽ ném lỗi lúc chạy.
- Không ai trong nhóm biết API của mình sinh ra bao nhiêu câu SQL → bật `LogTo` ngay hôm nay.

## Ghi nhớ

- `IQueryable` chạy ở database, `IEnumerable` chạy trong bộ nhớ — luôn biết mình đang cầm cái nào.
- Lọc, sắp xếp, phân trang **trước** khi chốt bằng `ToListAsync`.
- N + 1 sửa bằng `Include` hoặc `Select` sang DTO.
- Bật log SQL lúc phát triển: nhìn câu SQL thật là cách nhanh nhất để biết truy vấn có đúng ý không.

## Bước tiếp theo

Hết chương **Collection và LINQ**. Chương kế của khoá sẽ là **Ngoại lệ và tài
nguyên**: xử lý lỗi cho đúng, và đóng file, connection, socket đúng lúc bằng
`using`.

```quiz
[
  {
    "prompt": "Đoạn này sinh ra bao nhiêu câu SQL, với 20 đơn hàng?",
    "code": "var dons = await db.Orders.Take(20).ToListAsync();\n\nforeach (var d in dons)\n    Console.WriteLine(d.Customer.Name);",
    "options": ["1", "2", "21", "40"],
    "answer": 3,
    "explain": "Một câu lấy 20 đơn, rồi mỗi lần chạm d.Customer là thêm một câu — tổng 21. Sửa bằng Include(o => o.Customer) hoặc Select sang DTO."
  },
  {
    "prompt": "Câu nào chạy ở database, câu nào kéo cả bảng về bộ nhớ?",
    "code": "var a = db.Orders.AsEnumerable()\n    .Where(o => o.Total > 100_000);\n\nvar b = db.Orders\n    .Where(o => o.Total > 100_000);",
    "options": [
      "Cả hai đều chạy ở database",
      "a chạy ở database, b trong bộ nhớ",
      "b chạy ở database, a kéo cả bảng về rồi mới lọc",
      "Cả hai đều kéo cả bảng về"
    ],
    "answer": 3,
    "explain": "AsEnumerable() cắt đứt IQueryable: từ đó trở đi mọi phép chạy trong bộ nhớ, nên dữ liệu phải về trước."
  },
  {
    "prompt": "Bạn viết db.Orders.Where(o => TinhDiem(o) > 10) với TinhDiem là method C# của bạn. Chuyện gì xảy ra?",
    "options": [
      "EF Core tự dịch method sang SQL",
      "Ném exception lúc chạy vì không dịch được sang SQL",
      "Chạy được nhưng chậm",
      "Lỗi compile"
    ],
    "answer": 2,
    "explain": "EF Core chỉ dịch được các biểu thức nó hiểu. Muốn dùng logic C#, hãy lọc ở database trước cho tập đủ nhỏ rồi ToListAsync, sau đó mới tính."
  },
  {
    "prompt": "Truy vấn chỉ để hiển thị danh sách, không sửa gì. Thêm gì để nhẹ hơn?",
    "options": [
      "AsEnumerable()",
      "AsNoTracking()",
      "ToList() sớm nhất có thể",
      "Include() mọi quan hệ"
    ],
    "answer": 2,
    "explain": "Mặc định EF Core theo dõi mọi thực thể đã tải để phát hiện thay đổi. Truy vấn chỉ đọc thì bỏ phần theo dõi đó đi."
  }
]
```
