---
title: LINQ khi chạm database
minutes: 9
---

Cùng một câu LINQ, chạy trên `List` thì vô hại, chạy trên **EF Core** có thể
thành một truy vấn tải cả bảng về máy chủ ứng dụng. Khác nhau nằm ở một chữ:
`IQueryable`.

## IEnumerable và IQueryable

```csharp
IEnumerable<Order> a = db.Orders.AsEnumerable().Where(o => o.Total > 100_000);
IQueryable<Order>  b = db.Orders.Where(o => o.Total > 100_000);
```

- `IEnumerable<T>`: lọc **trong bộ nhớ**, tức là phải tải dữ liệu về trước. Dòng trên kéo **cả bảng** `Orders` về rồi mới lọc.
- `IQueryable<T>`: điều kiện được **dịch sang SQL** và gửi cho database. Dòng dưới sinh ra `WHERE Total > 100000`.

Với bảng vài chục dòng thì không ai thấy khác biệt. Với bảng vài triệu dòng thì
một bên trả về trong mili giây, một bên làm sập service.

## Xếp thứ tự các phép

```csharp
var top = await db.Orders
    .Where(o => o.IsPaid)                       // còn IQueryable → vào SQL
    .OrderByDescending(o => o.CreatedAt)
    .Take(20)
    .Select(o => new OrderDto(o.Id, o.Total))   // chỉ chọn cột cần
    .ToListAsync();                             // chốt: gửi truy vấn, nhận kết quả
```

Mọi phép trước `ToListAsync` đều gộp lại thành **một** câu SQL. Sau `ToList` thì
dữ liệu đã nằm trong bộ nhớ, mọi phép tiếp theo là LINQ thường.

```csharp
var sai = (await db.Orders.ToListAsync())       // tải hết bảng về
    .Where(o => o.IsPaid)                       // rồi mới lọc trong bộ nhớ
    .Take(20);
```

## N + 1 — cái bẫy kinh điển

```csharp
var orders = await db.Orders.Take(20).ToListAsync();
foreach (var o in orders)
    Console.WriteLine(o.Customer.Name);   // mỗi vòng lặp là MỘT truy vấn nữa
```

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

Một truy vấn lấy đơn hàng, rồi 20 truy vấn lấy khách hàng: **N + 1**. Lấy sẵn dữ
liệu liên quan trong cùng một lần:

```csharp
var orders = await db.Orders
    .Include(o => o.Customer)      // join sẵn
    .Take(20)
    .ToListAsync();
```

Hoặc tốt hơn, chiếu thẳng sang DTO — EF Core chỉ lấy đúng cột được dùng:

```csharp
var rows = await db.Orders
    .Select(o => new OrderRow(o.Id, o.Customer.Name, o.Total))
    .Take(20)
    .ToListAsync();
```

## Không phải hàm nào cũng dịch được

```csharp
var q = db.Orders.Where(o => TinhDiem(o) > 10);   // hàm C# của bạn
```

EF Core không biết dịch `TinhDiem` sang SQL nên sẽ **ném exception** lúc chạy.
Muốn dùng logic C# thì phải chốt dữ liệu về bộ nhớ trước — và chỉ nên làm vậy sau
khi đã lọc cho tập đủ nhỏ:

```csharp
var canXet = await db.Orders
    .Where(o => o.IsPaid && o.CreatedAt >= tuNgay)   // lọc ở database
    .ToListAsync();

var ketQua = canXet.Where(o => TinhDiem(o) > 10);    // tính trong bộ nhớ
```

## Truy vấn chỉ để đọc

```csharp
var rows = await db.Orders
    .AsNoTracking()          // không theo dõi thay đổi
    .Where(o => o.IsPaid)
    .ToListAsync();
```

Mặc định EF Core ghi nhớ mọi thực thể đã tải để phát hiện thay đổi khi lưu. Với
truy vấn chỉ để hiển thị, `AsNoTracking` bỏ phần ghi nhớ đó — nhanh hơn và tốn ít
bộ nhớ hơn.

## Đếm và tồn tại

```csharp
bool co = await db.Orders.AnyAsync(o => o.CustomerId == id);   // SELECT EXISTS
int n   = await db.Orders.CountAsync(o => o.IsPaid);           // SELECT COUNT
```

Đừng `ToListAsync()` rồi mới `.Count` — đó là kéo cả tập về chỉ để đếm.

## Ghi nhớ

- `IQueryable` → chạy ở database; `IEnumerable` → chạy trong bộ nhớ. Biết mình đang cầm cái nào.
- Lọc, sắp xếp, phân trang **trước** khi chốt bằng `ToListAsync`.
- Duyệt vòng lặp mà chạm tới navigation property là dấu hiệu N + 1: dùng `Include` hoặc `Select`.
- Bật log SQL của EF Core khi phát triển — nhìn câu SQL thật là cách nhanh nhất để biết truy vấn có đúng ý không.
