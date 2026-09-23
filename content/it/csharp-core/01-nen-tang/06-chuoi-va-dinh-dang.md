---
title: Chuỗi và định dạng
minutes: 11
---

API của bạn gửi số tiền sang hệ thống đối tác dưới dạng chuỗi.

Máy dev gửi `1.5`. Server ở Đức gửi `1,5`. Đối tác đọc thành mười lăm, và trừ
tiền khách gấp mười lần. Không ai sửa một dòng code nào — chỉ là hai máy có
ngôn ngữ hệ thống khác nhau.

> **Học xong bài này bạn sẽ:** định dạng số, tiền và ngày đúng ý; biết khi nào
> phải dùng `InvariantCulture`; và không còn nối chuỗi trong vòng lặp.
>
> **Cần biết trước:** biến, kiểu `string`, vòng lặp.

## String interpolation và bảng định dạng

```csharp
var name = "Huy";
var amount = 1500000m;

var message =
    $"Chào {name}, đơn {amount:N0} VND đã ghi nhận.";
```

Dấu `$` bật **string interpolation**: chèn biểu thức thẳng vào chuỗi. Phần sau
dấu `:` là **format string**.

| Viết | Kết quả | Dùng khi |
|---|---|---|
| `$"{x:N0}"` | `1,500,000` | số có phân cách nghìn |
| `$"{x:C}"` | `1.500.000 ₫` trên máy `vi-VN` | tiền theo culture hiện tại — đổi máy là đổi kết quả |
| `$"{x:P1}"` | `12.3%` | phần trăm |
| `$"{d:dd/MM/yyyy}"` | `23/09/2026` | ngày cho người Việt đọc |
| `$"{id:D6}"` | `000042` | đệm số 0 cho đủ 6 chữ số |

## Thử ngay: chuỗi không đổi được

```csharp
var s = "abc";

s.ToUpper();
Console.WriteLine(s);

s = s.ToUpper();
Console.WriteLine(s);
```

**Đoán trước khi chạy:** hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
abc
ABC
```

`string` là **immutable**. Mọi method của nó trả về chuỗi mới chứ không sửa
chuỗi cũ. Không gán lại thì kết quả bị vứt đi ngay.

</details>

## Nối trong vòng lặp thì dùng StringBuilder

Chuỗi bất biến kéo theo một hệ quả về hiệu năng. Mỗi lần `+=` là một chuỗi mới
được cấp phát, còn chuỗi cũ bỏ lại cho GC.

```csharp
// SAI — 10.000 vòng là 10.000 chuỗi rác
var report = "";
foreach (var item in items)
    report += $"- {item.Name}\n";
```

```csharp
// ĐÚNG — một bộ đệm, chỉ ghi thêm vào
var sb = new StringBuilder();
foreach (var item in items)
    sb.AppendLine($"- {item.Name}");

var report2 = sb.ToString();
```

Quy tắc thực dụng: dưới chục lần nối thì `+` hay interpolation đều ổn. Nối
trong vòng lặp không biết trước số lần thì `StringBuilder`.

## Kiểm tra rỗng và so sánh

```csharp
string.IsNullOrEmpty(s);        // null hoặc ""
string.IsNullOrWhiteSpace(s);   // thêm: toàn dấu cách

var joined = string.Join(", ", tags);   // "a, b, c"
var parts = csv.Split(',');
```

Với dữ liệu người dùng nhập, `IsNullOrWhiteSpace` gần như luôn đúng. Người ta
gõ một dấu cách rồi bấm gửi nhiều hơn bạn tưởng.

| Cách so sánh | Đặc điểm |
|---|---|
| `a == b` | so nội dung, **phân biệt** hoa thường |
| `StringComparison.Ordinal` | so theo mã ký tự, không phụ thuộc culture |
| `OrdinalIgnoreCase` | như trên, bỏ qua hoa thường |
| So theo culture | chỉ khi cần sắp xếp chữ cho người đọc |

```csharp
string.Equals(a, b,
    StringComparison.OrdinalIgnoreCase);
```

Mã đơn hàng, email, tên file đều nên so bằng `Ordinal`. Chúng là dữ liệu máy,
không phải tiếng người.

## Culture: cái bẫy chỉ lộ ra trên production

```csharp
using System.Globalization;

// máy dev tiếng Anh: "1.5"
// server ở Đức:      "1,5"
var text = value.ToString();

// luôn như nhau ở mọi máy
var stable = value.ToString(
    CultureInfo.InvariantCulture);
```

Chuỗi có hai loại người đọc. Bạn phải biết mình đang viết cho ai.

| Chuỗi để | Dùng |
|---|---|
| Máy đọc: file, API, SQL, khoá cache | `InvariantCulture` |
| Người đọc: màn hình, email, hoá đơn | culture của người dùng |

Nhầm hai loại này sinh ra đúng cái bug ở đầu bài. Nó chỉ xuất hiện sau khi
deploy, trên máy có ngôn ngữ khác máy bạn.

## Raw string literal giữ nguyên mọi ký tự bên trong

```csharp
var json = """
    { "id": 1, "name": "Huy" }
    """;
```

Ba dấu nháy kép (C# 11) giữ nguyên nội dung bên trong, không cần escape. Rất
tiện khi viết JSON, SQL hay HTML mẫu trong test.

Trước đây muốn viết một chuỗi JSON là phải escape từng dấu nháy, đọc rối mắt
và sửa thì dễ sai. Thụt lề của khối cũng được cắt theo dấu nháy đóng, nên chuỗi
không dính thêm khoảng trắng thừa.

## Dấu hiệu trong code của bạn

- `+=` trên `string` nằm trong vòng lặp → đổi sang `StringBuilder`.
- `ToString()` hay `decimal.Parse` không truyền culture, mà giá trị đó đi ra API, file hoặc SQL → thêm `InvariantCulture`.
- `==` so sánh mã đơn hàng, email, tên file → cân nhắc `OrdinalIgnoreCase`.
- Câu SQL nối từ biến chuỗi → đó là **SQL injection**, phải chuyển sang tham số hoá.

## Ghi nhớ

- `string` là reference type nhưng **immutable**, và `==` so theo nội dung.
- Nối trong vòng lặp thì dùng `StringBuilder`.
- Chuỗi cho máy đọc dùng `InvariantCulture`, cho người đọc theo culture người dùng.
- So mã, email, tên file bằng `Ordinal` hoặc `OrdinalIgnoreCase`.
- Không bao giờ dựng câu SQL bằng cách nối chuỗi.

## Bước tiếp theo

Hết chương **Nền tảng**. Bạn đã viết được code chạy đúng.

Chương sau, **Kiểu và bộ nhớ**, trả lời câu hỏi còn treo từ bài Method: vì sao
sửa object trong method thì bên ngoài thấy được.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "var s = \"abc\";\ns.ToUpper();\nConsole.WriteLine(s);",
    "options": [
      "ABC",
      "Lỗi compile",
      "Chuỗi rỗng",
      "abc"
    ],
    "answer": 4,
    "explain": "string là immutable: ToUpper() trả về chuỗi mới, không sửa s. Phải gán lại s = s.ToUpper()."
  },
  {
    "prompt": "Job xuất báo cáo nối 50.000 dòng vào một biến string bằng +=. Vấn đề là gì?",
    "options": [
      "Không có vấn đề, C# tối ưu sẵn",
      "Mỗi lần += sinh một chuỗi mới, tốn bộ nhớ và thời gian",
      "Chuỗi bị giới hạn độ dài nên sẽ mất dữ liệu",
      "Kết quả sai thứ tự dòng"
    ],
    "answer": 2,
    "explain": "Chuỗi bất biến nên mỗi phép nối chép lại toàn bộ nội dung cũ. StringBuilder giữ một bộ đệm và chỉ ghi thêm."
  },
  {
    "prompt": "Số tiền 1.5 gửi sang API đối tác. Trên server Đức nó thành \"1,5\". Sửa thế nào?",
    "options": [
      "Đổi ngôn ngữ hệ thống của server về tiếng Anh",
      "Nhân 100 rồi gửi số nguyên",
      "Dùng value.ToString(CultureInfo.InvariantCulture)",
      "Thay dấu phẩy bằng dấu chấm sau khi ToString()"
    ],
    "answer": 3,
    "explain": "Chuỗi cho máy đọc phải độc lập với culture của máy chủ. Đổi cấu hình server chỉ giấu bug tới lần deploy sau."
  },
  {
    "prompt": "So sánh mã đơn hàng người dùng nhập với mã trong database, cần bỏ qua hoa thường. Cách đúng?",
    "options": [
      "string.Equals(a, b, StringComparison.OrdinalIgnoreCase)",
      "a.ToLower() == b.ToLower()",
      "a.Equals(b)",
      "a.CompareTo(b) == 0"
    ],
    "answer": 1,
    "explain": "OrdinalIgnoreCase so theo mã ký tự, không phụ thuộc culture và không cấp phát chuỗi mới như ToLower()."
  }
]
```
