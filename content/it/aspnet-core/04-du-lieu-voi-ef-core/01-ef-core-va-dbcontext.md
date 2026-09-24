---
title: EF Core và DbContext
minutes: 5
---

Dữ liệu trong bộ nhớ mất mỗi lần tắt server. Cửa hàng thật cần lưu sản phẩm
vào database. EF Core cho phép làm việc với database bằng class và LINQ quen
thuộc, không phải tự viết SQL cho từng thao tác.

## Khái niệm

🗄️ **ORM (Object-Relational Mapper)**: thư viện chuyển qua lại giữa object C# và dòng trong bảng database. EF Core là ORM chính thức của .NET.

🧭 **DbContext**: class đại diện cho một phiên làm việc với database. Mỗi `DbSet<T>` trong nó ứng với một bảng.

| C# | Database |
|---|---|
| class `Product` | bảng `Products` |
| property `Name` | cột `Name` |
| một object `Product` | một dòng trong bảng |
| property `Id` | khoá chính, tự tăng |

## Ví dụ

Cài package cho SQLite:

```bash
dotnet add package Microsoft.EntityFrameworkCore.Sqlite
```

Tạo `DbContext` và đăng ký vào `Program.cs`:

```csharp
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();
builder.Services.AddDbContext<ShopDbContext>(options =>
    options.UseSqlite("Data Source=shop.db"));

var app = builder.Build();
app.MapControllers();
app.Run();

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
}

public class ShopDbContext : DbContext
{
    public ShopDbContext(
        DbContextOptions<ShopDbContext> options)
        : base(options)
    {
    }

    public DbSet<Product> Products => Set<Product>();
}

[ApiController]
[Route("api/products")]
public class ProductsController : ControllerBase
{
    private readonly ShopDbContext _db;

    public ProductsController(ShopDbContext db)
    {
        _db = db;
    }

    [HttpGet]
    public async Task<List<Product>> GetAll() =>
        await _db.Products.ToListAsync();
}
```

- `ShopDbContext` kế thừa `DbContext`, có một `DbSet<Product>` ứng với bảng
  `Products`.
- `UseSqlite("Data Source=shop.db")` chọn SQLite, dữ liệu lưu trong file
  `shop.db` cạnh project.
- `AddDbContext` đăng ký `ShopDbContext` với vòng đời `Scoped`, mỗi request
  một phiên làm việc.
- `ToListAsync()` đọc cả bảng. Dùng bản `Async` vì đọc database là việc chậm.

## Thử ngay

Chạy server với code trên, rồi gọi:

```bash
curl -i http://localhost:5000/api/products
```

**Đoán trước khi chạy:** code biên dịch được và server chạy bình thường. Lần
gọi này trả về danh sách rỗng hay lỗi?

<details>
<summary>Xem kết quả</summary>

```text
HTTP/1.1 500 Internal Server Error

Log của server:
SQLite Error 1: 'no such table: Products'.
```

Lỗi 500. EF Core mở được file `shop.db`, nhưng trong đó chưa có bảng
`Products`. Class C# không tự sinh ra bảng. Bài sau dùng migration để tạo
bảng.

</details>

## Lỗi hay gặp

**Quên đăng ký `DbContext`.** Controller cần `ShopDbContext` nhưng container
không biết cách tạo, nên request trả 500.

```csharp
// SAI — thiếu AddDbContext
var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();
var app = builder.Build();
```

**Viết cứng chuỗi kết nối trong code.** Mỗi môi trường thường dùng một
database khác nhau.
Đặt chuỗi kết nối vào `appsettings.json` như bài Cấu hình:

```csharp
// ĐÚNG — đọc từ mục ConnectionStrings:Shop
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);
var cs = builder.Configuration
    .GetConnectionString("Shop");
builder.Services.AddDbContext<ShopDbContext>(options =>
    options.UseSqlite(cs));
```

## Tóm tắt

- EF Core là ORM: class thành bảng, object thành dòng.
- `DbContext` là phiên làm việc với database, mỗi `DbSet<T>` là một bảng.
- Đăng ký bằng `AddDbContext` trong `Program.cs`, nhận qua constructor.
- Chỉ có class thì chưa đủ, phải tạo bảng trong database trước khi dùng.

```quiz
[
  {
    "prompt": "DbContext có DbSet<Customer> Customers. Trong database, nó ứng với gì?",
    "options": [
      "Một dòng dữ liệu",
      "Một cột",
      "Bảng Customers",
      "Một file database"
    ],
    "answer": 3,
    "explain": "Mỗi DbSet<T> ứng với một bảng. Mỗi object Customer là một dòng trong bảng đó."
  },
  {
    "prompt": "AddDbContext đăng ký DbContext với vòng đời nào?",
    "options": [
      "Scoped, mỗi request một phiên",
      "Singleton",
      "Transient",
      "Không đăng ký vào container"
    ],
    "answer": 1,
    "explain": "Mặc định là Scoped: mỗi request có một DbContext riêng, xong request thì bỏ đi."
  },
  {
    "prompt": "Gọi API nhận 500, log báo \"no such table: Orders\". Nguyên nhân?",
    "options": [
      "Thiếu [ApiController]",
      "Chưa cài .NET",
      "Sai địa chỉ server",
      "Database chưa có bảng Orders, cần tạo bằng migration"
    ],
    "answer": 4,
    "explain": "EF Core kết nối được database nhưng bảng chưa tồn tại. Class trong C# không tự tạo bảng."
  }
]
```
