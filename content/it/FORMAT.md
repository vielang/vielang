# Chuẩn viết bài học IT

Bài mẫu: `csharp-core/03-collection-va-linq/02-ienumerable-va-hoan-thuc-thi.md`.
Bài nào cũng viết theo khuôn dưới đây, không phải vì đồng phục cho đẹp mà vì
mỗi mục giải quyết một việc cụ thể trong lúc học.

File này không nằm trong thư mục khoá nào nên bước build bỏ qua nó.

## Khuôn một bài

| Mục | Vì sao có | Bắt buộc |
|---|---|---|
| Tình huống mở đầu (2–3 câu) | Cho người học lý do đọc tiếp, trước khi có định nghĩa | ✔ |
| Khối "Học xong bạn sẽ / Cần biết trước" | Nói rõ đích và điều kiện vào bài | ✔ |
| Khái niệm cốt lõi | Phần lý thuyết, giữ ngắn | ✔ |
| "Thử ngay" | Đoạn chạy được thật, kèm lệnh cụ thể | ✔ với bài có code |
| Câu hỏi dự đoán, kết quả giấu trong thẻ details | Đoán sai một lần nhớ lâu hơn đọc đúng mười lần — mà kết quả bày sẵn ngay dưới thì mắt đọc lướt qua là mất luôn cơ hội đoán | ✔ trong "Thử ngay" |
| Sơ đồ | Chỉ khi hình hơn hẳn chữ | tuỳ |
| Bẫy thực tế, cặp SAI/ĐÚNG | Người ta nhớ tương phản, không nhớ lời khuyên | ✔ |
| "Dấu hiệu trong code của bạn" | Biến bài học thành việc làm được ngay hôm nay | ✔ |
| "Ghi nhớ" | Chốt lại, đọc lướt được khi ôn | ✔ |
| "Bước tiếp theo" | Nối sang bài sau để mạch học không đứt | ✔ |
| "Tự kiểm tra" (khối ```quiz) | Nhớ lại chủ động, và chấm được | ✔ |

Chỉ có **một** mục tóm tắt cuối bài. Từng có cả "Quy tắc thực dụng" lẫn "Ghi
nhớ" nằm cạnh nhau — hai mục làm cùng một việc, tốn gần một màn hình cuộn.

## Giọng văn

- Tiếng Việt giải thích, **thuật ngữ giữ tiếng Anh**: deferred execution, boxing, pattern matching… để người học đọc được tài liệu gốc và đi phỏng vấn.
- Nói với một người đang làm việc thật, không nói với lớp học. "Mở project đang làm và tìm bốn thứ này" chứ không phải "chúng ta hãy cùng tìm hiểu".
- Nêu hệ quả bằng con số khi có: "ba lần đi mạng, ba câu SQL" mạnh hơn "kém hiệu quả".
- Không đùa cho vui, không emoji. Sự cuốn hút đến từ tình huống thật và từ chỗ người học tự làm được, không đến từ giọng điệu.

## Code trong bài

- **Tối đa 56 ký tự một dòng.** Đo trên màn 390px: quá ngưỡng này là điện thoại phải cuộn ngang mới đọc hết. `npm run check-csharp` đếm số dòng vi phạm theo từng file.
- Ngoại lệ: log và stack trace thật thì giữ nguyên, vì người học cần nhận ra đúng cái họ sẽ thấy.
- Đặt tên biến ngắn nhưng vẫn là tiếng Việt không dấu có nghĩa (`cho`, `donHang`), tránh `a`, `tmp`.
- Comment trong code nói **vì sao**, không nhắc lại code.
- Mọi khối C# phải qua `npm run check-csharp` (parse bằng Roslyn, bắt lỗi cú pháp).

## Sơ đồ

- Viết bằng khối ```` ```mermaid ````, chữ sau chữ "mermaid" là **chú thích** — hiện dưới sơ đồ và cũng là nhãn cho trình đọc màn hình.
- Chỉ vẽ khi hình hơn hẳn chữ: bố cục bộ nhớ, thứ tự thời gian, luồng rẽ nhánh. Đừng vẽ lại thứ vừa liệt kê bằng gạch đầu dòng.
- **Tối đa 3 nhánh ngang.** Rộng hơn thì trên điện thoại chữ bị co nhỏ khó đọc — chuyển sang `flowchart TD` xếp dọc.
- Hỗ trợ: flowchart, sequence, state, class, ER. Không có gantt/pie/mindmap.

## Câu tự kiểm tra

- 3–5 câu, viết bằng khối ```` ```quiz ````, `answer` đánh số từ 1.
- **Đặt tình huống mới**, đừng hỏi lại đúng ví dụ vừa trình bày — hỏi lại ví dụ chỉ đo trí nhớ ngắn hạn, không đo hiểu.
- Ưu tiên dạng "đọc đoạn code này, chuyện gì xảy ra" và "thấy lỗi này trong log, chỗ nào đáng ngờ".
- Phương án sai phải là **hiểu nhầm có thật**, không phải phương án cho có.
- `explain` viết cho người chọn sai: nói vì sao sai và sửa thế nào.
- Mục nào chiếm hẳn một phần trong bài thì phải có câu hỏi chạm tới.

## Độ dài

- 8–12 phút đọc, khoảng 6–9 màn hình trên điện thoại.
- Dài hơn thì tách bài, đừng cắt phần "Thử ngay" hay phần tự kiểm tra — đó là hai chỗ tạo ra việc học thật.
