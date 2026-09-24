---
title: Nhận dữ liệu từ request
minutes: 5
---

API cần biết client muốn gì: sản phẩm số mấy, tìm theo tên nào, tạo sản phẩm
với tên và giá bao nhiêu. Dữ liệu đó nằm ở ba chỗ trong request. Bài này chỉ
cách nhận cả ba vào tham số của action.

## Khái niệm

🔗 **Model binding**: cơ chế ASP.NET Core tự lấy dữ liệu trong request và gán vào tham số của action.

| Nguồn | Ví dụ trong request | Attribute |
|---|---|---|
| Route | `/api/products/5` | `[FromRoute]` |
| Query string | `?name=pen&maxPrice=10000` | `[FromQuery]` |
| Body (JSON) | `{ "name": "Pen", "price": 5000 }` | `[FromBody]` |

Với `[ApiController]`, bạn thường không cần ghi attribute. Tham số có tên
trong route lấy từ route, kiểu đơn giản như `int`, `string` lấy từ query,
còn object lấy từ body.

## Ví dụ

```csharp
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/products")]
public class ProductsController : ControllerBase
{
    [HttpGet("{id}")]
    public string GetById(int id) => $"Sản phẩm {id}";

    [HttpGet("search")]
    public string Search(
        string name, decimal maxPrice) =>
        $"Tìm {name}, giá tối đa {maxPrice}";

    [HttpPost]
    public Product Create(Product product) => product;
}

public class Product
{
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
}
```

- `GetById(int id)`: `id` có trong route `{id}` nên lấy từ URL.
- `Search(string name, decimal maxPrice)`: không có trong route nên lấy từ
  query string `?name=...&maxPrice=...`.
- `Create(Product product)`: `Product` là object nên lấy từ body JSON.
- ASP.NET Core tự đổi chữ trong URL thành `int`, `decimal`. Đổi không được
  thì trả lỗi 400.

## Thử ngay

Chạy server, rồi gọi hai request GET:

```bash
curl -i "http://localhost:5000/api/products/5"
curl -i "http://localhost:5000/api/products/search?name=pen&maxPrice=10000"
```

Để gửi POST, tạo file `product.json`:

```json
{ "name": "Pen", "price": 5000 }
```

Rồi gửi file đó làm body:

```bash
curl -i -X POST http://localhost:5000/api/products -H "Content-Type: application/json" -d @product.json
```

**Đoán trước khi chạy:** JSON gửi lên viết `name` chữ thường, còn property
trong C# là `Name`. Action có nhận được tên sản phẩm không?

<details>
<summary>Xem kết quả</summary>

```http
HTTP/1.1 200 OK
{"name":"Pen","price":5000}
```

Có. ASP.NET Core không phân biệt hoa thường khi đọc JSON vào object, nên
`name` vẫn gán được vào `Name`. Action trả lại đúng object vừa nhận.

</details>

## Lỗi hay gặp

**Gửi chữ vào tham số số.** `id` là `int` mà URL lại là chữ, model binding
không đổi được và trả về 400.

```http
// SAI — "abc" không phải số, nhận về 400 Bad Request
GET /api/products/abc
```

```http
// ĐÚNG
GET /api/products/5
```

**Quên header `Content-Type`.** Không có `-H`, curl gửi body với kiểu của
form (`application/x-www-form-urlencoded`). Server không đọc kiểu này như
JSON và trả về 415 Unsupported Media Type.

```bash
# SAI — thiếu -H "Content-Type: application/json"
curl -i -X POST http://localhost:5000/api/products -d @product.json
```

```bash
# ĐÚNG
curl -i -X POST http://localhost:5000/api/products -H "Content-Type: application/json" -d @product.json
```

## Tóm tắt

- Model binding tự gán dữ liệu request vào tham số của action.
- Dữ liệu đến từ route (`{id}`), query string (`?name=`) hoặc body JSON.
- Với `[ApiController]`: tên trong route thì lấy từ route, kiểu đơn giản lấy
  từ query, object lấy từ body.
- Gửi JSON phải có header `Content-Type: application/json`.

```quiz
[
  {
    "prompt": "Action [HttpGet(\"{orderId}/lines\")] GetLines(int orderId, int page). Gọi GET /api/orders/7/lines?page=2 thì orderId và page bằng bao nhiêu?",
    "options": [
      "orderId = 2, page = 7",
      "orderId = 7, page = 2",
      "orderId = 7, page = 0",
      "Lỗi 400"
    ],
    "answer": 2,
    "explain": "orderId có trong route nên lấy từ URL là 7. page không có trong route nên lấy từ query string là 2."
  },
  {
    "prompt": "Client gửi thông tin đơn hàng mới gồm nhiều trường. Dữ liệu này nên đặt ở đâu trong request?",
    "options": [
      "Route",
      "Query string",
      "Body, dạng JSON",
      "Tên của action"
    ],
    "answer": 3,
    "explain": "Dữ liệu nhiều trường để tạo mới gửi trong body JSON. Route và query dùng cho giá trị đơn giản như id hay điều kiện lọc."
  },
  {
    "prompt": "POST body JSON đúng, nhưng server trả 415 Unsupported Media Type. Nguyên nhân khả dĩ nhất?",
    "options": [
      "Sai tên action",
      "Thiếu [HttpPost]",
      "JSON có chữ viết thường",
      "Thiếu header Content-Type"
    ],
    "answer": 4,
    "explain": "415 nghĩa là server không nhận loại dữ liệu này. Thiếu Content-Type thì server không biết body là JSON."
  }
]
```
