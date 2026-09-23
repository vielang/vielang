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
| **Phần tra cứu** | Định nghĩa một câu, BẢNG từ khoá đi kèm, BẢNG so sánh những cái dễ nhầm — đặt trước phần kể chuyện | ✔ |
| "Thử ngay" | Đoạn chạy được thật, kèm lệnh cụ thể | ✔ với bài có code |
| Câu hỏi dự đoán, kết quả giấu trong thẻ details | Đoán sai một lần nhớ lâu hơn đọc đúng mười lần — mà kết quả bày sẵn ngay dưới thì mắt đọc lướt qua là mất luôn cơ hội đoán | ✔ trong "Thử ngay" |
| Sơ đồ | Chỉ khi hình hơn hẳn chữ | tuỳ |
| Bẫy thực tế, cặp SAI/ĐÚNG | Người ta nhớ tương phản, không nhớ lời khuyên | ✔ |
| "Dấu hiệu trong code của bạn" | Biến bài học thành việc làm được ngay hôm nay | ✔ |
| "Ghi nhớ" | Chốt lại, đọc lướt được khi ôn | ✔ |
| "Bước tiếp theo" | Nối sang bài sau để mạch học không đứt | ✔ |
| "Tự kiểm tra" (khối ```quiz) | Nhớ lại chủ động, và chấm được | ✔ |

## Phần tra cứu gồm ba thứ

Đây **không** phải một mục tên là "Cốt lõi". Tiêu đề vẫn phải là câu khẳng định
như mọi tiêu đề khác (xem mục dưới) — quy tắc cũ đòi một cái nhãn "Cốt lõi" nên
nó tự đụng với quy tắc tiêu đề, và đã bỏ. Thứ bắt buộc là **nội dung**: mục đầu
tiên sau hộp mục tiêu phải tra cứu được.

1. **Định nghĩa một câu**, chính xác, không ẩn dụ. Đọc xong phải trả lời được "nó là gì".
2. **Bảng từ khoá**: mọi từ khoá C# mà bài dùng tới, kèm nghĩa ngắn. Không dùng từ khoá nào mà chưa có trong bảng — bài "đóng gói" từng dùng `public`/`private` 25 lần mà không định nghĩa chúng lần nào, còn `internal` thì cả 22 bài không hề nhắc tới.
3. **Bảng so sánh** những thứ dễ nhầm đặt cạnh nhau: `set` / `init` / `private set`, `readonly` / `const`, field / property, interface / abstract class.

Bảng là bắt buộc chứ không phải gạch đầu dòng: người học quay lại bài cũ là để TRA, mà tra thì bảng nhanh hơn đoạn văn.

Mục này đứng ngay sau hộp mục tiêu và TRƯỚC "Thử ngay". Kể chuyện trước khi định nghĩa thì người chưa biết gì sẽ trôi tuột.

Chỉ có **một** mục tóm tắt cuối bài. Từng có cả "Quy tắc thực dụng" lẫn "Ghi
nhớ" nằm cạnh nhau — hai mục làm cùng một việc, tốn gần một màn hình cuộn.

## Tiêu đề phải tự nói được nội dung

Mục lục đầu bài dựng tự động từ các thẻ `h2`, và **đọc riêng mục lục thì phải nắm được gần hết bài**. Vì vậy tiêu đề là một CÂU KHẲNG ĐỊNH, không phải một cái nhãn.

| Nhãn (tránh) | Câu khẳng định (dùng) |
|---|---|
| Cốt lõi | Đóng gói: dữ liệu và quy tắc nằm chung một chỗ |
| Access modifier | Access modifier quyết định ai nhìn thấy được gì |
| Đừng lộ collection | Bỏ `set` không khoá được nội dung collection |
| Anemic model | Anemic model: khi quy tắc rời khỏi class và tản đi khắp nơi |

Ba mục cuối giữ nguyên tên cố định vì chúng là cấu trúc chứ không phải nội dung: **Dấu hiệu trong code của bạn**, **Ghi nhớ**, **Bước tiếp theo**.

Dùng `h2` cho mọi mục đáng vào mục lục; `h3` chỉ dành cho phần phụ không cần tra tới. Số thứ tự do giao diện tự đánh — đừng gõ số vào tiêu đề, vì chèn thêm một mục là phải đánh số lại cả bài.

## Giọng văn

- Tiếng Việt giải thích, **thuật ngữ giữ tiếng Anh**: deferred execution, boxing, pattern matching… để người học đọc được tài liệu gốc và đi phỏng vấn.
- Nói với một người đang làm việc thật, không nói với lớp học. "Mở project đang làm và tìm bốn thứ này" chứ không phải "chúng ta hãy cùng tìm hiểu".
- **Mỗi câu một ý.** Câu quá 25 từ thì tách. Một đoạn chỉ nên có tối đa một dấu gạch ngang — nhiều mệnh đề nối nhau làm câu nặng dù từng chữ đều dễ.
- Ví dụ đơn giản nhất trước, biến thể sau. Đừng mở đầu bằng trường hợp đầy đủ nhất.
- Nêu hệ quả bằng con số khi có: "ba lần đi mạng, ba câu SQL" mạnh hơn "kém hiệu quả".
- Không đùa cho vui, không emoji. Sự cuốn hút đến từ tình huống thật và từ chỗ người học tự làm được, không đến từ giọng điệu.

## Viết cho liền mạch, đừng viết thành mẩu rời

Lỗi nặng nhất của bản nháp đầu tiên không phải là từ khó, mà là **kết cấu rời**: mỗi mục thả ra một định nghĩa, một bảng, vài gạch đầu dòng rồi hết. Đo trên bài "Đóng gói" bản cũ: 39 dòng danh sách so với 36 dòng văn xuôi, 19 đoạn văn mà phần lớn chỉ 1–2 câu, và không mục nào mở đầu bằng câu nối với mục trước.

Bốn quy tắc để bài đọc thành một mạch:

1. **Mỗi mục mở bằng một câu nối.** Câu đó phải dính vào mục vừa xong hoặc gọi lại sự cố ở đầu bài — "Biết ai nhìn thấy rồi thì tới câu hỏi thứ hai…". Mở bằng định nghĩa khô ("X là Y") làm người đọc rơi xuống từ trên trời.
2. **Đoạn văn 3–5 câu**, có dẫn, có triển khai, có chốt. Đoạn một câu đứng trơ là dấu hiệu bạn đang ghi chú chứ chưa viết.
3. **Bảng và danh sách không đứng trần.** Trước bảng phải có câu dẫn nói bảng này giải quyết gì, sau bảng phải có câu rút ra điều đáng nhớ nhất.
4. **Giữ một nhân vật xuyên suốt.** Sự cố mở bài và lớp ví dụ phải quay lại ở các mục sau, kể cả ở mục "Ghi nhớ" và "Bước tiếp theo". Bài có mạch khi người đọc thấy mình đang đi tới đâu đó.

Kiểm tra nhanh trước khi chốt một bài: đếm dòng văn xuôi so với dòng danh sách. Văn xuôi phải nhiều hơn hẳn.

## Nhịp văn: thước đo bắt buộc

Văn khó đọc hiếm khi vì từ khó. Nó khó vì mọi câu dài bằng nhau và cùng một khuôn: *mệnh đề chính — phẩy — "và" — mệnh đề phụ*. Người đọc không có chỗ thở, và không câu nào đánh mạnh.

Đo xong một bài, ba con số phải đạt:

| Chỉ số | Ngưỡng |
|---|---|
| Câu dài quá 22 từ | tối đa **3 câu** một bài |
| Câu ngắn dưới 10 từ | khoảng **40%** tổng số câu |
| Độ dài trung bình | **11–14 từ** |

Cách viết để đạt: **ý chốt luôn rơi vào câu ngắn**. Câu dài dùng để dựng bối cảnh, câu ngắn dùng để đóng đinh. "`public set` thì tiện thật. Tiện như trao chìa khoá nhà cho cả phố."

Vài thói quen nên bỏ:

- Nối hai ý bằng dấu phẩy và chữ "và" khi ý sau mới là ý chính. Tách ra thành câu riêng.
- Mở đoạn bằng "Nó", "Điều này", "Việc…" — danh từ hoá làm câu nặng mà chẳng thêm nghĩa.
- Giải thích lại điều người đọc vừa thấy trong code. Code đã nói rồi, câu văn nói phần code không nói được.

Một hình ảnh đời thường đắt hơn ba câu giải thích, nhưng mỗi mục chỉ nên có một. Nhiều quá thành ra làm dáng.

## Code trong bài

- **Tối đa 56 ký tự một dòng.** Đo trên màn 390px: quá ngưỡng này là điện thoại phải cuộn ngang mới đọc hết. `npm run check-csharp` đếm số dòng vi phạm theo từng file.
- Ngoại lệ: log và stack trace thật thì giữ nguyên, vì người học cần nhận ra đúng cái họ sẽ thấy.
- **Định danh viết bằng TIẾNG ANH**, theo đúng quy ước .NET: `Order`, `UnitPrice`, `ChangeQuantity`, `_items`. Người học phải gõ ra thứ giống code thật ở công ty và khớp với tài liệu gốc; định danh tiếng Việt (`DonHang`, `soLuong`) không giống bất kỳ dự án .NET nào.
- **Comment và văn xuôi vẫn tiếng Việt.** Tên thì tiếng Anh, giải thích thì tiếng Việt — đó cũng là cách nhiều nhóm ở Việt Nam làm thật.
- Tránh tên vô nghĩa (`a`, `tmp`, `x1`) và cũng tránh viết tắt lạ; `qty` thì được, `sl` thì không.
- **Một bài dùng MỘT bối cảnh xuyên suốt**, tối đa khoảng ba kiểu. Mỗi lần đổi ví dụ là người học phải nạp lại bối cảnh từ đầu — bài interface từng dựng tới 11 kiểu khác nhau và đó là lý do nó khó đọc.
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

## Bốn thứ chỉ máy bắt được

Bốn lỗi này viết tay không ai thấy, nên `check-prose` và `check-code` kiểm hộ.

1. **Mọi khối code phải biên dịch được.** Dán vào `Program.cs` là chạy, nghĩa là khai báo `class`/`record` đặt **sau** các câu lệnh top-level, không phải trước. `npm run check-code` biên dịch thật từng khối, kể cả code trong câu hỏi trắc nghiệm. Khối phản ví dụ thì ghi `// SAI` để công cụ biết lỗi là đúng ý — và nếu comment hứa "lỗi compile" thì khối buộc phải lỗi thật.
2. **Dấu `|` trong bảng phải escape thành `\|`, kể cả khi nằm trong backtick.** Markdown vẫn cắt ô, nên `` `Read|Write` `` làm mất nửa sau của ô — lỗi chỉ lộ ra trên web, đọc file không thấy.
3. **Không mục nào mở thẳng bằng code hay bảng.** Một câu bản lề trước đã, nối vào mục trên hoặc vào sự cố mở bài. Thiếu nó thì văn xuôi tụt xuống vai thuyết minh lại thứ người đọc vừa thấy.
4. **Đáp án trắc nghiệm phải rải đều 1–4.** Dồn về một vị trí là người học đoán được mà không cần đọc đề. Và câu hỏi không được chép lại code của thân bài.

Bài học rút ra từ lần kiểm trước: `check-prose` chỉ đo nhịp câu, nên nó báo
"22/22 đạt chuẩn" trong khi 30 khối code không biên dịch được, 80% đáp án nằm ở
vị trí 2, và một ô bảng mất chữ khi render. **Con số của công cụ chỉ nói được
về thứ nó đo.**

## Độ dài

- **Ngân sách chữ: 700–1200 từ văn xuôi một bài, nhắm 900–1100.** Đếm bằng cách bỏ code và bảng ra. `check-prose` chặn ở 700 và 1200; khoảng 900–1100 là chỗ một bài đủ chỗ cho câu bản lề giữa các mục mà chưa nói lại điều vừa nói. Ngưỡng trong `scripts/check-prose.ts` phải khớp đúng hai con số 700 và 1200 này — trước đây FORMAT ghi 900 mà công cụ chặn ở 700, nên "đạt chuẩn" không có nghĩa gì.
- 9–11 phút đọc, khoảng 8–10 màn hình trên điện thoại.
- Mỗi câu phải mang thông tin mới. Xoá thử một câu: người đọc vẫn nắm được thì câu đó thừa.
- Dài hơn thì tách bài, đừng cắt phần "Thử ngay" hay phần tự kiểm tra — đó là hai chỗ tạo ra việc học thật.
