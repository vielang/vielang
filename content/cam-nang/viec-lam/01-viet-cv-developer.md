---
title: Viết CV developer cho người mới
summary: Cấu trúc CV một trang cho fresher và intern, cách viết dự án khi chưa có kinh nghiệm đi làm.
updated: 2026-09-27
---

Người tuyển dụng thường chỉ lướt CV trong thời gian ngắn trước khi quyết định
đọc tiếp. CV của người mới cần cho thấy ngay hai điều: bạn biết công nghệ họ
cần, và bạn đã tự làm ra thứ chạy được.

## Cấu trúc một trang

Người mới chỉ cần **một trang**, theo thứ tự:

1. **Thông tin liên hệ**: họ tên, email, số điện thoại, link GitHub. Không
   cần ảnh, ngày sinh, tình trạng hôn nhân trừ khi công ty yêu cầu.
2. **Mục tiêu** (không bắt buộc): một hai câu về vị trí bạn nhắm tới, ví dụ
   "Fresher .NET backend".
3. **Kỹ năng**: chia nhóm, chỉ ghi thứ bạn trả lời được câu hỏi phỏng vấn.
4. **Dự án**: phần quan trọng nhất khi chưa có kinh nghiệm.
5. **Kinh nghiệm**: thực tập, làm thêm liên quan (nếu có).
6. **Học vấn và chứng chỉ**.

Ví dụ mục Kỹ năng:

```
Ngôn ngữ:   C#, SQL
Framework:  ASP.NET Core Web API, EF Core, WinForms
Database:   Oracle, SQL Server
Công cụ:    Git, Docker, Visual Studio, Postman
```

## Viết mục dự án

Mỗi dự án gồm: tên, link GitHub, công nghệ, và 2–4 gạch đầu dòng nói **bạn đã
làm gì**, bắt đầu bằng động từ.

```
ShopApi — Web API quản lý cửa hàng           github.com/<tên>/shopapi
ASP.NET Core 9, EF Core, Oracle, JWT, Docker
- Thiết kế database 6 bảng, viết migration bằng EF Core
- Xây API CRUD cho sản phẩm và đơn hàng, validation và xử lý lỗi thống nhất
- Đăng nhập bằng JWT, phân quyền admin và nhân viên
- Viết unit test cho service tính tiền đơn hàng
```

Tránh viết chung chung như "Tham gia xây dựng hệ thống" hay "Tìm hiểu về
ASP.NET Core". Nói cụ thể đã làm phần nào.

## Lỗi hay gặp

- Liệt kê quá nhiều công nghệ chỉ mới nghe qua. Người phỏng vấn sẽ hỏi đúng
  những thứ đó.
- Chấm điểm kỹ năng bằng thanh hoặc sao ("C#: 4/5"). Không ai biết 4/5 là
  mức nào.
- Link GitHub dẫn tới repo trống hoặc chỉ có code mẫu từ khoá học.
- Sai chính tả tên công nghệ: "Asp.net core", "Sql". Viết đúng: ASP.NET Core,
  SQL, JavaScript, GitHub.
- CV tiếng Anh dùng máy dịch mà không đọc lại.

## Trước khi gửi

- [ ] CV gói gọn trong một trang, xuất ra PDF
- [ ] Tên file dạng `HoTen_DotNet_Fresher.pdf`
- [ ] Mọi link bấm được và mở đúng trang
- [ ] Mỗi dự án có README và chạy được theo hướng dẫn
- [ ] Đọc lại mô tả công việc, đưa kỹ năng họ yêu cầu (mà bạn có) lên đầu
- [ ] Nhờ một người khác đọc lại chính tả

## Nguồn

- [Tech Interview Handbook — Resume guide](https://www.techinterviewhandbook.org/resume/)
- [GitHub Docs — Managing your profile README](https://docs.github.com/en/account-and-profile/how-tos/profile-customization/managing-your-profile-readme)
