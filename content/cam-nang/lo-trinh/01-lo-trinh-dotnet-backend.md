---
title: Lộ trình .NET backend từ số 0 đến fresher
summary: Học gì, theo thứ tự nào và làm dự án gì để đủ sức ứng tuyển vị trí fresher .NET.
updated: 2026-09-27
---

Fresher .NET cần làm được một việc: viết một Web API có database, đưa lên
GitHub và giải thích được code của mình. Lộ trình dưới đây đi thẳng tới đích
đó, theo đúng thứ tự các khoá trên VieLang.

## Thứ tự học

| # | Khoá | Học xong làm được |
|---|---|---|
| 1 | [C# Core](/it/csharp-core) | Viết chương trình console: biến, vòng lặp, method, collection, class, LINQ, exception, async |
| 2 | [OOP và thiết kế](/it/oop-thiet-ke) | Chia code thành class và interface hợp lý, hiểu SOLID |
| 3 | [SQL với Oracle](/it/sql) | Truy vấn, nối bảng, tổng hợp, transaction, index |
| 4 | [ASP.NET Core Web API](/it/aspnet-core) | Xây API CRUD có DI, EF Core, xử lý lỗi, JWT |
| 5 | [Cấu trúc dữ liệu và giải thuật](/it/dsa) | Chọn đúng cấu trúc dữ liệu, giải bài phỏng vấn cơ bản |
| 6 | [Kiến trúc và chất lượng code](/it/kien-truc) | Git, test, refactor, chia tầng |

[WinForms với Oracle](/it/winforms) học khi công ty bạn nhắm tới làm phần
mềm desktop nội bộ. Nhiều công ty outsource và doanh nghiệp vẫn duy trì app
WinForms.

## Mỗi giai đoạn: học một phần, làm một phần

Đọc bài xong mà không gõ lại thì quên rất nhanh. Với mỗi khoá:

- Gõ lại mọi ví dụ, không copy.
- Làm hết phần "Thử ngay" và câu hỏi tự kiểm tra cuối bài.
- Cuối khoá, tự làm một bài tập nhỏ không nhìn bài mẫu.

Gợi ý bài tập cuối mỗi khoá:

- **C# Core**: app console quản lý chi tiêu, lưu ra file JSON.
- **OOP**: tách app chi tiêu thành các class có interface, đổi cách lưu từ
  file sang bộ nhớ mà không sửa code gọi.
- **SQL**: thiết kế database cho thư viện sách (sách, độc giả, phiếu mượn),
  viết 10 câu truy vấn báo cáo.
- **ASP.NET Core**: Web API cho thư viện sách, dùng database vừa thiết kế.

## Dự án để đi xin việc

Một dự án làm kỹ có giá trị hơn năm dự án dở dang. Dự án tối thiểu nên có:

- Web API ASP.NET Core, EF Core, một database quan hệ.
- Đăng nhập bằng JWT, phân quyền ít nhất hai vai trò.
- Validation, xử lý lỗi thống nhất, log.
- Vài unit test cho phần logic.
- README ghi cách chạy trong một lệnh (xem bài
  [Docker cơ bản](/cam-nang/cong-cu/docker-cho-dotnet)).
- Lịch sử commit rõ ràng trên GitHub.

## Khi nào thì đi ứng tuyển

Bạn sẵn sàng khi tự làm được những việc sau mà không cần xem hướng dẫn:

- [ ] Tạo project Web API mới, thêm controller CRUD cho một bảng
- [ ] Viết câu SQL có JOIN, GROUP BY, HAVING
- [ ] Giải thích class, interface, kế thừa, đa hình bằng ví dụ của mình
- [ ] Giải thích dependency injection trong ASP.NET Core làm gì
- [ ] Tạo nhánh, commit, mở pull request, xử lý conflict
- [ ] Có một dự án trên GitHub kèm README

Không cần chờ học hết khoá DSA hay Kiến trúc mới nộp đơn. Vừa ứng tuyển
vừa học tiếp là bình thường.

## Nguồn

- [roadmap.sh — ASP.NET Core Roadmap](https://roadmap.sh/aspnet-core)
- [Microsoft Learn — C# documentation](https://learn.microsoft.com/en-us/dotnet/csharp/)
- [Microsoft Learn — ASP.NET Core documentation](https://learn.microsoft.com/en-us/aspnet/core/)
