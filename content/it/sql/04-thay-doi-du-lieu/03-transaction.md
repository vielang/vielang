---
title: Transaction
minutes: 5
---

Đặt một đơn hàng gồm ba bước: thêm đơn, thêm dòng hàng, trừ tồn kho. Nếu mất
điện sau bước thứ hai thì đơn đã tạo mà kho chưa trừ, dữ liệu bị lệch. Ba bước
phải cùng thành công hoặc cùng bị huỷ. Transaction đảm bảo điều đó.

## Khái niệm

🔄 **Transaction**: nhóm câu lệnh thay đổi dữ liệu được thực hiện trọn vẹn cùng nhau, hoặc tất cả có hiệu lực, hoặc không câu nào có hiệu lực.

✅ **COMMIT**: xác nhận mọi thay đổi trong transaction, từ lúc này người khác mới thấy chúng và chúng không còn bị mất.

↩️ **ROLLBACK**: huỷ mọi thay đổi chưa `COMMIT`, đưa dữ liệu về như lúc bắt đầu transaction.

## Ví dụ

```sql
INSERT INTO orders (customer_id, order_date, status)
  VALUES (4, DATE '2025-03-01', 'NEW');

INSERT INTO order_lines
  (order_id, product_id, quantity, unit_price)
  VALUES (5, 3, 1, 350000);

UPDATE products SET stock = stock - 1
WHERE product_id = 3;

COMMIT;
```

- Trong Oracle, câu `INSERT`, `UPDATE` hoặc `DELETE` đầu tiên tự mở một
  transaction.
- Các thay đổi chỉ được lưu hẳn khi chạy `COMMIT`.
- Có lỗi giữa chừng thì gọi `ROLLBACK` để huỷ cả ba bước.
- Trước khi `COMMIT`, chỉ phiên (một kết nối đang mở tới Oracle) đã thực hiện
  thay đổi mới thấy chúng. Người khác vẫn thấy dữ liệu cũ.

```mermaid Một transaction kết thúc bằng COMMIT hoặc ROLLBACK
stateDiagram-v2
    state "Đang dở, chỉ phiên này thấy" as Do
    state "Đã lưu hẳn, mọi phiên đều thấy" as Luu
    state "Đã huỷ, dữ liệu như trước transaction" as Huy
    [*] --> Do : INSERT, UPDATE, DELETE
    Do --> Do : thêm câu lệnh
    Do --> Luu : COMMIT
    Do --> Huy : ROLLBACK
```

## Thử ngay

```sql
UPDATE products SET stock = stock - 5
WHERE product_id = 1;

ROLLBACK;

SELECT name, stock FROM products
WHERE product_id = 1;
```

**Đoán trước khi chạy:** bút bi có 120 cây, vừa trừ 5 rồi `ROLLBACK`. Tồn kho
là 115 hay 120?

<details>
<summary>Xem kết quả</summary>

| NAME | STOCK |
|---|---|
| Bút bi | 120 |

Là 120. `ROLLBACK` huỷ câu `UPDATE` vì nó chưa được `COMMIT`.

</details>

## Lỗi hay gặp

**Quên `COMMIT`.** Bạn thấy dữ liệu đã đổi trong phiên của mình, nhưng người
khác thì không. Đóng kết nối mà chưa `COMMIT` thì thay đổi có thể mất.

**Chạy `CREATE TABLE` giữa transaction.** Câu lệnh tạo bảng (DDL) trong Oracle
tự `COMMIT` mọi thay đổi đang dở. `ROLLBACK` sau đó không huỷ được câu
`UPDATE` phía trước nữa.

```sql
-- SAI — CREATE TABLE đã tự COMMIT câu UPDATE
UPDATE products SET stock = 0 WHERE product_id = 1;
CREATE TABLE temp_log (note VARCHAR2(100));
ROLLBACK;
SELECT stock FROM products WHERE product_id = 1;
```

Câu `SELECT` cuối trả về 0 chứ không phải 120. Tạo bảng trước, rồi mới bắt
đầu thay đổi dữ liệu.

## Tóm tắt

- Transaction gom nhiều câu lệnh thành một khối: cùng thành công hoặc cùng
  bị huỷ.
- `COMMIT` lưu hẳn thay đổi, `ROLLBACK` huỷ thay đổi chưa lưu.
- Chưa `COMMIT` thì người khác chưa thấy thay đổi.
- `CREATE TABLE` và các lệnh DDL khác tự `COMMIT` trong Oracle.

```quiz
[
  {
    "prompt": "Chuyển 100.000đ từ tài khoản A sang B gồm hai câu UPDATE. Câu thứ hai lỗi. Nên làm gì?",
    "options": [
      "COMMIT để giữ câu thứ nhất",
      "Chạy lại câu thứ hai rồi thôi",
      "ROLLBACK để huỷ cả câu thứ nhất",
      "Không cần làm gì"
    ],
    "answer": 3,
    "explain": "Hai câu phải cùng thành công. Câu hai lỗi thì ROLLBACK, nếu không A bị trừ tiền mà B không được cộng."
  },
  {
    "prompt": "Bạn UPDATE giá sản phẩm nhưng chưa COMMIT. Đồng nghiệp SELECT cùng sản phẩm đó ở máy khác. Họ thấy giá nào?",
    "options": [
      "Giá cũ",
      "Giá mới",
      "Báo lỗi",
      "NULL"
    ],
    "answer": 1,
    "explain": "Thay đổi chưa COMMIT chỉ phiên của bạn thấy. Người khác vẫn đọc dữ liệu đã được COMMIT trước đó."
  },
  {
    "prompt": "UPDATE ...; CREATE TABLE ...; ROLLBACK; Câu UPDATE có bị huỷ không?",
    "options": [
      "Có, ROLLBACK huỷ mọi thứ",
      "Chỉ huỷ nếu CREATE TABLE lỗi",
      "Báo lỗi vì không được CREATE TABLE sau UPDATE",
      "Không, vì CREATE TABLE đã tự COMMIT câu UPDATE"
    ],
    "answer": 4,
    "explain": "Trong Oracle, lệnh DDL như CREATE TABLE tự COMMIT những thay đổi đang dở trước khi chạy."
  }
]
```
