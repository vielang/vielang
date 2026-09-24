---
title: Commit và lịch sử
minutes: 6
---

Sửa code xong, chạy thấy hỏng, mà không nhớ đã đổi những gì. Hoặc muốn quay
lại bản chạy được hôm qua nhưng đã lưu đè mất. Git lưu lại từng mốc thay đổi
của project để bạn xem lại hay quay về lúc nào cũng được.

## Khái niệm

📸 **Commit**: một mốc lưu lại nội dung các file của project tại một thời điểm, kèm lời mô tả và tên người tạo.

🛒 **Staging area**: nơi gom những thay đổi sẽ đi vào commit tiếp theo, thêm vào bằng `git add`.

🙈 **.gitignore**: file liệt kê những file, thư mục Git bỏ qua, không bao giờ đưa vào commit.

Một thay đổi đi qua ba chỗ: sửa trong thư mục làm việc, `git add` vào
staging area, rồi `git commit` để thành một mốc trong lịch sử.

## Ví dụ

Cài Git xong, khai báo tên và email một lần. Git gắn chúng vào mỗi commit:

```bash
git config --global user.name "An"
git config --global user.email "an@shop.vn"
```

Mở terminal ở thư mục chứa `ShopApi`, `ShopApi.Tests` và `ShopDesk`, rồi đưa
cả ba vào một kho Git:

```bash
git init -b main
dotnet new gitignore
# mở .gitignore, thêm dòng appsettings.Development.json
git add .
git commit -m "Commit đầu tiên"
git log --oneline
```

- `git init -b main` tạo kho Git (repository) trong thư mục hiện tại, nhánh
  chính tên `main`.
- `dotnet new gitignore` tạo sẵn `.gitignore` cho project .NET, bỏ qua các
  thư mục do build sinh ra như `bin/`, `obj/`.
- `appsettings.Development.json` đang chứa mật khẩu Oracle từ bài EF Core
  và DbContext. Mật khẩu không được commit, nên thêm file này vào
  `.gitignore` trước khi `git add`.
- `git add .` đưa mọi file vào staging area, `git commit -m` tạo commit kèm
  lời mô tả.
- `git log --oneline` in mỗi commit một dòng: mã commit và lời mô tả.

## Thử ngay

Thêm một dòng comment vào cuối `ShopApi/Program.cs`, tạo file mới
`Notes.txt` ở thư mục gốc, rồi chạy:

```bash
git status
```

**Đoán trước khi chạy:** Git nói gì về hai file này?

<details>
<summary>Xem kết quả</summary>

```text
On branch main
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   ShopApi/Program.cs

Untracked files:
  (use "git add <file>..." to include in what will be committed)
	Notes.txt

no changes added to commit (use "git add" and/or "git commit -a")
```

`ShopApi/Program.cs` đã có trong commit trước nên là "modified", đã sửa
nhưng chưa `git add`. Git ghi đường dẫn tính từ thư mục gốc của kho.

`Notes.txt` là file mới, Git chưa theo dõi nên là "untracked".
Muốn đưa cả hai vào commit tiếp theo thì `git add` chúng.

</details>

## Lỗi hay gặp

**Commit trước khi có `.gitignore`.** `bin/`, `obj/` nặng và đổi sau mỗi lần
build, làm lịch sử rối.

```bash
# SAI — add hết khi chưa có .gitignore
git init -b main
git add .
```

```bash
# ĐÚNG — tạo .gitignore trước khi add
git init -b main
dotnet new gitignore
git add .
```

Tốt hơn nữa là không để mật khẩu trong file của project: bài Cấu hình với
appsettings khuyên dùng user-secrets.

## Tóm tắt

- Commit là một mốc trạng thái của project, kèm lời mô tả.
- Sửa file, `git add` vào staging area, rồi `git commit`.
- `git status` cho biết file nào đã sửa, file nào mới. `git log` xem lịch sử.
- Tạo `.gitignore` trước commit đầu tiên, không commit `bin/`, `obj/` và
  file chứa mật khẩu.

```quiz
[
  {
    "prompt": "Sửa Product.cs rồi chạy git commit -m \"Sửa giá\" mà chưa git add. Commit mới có thay đổi đó không?",
    "options": [
      "Không, chỉ phần đã git add mới vào",
      "Có, commit tự lấy mọi thay đổi",
      "Có, nếu file đã lưu trong editor",
      "Git báo lỗi và bỏ thay đổi đó"
    ],
    "answer": 1,
    "explain": "Commit lấy nội dung trong staging area. Thay đổi chưa add vẫn nằm ở thư mục làm việc."
  },
  {
    "prompt": "git status báo một file là \"Untracked\". Nghĩa là gì?",
    "options": [
      "File đã bị xoá",
      "File mới, Git chưa từng theo dõi",
      "File đã commit nhưng bị sửa",
      "File nằm trong .gitignore"
    ],
    "answer": 2,
    "explain": "Untracked là file chưa từng được add. File trong .gitignore thì git status không hiện ra."
  },
  {
    "prompt": "Thư mục nào không nên commit trong project .NET?",
    "options": [
      "Controllers",
      "Migrations",
      "bin và obj",
      "Properties"
    ],
    "answer": 3,
    "explain": "bin và obj do build sinh ra, ai build cũng tạo lại được. Migrations là code nên phải commit."
  }
]
```
