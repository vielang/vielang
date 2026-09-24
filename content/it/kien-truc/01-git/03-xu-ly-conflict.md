---
title: Xử lý conflict
minutes: 5
---

Hai người cùng sửa giá Bút bi, mỗi người trên một nhánh: một người ghi 5500,
người kia ghi 6000. Tới lúc merge, Git không thể tự chọn giá nào đúng. Đó là
conflict, và người merge phải tự quyết định.

## Khái niệm

⚔️ **Conflict (xung đột)**: khi hai nhánh cùng sửa một chỗ trong cùng một file, Git dừng merge lại và đánh dấu chỗ đó để người dùng tự chọn nội dung.

Sửa hai file khác nhau, hoặc hai chỗ khác nhau trong cùng một file, thì Git
tự gộp được. Conflict chỉ xảy ra khi cùng một dòng, hoặc các dòng sát nhau, bị sửa ở cả
hai phía.

## Ví dụ

Trên `main`, `prices.txt` có dòng `Bút bi: 5000`. Nhánh `tang-gia` sửa thành
6000, còn `main` sửa thành 5500:

```bash
git switch -c tang-gia
# sửa "Bút bi: 5000" thành "Bút bi: 6000"
git commit -am "Tăng giá Bút bi"
git switch main
# sửa "Bút bi: 5000" thành "Bút bi: 5500"
git commit -am "Giá Bút bi 5500"
git merge tang-gia
```

Git dừng lại và mở `prices.txt` ra thì thấy:

```text
<<<<<<< HEAD
Bút bi: 5500
=======
Bút bi: 6000
>>>>>>> tang-gia
Vở: 12000
Thước: 7000
```

- Phần giữa `<<<<<<< HEAD` và `=======` là nội dung của nhánh đang đứng
  (`main`).
- Phần giữa `=======` và `>>>>>>> tang-gia` là nội dung của nhánh đang merge
  vào.
- Các dòng khác không bị hai bên cùng sửa nên Git đã tự gộp.

Cách giải quyết: sửa file cho đúng nội dung cuối cùng, xoá cả ba dòng đánh
dấu, rồi:

```bash
git add prices.txt
git commit -m "Merge tang-gia, chốt giá 6000"
```

## Thử ngay

Tiếp tục với `prices.txt` của bài trước, làm lại các bước trong ví dụ tới
lệnh `git merge tang-gia`.

**Đoán trước khi chạy:** Git in ra gì, và `git status --short` báo gì về
`prices.txt`?

<details>
<summary>Xem kết quả</summary>

```text
Auto-merging prices.txt
CONFLICT (content): Merge conflict in prices.txt
Automatic merge failed; fix conflicts and then commit the result.

UU prices.txt
```

`CONFLICT (content)` là conflict trong nội dung file. `UU` nghĩa là cả hai
phía cùng sửa file này và chưa được giải quyết. Merge đang dừng giữa chừng,
chờ bạn sửa file, `git add` và `git commit`.

</details>

## Lỗi hay gặp

**Commit mà quên xoá dòng đánh dấu.** Git vẫn cho commit, nhưng file giờ
chứa `<<<<<<<` và `=======`. Nếu đó là file `.cs` thì project không build
được nữa.

```text
# SAI — còn nguyên dấu conflict
<<<<<<< HEAD
Bút bi: 5500
=======
Bút bi: 6000
>>>>>>> tang-gia
```

```text
# ĐÚNG — chỉ giữ nội dung đã chốt
Bút bi: 6000
```

Trước khi `git add`, chạy `git diff --check`: còn sót dấu conflict thì Git
báo `leftover conflict marker` kèm số dòng.

## Tóm tắt

- Conflict xảy ra khi hai nhánh cùng sửa một chỗ trong một file.
- Git đánh dấu bằng `<<<<<<<`, `=======`, `>>>>>>>`: phía trên là nhánh
  đang đứng, phía dưới là nhánh đang merge vào.
- Sửa file cho đúng, xoá dấu, `git add` rồi `git commit`.
- Merge nhánh thường xuyên để conflict nhỏ và dễ gỡ.

```quiz
[
  {
    "prompt": "Nhánh A sửa Product.cs, nhánh B sửa Order.cs. Merge B vào A thì sao?",
    "options": [
      "Luôn conflict",
      "Git tự gộp được, vì hai nhánh sửa hai file khác nhau",
      "Git xoá một trong hai file",
      "Phải merge bằng tay từng dòng"
    ],
    "answer": 2,
    "explain": "Conflict chỉ xảy ra khi cùng một chỗ bị sửa ở cả hai phía."
  },
  {
    "prompt": "Trong file conflict, nội dung giữa <<<<<<< HEAD và ======= là của ai?",
    "options": [
      "Nhánh đang merge vào",
      "Commit đầu tiên của project",
      "Git tự đoán",
      "Nhánh đang đứng"
    ],
    "answer": 4,
    "explain": "HEAD là nhánh hiện tại. Phần của nhánh đang merge vào nằm giữa ======= và >>>>>>>."
  },
  {
    "prompt": "Đã sửa xong nội dung file conflict. Bước tiếp theo là gì?",
    "options": [
      "git merge lại từ đầu",
      "Xoá nhánh kia",
      "git add file đó rồi git commit",
      "Không cần làm gì"
    ],
    "answer": 3,
    "explain": "git add đánh dấu conflict đã giải quyết, git commit hoàn tất merge."
  }
]
```
