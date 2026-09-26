# Chuẩn viết bài Cẩm nang

File này không nằm trong thư mục mục nào nên bước build bỏ qua nó.

Cẩm nang là kiến thức nghề cho người học lập trình theo lộ trình .NET trên
VieLang: lộ trình, phỏng vấn, công cụ, việc làm IT. Khác bài học trong
`content/it` (mỗi bài dạy một khái niệm), bài cẩm nang là hướng dẫn để làm
theo: chuẩn bị phỏng vấn, viết CV, dựng môi trường.

## Nguyên tắc nội dung

- **Chỉ viết điều có nguồn.** Kiến thức kỹ thuật dựa trên tài liệu chính
  thức (learn.microsoft.com, git-scm.com, docs.docker.com, tài liệu Oracle,
  PostgreSQL) hoặc nguồn uy tín (roadmap.sh). Không tìm được nguồn thì KHÔNG
  viết, đừng đoán.
- **Không bịa số liệu.** Không đưa mức lương, tỉ lệ, thống kê tự nghĩ ra.
  Cần nói về lương thì nói định tính, hoặc dẫn một khảo sát công khai có
  thật, ghi rõ năm khảo sát.
- **Code phải chạy được** với phiên bản các khoá IT đang dùng (.NET 9,
  Oracle Free trong Docker, database mẫu của khoá SQL).
- **Không hứa hẹn** ("chắc chắn đậu phỏng vấn"), **không quảng cáo** trung
  tâm, công ty nào.
- Ngắn, đi thẳng vào việc: mở bài một hai câu, không câu nối rỗng, không ghi
  chú bên lề.
- Thuật ngữ kỹ thuật giữ tiếng Anh (pull request, dependency injection,
  index), viết đúng tên riêng: ASP.NET Core, GitHub, JavaScript.

## Cấu trúc thư mục

```
content/cam-nang/lo-trinh/<slug>.md    — lộ trình học và phát triển nghề
content/cam-nang/phong-van/<slug>.md   — chuẩn bị và câu hỏi phỏng vấn
content/cam-nang/cong-cu/<slug>.md     — công cụ làm việc hằng ngày
content/cam-nang/viec-lam/<slug>.md    — CV, portfolio, tìm việc, lương, remote
```

Danh sách mục nằm ở `src/lib/guide-sections.ts`; thêm mục = thêm thư mục và
một dòng ở đó.

`slug`: chữ thường, không dấu, nối bằng gạch ngang (vd `git-hang-ngay`).
Đặt số đầu để sắp thứ tự (`01-git-hang-ngay.md`), số bị bỏ khỏi slug:
URL là `/cam-nang/cong-cu/git-hang-ngay`.

## Phần khai báo đầu file

Mỗi dòng `khoá: giá trị`, giá trị là chữ trên MỘT dòng, KHÔNG đặt trong
ngoặc kép (ngoặc kép sẽ hiện nguyên trên trang).

```
---
title: Git hằng ngày: quy trình làm việc theo nhánh
summary: Một câu nói bài này giúp người đọc làm gì.
updated: 2026-09-27
---
```

- `title`, `updated` (YYYY-MM-DD) bắt buộc; thiếu thì bước build báo lỗi.
- `summary`: một câu, hiện dưới tiêu đề và trong danh sách bài.
- `group`: chỉ khi mục đó có chia nhóm trong `src/lib/guide-groups.ts`. Mục
  đã chia nhóm thì mọi bài phải có `group`, gõ sai tên nhóm thì build báo
  lỗi. Trong một nhóm, bài xếp theo số đầu tên file.

## Thân bài

- Dùng `##` cho các mục chính (mục lục lấy từ đây). Mục cuối cùng luôn là
  `## Nguồn`.
- Khối code ghi rõ ngôn ngữ: ` ```csharp `, ` ```sql `, ` ```bash `,
  ` ```dockerfile `, ` ```yaml `.
- Bảng dùng cho tra cứu nhanh (lệnh, so sánh). Bảng rộng tự cuộn ngang trên
  điện thoại.
- Link tới bài học trên site bằng đường dẫn tương đối: khoá `/it/<khoá>`,
  bài `/learn/<khoá>/<chương>/<bài>`, bài cẩm nang khác
  `/cam-nang/<mục>/<slug>`.

### Checklist

Việc người đọc cần làm và muốn đánh dấu (chuẩn bị phỏng vấn, kiểm tra CV
trước khi gửi) viết thành checklist. App cho tick và nhớ trạng thái:

```
## Chuẩn bị trước buổi phỏng vấn

- [ ] Đọc kỹ mô tả công việc
- [ ] Chuẩn bị phần giới thiệu bản thân
```

Mỗi dòng được nhận diện theo nội dung chữ: sửa chữ của một dòng thì dấu tick
cũ của dòng đó mất.

### Bài phỏng vấn

Câu hỏi in đậm, đáp án ngay dưới, ngắn đủ ý. Có một checklist
`## Chuẩn bị trước buổi phỏng vấn` ở đầu bài.

### Mục Nguồn

2–4 link tài liệu chính thức hoặc uy tín, ghi rõ tên trang. Mở từng link
kiểm tra còn sống trước khi thêm.

```
## Nguồn

- [Pro Git — Branching Workflows](https://git-scm.com/book/en/v2/Git-Branching-Branching-Workflows)
- [Microsoft Learn — Dependency injection in ASP.NET Core](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/dependency-injection)
```
