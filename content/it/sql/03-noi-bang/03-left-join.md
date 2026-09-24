---
title: LEFT JOIN
minutes: 5
---

Phòng marketing muốn gửi mã giảm giá cho những khách chưa từng mua hàng.
`INNER JOIN` chỉ giữ khách có đơn, nên chính những khách cần tìm lại bị loại.
`LEFT JOIN` giữ lại cả những khách không có đơn nào.

## Khái niệm

🫲 **LEFT JOIN**: giữ mọi dòng của bảng bên trái, dòng không khớp với bảng bên phải thì các cột của bảng phải là NULL.

## Ví dụ

```sql
SELECT c.name, o.order_id
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
ORDER BY c.customer_id, o.order_id;
```

- Bảng bên trái là `customers`, đứng trước `LEFT JOIN`.
- An có hai đơn nên hiện hai dòng. Bình và Chi mỗi người một dòng.
- Dũng không có đơn nào nhưng vẫn có một dòng, với `order_id` là NULL.
- Nếu đổi thành `JOIN` thường thì dòng của Dũng biến mất.

## Thử ngay

Tìm những khách chưa từng đặt hàng:

```sql
SELECT c.name
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
WHERE o.order_id IS NULL;
```

**Đoán trước khi chạy:** ai nằm trong danh sách?

<details>
<summary>Xem kết quả</summary>

| NAME |
|---|
| Dũng |

`LEFT JOIN` giữ Dũng lại với `order_id` là NULL. `WHERE o.order_id IS NULL`
lọc đúng những khách không khớp với đơn nào.

</details>

## Lỗi hay gặp

**Dùng `INNER JOIN` khi cần cả dòng không khớp.** Câu dưới không bao giờ tìm
được khách chưa mua hàng, vì `JOIN` đã loại họ từ trước.

```sql
-- SAI — Dũng bị loại ngay ở bước JOIN
SELECT c.name
FROM customers c
JOIN orders o ON o.customer_id = c.customer_id
WHERE o.order_id IS NULL;
```

```sql
-- ĐÚNG — LEFT JOIN giữ Dũng lại để WHERE tìm ra
SELECT c.name
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
WHERE o.order_id IS NULL;
```

**Lọc bảng bên phải ở `WHERE`.** Điều kiện `o.status = 'PAID'` loại mọi dòng
không phải `PAID`, kể cả dòng của khách không có đơn (ở đó `o.status` là
NULL), nên `LEFT JOIN` hoạt động như `JOIN`.
Điều kiện cho bảng bên phải đặt trong `ON`.

```sql
-- SAI — mất Chi và Dũng
SELECT c.name, o.order_id
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
WHERE o.status = 'PAID';
```

```sql
-- ĐÚNG — mọi khách, kèm đơn đã thanh toán nếu có
SELECT c.name, o.order_id
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
                  AND o.status = 'PAID';
```

## Tóm tắt

- `LEFT JOIN` giữ mọi dòng bảng bên trái, dòng không khớp thì cột bảng phải
  là NULL.
- Tìm dòng "không có": `LEFT JOIN` rồi `WHERE cột_bên_phải IS NULL`.
- Điều kiện lọc bảng bên phải đặt trong `ON`, không đặt ở `WHERE`.
- `INNER JOIN` chỉ giữ cặp khớp nhau, `LEFT JOIN` giữ đủ bảng trái.

```quiz
[
  {
    "prompt": "Muốn liệt kê mọi sản phẩm kèm số lượng đã bán, kể cả sản phẩm chưa bán được lần nào. Nên dùng gì?",
    "options": [
      "products JOIN order_lines",
      "order_lines JOIN products",
      "Chỉ SELECT từ order_lines",
      "products LEFT JOIN order_lines"
    ],
    "answer": 4,
    "explain": "products đứng bên trái của LEFT JOIN thì sản phẩm chưa bán vẫn được giữ, với các cột của order_lines là NULL."
  },
  {
    "prompt": "customers LEFT JOIN orders, khách X không có đơn nào. Cột o.order_id trên dòng của X là gì?",
    "options": [
      "NULL",
      "0",
      "Dòng của X không xuất hiện",
      "Báo lỗi"
    ],
    "answer": 1,
    "explain": "LEFT JOIN vẫn giữ dòng của X, các cột lấy từ bảng bên phải không có dữ liệu nên là NULL."
  },
  {
    "prompt": "Đếm số đơn của mỗi khách, khách chưa có đơn thì hiện 0. Hàm nào đúng sau LEFT JOIN và GROUP BY c.name?",
    "options": [
      "COUNT(*)",
      "COUNT(o.order_id)",
      "SUM(o.order_id)",
      "MAX(o.order_id)"
    ],
    "answer": 2,
    "explain": "COUNT(o.order_id) bỏ qua NULL, nên khách không có đơn được đếm là 0. COUNT(*) đếm cả dòng NULL, ra 1."
  }
]
```
