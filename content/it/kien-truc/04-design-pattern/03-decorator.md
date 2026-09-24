---
title: Decorator
minutes: 6
---

Cần ghi log mỗi lần đọc danh sách sản phẩm để xem vì sao API chậm. Sửa thẳng
vào `DbProductStore` thì trộn việc log với việc đọc database, và
`FakeProductStore` muốn log cũng phải sửa theo. Decorator thêm việc log bằng
một class bọc bên ngoài, không đụng tới class gốc.

## Khái niệm

🎁 **Decorator**: class implement cùng interface với class gốc, giữ một object gốc bên trong, thêm việc trước hoặc sau rồi chuyển lời gọi cho object đó.

Nơi dùng vẫn chỉ thấy `IProductStore`, không biết mình đang dùng bản gốc hay
bản đã bọc. Bọc nhiều lớp cũng được, mỗi lớp thêm một việc.

## Ví dụ

`IProductStore` và `Product` như bài Truy vấn với EF Core. `MemoryProductStore`
đứng thay `DbProductStore` cho gọn:

```csharp
IProductStore store =
    new LoggingProductStore(new MemoryProductStore());
var products = await store.InStockAsync();

public class LoggingProductStore : IProductStore
{
    private readonly IProductStore _inner;

    public LoggingProductStore(IProductStore inner)
    {
        _inner = inner;
    }

    public async Task<List<Product>> InStockAsync()
    {
        Console.WriteLine("Bắt đầu đọc sản phẩm");
        var result = await _inner.InStockAsync();
        Console.WriteLine(
            $"Xong, {result.Count} sản phẩm");
        return result;
    }

    public Task<bool> SetStockAsync(int id, int stock)
    {
        return _inner.SetStockAsync(id, stock);
    }
}

public class MemoryProductStore : IProductStore
{
    public Task<List<Product>> InStockAsync()
    {
        var list = new List<Product>
        {
            new Product { Id = 1, Name = "Bút bi" }
        };
        return Task.FromResult(list);
    }

    public Task<bool> SetStockAsync(int id, int stock)
    {
        return Task.FromResult(true);
    }
}

// Có sẵn trong ShopApi từ khoá ASP.NET Core
public interface IProductStore
{
    Task<List<Product>> InStockAsync();
    Task<bool> SetStockAsync(int id, int stock);
}

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
    public int Stock { get; set; }
}
```

- `LoggingProductStore` implement `IProductStore` và giữ `_inner` cũng là
  `IProductStore`.
- `InStockAsync` in log, gọi `_inner`, in log tiếp. `SetStockAsync` chuyển
  thẳng cho `_inner`.
- Bọc `DbProductStore` hay `FakeProductStore` đều được, vì decorator chỉ biết
  interface.
- Trong ASP.NET Core, đăng ký bản đã bọc vào container là controller dùng
  ngay, không sửa controller.

## Thử ngay

Sửa `LoggingProductStore` nhận thêm tên lớp bọc và in tên đó ở hai dòng log,
rồi bọc hai lớp:

```csharp
IProductStore inner = new LoggingProductStore(
    "B", new MemoryProductStore());
IProductStore store =
    new LoggingProductStore("A", inner);
await store.InStockAsync();

public class LoggingProductStore : IProductStore
{
    private readonly string _name;
    private readonly IProductStore _inner;

    public LoggingProductStore(
        string name, IProductStore inner)
    {
        _name = name;
        _inner = inner;
    }

    public async Task<List<Product>> InStockAsync()
    {
        Console.WriteLine(_name + ": bắt đầu");
        var result = await _inner.InStockAsync();
        Console.WriteLine(
            $"{_name}: xong, {result.Count} sản phẩm");
        return result;
    }

    public Task<bool> SetStockAsync(int id, int stock)
    {
        return _inner.SetStockAsync(id, stock);
    }
}
```

**Đoán trước khi chạy:** bốn dòng log của A và B in ra theo thứ tự nào?

<details>
<summary>Xem kết quả</summary>

```text
A: bắt đầu
B: bắt đầu
B: xong, 1 sản phẩm
A: xong, 1 sản phẩm
```

Lời gọi đi từ lớp ngoài vào trong, kết quả đi từ trong ra ngoài. Đây đúng là
cách middleware chạy trong bài Middleware và pipeline của khoá ASP.NET Core.

</details>

## Lỗi hay gặp

**Kế thừa class gốc để thêm log.** Class log chỉ dùng được với đúng một class
gốc, bọc `FakeProductStore` phải viết thêm một class log khác.

```csharp
// SAI — gắn chặt với DbProductStore
public class LoggingDbProductStore : DbProductStore
{
    public LoggingDbProductStore(ShopDbContext db)
        : base(db)
    {
    }
}
```

```csharp
// ĐÚNG — bọc bất kỳ IProductStore nào
public class LoggingProductStore : IProductStore
{
    private readonly IProductStore _inner;

    public LoggingProductStore(IProductStore inner)
    {
        _inner = inner;
    }

    public Task<List<Product>> InStockAsync()
    {
        return _inner.InStockAsync();
    }

    public Task<bool> SetStockAsync(int id, int stock)
    {
        return _inner.SetStockAsync(id, stock);
    }
}
```

Đây là "composition hơn kế thừa" ở bài Composition của khoá OOP.

## Tóm tắt

- Decorator bọc object gốc, cùng interface, thêm việc trước hoặc sau.
- Class gốc và nơi dùng đều không phải sửa.
- Bọc nhiều lớp: lời gọi đi từ ngoài vào, kết quả đi từ trong ra.
- Dùng cho log, cache, đo thời gian, kiểm quyền.

```quiz
[
  {
    "prompt": "Muốn cache kết quả InStockAsync mà không sửa DbProductStore. Cách nào hợp nhất?",
    "options": [
      "Sửa DbProductStore thêm cache",
      "Viết CachingProductStore implement IProductStore, bọc DbProductStore",
      "Kế thừa DbProductStore",
      "Thêm cache vào controller"
    ],
    "answer": 2,
    "explain": "Decorator thêm việc cache bên ngoài, class gốc giữ nguyên và bản cache bọc được mọi IProductStore."
  },
  {
    "prompt": "Decorator khác kế thừa ở điểm nào?",
    "options": [
      "Decorator chạy nhanh hơn",
      "Không khác gì",
      "Decorator chỉ dùng cho log",
      "Decorator bọc bất kỳ object nào cùng interface, kế thừa gắn với một class cụ thể"
    ],
    "answer": 4,
    "explain": "Decorator giữ interface bên trong nên bọc được mọi implementation."
  },
  {
    "prompt": "Bọc Timing(Logging(Db)). Gọi InStockAsync thì lớp nào chạy phần \"trước\" đầu tiên?",
    "options": [
      "Timing",
      "Logging",
      "Db",
      "Cùng lúc"
    ],
    "answer": 1,
    "explain": "Lời gọi đi từ lớp ngoài cùng vào trong, nên Timing chạy trước, rồi Logging, rồi Db."
  }
]
```
