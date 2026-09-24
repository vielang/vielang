---
title: EF Core và DbContext
minutes: 6
---

Dữ liệu trong bộ nhớ mất mỗi lần tắt server. Cửa hàng thật cần lưu sản phẩm
vào database, và bạn đã có sẵn Oracle từ khoá SQL. EF Core cho phép làm việc
với Oracle bằng class và LINQ quen thuộc, không phải tự viết SQL cho từng thao
tác.

## Khái niệm

🗄️ **ORM (Object-Relational Mapper)**: thư viện chuyển qua lại giữa object C# và dòng trong bảng database. EF Core là ORM chính thức của .NET.

🧭 **DbContext**: class đại diện cho một phiên làm việc với database. Mỗi `DbSet<T>` trong nó ứng với một bảng.

| C# | Oracle |
|---|---|
| class `Product`, `DbSet` tên `Products` | bảng `PRODUCTS` |
| property `Name` | cột `NAME` |
| một object `Product` | một dòng trong bảng |
| property `Id` | khoá chính, tự tăng |

## Tạo user cho API

API dùng một user riêng tên `shopapi`, tách khỏi user `shop` của khoá SQL.
Mở VS Code, kết nối Oracle bằng user `system`, mật khẩu `oracle_pw`, service
name `FREEPDB1`, rồi chạy một lần:

```sql
CREATE USER shopapi IDENTIFIED BY shopapi_pw
  QUOTA UNLIMITED ON USERS;
GRANT CREATE SESSION, CREATE TABLE, CREATE SEQUENCE
  TO shopapi;
```

User mới chưa có bảng nào. Bài sau dùng migration để EF Core tự tạo bảng.

## Ví dụ

Cài package cho Oracle:

```bash
dotnet add package Oracle.EntityFrameworkCore
dotnet add package EFCore.NamingConventions
```

Thêm chuỗi kết nối vào `appsettings.json`:

```json
"ConnectionStrings": {
  "Shop": "User Id=shopapi;Password=shopapi_pw;Data Source=localhost:1521/FREEPDB1"
}
```

Tạo `DbContext` và đăng ký vào `Program.cs`:

```csharp
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);
var cs = builder.Configuration
    .GetConnectionString("Shop");

builder.Services.AddControllers();
builder.Services.AddDbContext<ShopDbContext>(options =>
    options.UseOracle(cs)
        .UseUpperSnakeCaseNamingConvention());

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

- `UseOracle(cs)` chọn Oracle, đọc chuỗi kết nối từ cấu hình.
- `UseUpperSnakeCaseNamingConvention()` đổi tên sang chữ hoa nối gạch dưới:
  `Products` thành `PRODUCTS`, `CustomerName` thành `CUSTOMER_NAME`. Tên
  khớp với cách viết của khoá SQL.
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
ORA-00942: table or view "SHOPAPI"."PRODUCTS" does not exist
```

Lỗi 500. EF Core kết nối được Oracle, nhưng user `shopapi` chưa có bảng
`PRODUCTS`. Class C# không tự sinh ra bảng. Bài sau dùng migration để tạo
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

**Bỏ `UseUpperSnakeCaseNamingConvention()`.** EF Core với Oracle đặt tên bảng
trong ngoặc kép, giữ nguyên chữ hoa chữ thường, ví dụ `"Products"`. Viết
`SELECT * FROM products` như khoá SQL sẽ báo `ORA-00942`, vì Oracle đổi
`products` không có ngoặc kép thành `PRODUCTS`, khác với `"Products"`.

## Tóm tắt

- EF Core là ORM: class thành bảng, object thành dòng.
- `DbContext` là phiên làm việc với database, mỗi `DbSet<T>` là một bảng.
- Kết nối Oracle bằng `UseOracle`, đặt tên chữ hoa bằng
  `UseUpperSnakeCaseNamingConvention()`.
- API dùng user riêng. Có class chưa đủ, phải tạo bảng trước khi dùng.

```quiz
[
  {
    "prompt": "DbContext có DbSet<Customer> Customers. Với UseUpperSnakeCaseNamingConvention, bảng trong Oracle tên là gì?",
    "options": [
      "customers",
      "\"Customers\"",
      "CUSTOMERS",
      "Customer"
    ],
    "answer": 3,
    "explain": "Tên DbSet Customers được đổi sang chữ hoa nối gạch dưới thành CUSTOMERS, khớp với cách Oracle lưu tên không có ngoặc kép."
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
    "prompt": "Gọi API nhận 500, log báo ORA-00942 table or view \"SHOPAPI\".\"ORDERS\" does not exist. Nguyên nhân?",
    "options": [
      "Thiếu [ApiController]",
      "Chưa cài .NET",
      "Sai địa chỉ server",
      "User shopapi chưa có bảng ORDERS, cần tạo bằng migration"
    ],
    "answer": 4,
    "explain": "EF Core kết nối được Oracle nhưng bảng chưa tồn tại. Class trong C# không tự tạo bảng."
  }
]
```
