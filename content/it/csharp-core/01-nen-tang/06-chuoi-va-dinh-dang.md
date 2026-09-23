---
title: Chuỗi và định dạng
minutes: 10
---

API của bạn gửi số tiền sang hệ thống đối tác dưới dạng chuỗi. Trên máy dev nó
gửi `1.5`, trên server Đức nó gửi `1,5`, và đối tác đọc thành mười lăm. Không
ai sửa code cả — chỉ là máy chủ có ngôn ngữ hệ thống khác.

> **Học xong bài này bạn sẽ:** định dạng số, tiền và ngày đúng ý; biết khi nào
> phải dùng `InvariantCulture`; và không còn nối chuỗi trong vòng lặp.
>
> **Cần biết trước:** biến, kiểu `string`, vòng lặp.

## String interpolation

```csharp
var ten = "Huy";
var tien = 1500000m;

var tin = $"Chào {ten}, đơn {tien:N0} VND đã ghi nhận.";
// Chào Huy, đơn 1,500,000 VND đã ghi nhận.
```

Dấu `$` bật **string interpolation**: chèn biểu thức thẳng vào chuỗi. Phần sau
dấu `:` là **format string** — `N0` là số có phân cách nghìn, không lấy phần lẻ.

| Viết | Kết quả | Dùng khi |
|---|---|---|
| `$"{gia:C}"` | `₫1,500,000` | tiền theo culture hiện tại |
| `$"{tiLe:P1}"` | `12.3%` | phần trăm |
| `$"{ngay:dd/MM/yyyy}"` | `23/09/2026` | ngày cho người Việt đọc |
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

`string` là **immutable**: mọi method của nó trả về chuỗi MỚI chứ không sửa
chuỗi cũ. Không gán lại thì kết quả bị vứt đi ngay.

</details>

Hệ quả thực tế: nối chuỗi trong vòng lặp là cái bẫy hiệu năng, vì mỗi lần `+=`
sinh một chuỗi mới rồi bỏ chuỗi cũ cho GC dọn.

```csharp
// SAI — 10.000 vòng là 10.000 chuỗi rác
var bao = "";
foreach (var x in items)
    bao += $"- {x.Name}\n";
```

```csharp
// ĐÚNG — một bộ đệm, ghi thêm vào
var sb = new StringBuilder();
foreach (var x in items)
    sb.AppendLine($"- {x.Name}");

var bao = sb.ToString();
```

Quy tắc thực dụng: dưới chục lần nối thì `+` hay interpolation đều ổn; nối
trong vòng lặp không biết trước số lần thì `StringBuilder`.

## Chuỗi rỗng và null

```csharp
string.IsNullOrEmpty(s);        // null hoặc ""
string.IsNullOrWhiteSpace(s);   // thêm: toàn dấu cách

var sach = (input ?? "").Trim();
var gop = string.Join(", ", tags);   // "a, b, c"
var phan = csv.Split(',');
```

Với dữ liệu người dùng nhập, `IsNullOrWhiteSpace` gần như luôn là lựa chọn
đúng — người ta gõ một dấu cách rồi bấm gửi nhiều hơn bạn tưởng.

## So sánh chuỗi

```csharp
a == b;   // so nội dung, phân biệt hoa thường

string.Equals(a, b,
    StringComparison.OrdinalIgnoreCase);

a.Contains("abc",
    StringComparison.OrdinalIgnoreCase);
```

`Ordinal` so theo mã ký tự — nhanh và không phụ thuộc ngôn ngữ máy chủ, đúng
cho mã đơn hàng, email, tên file. Chỉ so theo **culture** khi thật sự cần sắp
xếp chữ theo tiếng người đọc.

## Culture: cái bẫy của server

```csharp
// máy dev tiếng Việt: "1,5"
// server tiếng Anh:   "1.5"
var text = value.ToString();

// luôn như nhau ở mọi máy
var on = value.ToString(
    CultureInfo.InvariantCulture);
```

Chuỗi để **máy đọc** (ghi file, gọi API, sinh SQL, khoá cache) thì dùng
`InvariantCulture`. Chuỗi để **người đọc** mới theo culture của người dùng.
Nhầm hai thứ này sinh ra đúng loại bug chỉ xảy ra trên production.

## Raw string literal

```csharp
var json = """
    { "id": 1, "name": "Huy" }
    """;
```

Ba dấu nháy kép (C# 11) giữ nguyên nội dung bên trong, không cần escape — tiện
khi viết JSON, SQL hay HTML mẫu trong test.

## Dấu hiệu trong code của bạn

- `+=` trên `string` nằm trong vòng lặp → đổi sang `StringBuilder`.
- `ToString()` hoặc `decimal.Parse` không truyền culture, mà giá trị đó đi ra API, file hay SQL → thêm `InvariantCulture`.
- `==` so sánh mã đơn hàng, email, tên file → cân nhắc `OrdinalIgnoreCase`.
- Câu SQL được nối từ biến chuỗi → đó là **SQL injection**, phải chuyển sang tham số hoá.

## Ghi nhớ

- `string` là reference type nhưng **immutable**, và `==` so theo nội dung.
- Nối trong vòng lặp → `StringBuilder`.
- Chuỗi cho máy đọc dùng `InvariantCulture`; chuỗi cho người đọc theo culture người dùng.
- So sánh mã, email, tên file bằng `Ordinal`/`OrdinalIgnoreCase`.
- Không bao giờ dựng câu SQL bằng cách nối chuỗi từ dữ liệu người dùng.

## Bước tiếp theo

Hết chương **Nền tảng**. Chương sau — **Kiểu và bộ nhớ** — trả lời câu hỏi còn
treo từ bài Method: vì sao sửa object trong method thì bên ngoài thấy, và
value type khác reference type ở chỗ nào trong bộ nhớ.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "var s = \"abc\";\ns.ToUpper();\nConsole.WriteLine(s);",
    "options": ["ABC", "abc", "Chuỗi rỗng", "Lỗi compile"],
    "answer": 2,
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
      "Dùng value.ToString(CultureInfo.InvariantCulture)",
      "Nhân 100 rồi gửi số nguyên",
      "Thay dấu phẩy bằng dấu chấm sau khi ToString()"
    ],
    "answer": 2,
    "explain": "Chuỗi cho máy đọc phải độc lập với culture của máy chủ. Đổi cấu hình server chỉ giấu bug tới lần deploy sau."
  },
  {
    "prompt": "So sánh mã đơn hàng người dùng nhập với mã trong database, cần bỏ qua hoa thường. Cách đúng?",
    "options": [
      "a.ToLower() == b.ToLower()",
      "string.Equals(a, b, StringComparison.OrdinalIgnoreCase)",
      "a.Equals(b)",
      "a.CompareTo(b) == 0"
    ],
    "answer": 2,
    "explain": "OrdinalIgnoreCase so theo mã ký tự, không phụ thuộc culture và không cấp phát chuỗi mới như ToLower()."
  }
]
```
