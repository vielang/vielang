---
title: Chuỗi và định dạng
minutes: 8
---

Backend nào cũng nối chuỗi suốt ngày: dựng log, dựng truy vấn, dựng thông báo.
Vài chi tiết nhỏ ở đây ảnh hưởng thẳng tới hiệu năng và tới bug ngày/giờ.

## String interpolation

```csharp
var name = "Huy";
var total = 1500000m;

var message = $"Xin chào {name}, đơn hàng {total:N0} VND đã được ghi nhận.";
// Xin chào Huy, đơn hàng 1,500,000 VND đã được ghi nhận.
```

Dấu `$` bật **string interpolation**: chèn biểu thức thẳng vào chuỗi. Phần sau
dấu `:` là **format string** — `N0` là số có phân cách nghìn, không lấy thập phân.

| Viết | Kết quả | Dùng khi |
|---|---|---|
| `$"{price:C}"` | `₫1,500,000` | tiền theo culture hiện tại |
| `$"{ratio:P1}"` | `12.3%` | phần trăm |
| `$"{date:dd/MM/yyyy}"` | `23/09/2026` | ngày cho người Việt đọc |
| `$"{id:D6}"` | `000042` | đệm số 0 cho đủ 6 chữ số |

## Chuỗi là immutable

```csharp
var s = "abc";
s.ToUpper();          // KHÔNG đổi s — trả về chuỗi mới
s = s.ToUpper();      // phải gán lại
```

Mọi method của `string` đều trả về chuỗi mới. Vì vậy nối chuỗi trong vòng lặp là
cái bẫy hiệu năng: mỗi lần `+=` sinh một chuỗi mới rồi vứt chuỗi cũ đi.

```csharp
var sb = new StringBuilder();
foreach (var item in items)
    sb.AppendLine($"- {item.Name}");

var report = sb.ToString();
```

Quy tắc thực dụng: dưới chục lần nối thì `+` hay interpolation đều ổn; nối trong
vòng lặp không biết trước số lần thì dùng `StringBuilder`.

## Chuỗi rỗng và null

```csharp
string.IsNullOrEmpty(s);        // null hoặc ""
string.IsNullOrWhiteSpace(s);   // null, "" hoặc chỉ toàn khoảng trắng

var clean = (input ?? "").Trim();
var joined = string.Join(", ", tags);       // "a, b, c"
var parts  = csv.Split(',');                // tách thành mảng
```

Với dữ liệu người dùng nhập, `IsNullOrWhiteSpace` gần như luôn là lựa chọn đúng —
người ta gõ một dấu cách rồi bấm gửi nhiều hơn bạn tưởng.

## So sánh chuỗi

```csharp
a == b;                                                   // so sánh nội dung, phân biệt hoa thường
string.Equals(a, b, StringComparison.OrdinalIgnoreCase);  // bỏ qua hoa thường
a.Contains("abc", StringComparison.OrdinalIgnoreCase);
```

`Ordinal` so sánh theo mã ký tự — nhanh và không phụ thuộc ngôn ngữ máy chủ, là
lựa chọn đúng cho mã đơn hàng, email, tên file. Chỉ dùng so sánh theo **culture**
khi thật sự cần sắp xếp chữ theo tiếng người đọc.

## Culture: cái bẫy của server

```csharp
// Máy dev tiếng Việt: "1,5"  — server tiếng Anh: "1.5"
var text = value.ToString();

// Luôn ra như nhau ở mọi máy
var stable = value.ToString(CultureInfo.InvariantCulture);
```

Chuỗi để **máy đọc** (ghi file, gọi API, sinh SQL) thì dùng `InvariantCulture`.
Chuỗi để **người đọc** mới theo culture của người dùng. Nhầm hai thứ này là nguồn
của những bug chỉ xảy ra trên production.

## Raw string literal

```csharp
var json = """
    { "id": 1, "name": "Huy" }
    """;
```

Ba dấu nháy kép (C# 11) giữ nguyên nội dung bên trong, không cần escape — rất
tiện khi viết JSON, SQL hay HTML mẫu trong test.

## Ghi nhớ

- `string` là reference type nhưng **so sánh bằng `==` theo nội dung**, khác với class thông thường.
- Nối chuỗi trong vòng lặp → `StringBuilder`.
- Không bao giờ dựng câu SQL bằng cách nối chuỗi từ dữ liệu người dùng — đó là lỗ hổng **SQL injection**; dùng tham số hoá.
