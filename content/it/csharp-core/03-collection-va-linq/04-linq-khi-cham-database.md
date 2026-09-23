---
title: LINQ khi chạm database
minutes: 11
---

Đoạn này nằm trong một controller. Bốn dòng, và đọc qua thì không có gì đáng
ngờ.

```csharp
var orders = await db.Orders
    .Take(20)
    .ToListAsync();

foreach (var o in orders)
    rows.Add(new OrderRow(
        o.Id, o.Customer.Name, o.Total));
```

Không `Include`. Không `Select`. Chỉ một vòng lặp lấy ra hai mươi dòng.

Nó gửi xuống database **hai mươi mốt câu SQL**, và API mất 3 giây cho đúng hai
mươi dòng ấy.

> **Học xong bài này bạn sẽ:** biết câu LINQ của mình chạy ở database hay
> trong bộ nhớ; nhận ra và sửa bẫy N + 1; đọc được SQL mà EF Core sinh ra.
>
> **Cần biết trước:** LINQ cơ bản và hoãn thực thi (hai bài trước). Phần Thử
> ngay cần một project EF Core đã nối được database; chưa có thì đọc câu SQL in
> ra là đủ.

## IQueryable chạy ở database, IEnumerable chạy trong bộ nhớ

Hai mươi mốt câu SQL cho một vòng `foreach`. Muốn hiểu vì sao, phải biết mình
đang cầm kiểu nào.

```csharp
// lọc TRONG BỘ NHỚ — kéo cả bảng về trước
IEnumerable<Order> a = db.Orders
    .AsEnumerable()
    .Where(o => o.Total > 100_000);

// lọc Ở DATABASE — dịch sang SQL
IQueryable<Order> b = db.Orders
    .Where(o => o.Total > 100_000);
```

| | `IEnumerable<T>` | `IQueryable<T>` |
|---|---|---|
| Phép `Where` chạy ở đâu | trong bộ nhớ | ở database |
| Dữ liệu đi về | **cả bảng** | chỉ kết quả |
| Dịch sang SQL | không | có |

Nhìn code thì hai dòng gần như giống hệt, khác đúng một chữ `AsEnumerable`.

Nhưng chữ ấy là ranh giới. Trước nó, câu lệnh mới là bản thiết kế chờ gửi cho
database.

Sau nó, mọi phép lọc là phép của C#. Nên tới lúc có người duyệt, cả bảng phải
về bộ nhớ máy chủ ứng dụng trước đã.

Với bảng vài chục dòng thì không ai thấy khác biệt. Với bảng vài triệu dòng
thì một bên trả về trong mili giây, một bên làm sập service.

## Thử ngay: nhìn SQL thật

Bật log SQL trong `DbContext`. Một dòng thôi, và nên bật suốt lúc phát triển.

```csharp
protected override void OnConfiguring(
    DbContextOptionsBuilder o)
{
    o.UseSqlServer(conn)
     .LogTo(Console.WriteLine, LogLevel.Information);
}
```

Rồi chạy đoạn này.

```csharp
var orders = await db.Orders.Take(20).ToListAsync();

foreach (var o in orders)
    Console.WriteLine(o.Customer.Name);
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

Console in ra **21 câu**. Một câu cho đơn hàng, rồi mỗi vòng lặp thêm một câu
nữa để lấy khách.

Tên gọi của nó là **N + 1**, và đó là chỗ ba giây kia trốn.

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

Có hai cách sửa.

```csharp
// Lấy sẵn dữ liệu liên quan trong cùng một câu
var orders = await db.Orders
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

`Include` bảo EF Core lấy sẵn khách hàng ngay trong cùng một câu SQL. `Select`
đi xa hơn một bước và chỉ lấy đúng ba cột bạn cần.

Hai mươi mốt câu vừa rút về một.

## Mọi phép trước ToListAsync gộp thành một câu SQL

Sửa được N + 1 rồi, còn một câu hỏi nữa: chốt truy vấn ở chỗ nào.

```csharp
var top = await db.Orders
    .Where(o => o.IsPaid)          // vào SQL
    .OrderByDescending(o => o.CreatedAt)
    .Take(20)
    .Select(o => new OrderDto(o.Id, o.Total))
    .ToListAsync();                // gửi truy vấn
```

Mọi phép trước `ToListAsync` gộp lại thành **một** câu SQL. Sau `ToList` thì
dữ liệu đã nằm trong bộ nhớ. Mọi phép tiếp theo là LINQ thường.

```csharp
// SAI — tải cả bảng rồi mới lọc
var wrong = (await db.Orders.ToListAsync())
    .Where(o => o.IsPaid)
    .Take(20);
```

Đoạn này vẫn ra đúng 20 đơn, chỉ có điều nó tải cả triệu dòng về rồi mới vứt
đi 999.980 dòng.

Quy tắc gọn lại một câu: lọc, sắp xếp và phân trang phải đứng **trước** dấu
chốt.

## Không phải hàm nào cũng dịch được

EF Core dịch được `Where` và `OrderBy`. Nhưng nó không dịch được method của
bạn.

```csharp
var q = db.Orders.Where(o => Score(o) > 10);
```

EF Core không biết dịch `Score` sang SQL, nên nó **ném exception lúc chạy**.

Muốn dùng logic C# thì chốt dữ liệu về bộ nhớ trước. Và chỉ làm vậy sau khi đã
lọc cho tập đủ nhỏ.

```csharp
var candidates = await db.Orders
    .Where(o => o.IsPaid && o.CreatedAt >= fromDate)
    .ToListAsync();

var result = candidates.Where(o => Score(o) > 10);
```

## Truy vấn chỉ để hiển thị thì bỏ phần theo dõi thay đổi

Còn hai thói quen nhỏ, và cả hai đều giúp truy vấn nhẹ đi mà không phải viết
thêm gì.

```csharp
var rows = await db.Orders
    .AsNoTracking()
    .Where(o => o.IsPaid)
    .ToListAsync();

bool exists = await db.Orders.AnyAsync(o => o.Id == id);
int paid = await db.Orders.CountAsync(o => o.IsPaid);
```

Mặc định EF Core **theo dõi** mọi thực thể đã tải, để phát hiện thay đổi khi
lưu. `AsNoTracking` chính là tắt phần theo dõi đó.

Truy vấn chỉ để hiển thị thì đâu cần theo dõi gì.

Và đừng `ToListAsync()` rồi mới `.Count`. Đó là kéo cả tập về chỉ để đếm.

## Dấu hiệu trong code của bạn

- Vòng lặp chạm tới navigation property (`o.Customer.Name`) mà truy vấn không có `Include` hay `Select` → N + 1.
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

Hết chương **Collection và LINQ**. Bạn đã biết chọn chỗ chứa dữ liệu, xử lý
chúng bằng LINQ, và nhìn ra câu SQL thật phía sau.

Chương kế của khoá là **Ngoại lệ và tài nguyên**. Nó mở bằng hai khối `catch`
khác nhau đúng một chữ — và chữ ấy quyết định lúc hai giờ sáng bạn có tìm được
bug hay không.

```quiz
[
  {
    "prompt": "Đoạn này sinh ra bao nhiêu câu SQL, với 20 đơn hàng?",
    "code": "var orders = await db.Orders.Take(20).ToListAsync();\n\nforeach (var d in orders)\n    Console.WriteLine(o.Customer.Name);",
    "options": [
      "1",
      "21",
      "2",
      "40"
    ],
    "answer": 2,
    "explain": "Một câu lấy 20 đơn, rồi mỗi lần chạm o.Customer là thêm một câu — tổng 21. Sửa bằng Include(o => o.Customer) hoặc Select sang DTO."
  },
  {
    "prompt": "Câu nào chạy ở database, câu nào kéo cả bảng về bộ nhớ?",
    "code": "var a = db.Orders.AsEnumerable()\n    .Where(o => o.Total > 100_000);\n\nvar b = db.Orders\n    .Where(o => o.Total > 100_000);",
    "options": [
      "Cả hai đều chạy ở database",
      "a chạy ở database, b trong bộ nhớ",
      "Cả hai đều kéo cả bảng về",
      "b chạy ở database, a kéo cả bảng về rồi mới lọc"
    ],
    "answer": 4,
    "explain": "AsEnumerable() cắt đứt IQueryable: từ đó trở đi mọi phép chạy trong bộ nhớ, nên dữ liệu phải về trước."
  },
  {
    "prompt": "Bạn viết db.Orders.Where(o => Score(o) > 10) với Score là method C# của bạn. Chuyện gì xảy ra?",
    "options": [
      "Ném exception lúc chạy vì không dịch được sang SQL",
      "EF Core tự dịch method sang SQL",
      "Chạy được nhưng chậm",
      "Lỗi compile"
    ],
    "answer": 1,
    "explain": "EF Core chỉ dịch được các biểu thức nó hiểu. Muốn dùng logic C#, hãy lọc ở database trước cho tập đủ nhỏ rồi ToListAsync, sau đó mới tính."
  },
  {
    "prompt": "Truy vấn chỉ để hiển thị danh sách, không sửa gì. Thêm gì để nhẹ hơn?",
    "options": [
      "AsEnumerable()",
      "ToList() sớm nhất có thể",
      "AsNoTracking()",
      "Include() mọi quan hệ"
    ],
    "answer": 3,
    "explain": "Mặc định EF Core theo dõi mọi thực thể đã tải để phát hiện thay đổi. Truy vấn chỉ đọc thì bỏ phần theo dõi đó đi."
  }
]
```
