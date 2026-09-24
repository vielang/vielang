# Chuẩn viết bài học IT

Bài mẫu: `oop-thiet-ke/01-bon-tru-cot/02-ke-thua.md`.

File này không nằm trong thư mục khoá nào nên bước build bỏ qua nó.

## Tinh thần

Đơn giản, đi thẳng vào vấn đề, không lan man. Người học đọc một bài trong
khoảng 5 phút và làm được ngay một việc cụ thể.

- **Mỗi bài một khái niệm.** Thấy cần giải thích thêm khái niệm thứ hai thì
  tách thành bài khác.
- **Chỉ dạy phần cốt lõi.** Trường hợp hiếm, cú pháp nâng cao, lịch sử phiên
  bản: bỏ, hoặc nhiều nhất là một dòng trong "Tóm tắt".
- **Chưa dạy thì không dùng.** Code và lời giải thích chỉ dùng những gì đã có
  ở các bài trước (xem mục lục cuối file). Bài OOP không có `async`, bài
  `if` không có LINQ.
- Không kể chuyện dài, không ví von, không câu chốt làm dáng, không câu nối
  rỗng ("Đủ lý thuyết rồi", "Chúng ta hãy cùng tìm hiểu").
- Câu đầy đủ chủ ngữ, vị ngữ. Ngắn nhưng không cụt.

## Định nghĩa: một dòng, có emoji

Mỗi khái niệm được định nghĩa đúng một lần, trên một dòng riêng:

```
🧬 **Kế thừa (inheritance)**: class con nhận lại field và method của class cha.
```

- Mở đầu bằng **một** emoji gợi hình cho khái niệm.
- Thuật ngữ in đậm, kèm tên tiếng Anh (hoặc tiếng Việt) trong ngoặc.
- Sau dấu `:` là **một câu** nói nó là gì. Không ẩn dụ, không "giống như".
- Emoji chỉ dùng ở dòng định nghĩa, không rải khắp bài.

`check-prose` nhận định nghĩa theo đúng mẫu `**...**:` này.

## Khuôn một bài

Các mục `h2` có tên cố định, theo đúng thứ tự:

| Mục | Nội dung | Bắt buộc |
|---|---|---|
| (không tiêu đề) | 2–3 câu: bài này giải quyết vấn đề gì | ✔ |
| `## Khái niệm` | các dòng định nghĩa, mỗi dòng kèm 1–2 câu giải thích nếu cần | ✔ |
| `## Ví dụ` | đoạn code nhỏ nhất thể hiện khái niệm, vài gạch đầu dòng giải thích | ✔ |
| `## Thử ngay` | sửa tiếp code của "Ví dụ" (không chép lại class), câu hỏi đoán kết quả, kết quả giấu trong `<details>` | ✔ |
| `## Lỗi hay gặp` | 1–2 lỗi, mỗi lỗi một cặp `// SAI` / `// ĐÚNG` | ✔ |
| `## Tóm tắt` | 3–5 gạch đầu dòng | ✔ |
| khối ```` ```quiz ```` | 3 câu tự kiểm tra | ✔ |

Được thêm tối đa **hai** mục `h2` riêng (đặt giữa "Ví dụ" và "Thử ngay") khi
khái niệm có một phần phụ quan trọng, ví dụ `## Từ khoá base`. Tên mục ngắn,
nói đúng nội dung.

Không có mục "Học xong bạn sẽ", "Bước tiếp theo", "Ghi nhớ": mục lục và "Tóm
tắt" đã làm việc đó.

## Độ dài

- **200–700 chữ văn xuôi** (không tính code và bảng). `check-prose` chặn hai
  ngưỡng này.
- Câu dài quá 40 âm tiết thì tách. Mỗi bài tối đa 2 câu như vậy.
- Mỗi đoạn văn 1–3 câu.

## Code

- Mọi khối ```` ```csharp ```` phải biên dịch được: `npm run check-code`.
  Dán vào `Program.cs` là chạy, nên câu lệnh top-level đứng **trước** khai báo
  `class`.
- Khối phản ví dụ ghi `// SAI` ở dòng đầu để công cụ biết lỗi là cố ý.
- Tối đa 56 ký tự mỗi dòng (vừa màn điện thoại 390px).
- Định danh tiếng Anh theo quy ước .NET (`Order`, `UnitPrice`). Comment và
  văn xuôi tiếng Việt.
- Một bài dùng **một** bối cảnh ví dụ xuyên suốt, tối đa ba class.
- Ví dụ lấy từ bối cảnh bán hàng quen thuộc: `Product`, `Order`, `Customer`.

## Quiz

- 3 câu, khối ```` ```quiz ````, `answer` đánh số từ 1.
- Tình huống mới, không hỏi lại đúng ví dụ trong bài.
- Phương án sai phải là hiểu nhầm có thật.
- `explain` một hai câu: vì sao đáp án đúng.
- Đáp án đúng không dồn vào một vị trí.

## Kỹ thuật

- Dấu `|` trong bảng phải viết `\|`, kể cả trong backtick.
- Sơ đồ dùng ```` ```mermaid Chú thích ````, chỉ khi hình rõ hơn chữ. Tối đa 3
  nhánh ngang.
- Thuật ngữ giữ tiếng Anh khi đó là từ dân IT dùng hằng ngày (class, method,
  interface), kèm tiếng Việt trong định nghĩa.

## Kiểm tra trước khi commit

```bash
npm run check-prose   # khuôn bài, độ dài, định nghĩa, quiz
npm run check-code    # biên dịch mọi khối C#
npm run check-sql     # chạy thật khối SQL trên Oracle (cần Docker)
npm run build-content # dựng courses.json
npx tsc --noEmit      # next build kiểm kiểu cả scripts/*.mts
```

## Mục lục

Thứ tự bài là thứ tự dạy. Một bài chỉ được dùng khái niệm của các bài đứng
trước nó.

### C# Core (`csharp-core`)

| Chương | Bài |
|---|---|
| `01-bat-dau` | `01-chuong-trinh-dau-tien` · `02-bien-va-kieu-du-lieu` · `03-toan-tu-va-ep-kieu` · `04-if-else-va-switch` · `05-vong-lap` (`for`, `while`) · `06-method` |
| `02-du-lieu` | `01-string` · `02-array-va-list` (kèm `foreach`) · `03-dictionary` |
| `03-class-co-ban` | `01-class-va-object` · `02-property-va-constructor` · `03-value-type-va-reference-type` · `04-null-va-nullable` · `05-static` · `06-enum` |
| `04-cong-cu-hang-ngay` | `01-lambda` · `02-linq-co-ban` · `03-exception` · `04-async-await-co-ban` |

### OOP (`oop-thiet-ke`)

| Chương | Bài |
|---|---|
| `01-bon-tru-cot` | `01-dong-goi` · `02-ke-thua` · `03-da-hinh` · `04-abstract-class` |
| `02-interface-va-composition` | `01-interface` · `02-interface-hay-abstract-class` · `03-composition` |
| `03-solid` | `01-srp` · `02-ocp` · `03-lsp` · `04-isp` · `05-dip-va-dependency-injection` |

### ASP.NET Core Web API (`aspnet-core`)

Dùng controller, không dùng minimal API. Database là SQLite qua EF Core.

| Chương | Bài |
|---|---|
| `01-nen-tang-web` | `01-http-va-rest` · `02-tao-web-api-dau-tien` · `03-controller-va-routing` |
| `02-xay-api-crud` | `01-nhan-du-lieu-tu-request` · `02-tra-ve-ket-qua` · `03-dto` · `04-validation` · `05-crud-hoan-chinh` |
| `03-cau-truc-ung-dung` | `01-dependency-injection-trong-aspnet` · `02-cau-hinh-appsettings` · `03-logging` · `04-middleware-va-pipeline` |
| `04-du-lieu-voi-ef-core` | `01-ef-core-va-dbcontext` · `02-migration` · `03-truy-van-voi-ef-core` · `04-quan-he-mot-nhieu` |
| `05-hoan-thien-api` | `01-xu-ly-loi-tap-trung` · `02-xac-thuc-jwt` · `03-test-api` |

Riêng bài web:

- Project tạo bằng `dotnet new webapi --use-controllers -o ShopApi`, chạy
  bằng `dotnet run --urls http://localhost:5000` để mọi bài cùng một địa chỉ.
- "Thử ngay" là chạy server rồi gọi API bằng `curl -i`. Câu đoán hỏi về
  status code hoặc JSON trả về. Kết quả trong `<details>` là response.
- Request có body: lưu JSON vào file rồi gửi bằng `-d @ten-file.json`, kèm
  `-H "Content-Type: application/json"`. Viết JSON thẳng trong lệnh dễ vỡ
  dấu nháy khác nhau giữa PowerShell, cmd và bash.
- Request, response mẫu viết trong khối ```` ```http ````.
- Code C# ghi rõ `using Microsoft.AspNetCore.Mvc;`,
  `using Microsoft.EntityFrameworkCore;` vì template không tự thêm.
  `check-code` biên dịch bài khoá này như một project web thật.

### SQL với Oracle (`sql`)

| Chương | Bài |
|---|---|
| `01-doc-du-lieu` | `01-database-bang-va-khoa` · `02-select-va-where` · `03-sap-xep-va-phan-trang` · `04-null-trong-sql` |
| `02-tong-hop` | `01-ham-tong-hop` · `02-group-by-va-having` |
| `03-noi-bang` | `01-khoa-ngoai-va-quan-he` · `02-inner-join` · `03-left-join` · `04-subquery` |
| `04-thay-doi-du-lieu` | `01-insert-update-delete` · `02-create-table-va-rang-buoc` · `03-transaction` |
| `05-hieu-nang-va-an-toan` | `01-index` · `02-chuan-hoa` · `03-sql-injection` |

Riêng bài SQL:

- Database là **Oracle**. Chỉ dùng cú pháp chạy được từ Oracle 19c trở lên
  (`FETCH FIRST`, `GENERATED AS IDENTITY`), không dùng tính năng chỉ có ở
  23ai như kiểu `BOOLEAN` hay `SELECT` không có `FROM`.
- Người học chạy SQL trên **Oracle Live SQL** (livesql.oracle.com), không cần
  cài gì.
- Database mẫu gồm bốn bảng `customers`, `products`, `orders`,
  `order_lines`. Script tạo bảng và dữ liệu nằm **duy nhất** ở bài
  `01-database-bang-va-khoa`, trong khối ```` ```sql setup ````. Mọi bài sau
  dùng đúng dữ liệu đó.
- Tên bảng, tên cột viết thường, nối bằng dấu gạch dưới: `customer_id`.
- "Thử ngay" là một khối ```` ```sql ```` chạy trên dữ liệu mẫu. Kết quả trong
  `<details>` viết thành bảng Markdown, tên cột viết hoa như Oracle trả về.
  `check-sql` chạy thật câu SQL và so với bảng này.
- Khối phản ví dụ ghi `-- SAI` ở dòng đầu. Nếu comment nói "lỗi" thì câu
  lệnh buộc phải báo lỗi khi chạy.

### Cú pháp chưa dạy thì chưa dùng

- Method viết đầy đủ `{ return ...; }`. Dạng gọn `=> ...` chỉ dùng sau bài
  `01-lambda`.
- Viết `new Address()`, không viết tắt `new()`.
