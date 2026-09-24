---
title: Logging
minutes: 5
---

Khách báo không đặt được hàng lúc 2 giờ sáng. Không ai ngồi xem server lúc
đó. Thứ duy nhất còn lại để biết chuyện gì đã xảy ra là log. Bài này hướng dẫn
ghi log đúng cách trong ASP.NET Core.

## Khái niệm

📝 **Logging**: ghi lại những gì ứng dụng đang làm, để tra cứu khi có sự cố.

📶 **Log level**: mức độ quan trọng của một dòng log, từ thấp tới cao là Trace, Debug, Information, Warning, Error, Critical.

| Level | Khi nào dùng |
|---|---|
| Debug | chi tiết để dò lỗi lúc dev |
| Information | sự kiện bình thường: tạo đơn, đăng nhập |
| Warning | bất thường nhưng vẫn chạy được |
| Error | lỗi làm hỏng một thao tác |

## Ví dụ

```csharp
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/products")]
public class ProductsController : ControllerBase
{
    private readonly ILogger<ProductsController>
        _logger;

    public ProductsController(
        ILogger<ProductsController> logger)
    {
        _logger = logger;
    }

    [HttpGet("{id}")]
    public IActionResult GetById(int id)
    {
        _logger.LogDebug(
            "Bắt đầu tìm sản phẩm {Id}", id);

        if (id > 100)
        {
            _logger.LogWarning(
                "Không tìm thấy sản phẩm {Id}", id);
            return NotFound();
        }
        return Ok(id);
    }
}
```

- `ILogger<ProductsController>` có sẵn trong container. Tên class trong
  `< >` được ghi kèm mỗi dòng log để biết log đến từ đâu.
- `{Id}` là chỗ giữ: giá trị của `id` được điền vào đúng vị trí đó. Công
  cụ xem log lọc được theo giá trị này.
- `appsettings.json` quy định level thấp nhất được ghi. Mặc định là
  `Information`, nên dòng `LogDebug` không hiện.

## Thử ngay

Dùng controller ở ví dụ trên, chạy server rồi gọi:

```bash
curl -i http://localhost:5000/api/products/999
```

**Đoán trước khi chạy:** action gọi cả `LogDebug` và `LogWarning`. Cửa sổ
đang chạy server hiện mấy dòng log mới?

<details>
<summary>Xem kết quả</summary>

```text
warn: ProductsController[0]
      Không tìm thấy sản phẩm 999
```

Chỉ một dòng `warn` của action. `LogDebug` thấp hơn mức `Information` mặc
định nên bị bỏ qua. Lần gọi đầu có thể kèm một dòng `warn` của
`HttpsRedirection` báo không tìm thấy cổng https, bỏ qua được.

Muốn thấy log Debug khi dev, đặt `"Default": "Debug"` trong mục
`Logging:LogLevel` của `appsettings.Development.json`.

</details>

## Lỗi hay gặp

**Ghép chuỗi vào log.** Dòng log vẫn đọc được, nhưng công cụ xem log không lọc
được theo `Id` nữa.

```csharp
// SAI — giá trị bị ghép thẳng vào chuỗi
_logger.LogWarning($"Không tìm thấy sản phẩm {id}");
```

```csharp
// ĐÚNG — dùng chỗ giữ {Id}
_logger.LogWarning("Không tìm thấy sản phẩm {Id}", id);
```

**Ghi dữ liệu nhạy cảm.** Mật khẩu, số thẻ, token không bao giờ được ghi vào
log, vì nhiều người và nhiều hệ thống đọc được log.

## Tóm tắt

- Nhận `ILogger<T>` qua constructor để ghi log.
- Chọn level theo mức độ: Information cho sự kiện thường, Warning cho bất
  thường, Error cho lỗi.
- Dùng chỗ giữ `{Ten}` thay vì ghép chuỗi.
- Level thấp nhất được ghi đặt trong `appsettings.json`.

```quiz
[
  {
    "prompt": "Thanh toán thất bại vì cổng thanh toán trả lỗi, đơn hàng không tạo được. Nên ghi log level nào?",
    "options": [
      "Debug",
      "Information",
      "Error",
      "Trace"
    ],
    "answer": 3,
    "explain": "Một thao tác quan trọng đã hỏng, nên dùng Error."
  },
  {
    "prompt": "Mức log thấp nhất đặt là Warning. Dòng nào sẽ được ghi?",
    "options": [
      "_logger.LogInformation(...)",
      "_logger.LogDebug(...)",
      "_logger.LogTrace(...)",
      "_logger.LogError(...)"
    ],
    "answer": 4,
    "explain": "Chỉ log từ Warning trở lên được ghi. Error cao hơn Warning, ba level còn lại thấp hơn nên bị bỏ."
  },
  {
    "prompt": "Cách viết log nào tốt nhất?",
    "options": [
      "_logger.LogInformation(\"Đơn {OrderId} đã tạo\", orderId);",
      "_logger.LogInformation(\"Đơn \" + orderId + \" đã tạo\");",
      "_logger.LogInformation($\"Đơn {orderId} đã tạo\");",
      "Console.WriteLine(\"Đơn đã tạo\");"
    ],
    "answer": 1,
    "explain": "Chỗ giữ {OrderId} lưu orderId thành một giá trị riêng, nên công cụ xem log lọc được theo từng đơn hàng."
  }
]
```
