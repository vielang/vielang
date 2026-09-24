---
title: Chuẩn hoá
minutes: 5
---

Một bảng duy nhất chứa cả đơn hàng lẫn tên và email khách thì trông tiện. Đến
khi khách đổi email, bạn phải sửa mọi đơn của họ, và chỉ cần sót một đơn là
dữ liệu lệch nhau. Chuẩn hoá chia dữ liệu sao cho mỗi thông tin chỉ nằm ở một
chỗ.

## Khái niệm

🧩 **Chuẩn hoá (normalization)**: cách chia dữ liệu thành nhiều bảng để mỗi thông tin chỉ lưu ở một chỗ, tránh lặp lại và lệch nhau.

Ba mức chuẩn hoá cơ bản, mỗi mức gồm cả mức trước:

| Mức | Quy tắc | Ví dụ vi phạm |
|---|---|---|
| 1NF | mỗi ô chỉ chứa một giá trị | ô `products` ghi "Bút bi, Vở" |
| 2NF | mọi cột phụ thuộc vào **toàn bộ** khoá chính | bảng có khoá (đơn, sản phẩm) lại lưu ngày đặt đơn |
| 3NF | cột không phụ thuộc vào một cột khác ngoài khoá | bảng đơn hàng lưu email của khách |

## Ví dụ

Bảng chưa chuẩn hoá:

| order_id | customer_name | customer_email | products |
|---|---|---|---|
| 1 | An | an@shop.vn | Bút bi, Vở |
| 2 | An | an@shop.vn | Balo |

Sau khi chuẩn hoá, ta có bốn bảng mà cửa hàng mẫu đang dùng:

- Thông tin khách nằm ở `customers`, mỗi khách một dòng.
- `orders` chỉ lưu `customer_id` trỏ tới khách.
- Mỗi sản phẩm trong đơn là một dòng ở `order_lines`, không gộp chung trong
  một ô.
- Khách đổi email thì chỉ sửa một dòng ở `customers`.
- `order_lines.unit_price` không phải dữ liệu lặp: đó là giá lúc bán, vì giá ở
  `products` có thể đổi.

## Thử ngay

Dựng bảng chưa chuẩn hoá, rồi sửa email của An ở một đơn:

```sql
CREATE TABLE orders_flat (
  order_id       NUMBER,
  customer_name  VARCHAR2(50 CHAR),
  customer_email VARCHAR2(100)
);

INSERT INTO orders_flat VALUES (1, 'An', 'an@shop.vn');
INSERT INTO orders_flat VALUES (2, 'An', 'an@shop.vn');

UPDATE orders_flat SET customer_email = 'an.new@shop.vn'
WHERE order_id = 1;

SELECT DISTINCT customer_email
FROM orders_flat
WHERE customer_name = 'An'
ORDER BY customer_email;
```

`SELECT DISTINCT` bỏ các dòng trùng nhau trong kết quả.

**Đoán trước khi chạy:** theo dữ liệu này, An có mấy email?

<details>
<summary>Xem kết quả</summary>

| CUSTOMER_EMAIL |
|---|
| an.new@shop.vn |
| an@shop.vn |

Hai email cho cùng một người. Sửa một đơn mà quên đơn kia là dữ liệu lệch.
Khi có bảng `customers` riêng, email chỉ nằm đúng một chỗ nên không thể lệch.

</details>

## Lỗi hay gặp

**Gộp nhiều giá trị vào một ô.** Ô `products` ghi "Bút bi, Vở" thì không
`JOIN` được với bảng sản phẩm, không đếm được mỗi món bán bao nhiêu, và muốn
tìm đơn có Vở phải dùng `LIKE '%Vở%'`, dễ khớp nhầm.

```sql
-- SAI — nhiều sản phẩm trong một ô
CREATE TABLE bad_orders (
  order_id NUMBER PRIMARY KEY,
  products VARCHAR2(500 CHAR)
);
```

Tách thành bảng dòng hàng, mỗi sản phẩm một dòng, như `order_lines`.

## Tóm tắt

- Chuẩn hoá chia dữ liệu để mỗi thông tin chỉ nằm ở một chỗ.
- 1NF: mỗi ô một giá trị. 2NF và 3NF: mọi cột phụ thuộc vào toàn bộ khoá
  chính và chỉ vào khoá chính.
- Dữ liệu lặp lại ở nhiều dòng sẽ lệch nhau khi sửa sót.
- Nối các bảng đã tách bằng khoá ngoại và `JOIN`.

```quiz
[
  {
    "prompt": "Bảng order_lines lưu cả product_name bên cạnh product_id. Vấn đề là gì?",
    "options": [
      "Không có vấn đề gì",
      "Thiếu khoá chính",
      "Tên sản phẩm bị lặp ở nhiều dòng, đổi tên thì phải sửa khắp nơi",
      "product_id phải là chuỗi"
    ],
    "answer": 3,
    "explain": "Tên sản phẩm phụ thuộc vào product_id, nên chỉ nên nằm ở bảng products. Lưu lặp thì dễ lệch khi sửa."
  },
  {
    "prompt": "Cột phone_numbers lưu '0901..., 0912...' trong cùng một ô. Bảng vi phạm mức nào?",
    "options": [
      "1NF",
      "2NF",
      "3NF",
      "Không vi phạm"
    ],
    "answer": 1,
    "explain": "1NF yêu cầu mỗi ô chỉ chứa một giá trị. Nhiều số điện thoại nên tách ra bảng riêng."
  },
  {
    "prompt": "Sau khi chuẩn hoá, khách đổi địa chỉ email. Cần sửa bao nhiêu dòng?",
    "options": [
      "Mọi đơn hàng của khách",
      "Mọi dòng hàng của khách",
      "Không cần sửa",
      "Một dòng trong bảng customers"
    ],
    "answer": 4,
    "explain": "Email chỉ lưu ở customers. Các đơn hàng trỏ tới khách qua customer_id nên tự thấy email mới."
  }
]
```
