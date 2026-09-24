---
title: Hàm tổng hợp
minutes: 5
---

Chủ cửa hàng không cần xem từng dòng. Họ muốn biết có bao nhiêu sản phẩm, tổng
tồn kho là bao nhiêu, món rẻ nhất giá bao nhiêu. Để trả lời, cần gom cả bảng
thành một con số, và hàm tổng hợp làm việc đó.

## Khái niệm

🧮 **Hàm tổng hợp (aggregate function)**: hàm nhận nhiều dòng và trả về một giá trị duy nhất, như đếm, cộng, tìm lớn nhất.

| Hàm | Trả về |
|---|---|
| `COUNT(*)` | số dòng |
| `COUNT(cột)` | số dòng mà cột đó không NULL |
| `SUM(cột)` | tổng |
| `AVG(cột)` | trung bình |
| `MIN(cột)`, `MAX(cột)` | nhỏ nhất, lớn nhất |

Mọi hàm trong bảng, trừ `COUNT(*)`, đều bỏ qua giá trị NULL.

## Ví dụ

```sql
SELECT COUNT(*)   AS product_count,
       SUM(stock) AS total_stock,
       MIN(price) AS min_price,
       MAX(price) AS max_price
FROM products;
```

- Kết quả chỉ có **một dòng**: 5 sản phẩm, tổng tồn kho 181, rẻ nhất 5000,
  đắt nhất 450000.
- `AS product_count` đặt tên cho cột kết quả.
- Có thể kết hợp với `WHERE`: thêm `WHERE stock > 0` thì chỉ tính các món
  còn hàng.

## Thử ngay

```sql
SELECT COUNT(*)     AS total,
       COUNT(email) AS with_email
FROM customers;
```

**Đoán trước khi chạy:** có 4 khách hàng. Hai cột này có bằng nhau không?

<details>
<summary>Xem kết quả</summary>

| TOTAL | WITH_EMAIL |
|---|---|
| 4 | 3 |

Không bằng. `COUNT(*)` đếm mọi dòng, còn `COUNT(email)` bỏ qua dòng có email
NULL, tức là bỏ qua Chi.

</details>

## Lỗi hay gặp

**Lấy cột thường cùng với hàm tổng hợp.** Hàm tổng hợp gom cả bảng thành một
dòng, nên Oracle không biết phải hiện `name` của dòng nào và báo lỗi
`ORA-00937: not a single-group group function`.

```sql
-- SAI — lỗi: name không được gom lại
SELECT name, COUNT(*) FROM products;
```

```sql
-- ĐÚNG — chỉ lấy giá trị đã tổng hợp
SELECT COUNT(*) FROM products;
```

Muốn đếm theo từng nhóm, ví dụ theo thành phố, thì dùng `GROUP BY` (học ở bài
sau).

## Tóm tắt

- Hàm tổng hợp gom nhiều dòng thành một giá trị: `COUNT`, `SUM`, `AVG`,
  `MIN`, `MAX`.
- `COUNT(*)` đếm mọi dòng, `COUNT(cột)` bỏ qua NULL.
- Không lấy cột thường cùng hàm tổng hợp khi chưa có `GROUP BY`.
- Kết hợp `WHERE` để chỉ tổng hợp các dòng cần.

```quiz
[
  {
    "prompt": "Bảng orders có 10 dòng, cột shipped_at NULL ở 4 dòng. COUNT(shipped_at) trả về bao nhiêu?",
    "options": [
      "10",
      "4",
      "6",
      "0"
    ],
    "answer": 3,
    "explain": "COUNT(cột) chỉ đếm những dòng mà cột đó không NULL: 10 - 4 = 6."
  },
  {
    "prompt": "Muốn biết tổng số tiền của mọi dòng hàng, biết mỗi dòng có quantity và unit_price. Viết câu nào?",
    "options": [
      "SELECT SUM(quantity * unit_price) FROM order_lines",
      "SELECT COUNT(quantity * unit_price) FROM order_lines",
      "SELECT MAX(quantity * unit_price) FROM order_lines",
      "SELECT quantity * unit_price FROM order_lines"
    ],
    "answer": 1,
    "explain": "SUM cộng giá trị của biểu thức qua mọi dòng. Phương án cuối trả về từng dòng chứ không cộng lại."
  },
  {
    "prompt": "SELECT city, COUNT(*) FROM customers; báo ORA-00937. Vì sao?",
    "options": [
      "Bảng customers không có cột city",
      "COUNT(*) phải viết COUNT(city)",
      "Thiếu dấu chấm phẩy",
      "Có cột thường (city) cùng với hàm tổng hợp mà không có GROUP BY"
    ],
    "answer": 4,
    "explain": "Không có GROUP BY thì cả bảng thành một dòng, Oracle không biết hiện city nào. Thêm GROUP BY city để đếm theo từng thành phố."
  }
]
```
