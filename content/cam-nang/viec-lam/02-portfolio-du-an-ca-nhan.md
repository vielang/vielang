---
title: Portfolio và dự án cá nhân: làm gì để nhà tuyển dụng chú ý
summary: Chọn dự án, trình bày repo và README sao cho người tuyển dụng hiểu bạn làm được gì trong vài phút.
updated: 2026-09-27
---

Với người chưa có kinh nghiệm đi làm, dự án cá nhân là bằng chứng duy nhất
cho thấy bạn viết được code. Người xem thường chỉ mở README và vài file code,
nên phần trình bày quan trọng không kém phần code.

## Chọn dự án

Một dự án làm kỹ tốt hơn nhiều dự án dở dang. Dự án tốt cho vị trí .NET
backend có:

- Nghiệp vụ thật, dù nhỏ: quản lý thư viện, đặt lịch phòng khám, quản lý kho.
  Có quan hệ giữa các bảng và vài quy tắc nghiệp vụ.
- Đủ các phần công ty nào cũng cần: database, xác thực, validation, xử lý
  lỗi, log, test.
- Phần nào đó bạn tự nghĩ ra, không có trong khoá học.

Dự án ToDo hay clone theo video từng bước ít giá trị, vì người xem không
phân biệt được phần nào là của bạn.

## README

README là trang đầu tiên người tuyển dụng thấy. Khung gợi ý:

```markdown
# ShopApi

Web API quản lý cửa hàng: sản phẩm, đơn hàng, tồn kho.

## Công nghệ
ASP.NET Core 9, EF Core, Oracle, JWT, xUnit, Docker

## Chạy thử
docker compose up -d --build
Gọi thử: GET http://localhost:5000/api/products

## Tính năng
- Đăng nhập JWT, hai vai trò admin và nhân viên
- Đặt hàng tự trừ tồn kho trong một transaction

## Cấu trúc
src/ShopApi        Controller, cấu hình
src/Shop.Core      Entity, service, interface
tests/Shop.Tests   Unit test
```

- Chạy thử được bằng một lệnh (xem bài
  [Docker cơ bản](/cam-nang/cong-cu/docker-cho-dotnet)). Người xem hiếm khi
  tự cài database để thử.
- Thêm ảnh chụp màn hình nếu có giao diện, hoặc ví dụ request và response nếu là API.
- Ghi rõ phần khó nhất bạn đã giải quyết và cách làm.

## Repo gọn gàng

- `.gitignore` chuẩn cho .NET: không commit `bin/`, `obj/`.
- Không commit mật khẩu, connection string thật, khoá JWT. Dùng
  `appsettings.Development.json` với giá trị mẫu hoặc biến môi trường.
- Lịch sử commit có ý nghĩa, không phải một commit "init" chứa cả dự án.
- Ghim (pin) 2–3 repo tốt nhất lên trang GitHub cá nhân, thêm profile README
  giới thiệu ngắn.

## Kiểm tra trước khi đưa vào CV

- [ ] Clone repo về máy khác, làm theo README chạy được ngay
- [ ] Không còn secret trong code và lịch sử commit
- [ ] README có mô tả, công nghệ, cách chạy, tính năng chính
- [ ] Có ít nhất vài unit test chạy được bằng `dotnet test`
- [ ] Code không còn đoạn comment bỏ dở, file thừa
- [ ] Giải thích được mọi đoạn code trong repo nếu bị hỏi

## Nguồn

- [GitHub Docs — About READMEs](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes)
- [GitHub Docs — Managing your profile README](https://docs.github.com/en/account-and-profile/how-tos/profile-customization/managing-your-profile-readme)
- [Tech Interview Handbook — Resume guide](https://www.techinterviewhandbook.org/resume/)
