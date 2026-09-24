---
title: INSERT, UPDATE, DELETE
minutes: 5
---

Có khách mới đăng ký, sản phẩm tăng giá, một dòng hàng bị nhập nhầm. Ba việc
này tương ứng với ba câu lệnh thay đổi dữ liệu. Chúng dễ viết, nhưng chỉ cần
quên một dòng `WHERE` là sửa nhầm cả bảng.

## Khái niệm

➕ **INSERT**: câu lệnh thêm một dòng mới vào bảng.

✏️ **UPDATE**: câu lệnh sửa giá trị trong những dòng thoả điều kiện `WHERE`.

🗑️ **DELETE**: câu lệnh xoá những dòng thoả điều kiện `WHERE`.

## Ví dụ

```sql
INSERT INTO customers (name, email, city)
  VALUES ('Em', 'em@shop.vn', 'Huế');

UPDATE products
SET price = 5500
WHERE product_id = 1;

DELETE FROM order_lines
WHERE line_id = 6;
```

- `INSERT` liệt kê tên cột rồi giá trị theo đúng thứ tự đó. Cột không được
  liệt kê thì nhận NULL hoặc giá trị mặc định.
- `UPDATE ... SET cột = giá_trị` sửa được nhiều cột, cách nhau bằng dấu
  phẩy.
- `SET` dùng được giá trị cũ của cột: `SET stock = stock - 1`.
- `WHERE` của `UPDATE` và `DELETE` giống hệt `WHERE` của `SELECT`.

## Thử ngay

Tăng giá 10% cho mọi sản phẩm dưới 10.000đ, rồi xem lại:

```sql
UPDATE products
SET price = price * 1.1
WHERE price < 10000;

SELECT name, price
FROM products
ORDER BY product_id;
```

**Đoán trước khi chạy:** những món nào đổi giá, và giá mới là bao nhiêu?

<details>
<summary>Xem kết quả</summary>

| NAME | PRICE |
|---|---|
| Bút bi | 5500 |
| Vở | 12000 |
| Balo | 350000 |
| Thước | 7700 |
| Máy tính | 450000 |

Chỉ Bút bi và Thước đổi giá. Vở giá 12000, không dưới 10.000đ nên giữ
nguyên.

</details>

## Lỗi hay gặp

**`UPDATE` quên `WHERE`.** Câu lệnh vẫn chạy, không báo lỗi gì, nhưng sửa
**mọi** dòng trong bảng.

```sql
-- SAI — mọi sản phẩm đều thành 5500đ
UPDATE products SET price = 5500;
```

```sql
-- ĐÚNG — chỉ sửa đúng sản phẩm cần sửa
UPDATE products SET price = 5500
WHERE product_id = 1;
```

**`DELETE` quên `WHERE`.** Tương tự, cả bảng bị xoá sạch. Thói quen an toàn:
viết `SELECT` với cùng điều kiện `WHERE` trước, thấy đúng những dòng cần xoá
rồi mới đổi thành `DELETE`.

```sql
-- SAI — xoá hết mọi dòng hàng
DELETE FROM order_lines;
```

## Tóm tắt

- `INSERT INTO bảng (cột...) VALUES (giá_trị...)` thêm dòng mới.
- `UPDATE bảng SET cột = giá_trị WHERE ...` sửa các dòng thoả điều kiện.
- `DELETE FROM bảng WHERE ...` xoá các dòng thoả điều kiện.
- Không có `WHERE` thì `UPDATE` và `DELETE` tác động lên cả bảng. Chạy thử
  `SELECT` với cùng `WHERE` trước.

```quiz
[
  {
    "prompt": "Khách số 2 đổi email thành binh2@shop.vn. Câu nào đúng?",
    "options": [
      "UPDATE customers SET email = 'binh2@shop.vn'",
      "INSERT INTO customers (email) VALUES ('binh2@shop.vn')",
      "UPDATE customers SET email = 'binh2@shop.vn' WHERE customer_id = 2",
      "UPDATE customers WHERE customer_id = 2"
    ],
    "answer": 3,
    "explain": "UPDATE kèm WHERE chỉ sửa đúng khách số 2. Thiếu WHERE thì mọi khách đều đổi email."
  },
  {
    "prompt": "Nhập thêm 20 cây bút (product_id = 1) vào kho. Viết thế nào?",
    "options": [
      "UPDATE products SET stock = stock + 20 WHERE product_id = 1",
      "UPDATE products SET stock = 20 WHERE product_id = 1",
      "INSERT INTO products (stock) VALUES (20)",
      "UPDATE products SET stock + 20"
    ],
    "answer": 1,
    "explain": "SET stock = stock + 20 cộng thêm vào số cũ. Viết stock = 20 là ghi đè, mất số tồn cũ."
  },
  {
    "prompt": "Trước khi chạy DELETE FROM orders WHERE status = 'CANCELLED', thói quen nào an toàn nhất?",
    "options": [
      "Chạy luôn, nếu sai thì INSERT lại",
      "Bỏ WHERE cho nhanh",
      "Đổi thành UPDATE",
      "Chạy SELECT * FROM orders WHERE status = 'CANCELLED' để xem trước"
    ],
    "answer": 4,
    "explain": "SELECT với cùng điều kiện cho thấy chính xác những dòng sắp bị xoá. Thấy đúng rồi mới đổi SELECT * thành DELETE."
  }
]
```
