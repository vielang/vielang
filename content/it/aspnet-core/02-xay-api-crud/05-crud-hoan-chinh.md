---
title: CRUD hoàn chỉnh
minutes: 6
---

Mỗi bài trong bốn bài trước lo một phần: nhận dữ liệu, trả kết quả, DTO,
validation.
Bài này ghép tất cả lại thành API sản phẩm đủ bốn thao tác thêm, đọc, sửa,
xoá. Dữ liệu vẫn để trong bộ nhớ, chương 4 sẽ chuyển sang database.

## Khái niệm

🔄 **CRUD**: bốn thao tác cơ bản với dữ liệu gồm Create (tạo), Read (đọc), Update (sửa), Delete (xoá).

| Thao tác | Request | Trả về khi thành công |
|---|---|---|
| Create | `POST /api/products` | 201 + sản phẩm mới |
| Read | `GET /api/products/1` | 200 + sản phẩm |
| Update | `PUT /api/products/1` | 204 |
| Delete | `DELETE /api/products/1` | 204 |

Không tìm thấy sản phẩm thì Read, Update, Delete đều trả 404.

## Ví dụ

```csharp
using System.ComponentModel.DataAnnotations;
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/products")]
public class ProductsController : ControllerBase
{
    private static readonly List<Product> Products =
        new List<Product>();
    private static int _nextId = 1;

    [HttpGet("{id}")]
    public ActionResult<Product> GetById(int id)
    {
        var product = Find(id);
        if (product == null) return NotFound();
        return Ok(product);
    }

    [HttpPost]
    public ActionResult<Product> Create(
        ProductRequest req)
    {
        var product = new Product
        {
            Id = _nextId++,
            Name = req.Name,
            Price = req.Price,
        };
        Products.Add(product);
        return CreatedAtAction(
            nameof(GetById),
            new { id = product.Id },
            product);
    }

    [HttpPut("{id}")]
    public IActionResult Update(
        int id, ProductRequest req)
    {
        var product = Find(id);
        if (product == null) return NotFound();
        product.Name = req.Name;
        product.Price = req.Price;
        return NoContent();
    }

    [HttpDelete("{id}")]
    public IActionResult Delete(int id)
    {
        var product = Find(id);
        if (product == null) return NotFound();
        Products.Remove(product);
        return NoContent();
    }

    private static Product? Find(int id) =>
        Products.FirstOrDefault(p => p.Id == id);
}

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
}

public class ProductRequest
{
    [Required]
    public string Name { get; set; } = "";

    [Range(1, 100000000)]
    public decimal Price { get; set; }
}
```

- Tạo và sửa dùng chung `ProductRequest`, có validation sẵn.
- `IActionResult` dùng cho action không trả dữ liệu, chỉ trả status code.
- `id` của sản phẩm cần sửa lấy từ URL, không lấy từ body.
- `Find` gom phần tìm kiếm vào một chỗ cho cả ba action dùng.
- Để gọn, ví dụ trả thẳng `Product`. Dự án thật nên trả qua DTO response như
  bài trước.

## Thử ngay

Chạy server, tạo file `product.json` có nội dung `{ "name": "Pen", "price": 5000 }`,
rồi gọi lần lượt:

```bash
curl -i -X POST http://localhost:5000/api/products -H "Content-Type: application/json" -d @product.json
curl -i -X DELETE http://localhost:5000/api/products/1
curl -i -X DELETE http://localhost:5000/api/products/1
curl -i http://localhost:5000/api/products/1
```

**Đoán trước khi chạy:** xoá sản phẩm số 1 hai lần. Lần xoá thứ hai trả về
status code nào?

<details>
<summary>Xem kết quả</summary>

```text
POST            → 201 Created
DELETE lần 1    → 204 No Content
DELETE lần 2    → 404 Not Found
GET             → 404 Not Found
```

Lần xoá đầu thành công. Tới lần thứ hai thì sản phẩm đã không còn, nên
`Delete` trả 404. Tắt server rồi chạy lại thì mọi dữ liệu cũng mất, vì chúng
chỉ nằm trong bộ nhớ.

</details>

## Lỗi hay gặp

**Lấy `id` từ body khi sửa.** URL nói sửa sản phẩm số 1, body lại ghi số 2.
API không biết tin bên nào.

```csharp
// SAI — id nằm cả ở URL lẫn trong body
public class UpdateRequest
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
}
```

Chỉ lấy `id` từ URL (`[HttpPut("{id}")]`). DTO dùng để sửa không có `Id`.

**Sửa hay xoá mà không kiểm tra tồn tại.** Không tìm thấy vẫn trả 204, client
tưởng đã thành công.

## Tóm tắt

- CRUD: POST tạo, GET đọc, PUT sửa, DELETE xoá.
- Tạo trả 201, sửa và xoá trả 204, không tìm thấy trả 404.
- `id` lấy từ URL, dữ liệu sửa lấy từ body qua DTO.
- Dữ liệu trong bộ nhớ mất khi tắt server, chương 4 sẽ lưu vào database.

```quiz
[
  {
    "prompt": "PUT /api/products/5 với body hợp lệ, nhưng không có sản phẩm số 5. Nên trả gì?",
    "options": [
      "204 No Content",
      "201 Created",
      "404 Not Found",
      "200 OK"
    ],
    "answer": 3,
    "explain": "Không có gì để sửa nên trả 404. Trả 204 làm client tưởng đã sửa thành công."
  },
  {
    "prompt": "Action Delete chỉ cần báo thành công, không trả dữ liệu. Kiểu trả về nào hợp lý?",
    "options": [
      "IActionResult",
      "ActionResult<Product>",
      "List<Product>",
      "string"
    ],
    "answer": 1,
    "explain": "IActionResult dùng khi chỉ trả status code như NoContent() hay NotFound(), không có dữ liệu kèm theo."
  },
  {
    "prompt": "Tạo 3 sản phẩm rồi tắt server và chạy lại. Với cách lưu trong bài, GET /api/products/1 trả gì?",
    "options": [
      "200 và sản phẩm số 1",
      "201 Created",
      "500 Internal Server Error",
      "404 vì dữ liệu trong bộ nhớ đã mất"
    ],
    "answer": 4,
    "explain": "List static chỉ sống trong bộ nhớ của tiến trình. Tắt server là mất, nên cần database."
  }
]
```
