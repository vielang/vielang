---
title: Migration
minutes: 5
---

Bài trước dừng ở lỗi `ORA-00942`, bảng chưa tồn tại. Class `Product` đã có, nhưng database
chưa có bảng. Về sau, thêm property `Stock` thì database cũng phải đổi theo.
Migration giữ cho cấu trúc database luôn khớp với class trong code.

## Khái niệm

📜 **Migration**: file C# do EF Core sinh ra, mô tả cách đổi cấu trúc database từ phiên bản trước sang phiên bản mới.

Mỗi lần sửa class ảnh hưởng tới bảng, bạn làm hai bước:

| Bước | Lệnh | Làm gì |
|---|---|---|
| 1 | `dotnet ef migrations add Ten` | so class với lần trước, sinh file migration |
| 2 | `dotnet ef database update` | chạy các migration chưa chạy vào database |

## Ví dụ

Cài công cụ một lần, rồi thêm package cho project:

```bash
dotnet tool install --global dotnet-ef
dotnet add package Microsoft.EntityFrameworkCore.Design
```

Tạo migration đầu tiên và áp dụng:

```bash
dotnet ef migrations add InitialCreate
dotnet ef database update
```

EF Core sinh thư mục `Migrations`, trong đó có file dạng rút gọn như sau:

```csharp
using Microsoft.EntityFrameworkCore.Migrations;

public partial class InitialCreate : Migration
{
    protected override void Up(MigrationBuilder mb)
    {
        mb.CreateTable("PRODUCTS", t => new
        {
            ID = t.Column<int>(nullable: false)
                .Annotation("Oracle:Identity",
                    "START WITH 1 INCREMENT BY 1"),
            NAME = t.Column<string>(nullable: false),
            PRICE = t.Column<decimal>(nullable: false),
        },
        constraints: t =>
            t.PrimaryKey("PK_PRODUCTS", x => x.ID));
    }

    protected override void Down(MigrationBuilder mb) =>
        mb.DropTable(name: "PRODUCTS");
}
```

- `Up` chạy khi áp dụng migration: tạo bảng `PRODUCTS`, cột `ID` là khoá tự tăng.
  Đây là bản C# của `CREATE TABLE` ở khoá SQL: `nullable: false` là
  `NOT NULL`, `Oracle:Identity` là `GENERATED AS IDENTITY`.
- `Down` chạy khi huỷ migration: xoá bảng đó.
- Migration là code, được commit lên git cùng project. Cả team và server dùng
  chung một lịch sử thay đổi database.

## Thử ngay

Sau khi chạy `database update`, gọi lại `GET /api/products` sẽ nhận `[]`.
Giờ thêm property vào `Product`:

```csharp
public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
    public int Stock { get; set; }
}
```

Rồi chạy:

```bash
dotnet ef migrations add AddStock
dotnet ef database update
```

**Đoán trước khi chạy:** migration `AddStock` sẽ tạo lại cả bảng `PRODUCTS`,
hay chỉ thêm một cột?

<details>
<summary>Xem kết quả</summary>

```csharp
// Up của migration AddStock (rút gọn)
using Microsoft.EntityFrameworkCore.Migrations;

public partial class AddStock : Migration
{
    protected override void Up(MigrationBuilder mb) =>
        mb.AddColumn<int>(
            name: "STOCK", table: "PRODUCTS",
            nullable: false, defaultValue: 0);

    protected override void Down(MigrationBuilder mb) =>
        mb.DropColumn(name: "STOCK", table: "PRODUCTS");
}
```

Chỉ thêm một cột. EF Core so class hiện tại với lần migration trước và chỉ
sinh phần khác nhau. Các dòng đã có vẫn giữ nguyên, `Stock` nhận giá trị mặc
định 0.

</details>

## Lỗi hay gặp

**Sửa class mà quên tạo migration.** Code vẫn chạy, nhưng truy vấn báo lỗi vì
database chưa có cột mới.

```text
ORA-00904: "p"."STOCK": invalid identifier
```

Mỗi lần thêm, bớt hay đổi property của entity, chạy lại hai lệnh
`migrations add` và `database update`.

**Sửa file migration đã chạy trên server.** Server không chạy lại migration
đã chạy, nên phần vừa sửa không bao giờ được áp dụng. Muốn đổi thêm thì tạo
migration mới.

## Tóm tắt

- Migration là file C# mô tả cách đổi cấu trúc database.
- `migrations add` sinh migration từ phần class thay đổi.
- `database update` áp dụng các migration chưa chạy.
- Migration được commit lên git. Không sửa migration đã chạy, hãy tạo migration mới.

```quiz
[
  {
    "prompt": "Bạn thêm property Email vào class Customer. Cần làm gì để database có cột Email?",
    "options": [
      "Không cần làm gì, EF Core tự thêm",
      "Xoá file database rồi chạy lại",
      "dotnet ef migrations add AddEmail, rồi dotnet ef database update",
      "Chỉ cần build lại project"
    ],
    "answer": 3,
    "explain": "Sinh migration từ phần thay đổi, rồi áp dụng nó vào database."
  },
  {
    "prompt": "Method Down trong một migration dùng để làm gì?",
    "options": [
      "Huỷ đúng thay đổi mà Up đã làm",
      "Tải database về máy",
      "Chạy trước Up",
      "Xoá toàn bộ database"
    ],
    "answer": 1,
    "explain": "Up áp dụng thay đổi, Down đảo ngược nó. Ví dụ Up thêm cột thì Down xoá cột đó."
  },
  {
    "prompt": "Thư mục Migrations có nên commit lên git không?",
    "options": [
      "Không, mỗi người tự sinh",
      "Chỉ commit migration mới nhất",
      "Không, vì chứa mật khẩu",
      "Có, để cả team và server dùng chung lịch sử thay đổi"
    ],
    "answer": 4,
    "explain": "Migration là một phần của code. Mọi môi trường chạy cùng một chuỗi migration nên database giống nhau."
  }
]
```
