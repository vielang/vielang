---
title: Câu hỏi phỏng vấn SQL thường gặp (kèm đáp án)
summary: Các câu hỏi SQL về JOIN, GROUP BY, index, transaction hay gặp ở vòng fresher và junior, kèm đáp án và ví dụ trên Oracle.
updated: 2026-09-27
---

Ví dụ dùng database mẫu của khoá [SQL với Oracle](/it/sql): `customers`,
`products`, `orders`, `order_lines`. Vòng SQL thường có cả câu hỏi lý thuyết
và một bài viết truy vấn tại chỗ.

## Chuẩn bị trước buổi phỏng vấn

- [ ] Viết tay được câu JOIN, GROUP BY, HAVING không cần tra cú pháp
- [ ] Giải thích được index giúp gì và khi nào không giúp
- [ ] Nêu được bốn tính chất ACID kèm ví dụ chuyển tiền
- [ ] Biết cách chống SQL injection bằng parameter
- [ ] Hỏi trước (hoặc đoán qua mô tả công việc) công ty dùng Oracle, SQL Server hay PostgreSQL

## Truy vấn

**`INNER JOIN` và `LEFT JOIN` khác nhau thế nào?**

`INNER JOIN` chỉ giữ dòng khớp ở cả hai bảng. `LEFT JOIN` giữ mọi dòng của
bảng bên trái; cột bên phải là `NULL` khi không khớp.

Khách chưa đặt đơn nào:

```sql
SELECT c.customer_id, c.name
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
WHERE o.order_id IS NULL;
```

**`WHERE` và `HAVING` khác nhau thế nào?**

`WHERE` lọc từng dòng trước khi gom nhóm. `HAVING` lọc nhóm sau `GROUP BY`,
dùng được hàm tổng hợp.

```sql
SELECT customer_id, COUNT(*) AS order_count
FROM orders
GROUP BY customer_id
HAVING COUNT(*) >= 3;
```

**Lấy sản phẩm có giá cao thứ hai.**

```sql
SELECT name, price
FROM products
ORDER BY price DESC
OFFSET 1 ROWS FETCH NEXT 1 ROWS ONLY;
```

Nếu nhiều sản phẩm cùng giá cao nhất, hỏi lại người phỏng vấn "cao thứ hai"
tính theo dòng hay theo mức giá. Theo mức giá thì dùng `DENSE_RANK()`.

**`DELETE`, `TRUNCATE`, `DROP` khác nhau thế nào?**

- `DELETE`: xoá dòng, có `WHERE`, rollback được.
- `TRUNCATE`: xoá mọi dòng, nhanh hơn, không có `WHERE`. Trong Oracle đây là
  lệnh DDL, tự commit, không rollback được.
- `DROP`: xoá cả bảng.

**`UNION` và `UNION ALL` khác nhau thế nào?** `UNION` bỏ dòng trùng (tốn thêm
bước sắp xếp hoặc so sánh), `UNION ALL` giữ nguyên.

## Thiết kế

**Primary key và foreign key là gì?**

Primary key định danh duy nhất mỗi dòng, không được `NULL`. Foreign key là cột
tham chiếu tới primary key của bảng khác, đảm bảo không có đơn hàng trỏ tới
khách không tồn tại.

**Chuẩn hoá là gì?**

Tách dữ liệu thành các bảng để mỗi thông tin chỉ lưu một chỗ. Ví dụ không lưu
tên khách trong `orders` mà chỉ lưu `customer_id`. Sửa tên khách thì sửa một
dòng.

## Hiệu năng

**Index là gì?**

Cấu trúc phụ (thường là B-tree) giúp tìm dòng theo cột mà không phải quét cả
bảng. Đổi lại, `INSERT`, `UPDATE`, `DELETE` chậm hơn và tốn thêm dung lượng.

**Khi nào có index mà database không dùng?**

- Bọc cột trong hàm: `WHERE UPPER(name) = 'PEN'` không dùng được index thường
  trên `name`.
- Điều kiện trả về phần lớn bảng: quét cả bảng có thể nhanh hơn.
- `LIKE '%pen'`: ký tự đại diện ở đầu.

**Làm sao biết câu truy vấn chậm ở đâu?** Xem execution plan. Trong Oracle:
`EXPLAIN PLAN FOR <câu lệnh>;` rồi
`SELECT * FROM TABLE(DBMS_XPLAN.DISPLAY);`.

## Transaction

**ACID là gì?**

- Atomicity: transaction chạy hết hoặc không chạy gì.
- Consistency: dữ liệu luôn thoả các ràng buộc.
- Isolation: các transaction chạy song song không thấy trạng thái dở của nhau.
- Durability: đã commit thì không mất, kể cả khi server sập.

**Isolation level mặc định của Oracle là gì?** `READ COMMITTED`: mỗi câu lệnh
chỉ thấy dữ liệu đã commit trước khi câu lệnh bắt đầu.

## Bảo mật

**SQL injection là gì, chống thế nào?**

Kẻ tấn công chèn SQL vào dữ liệu nhập khi code nối chuỗi thành câu lệnh.
Chống bằng parameter: giá trị được gửi riêng, không bao giờ thành SQL.

```csharp
// SAI: nối chuỗi
var sql = "SELECT * FROM customers WHERE email = '" + email + "'";

// ĐÚNG: EF Core tự tạo parameter
var customer = await db.Customers.FirstOrDefaultAsync(c => c.Email == email);
```

## Nguồn

- [Oracle — SQL Language Reference (19c)](https://docs.oracle.com/en/database/oracle/oracle-database/19/sqlrf/)
- [Oracle — Indexes and Index-Organized Tables (Database Concepts 19c)](https://docs.oracle.com/en/database/oracle/oracle-database/19/cncpt/indexes-and-index-organized-tables.html)
- [PostgreSQL — Transaction Isolation](https://www.postgresql.org/docs/current/transaction-iso.html)
- [Use The Index, Luke — SQL indexing and tuning](https://use-the-index-luke.com/)
