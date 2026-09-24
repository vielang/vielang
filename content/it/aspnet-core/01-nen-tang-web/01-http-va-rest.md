---
title: HTTP và REST
minutes: 5
---

App bán hàng trên điện thoại cần lấy danh sách sản phẩm từ server. Hai bên
nói chuyện với nhau bằng HTTP. Trước khi viết API, bạn cần biết một request
gồm những gì và server trả lời ra sao.

## Khái niệm

🌐 **HTTP**: giao thức để client và server trao đổi dữ liệu. Client gửi request, server trả về response.

📮 **HTTP method**: phần của request cho biết client muốn làm gì, gồm GET, POST, PUT, DELETE.

🔢 **Status code**: số ba chữ số trong response cho biết kết quả. 2xx là thành công, 4xx là lỗi phía client, 5xx là lỗi phía server.

🧭 **REST**: cách thiết kế API mà mỗi URL trỏ tới một tài nguyên (danh từ), còn việc cần làm thể hiện bằng HTTP method.

## Ví dụ

Client xin sản phẩm số 1:

```http
GET /api/products/1 HTTP/1.1
Host: localhost:5000
```

Server trả lời:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{ "id": 1, "name": "Pen", "price": 5000 }
```

- Dòng đầu của request gồm method (`GET`) và đường dẫn (`/api/products/1`).
- Dòng đầu của response có status code (`200 OK`).
- Dữ liệu trả về ở dạng JSON, một định dạng chữ gồm các cặp tên và giá trị.

Một API sản phẩm theo kiểu REST:

| Method | URL | Việc |
|---|---|---|
| GET | `/api/products` | lấy danh sách |
| GET | `/api/products/1` | lấy sản phẩm số 1 |
| POST | `/api/products` | tạo sản phẩm mới |
| PUT | `/api/products/1` | cập nhật sản phẩm số 1 |
| DELETE | `/api/products/1` | xoá sản phẩm số 1 |

Status code hay gặp:

| Code | Ý nghĩa |
|---|---|
| 200 OK | thành công, có dữ liệu trả về |
| 201 Created | đã tạo mới |
| 204 No Content | thành công, không có dữ liệu |
| 400 Bad Request | dữ liệu gửi lên sai |
| 404 Not Found | không tìm thấy |
| 500 Internal Server Error | server bị lỗi |

## Thử ngay

Gọi một API công khai bằng `curl`. Trên Windows, gõ `curl.exe` thay cho
`curl`, vì trong PowerShell chữ `curl` là một lệnh khác.

```bash
curl -i https://jsonplaceholder.typicode.com/users/1
curl -i https://jsonplaceholder.typicode.com/users/9999
```

Tuỳ chọn `-i` in cả phần đầu của response, trong đó có status code.

**Đoán trước khi chạy:** lần gọi thứ hai hỏi user không tồn tại. Status code
là bao nhiêu?

<details>
<summary>Xem kết quả</summary>

```text
Lần 1: status 200, body là JSON thông tin user số 1
Lần 2: status 404, body là {}
```

Server trả 404 Not Found vì không có user 9999. Chỉ cần nhìn status code là
client biết kết quả, không cần đọc body.

</details>

## Lỗi hay gặp

**Đặt động từ vào URL.** REST dùng method để nói việc cần làm.

```http
// SAI
GET /api/deleteProduct?id=1
```

```http
// ĐÚNG
DELETE /api/products/1
```

**Trả 200 cho cả trường hợp lỗi.** Client phải đọc body mới biết có lỗi hay
không.

```http
// SAI
HTTP/1.1 200 OK

{ "error": "Không tìm thấy sản phẩm" }
```

```http
// ĐÚNG
HTTP/1.1 404 Not Found
```

## Tóm tắt

- Client gửi request, server trả response, qua giao thức HTTP.
- Method nói việc cần làm: GET đọc, POST tạo, PUT sửa, DELETE xoá.
- Status code nói kết quả: 2xx thành công, 4xx lỗi phía client, 5xx lỗi
  phía server.
- REST: URL là danh từ chỉ tài nguyên, việc cần làm nằm ở method.

```quiz
[
  {
    "prompt": "Client muốn cập nhật giá của đơn hàng số 7. Request nào đúng kiểu REST?",
    "options": [
      "GET /api/updateOrder?id=7",
      "POST /api/orders/7/update",
      "PUT /api/orders/7",
      "DELETE /api/orders/7"
    ],
    "answer": 3,
    "explain": "PUT dùng để cập nhật, URL chỉ đúng tài nguyên là đơn hàng số 7. Không đặt động từ vào URL."
  },
  {
    "prompt": "Client gửi email sai định dạng khi đăng ký. Server nên trả status code nào?",
    "options": [
      "400 Bad Request",
      "200 OK",
      "404 Not Found",
      "500 Internal Server Error"
    ],
    "answer": 1,
    "explain": "Dữ liệu client gửi lên sai là lỗi phía client, nhóm 4xx. 400 Bad Request đúng cho trường hợp này."
  },
  {
    "prompt": "Server bị lỗi kết nối database khi xử lý request. Status code phù hợp thuộc nhóm nào?",
    "options": [
      "2xx",
      "3xx",
      "4xx",
      "5xx"
    ],
    "answer": 4,
    "explain": "Lỗi nằm ở phía server, client không làm gì sai, nên thuộc nhóm 5xx."
  }
]
```
