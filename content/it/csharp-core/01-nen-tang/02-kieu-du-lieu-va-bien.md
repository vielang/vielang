---
title: Kiểu dữ liệu và biến
minutes: 11
---

Hoá đơn in ra `0.6000000000000001` đồng.

Không ai gõ sai. Không ai làm tròn nhầm. Ba khoản 0,1 — 0,2 — 0,3 cộng lại,
máy trả về con số đó. Lỗi nằm ở một chữ trong dòng khai báo biến, viết từ sáu
tháng trước.

> **Học xong bài này bạn sẽ:** chọn đúng kiểu số cho tiền, cho phép đo, cho
> id; biết khi nào dùng `var`; và đọc được cảnh báo null của compiler thay vì
> tắt nó đi.
>
> **Cần biết trước:** chạy được một project console (bài trước).

## C# kiểm tra kiểu ngay lúc compile

Kiểu của biến chốt từ lúc build. Gán sai kiểu là báo lỗi ngay, không đợi tới
lúc chạy.

```csharp
int quantity = 5;
string name = "Huy";
quantity = "năm";   // lỗi compile, không phải runtime
```

Nghe thì phiền, nhưng đó là hàng rào đầu tiên: phần lớn bug kiểu dữ liệu
chết ngay trên máy bạn, trước khi kịp lên production.

## Thử ngay: bug tiền bạc kinh điển

Dán vào `Program.cs` rồi `dotnet run`:

```csharp
double a = 0.1, b = 0.2;
Console.WriteLine(a + b);
Console.WriteLine(a + b == 0.3);

decimal x = 0.1m, y = 0.2m;
Console.WriteLine(x + y);
Console.WriteLine(x + y == 0.3m);
```

**Đoán trước khi chạy:** `0.1 + 0.2` có bằng `0.3` không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0.30000000000000004
False
0.3
True
```

`double` lưu số thực dưới dạng nhị phân. Mà `0.1` không viết hết được trong hệ
nhị phân, y như `1/3` không viết hết được trong hệ thập phân. Máy đành lưu con
số gần đúng. Sai số cộng dồn, rồi ra hoá đơn.

`decimal` thì lưu theo hệ thập phân, nên tiền bạc ra đúng.

</details>

## Chọn kiểu số theo việc, không theo thói quen

Hoá đơn lệch một xu không phải lỗi của bạn. Nó là lỗi chọn kiểu, và bảng dưới
đây gỡ được hầu hết những lần chọn sai như thế.

| Việc | Kiểu | Vì sao |
|---|---|---|
| Tiền, tỉ lệ, đơn giá | `decimal` | chính xác theo hệ thập phân |
| Đo lường, toạ độ, tính khoa học | `double` | nhanh, đủ chính xác |
| Đếm, số lượng, id, chỉ số | `int` | tới hơn 2 tỉ |
| Id bảng lớn, mốc thời gian | `long` | `int` sẽ tràn |

Số `decimal` phải có hậu tố `m`, `float` thì `f`. Viết `decimal x = 0.1;` là
compiler chặn ngay, đòi bạn thêm `m` vào.

Chỗ nguy hiểm là khi không có kiểu nào viết ra: `var rate = 0.1;` cho bạn một
`double`, và bug vừa thấy quay lại y nguyên.

## var chỉ là cách viết gọn, kiểu vẫn cố định

Vừa nói tới `var rate = 0.1;`, nên phải nói rõ luôn `var` là gì. Nhiều người
tưởng nó giống JavaScript.

```csharp
var total = 10;             // compiler suy ra int
var name = "Huy";           // string
List<string> tags = new();  // target-typed new
```

`var` **không phải** kiểu động. Kiểu vẫn chốt lúc compile, bạn chỉ đỡ phải
gõ nó ra.

Dùng `var` khi kiểu đã hiện rõ ở vế phải. Còn `var result = Calculate(x);` thì
người đọc phải mở `Calculate` ra mới biết mình đang cầm cái gì. Viết kiểu ra
cho họ đỡ mất công.

## Value type chép giá trị, reference type chép tham chiếu

Mọi kiểu trong C# chia làm hai nhóm, và cái nhóm quyết định chuyện gì xảy ra
khi bạn gán biến này sang biến khác.

```csharp
int quantity = 5;       // value type — giá trị ở trong biến
string code = "AB-01";  // reference type — giữ tham chiếu
```

| Nhóm | Gồm | Gán là chép gì |
|---|---|---|
| Value type | `int`, `double`, `bool`, `decimal`, `DateTime`, `struct`, `enum` | chép **giá trị** |
| Reference type | `string`, `class`, mảng, `record class` | chép **tham chiếu** |

Khác biệt này đẻ ra cả một họ bug kiểu "sửa chỗ này sao chỗ kia đổi theo". Nó
có hẳn một bài riêng ở chương sau.

## Nullable reference types bắt lỗi null từ lúc build

Project .NET mới bật sẵn `<Nullable>enable</Nullable>`. Khi đó `string` nghĩa
là không bao giờ null. Muốn cho phép null thì viết `string?`.

```csharp
string? middleName = null;   // hợp lệ
string firstName = "Huy";    // không được null

int len = middleName.Length;       // cảnh báo CS8602
int safe = middleName?.Length ?? 0;
```

| Toán tử | Nghĩa |
|---|---|
| `?.` | gặp null thì dừng, trả về null |
| `??` | null thì lấy giá trị bên phải |
| `??=` | gán khi đang null |

Ba toán tử này thay cho những câu `if (x != null)` dài dòng. Nhưng nhớ: đây
chỉ là cảnh báo lúc compile. Runtime vẫn cho null lọt vào, nên dữ liệu từ JSON
hay database vẫn phải kiểm tra.

## Dấu hiệu trong code của bạn

- `double` hay `float` đứng cạnh `price`, `amount`, `total` → đổi sang `decimal`.
- `int.Parse` nhận dữ liệu từ request hay file → đổi sang `int.TryParse`, không thì một ký tự lạ là sập request.
- `var` mà nhìn vế phải không đoán ra kiểu → viết kiểu ra cho người đọc sau.
- Cảnh báo `CS8602` bị tắt hoặc dập bằng `!` → đó là `NullReferenceException` của tuần sau.

## Ghi nhớ

- `decimal` cho tiền, `double` cho đo lường. Số `decimal` phải có hậu tố `m`.
- `var` là cách viết gọn, không phải kiểu động.
- Value type chép **giá trị**, reference type chép **tham chiếu**.
- `int.Parse` ném exception, `int.TryParse` trả `false` — dữ liệu người dùng thì luôn `TryParse`.
- Nullable reference types chỉ cảnh báo lúc compile, runtime không chặn.

## Bước tiếp theo

Chọn được kiểu rồi, nhưng hai số khác kiểu gặp nhau thì sao?

Bài sau, **Toán tử và ép kiểu**, mở bằng một báo cáo hiện `0%` suốt cả tuần.
Công thức đúng, dữ liệu đúng, chỉ vì hai số đem chia đều là `int`.

```quiz
[
  {
    "prompt": "Hệ thống tính tiền đang dùng double và thỉnh thoảng hoá đơn lệch vài đồng. Sửa thế nào?",
    "options": [
      "Làm tròn kết quả bằng Math.Round ở chỗ hiển thị",
      "Nhân 100 rồi lưu bằng double",
      "Đổi sang float cho nhẹ hơn",
      "Đổi kiểu sang decimal cho mọi phép tính tiền"
    ],
    "answer": 4,
    "explain": "Làm tròn lúc hiển thị chỉ giấu sai số, cộng dồn qua nhiều phép vẫn lệch. decimal lưu theo hệ thập phân nên tiền ra đúng ngay từ phép tính."
  },
  {
    "prompt": "Người dùng gõ \"abc\" vào ô số lượng, code chạy int.Parse(input). Chuyện gì xảy ra?",
    "options": [
      "Trả về 0",
      "Ném FormatException, request lỗi 500",
      "Trả về null",
      "Compiler chặn từ lúc build"
    ],
    "answer": 2,
    "explain": "Parse ném exception khi chuỗi không phải số. Với dữ liệu từ bên ngoài, dùng int.TryParse để tự xử lý trường hợp sai."
  },
  {
    "prompt": "Dòng này có vấn đề gì?",
    "code": "var result = Calculate(input);",
    "options": [
      "Sai cú pháp, var phải đi với giá trị hằng",
      "Chậm hơn vì kiểu chỉ biết lúc chạy",
      "Không sai, nhưng người đọc phải mở Calculate mới biết kiểu",
      "var không dùng được với giá trị trả về của method"
    ],
    "answer": 3,
    "explain": "var vẫn là kiểu tĩnh, không chậm hơn. Vấn đề chỉ là người đọc: kiểu không hiện rõ ở vế phải thì nên viết kiểu ra."
  },
  {
    "prompt": "Project bật Nullable enable. Dòng string name = null; sẽ ra sao?",
    "options": [
      "Cảnh báo lúc compile, vẫn chạy được",
      "Lỗi compile, chương trình không build được",
      "Không sao cả, string vốn cho phép null",
      "Ném NullReferenceException lúc chạy"
    ],
    "answer": 1,
    "explain": "Nullable reference types chỉ sinh cảnh báo, trừ khi project bật TreatWarningsAsErrors. Runtime không chặn, nên dữ liệu từ JSON hay database vẫn có thể lọt null vào."
  }
]
```
