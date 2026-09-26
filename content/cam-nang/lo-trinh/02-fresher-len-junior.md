---
title: Từ fresher lên junior: 12 tháng đầu đi làm
summary: Việc cần tập trung theo từng giai đoạn trong năm đầu đi làm để được giao việc độc lập.
updated: 2026-09-27
---

Khác biệt chính giữa fresher và junior: junior nhận một task và tự làm xong,
chỉ cần hỏi khi thật sự bị chặn. Năm đầu đi làm là để đạt được điều đó.

## Tháng 1–3: hiểu dự án

- Cài được môi trường và chạy được dự án trên máy mình. Ghi lại các bước
  vướng, gửi cho team để bổ sung vào README.
- Đọc code theo một luồng cụ thể: từ một API endpoint đi xuống service, rồi
  xuống database.
- Nhận task nhỏ (sửa bug, thêm field) và làm theo đúng quy trình của team:
  tạo nhánh, commit, pull request, review.
- Học cách team đặt tên, chia tầng, viết log. Theo quy ước của dự án trước,
  kể cả khi bạn thích cách khác.

## Tháng 4–6: làm task độc lập

- Trước khi code, viết lại yêu cầu bằng lời của mình và hỏi lại người giao
  việc nếu có chỗ chưa rõ.
- Ước lượng thời gian cho task, rồi so với thời gian thực tế để ước lượng
  lần sau chính xác hơn.
- Tự test kỹ trước khi mở pull request: trường hợp thường, dữ liệu rỗng, dữ
  liệu sai.
- Đọc pull request của người khác. Đây là cách nhanh nhất để học code base.

## Tháng 7–12: mở rộng phạm vi

- Nhận một tính năng trọn vẹn, từ database tới API.
- Viết unit test cho phần mình làm.
- Tham gia review code của người khác, bắt đầu từ các lỗi dễ thấy: đặt tên,
  xử lý null, thiếu validation.
- Chọn một mảng để đào sâu: EF Core và hiệu năng truy vấn, bảo mật API, hoặc
  CI/CD.

## Thói quen nên có ngay từ đầu

- **Hỏi đúng cách.** Tự tìm khoảng 30 phút trước khi hỏi. Khi hỏi, nói rõ đã
  thử gì, lỗi gì, mong đợi gì.
- **Ghi chép.** Lệnh hay dùng, quy ước của dự án, lỗi từng gặp và cách sửa.
- **Commit nhỏ.** Mỗi commit một thay đổi có ý nghĩa, message nói rõ làm gì.
- **Báo sớm.** Task có nguy cơ trễ thì báo ngay, không đợi tới hạn chót.

## Tự đánh giá cuối năm

- [ ] Tự làm xong một tính năng từ yêu cầu tới khi lên production
- [ ] Viết unit test cho code của mình
- [ ] Review code cho đồng nghiệp
- [ ] Debug được lỗi trên môi trường test bằng log
- [ ] Giải thích được kiến trúc dự án cho người mới vào

## Nguồn

- [Microsoft Learn — C# coding conventions](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/coding-style/coding-conventions)
- [Google Engineering Practices — Code Review](https://google.github.io/eng-practices/review/)
- [roadmap.sh — Backend Developer Roadmap](https://roadmap.sh/backend)
