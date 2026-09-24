---
title: NULL trong SQL
minutes: 5
---

Khách Chi chưa cung cấp email. Ô email của Chi không phải chuỗi rỗng, cũng
không phải số 0, mà là "không có gì". SQL gọi đó là `NULL`, và nó không hành
xử như một giá trị bình thường.

## Khái niệm

🕳️ **NULL**: giá trị cho biết dữ liệu không có hoặc chưa biết, khác với 0 hay chuỗi rỗng.

🩹 **NVL**: hàm của Oracle trả về giá trị thay thế khi gặp NULL, ví dụ `NVL(email, 'chưa có')`.

Mọi phép so sánh với `NULL` bằng `=` hay `<>` đều cho kết quả không đúng
cũng không sai, nên `WHERE` loại dòng đó. Muốn kiểm tra NULL phải dùng
`IS NULL` hoặc `IS NOT NULL`.

Khác `null` của C#: `= NULL` không bao giờ đúng, và `''` cũng là NULL. `NVL`
thì giống toán tử `??`.

## Ví dụ

```sql
SELECT name
FROM customers
WHERE email IS NULL;
```

- `IS NULL` tìm những dòng chưa có email. Kết quả là Chi.
- `IS NOT NULL` thì ngược lại.
- Oracle coi chuỗi rỗng `''` là `NULL`. Lưu `''` vào cột nào thì ô đó thành
  `NULL`.

## Thử ngay

```sql
SELECT name, NVL(email, 'chưa có') AS email
FROM customers
ORDER BY customer_id;
```

`AS email` đặt tên cho cột kết quả.

**Đoán trước khi chạy:** dòng của Chi hiện gì ở cột email?

<details>
<summary>Xem kết quả</summary>

| NAME | EMAIL |
|---|---|
| An | an@shop.vn |
| Bình | binh@shop.vn |
| Chi | chưa có |
| Dũng | dung@shop.vn |

Email của Chi là `NULL`, nên `NVL` thay bằng "chưa có". Những dòng có email
thì giữ nguyên.

</details>

## Lỗi hay gặp

**So sánh với NULL bằng dấu `=`.** Câu lệnh chạy được nhưng không trả về dòng
nào, kể cả dòng của Chi.

```sql
-- SAI — không bao giờ có kết quả
SELECT name FROM customers WHERE email = NULL;
```

```sql
-- ĐÚNG
SELECT name FROM customers WHERE email IS NULL;
```

**Tìm chuỗi rỗng bằng `= ''`.** Trong Oracle, `''` chính là `NULL`, nên câu
này cũng không trả về dòng nào.

```sql
-- SAI — '' là NULL, so sánh bằng = luôn loại dòng
SELECT name FROM customers WHERE email = '';
```

## Tóm tắt

- `NULL` là không có dữ liệu, khác 0 và khác chuỗi rỗng.
- Kiểm tra bằng `IS NULL`, `IS NOT NULL`. Dùng `= NULL` thì không bao giờ có
  kết quả.
- `NVL(cột, giá_trị)` thay NULL bằng giá trị khác khi hiển thị.
- Oracle coi `''` là `NULL`.

```quiz
[
  {
    "prompt": "Bảng products có cột discount, một số dòng là NULL. Câu nào tìm các sản phẩm chưa có giảm giá?",
    "options": [
      "WHERE discount = NULL",
      "WHERE discount = 0",
      "WHERE discount IS NULL",
      "WHERE discount = ''"
    ],
    "answer": 3,
    "explain": "Chỉ IS NULL mới tìm được NULL. Dấu = với NULL không bao giờ đúng, còn 0 là một giá trị khác hẳn."
  },
  {
    "prompt": "Muốn hiện 0 thay cho NULL ở cột discount khi hiển thị. Viết thế nào?",
    "options": [
      "NVL(discount, 0)",
      "discount IS NULL",
      "NULL(discount, 0)",
      "discount = 0"
    ],
    "answer": 1,
    "explain": "NVL trả về giá trị thứ hai khi giá trị thứ nhất là NULL."
  },
  {
    "prompt": "Trong Oracle, INSERT một dòng với email = '' rồi tìm bằng WHERE email IS NULL. Dòng đó có được tìm thấy không?",
    "options": [
      "Không, vì '' khác NULL",
      "Báo lỗi khi INSERT",
      "Chỉ tìm thấy bằng WHERE email = ''",
      "Có, vì Oracle lưu '' thành NULL"
    ],
    "answer": 4,
    "explain": "Oracle coi chuỗi rỗng là NULL, nên dòng đó có email là NULL và được IS NULL tìm thấy."
  }
]
```
