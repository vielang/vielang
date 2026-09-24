---
title: Tạo Web API đầu tiên
minutes: 5
---

Bạn đã biết HTTP, giờ tới lúc tự dựng một server trả lời request. Bài này tạo
project Web API bằng ASP.NET Core, chạy nó, và gọi thử API có sẵn trong
template.

## Khái niệm

🏗️ **Web API**: chương trình chạy trên server, nhận HTTP request và trả về dữ liệu (thường là JSON) thay vì trang web.

🧩 **WebApplication**: object đại diện cho ứng dụng web, được tạo qua builder, cấu hình xong thì gọi `Run()` để bắt đầu nhận request.

## Ví dụ

Tạo và chạy project:

```bash
dotnet new webapi --use-controllers -o ShopApi
cd ShopApi
dotnet run --urls http://localhost:5000
```

- `--use-controllers` tạo project dùng controller, cách làm của cả khoá này.
- `--urls` cố định địa chỉ server, để mọi bài gọi cùng một địa chỉ.

Phần cốt lõi của `Program.cs`:

```csharp
var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();

var app = builder.Build();
app.MapControllers();
app.Run();
```

- `CreateBuilder` tạo builder để cấu hình ứng dụng.
- `AddControllers()` đăng ký các service mà controller cần.
- `Build()` tạo ra `app` từ cấu hình đó.
- `MapControllers()` nối URL tới các controller trong project.
- `Run()` bắt đầu nhận request, chạy cho tới khi bạn bấm Ctrl+C.

Template còn vài dòng khác trong `Program.cs`, như `AddOpenApi()` hay
`UseHttpsRedirection()`. Cứ giữ nguyên chúng, khoá này không cần sửa.

## Thử ngay

Template có sẵn một controller mẫu tên `WeatherForecast`. Để nguyên server đang
chạy, mở terminal thứ hai và gọi:

```bash
curl -i http://localhost:5000/weatherforecast
```

**Đoán trước khi chạy:** status code là bao nhiêu, và body có mấy phần tử?

<details>
<summary>Xem kết quả</summary>

```http
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8

[{"date":"...","temperatureC":12,"temperatureF":53,"summary":"Cool"},
 ... 5 phần tử]
```

Status 200, body là một mảng JSON 5 phần tử. Số liệu là ngẫu nhiên nên mỗi
lần gọi mỗi khác. Tên property trong JSON có chữ đầu viết thường
(`temperatureC`), dù trong C# là `TemperatureC`.

</details>

## Lỗi hay gặp

**Quên `MapControllers()`.** Server vẫn chạy, nhưng mọi URL đều trả 404 vì
không controller nào được nối vào.

```csharp
// SAI — gọi URL nào cũng 404
var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();

var app = builder.Build();
app.Run();
```

```csharp
// ĐÚNG
var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();

var app = builder.Build();
app.MapControllers();
app.Run();
```

**Quên `AddControllers()`.** Ứng dụng dừng ngay lúc khởi động, báo không tìm
thấy service cần thiết và nhắc bạn gọi `AddControllers`. Hai dòng
`AddControllers` và `MapControllers` luôn đi cùng nhau.

## Tóm tắt

- `dotnet new webapi --use-controllers` tạo project Web API dùng controller.
- `dotnet run --urls http://localhost:5000` chạy server ở địa chỉ cố định.
- `Program.cs`: tạo builder, đăng ký service, `Build()`, nối URL, `Run()`.
- `AddControllers()` và `MapControllers()` phải có đủ cả hai.

```quiz
[
  {
    "prompt": "Server chạy bình thường nhưng gọi URL nào cũng nhận 404. Nguyên nhân nhiều khả năng nhất trong Program.cs là gì?",
    "options": [
      "Thiếu builder.Build()",
      "Thiếu app.Run()",
      "Thiếu AddControllers()",
      "Thiếu app.MapControllers()"
    ],
    "answer": 4,
    "explain": "Không có MapControllers thì không URL nào được nối tới controller, nên mọi request đều 404."
  },
  {
    "prompt": "Controller trả về object C# có property ProductName. Trong JSON response, tên đó thường là gì?",
    "options": [
      "productName",
      "ProductName",
      "product_name",
      "PRODUCTNAME"
    ],
    "answer": 1,
    "explain": "ASP.NET Core mặc định đổi tên property sang dạng chữ đầu viết thường (camelCase) khi tạo JSON."
  },
  {
    "prompt": "Lệnh nào tạo project Web API dùng controller?",
    "options": [
      "dotnet new console -o ShopApi",
      "dotnet new webapi --use-controllers -o ShopApi",
      "dotnet run --use-controllers",
      "dotnet new webapi -o ShopApi --no-controllers"
    ],
    "answer": 2,
    "explain": "Template webapi kèm tuỳ chọn --use-controllers tạo project có sẵn thư mục Controllers và controller mẫu."
  }
]
```
