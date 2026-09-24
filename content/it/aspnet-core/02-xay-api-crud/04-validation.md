---
title: Validation
minutes: 5
---

Client gửi sản phẩm tên rỗng, giá âm. Nếu API cứ thế lưu vào thì dữ liệu hỏng
từ đầu. Validation chặn dữ liệu sai ngay ở cửa, trước khi action kịp chạy.

## Khái niệm

✅ **Validation**: bước kiểm tra dữ liệu client gửi lên có hợp lệ không trước khi xử lý.

🏷️ **Data annotation**: attribute gắn lên property của DTO để khai báo quy tắc, ví dụ `[Required]`, `[Range]`.

| Attribute | Quy tắc |
|---|---|
| `[Required]` | bắt buộc có, chuỗi không được rỗng |
| `[StringLength(100)]` | chuỗi tối đa 100 ký tự |
| `[Range(1, 100000000)]` | số nằm trong khoảng |
| `[EmailAddress]` | đúng định dạng email |

## Ví dụ

```csharp
using System.ComponentModel.DataAnnotations;
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/products")]
public class ProductsController : ControllerBase
{
    [HttpPost]
    public ActionResult<string> Create(
        CreateProductRequest request)
    {
        return Ok($"Đã tạo {request.Name}");
    }
}

public class CreateProductRequest
{
    [Required(ErrorMessage = "Tên không được trống")]
    [StringLength(100)]
    public string Name { get; set; } = "";

    [Range(1, 100000000,
        ErrorMessage = "Giá phải từ 1 đến 100 triệu")]
    public decimal Price { get; set; }
}
```

- Quy tắc nằm ngay trên DTO, cần `using System.ComponentModel.DataAnnotations`.
- `[ApiController]` tự kiểm tra DTO trước khi gọi action. Sai quy tắc thì
  trả 400 kèm danh sách lỗi, action không chạy.
- `ErrorMessage` đặt câu báo lỗi riêng thay cho câu mặc định bằng tiếng Anh.

## Thử ngay

Chạy server. Tạo file `product.json` với dữ liệu sai:

```json
{ "name": "", "price": -5 }
```

Rồi gửi:

```bash
curl -i -X POST http://localhost:5000/api/products -H "Content-Type: application/json" -d @product.json
```

**Đoán trước khi chạy:** action có chạy tới dòng `return Ok(...)` không, và
response báo mấy lỗi?

<details>
<summary>Xem kết quả</summary>

```http
HTTP/1.1 400 Bad Request
Content-Type: application/problem+json

{
  "title": "One or more validation errors occurred.",
  "status": 400,
  "errors": {
    "Name": ["Tên không được trống"],
    "Price": ["Giá phải từ 1 đến 100 triệu"]
  }
}
```

Action không chạy. `[ApiController]` chặn từ trước và trả 400 kèm hai lỗi,
mỗi lỗi gắn với tên property. Client dùng danh sách này để báo cho người dùng.

</details>

## Lỗi hay gặp

**Tự kiểm tra bằng `if` trong từng action.** Quy tắc bị chép rải rác, sửa một
chỗ dễ quên chỗ khác.

```csharp
// SAI — quy tắc nằm trong action, phải chép lại ở PUT
using Microsoft.AspNetCore.Mvc;

public class ItemsController : ControllerBase
{
    [HttpPost]
    public IActionResult Create(string name)
    {
        if (string.IsNullOrWhiteSpace(name))
        {
            return BadRequest("Tên không được trống");
        }
        return Ok();
    }
}
```

Đặt quy tắc thành attribute trên DTO thì mọi action dùng DTO đó đều được kiểm
tra.

**Quên `[ApiController]`.** Không có nó thì dữ liệu sai vẫn lọt vào action,
vì không ai tự trả 400 nữa.

## Tóm tắt

- Validation chặn dữ liệu sai trước khi action chạy.
- Khai báo quy tắc bằng attribute trên DTO: `[Required]`, `[Range]`,
  `[StringLength]`.
- `[ApiController]` tự trả 400 kèm danh sách lỗi theo từng property.
- Dùng `ErrorMessage` để có câu báo lỗi dễ hiểu cho người dùng.

```quiz
[
  {
    "prompt": "DTO có [Range(1, 10)] public int Quantity. Client gửi quantity = 0. Chuyện gì xảy ra?",
    "options": [
      "Action chạy với Quantity = 0",
      "Action chạy với Quantity = 1",
      "Trả 400, action không chạy",
      "Trả 500"
    ],
    "answer": 3,
    "explain": "0 nằm ngoài khoảng 1 đến 10. [ApiController] trả 400 trước khi action kịp chạy."
  },
  {
    "prompt": "Muốn email của khách hàng vừa bắt buộc vừa đúng định dạng. Gắn attribute nào?",
    "options": [
      "[Required] và [EmailAddress]",
      "Chỉ [Required]",
      "[Range] và [EmailAddress]",
      "[StringLength(1)]"
    ],
    "answer": 1,
    "explain": "[Required] bắt buộc có giá trị, [EmailAddress] kiểm tra định dạng. Hai quy tắc dùng cùng nhau được."
  },
  {
    "prompt": "Quy tắc validation nên đặt ở đâu?",
    "options": [
      "Trong từng action, bằng if",
      "Trong Program.cs",
      "Ở phía client là đủ",
      "Trên property của DTO request, bằng attribute"
    ],
    "answer": 4,
    "explain": "Đặt trên DTO thì mọi nơi dùng DTO đều được kiểm tra, không phải chép quy tắc. Chỉ kiểm tra ở client thì ai gọi thẳng API vẫn qua được."
  }
]
```
