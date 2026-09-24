---
title: Xử lý lỗi tập trung
minutes: 5
---

Database mất kết nối, một chỗ trong code ném exception không ai bắt. Client
nhận về một trang lỗi dài kèm stack trace, lộ cả tên class bên trong. Bọc
`try/catch` vào từng action thì quá dài. Bài này bắt mọi lỗi ở một chỗ.

## Khái niệm

🛡️ **Xử lý lỗi tập trung**: bắt mọi exception chưa được xử lý ở một chỗ duy nhất trong pipeline, rồi trả về response lỗi thống nhất.

📄 **ProblemDetails**: định dạng JSON chuẩn để mô tả lỗi trong HTTP API, gồm các trường như `title`, `status`, `detail`.

## Ví dụ

```csharp
using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Mvc;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddControllers();
builder.Services.AddProblemDetails();
builder.Services
    .AddExceptionHandler<AppExceptionHandler>();

var app = builder.Build();
app.UseExceptionHandler();
app.MapControllers();
app.Run();

public class AppExceptionHandler : IExceptionHandler
{
    private readonly ILogger<AppExceptionHandler>
        _logger;

    public AppExceptionHandler(
        ILogger<AppExceptionHandler> logger)
    {
        _logger = logger;
    }

    public async ValueTask<bool> TryHandleAsync(
        HttpContext context,
        Exception exception,
        CancellationToken ct)
    {
        _logger.LogError(
            exception, "Lỗi chưa được xử lý");

        context.Response.StatusCode = 500;
        await context.Response.WriteAsJsonAsync(
            new ProblemDetails
            {
                Status = 500,
                Title = "Something went wrong",
            },
            ct);
        return true;
    }
}
```

- `IExceptionHandler` là interface để viết bộ xử lý lỗi. `TryHandleAsync`
  được gọi khi có exception chưa ai bắt.
- Bộ xử lý ghi log đầy đủ exception cho team, còn client chỉ nhận một câu
  chung chung.
- Trả `true` nghĩa là lỗi đã được xử lý xong.
- `AddExceptionHandler` đăng ký bộ xử lý, `UseExceptionHandler()` thêm nó
  vào pipeline.
- `ct` dùng để huỷ việc ghi response khi client ngắt kết nối, ở đây chỉ cần
  truyền tiếp.

## Thử ngay

Thêm một action cố tình ném lỗi vào project:

```csharp
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/test")]
public class TestController : ControllerBase
{
    [HttpGet("boom")]
    public string Boom() =>
        throw new InvalidOperationException(
            "Mất kết nối tới kho ở Hà Nội");
}
```

Chạy server rồi gọi `curl -i http://localhost:5000/api/test/boom`.

**Đoán trước khi chạy:** câu "Mất kết nối tới kho ở Hà Nội" có xuất hiện
trong response mà client nhận không?

<details>
<summary>Xem kết quả</summary>

```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{"title":"Something went wrong","status":500}
```

Không. Câu đó chỉ nằm trong log của server, kèm stack trace đầy đủ. Client
chỉ thấy thông báo chung, không biết gì về cấu trúc bên trong hệ thống.

</details>

## Lỗi hay gặp

**Bọc `try/catch` vào từng action.** Cùng một đoạn xử lý lỗi bị chép lại ở
khắp nơi, sửa một chỗ dễ quên chỗ khác.

```csharp
// SAI — mọi action đều lặp lại khối này
using Microsoft.AspNetCore.Mvc;

public class OrdersController : ControllerBase
{
    [HttpGet("{id}")]
    public IActionResult Get(int id)
    {
        try
        {
            return Ok(id);
        }
        catch (Exception)
        {
            return StatusCode(500);
        }
    }
}
```

**Trả thẳng `exception.Message` hay stack trace cho client.** Thông báo lỗi
có thể chứa tên bảng, đường dẫn file, chuỗi kết nối. Chỉ ghi chúng vào log.

## Tóm tắt

- Bắt mọi exception chưa xử lý ở một chỗ bằng `IExceptionHandler`.
- Đăng ký bằng `AddExceptionHandler`, bật bằng `UseExceptionHandler()`.
- Ghi log đầy đủ cho team, trả `ProblemDetails` ngắn gọn cho client.
- Không bọc `try/catch` vào từng action, không lộ chi tiết lỗi ra ngoài.

```quiz
[
  {
    "prompt": "Service ném exception khi đọc database, không ai bắt. Với bộ xử lý lỗi tập trung ở trên, client nhận status code nào?",
    "options": [
      "200",
      "404",
      "500",
      "Không nhận được gì"
    ],
    "answer": 3,
    "explain": "Bộ xử lý bắt exception, đặt StatusCode = 500 và trả ProblemDetails."
  },
  {
    "prompt": "Thông tin chi tiết của exception (message, stack trace) nên đi đâu?",
    "options": [
      "Vào log của server",
      "Vào body response cho client",
      "Vào URL",
      "Bỏ đi, không cần lưu"
    ],
    "answer": 1,
    "explain": "Team cần chi tiết để sửa lỗi, nên ghi vào log. Client chỉ cần biết có lỗi, không cần biết bên trong."
  },
  {
    "prompt": "TryHandleAsync trả về true có ý nghĩa gì?",
    "options": [
      "Không có lỗi nào xảy ra",
      "Yêu cầu chạy lại request",
      "Tắt server",
      "Lỗi đã được xử lý xong, không cần bộ xử lý nào khác"
    ],
    "answer": 4,
    "explain": "true báo cho ASP.NET Core biết response đã được ghi xong. Trả false thì bộ xử lý tiếp theo sẽ thử."
  }
]
```
