---
title: Trả về kết quả
minutes: 5
---

Ở bài Controller và routing, gọi API lấy sản phẩm không tồn tại thì nhận về
204 thay vì 404. Client không phân biệt được "không có" với "thành công". Bài này chỉ
cách chọn đúng status code cho từng trường hợp.

## Khái niệm

📬 **ActionResult<T>**: kiểu trả về của action, chứa được hoặc dữ liệu kiểu `T`, hoặc một kết quả kèm status code như 404.

Các method có sẵn trong `ControllerBase` để tạo kết quả:

| Method | Status code | Khi nào dùng |
|---|---|---|
| `Ok(data)` | 200 | đọc thành công, có dữ liệu |
| `CreatedAtAction(...)` | 201 | tạo mới thành công |
| `NoContent()` | 204 | thành công, không có gì trả về |
| `BadRequest()` | 400 | dữ liệu gửi lên sai |
| `NotFound()` | 404 | không tìm thấy |

## Ví dụ

```csharp
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/products")]
public class ProductsController : ControllerBase
{
    private static readonly List<Product> Products =
        new List<Product>();

    [HttpGet("{id}")]
    public ActionResult<Product> GetById(int id)
    {
        var product =
            Products.FirstOrDefault(p => p.Id == id);
        if (product == null)
        {
            return NotFound();
        }
        return Ok(product);
    }

    [HttpPost]
    public ActionResult<Product> Create(Product product)
    {
        product.Id = Products.Count + 1;
        Products.Add(product);
        return CreatedAtAction(
            nameof(GetById),
            new { id = product.Id },
            product);
    }
}

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
}
```

- `ActionResult<Product>` cho phép trả `NotFound()` ở nhánh này và
  `Ok(product)` ở nhánh kia.
- `CreatedAtAction` trả 201, kèm header `Location` chỉ tới URL của sản phẩm
  vừa tạo, ví dụ `/api/products/1`.
- `nameof(GetById)` chỉ ra action dùng để dựng URL đó. Viết bằng `nameof`
  thì khi đổi tên method, compiler báo lỗi ngay.
- `new { id = product.Id }` tạo nhanh một object không cần khai báo class,
  dùng để điền `{id}` vào URL.

## Thử ngay

Chạy server. Tạo file `product.json` có nội dung `{ "name": "Pen", "price": 5000 }`,
rồi gọi:

```bash
curl -i -X POST http://localhost:5000/api/products -H "Content-Type: application/json" -d @product.json
curl -i http://localhost:5000/api/products/1
curl -i http://localhost:5000/api/products/99
```

**Đoán trước khi chạy:** lần gọi POST trả về status code nào, và lần gọi
cuối còn là 204 không?

<details>
<summary>Xem kết quả</summary>

```http
HTTP/1.1 201 Created
Location: http://localhost:5000/api/products/1
{"id":1,"name":"Pen","price":5000}

HTTP/1.1 200 OK
{"id":1,"name":"Pen","price":5000}

HTTP/1.1 404 Not Found
```

POST trả 201 kèm `Location`. Sản phẩm 99 giờ trả 404, vì action chủ động gọi
`NotFound()` thay vì trả `null`.

</details>

## Lỗi hay gặp

**Trả `null` khi không tìm thấy.** Client nhận 204, tưởng là thành công.

```csharp
// SAI — không tìm thấy mà vẫn là "thành công"
using Microsoft.AspNetCore.Mvc;

public class OrdersController : ControllerBase
{
    [HttpGet("{id}")]
    public string? GetById(int id)
    {
        return null;
    }
}
```

```csharp
// ĐÚNG
using Microsoft.AspNetCore.Mvc;

public class OrdersController : ControllerBase
{
    [HttpGet("{id}")]
    public ActionResult<string> GetById(int id)
    {
        return NotFound();
    }
}
```

**Tạo mới mà trả 200.** Tạo thành công nên trả 201 qua `CreatedAtAction`,
để client biết ngay URL của thứ vừa tạo.

## Tóm tắt

- Dùng `ActionResult<T>` để một action trả được cả dữ liệu lẫn lỗi.
- Đọc được thì `Ok`, tạo mới thì `CreatedAtAction`, không có thì `NotFound`.
- Không trả `null` khi không tìm thấy, vì client sẽ nhận 204.
- Status code đúng giúp client xử lý mà không phải đọc body.

```quiz
[
  {
    "prompt": "Action xoá sản phẩm thành công và không cần trả dữ liệu gì. Nên trả gì?",
    "options": [
      "NoContent()",
      "Ok(null)",
      "NotFound()",
      "BadRequest()"
    ],
    "answer": 1,
    "explain": "NoContent() trả 204: thành công và không có body. Đây là cách phổ biến cho DELETE."
  },
  {
    "prompt": "Action trả ActionResult<Order>. Không tìm thấy đơn hàng thì viết gì?",
    "options": [
      "return null;",
      "return NotFound();",
      "return Ok();",
      "throw new Exception(\"Không có\");"
    ],
    "answer": 2,
    "explain": "NotFound() trả 404 rõ ràng. Trả null thành 204, còn ném exception thành 500."
  },
  {
    "prompt": "Sau khi tạo khách hàng mới, response có header Location. Header này dùng để làm gì?",
    "options": [
      "Cho biết server đặt ở đâu",
      "Bắt client chuyển sang trang khác",
      "Chỉ tới URL để lấy khách hàng vừa tạo",
      "Cho biết định dạng của body"
    ],
    "answer": 3,
    "explain": "CreatedAtAction đặt Location là URL của tài nguyên mới, client gọi GET vào đó để lấy lại."
  }
]
```
