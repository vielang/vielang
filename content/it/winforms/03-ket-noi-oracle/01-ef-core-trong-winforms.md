---
title: EF Core trong WinForms
minutes: 6
---

Tới giờ sản phẩm vẫn nằm trong bộ nhớ, tắt app là mất. Bài này nối app với
database `shopapi` mà API của khoá ASP.NET Core đang dùng. Hai app cùng đọc
một bảng `PRODUCTS`, nên sản phẩm thêm qua API sẽ hiện luôn trên màn hình kho.

## Khái niệm

🔌 **DbContextOptionsBuilder**: class dựng cấu hình cho `DbContext`: database nào, chuỗi kết nối nào, quy ước đặt tên nào.

Ở khoá ASP.NET Core, `AddDbContext` dựng cấu hình này và container tạo
`ShopDbContext` cho controller. WinForms không có container sẵn, nên ta tự
dựng rồi tự `new`, đúng cách làm tay ở bài DIP của khoá OOP.

## Ví dụ

Cài hai package như khoá ASP.NET Core:

```bash
dotnet add package Oracle.EntityFrameworkCore
dotnet add package EFCore.NamingConventions
```

Bảng `PRODUCTS` đã được migration của API tạo, app này chỉ đọc ghi. Thay
toàn bộ `Program.cs`:

```csharp
using Microsoft.EntityFrameworkCore;

namespace ShopDesk;

static class Program
{
    [STAThread]
    static void Main()
    {
        ApplicationConfiguration.Initialize();

        var cs = "User Id=shopapi;Password=shopapi_pw;"
            + "Data Source=localhost:1521/FREEPDB1";
        var options =
            new DbContextOptionsBuilder<ShopDbContext>()
                .UseOracle(cs)
                .UseUpperSnakeCaseNamingConvention()
                .Options;
        var db = new ShopDbContext(options);

        Application.Run(new MainForm(db));
    }
}

class MainForm : Form
{
    private readonly ShopDbContext _db;
    private readonly BindingSource _source =
        new BindingSource();

    public MainForm(ShopDbContext db)
    {
        _db = db;
        Text = "Kho hàng";
        Width = 500;

        var grid = new DataGridView
        {
            Dock = DockStyle.Fill,
            DataSource = _source,
            ReadOnly = true,
            AllowUserToAddRows = false
        };
        Controls.Add(grid);
        Load += MainForm_Load;
    }

    private void MainForm_Load(
        object? sender, EventArgs e)
    {
        _source.DataSource = _db.Products.ToList();
    }
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

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
    public int Stock { get; set; }
}
```

- `ShopDbContext` và `Product` chép nguyên từ khoá ASP.NET Core. Cùng class,
  cùng quy ước tên, nên khớp đúng bảng `PRODUCTS` có sẵn.
- `Main` là nơi ghép các phần: dựng `options`, tạo `db`, đưa vào
  constructor của `MainForm`. Form không tự tạo database, nó nhận từ ngoài.
- Chuỗi kết nối viết trong code chỉ để thử trên máy, như bài Cấu hình đã
  nhắc.
- Event `Load` chạy ngay trước khi cửa sổ hiện lần đầu, hợp để đọc dữ liệu.

## Thử ngay

Trong VS Code, tạo kết nối Oracle với user `shopapi`, rồi chạy câu sau nhưng
**chưa** `COMMIT`:

```sql
INSERT INTO PRODUCTS (NAME, PRICE, STOCK)
VALUES ('Thước', 3000, 40);
```

Chạy app bằng `dotnet run`.

**Đoán trước khi chạy:** lưới có dòng "Thước" không? Nếu chưa có, cần làm
gì để nó hiện?

<details>
<summary>Xem kết quả</summary>

```text
Chưa có "Thước".
Chạy COMMIT trong VS Code, đóng app rồi mở lại: có "Thước".
```

App mở một kết nối riêng tới Oracle. Như bài Transaction của khoá SQL, thay
đổi chưa `COMMIT` chỉ phiên đã làm nó mới thấy.

</details>

## Lỗi hay gặp

**Quên `UseUpperSnakeCaseNamingConvention()`.** EF Core tìm bảng
`"Products"` thay vì `PRODUCTS`. Mở app là hiện hộp thoại lỗi báo `ORA-00942`,
giống hệt lỗi ở bài EF Core và DbContext của khoá ASP.NET Core.

```csharp
// SAI — tìm bảng "Products", không có
var options =
    new DbContextOptionsBuilder<ShopDbContext>()
        .UseOracle("...")
        .Options;
```

```csharp
// ĐÚNG — cùng quy ước với API
var options =
    new DbContextOptionsBuilder<ShopDbContext>()
        .UseOracle("...")
        .UseUpperSnakeCaseNamingConvention()
        .Options;
```

## Tóm tắt

- App WinForms dùng lại `ShopDbContext`, `Product` và database của API.
- Tự dựng `options` bằng `DbContextOptionsBuilder`, tự tạo `ShopDbContext`
  trong `Main`.
- Form nhận `ShopDbContext` qua constructor.
- Đọc dữ liệu trong event `Load`. App chỉ thấy dữ liệu đã `COMMIT`.

```quiz
[
  {
    "prompt": "Trong ASP.NET Core, AddDbContext làm hộ việc gì mà trong WinForms ta phải tự làm?",
    "options": [
      "Tạo bảng trong database",
      "Dựng options và tạo object ShopDbContext",
      "Viết câu SQL",
      "Mở Docker"
    ],
    "answer": 2,
    "explain": "AddDbContext dựng cấu hình và để container tạo ShopDbContext. WinForms không có container sẵn nên ta tự làm trong Main."
  },
  {
    "prompt": "Đồng nghiệp UPDATE giá trong VS Code nhưng chưa COMMIT. App kho mở lại thấy giá nào?",
    "options": [
      "Giá mới",
      "Không thấy sản phẩm nào",
      "Báo lỗi khoá dòng",
      "Giá cũ"
    ],
    "answer": 4,
    "explain": "Thay đổi chưa COMMIT chỉ phiên của đồng nghiệp thấy. Kết nối của app vẫn đọc giá đã COMMIT."
  },
  {
    "prompt": "MainForm nhận ShopDbContext qua constructor thay vì tự new bên trong. Đây là cách làm của bài nào ở khoá OOP?",
    "options": [
      "Đóng gói",
      "Kế thừa",
      "DIP và dependency injection",
      "Abstract class"
    ],
    "answer": 3,
    "explain": "Nhận phụ thuộc từ ngoài qua constructor là dependency injection. Main là nơi ghép các phần lại."
  }
]
```
