---
title: Git hằng ngày: quy trình làm việc theo nhánh
summary: Chuỗi lệnh Git một developer dùng mỗi ngày, từ nhận task tới khi pull request được merge.
updated: 2026-09-27
---

Phần lớn team làm theo cùng một khung: nhánh `main` luôn chạy được, mỗi task
làm trên một nhánh riêng, xong thì mở pull request để review rồi mới merge.
Khái niệm commit, branch, merge, conflict đã có ở chương
[Git](/learn/kien-truc/git/commit-va-lich-su) của khoá Kiến trúc; bài này ghép
chúng thành quy trình.

## Bắt đầu một task

Lấy code mới nhất rồi tạo nhánh từ `main`:

```bash
git switch main
git pull
git switch -c feature/123-them-bo-loc-san-pham
```

Tên nhánh theo quy ước của team. Kiểu hay gặp: `feature/…`, `fix/…`, kèm mã
task để tra ngược.

## Trong lúc làm

Commit nhỏ, mỗi commit một thay đổi có ý nghĩa:

```bash
git status
git diff
git add src/Products/ProductFilter.cs
git commit -m "Thêm bộ lọc sản phẩm theo khoảng giá"
```

- Xem `git status` và `git diff` trước khi `add`, tránh commit nhầm file
  cấu hình, file log.
- Message nói commit làm gì. Nhiều team dùng quy ước
  [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):
  `feat: …`, `fix: …`.
- Đẩy nhánh lên remote thường xuyên để không mất việc khi hỏng máy:

```bash
git push -u origin feature/123-them-bo-loc-san-pham   # lần đầu
git push                                              # các lần sau
```

## Cập nhật nhánh khi main đã đổi

Người khác merge vào `main` trong lúc bạn làm. Kéo thay đổi đó về nhánh của
mình để xử lý conflict sớm:

```bash
git switch main
git pull
git switch feature/123-them-bo-loc-san-pham
git merge main
```

Một số team yêu cầu `git rebase main` thay cho `merge` để lịch sử thẳng. Chỉ
rebase nhánh chưa ai khác dùng chung, và sau khi rebase phải đẩy lên bằng
`git push --force-with-lease`.

## Mở pull request

- Tự đọc lại diff trên GitHub trước khi gửi reviewer.
- Mô tả: làm gì, vì sao, cách test. Gắn link task.
- Sửa theo góp ý bằng commit mới trên cùng nhánh, pull request tự cập nhật.

Sau khi merge, dọn nhánh:

```bash
git switch main
git pull
git branch -d feature/123-them-bo-loc-san-pham
```

## Lệnh cứu nguy

| Tình huống | Lệnh |
|---|---|
| Bỏ thay đổi chưa commit của một file | `git restore <file>` |
| Bỏ file khỏi vùng chuẩn bị commit | `git restore --staged <file>` |
| Sửa message commit cuối (chưa push) | `git commit --amend` |
| Cất tạm thay đổi để đổi nhánh | `git stash`, xong việc thì `git stash pop` |
| Huỷ một commit đã push | `git revert <commit>` |
| Xem lịch sử gọn | `git log --oneline --graph` |

Không dùng `git push --force` lên nhánh chung như `main`: nó ghi đè commit
của người khác.

## Nguồn

- [Pro Git — Branching Workflows](https://git-scm.com/book/en/v2/Git-Branching-Branching-Workflows)
- [Git Documentation — git-rebase](https://git-scm.com/docs/git-rebase)
- [GitHub Docs — GitHub flow](https://docs.github.com/en/get-started/using-github/github-flow)
- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)
