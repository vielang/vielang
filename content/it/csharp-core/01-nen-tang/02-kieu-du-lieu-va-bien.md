---
title: Kiểu dữ liệu và biến
minutes: 10
---

Một hệ thống bán hàng cộng tiền ba đơn: 0,1 + 0,2 + 0,3. Kết quả in ra hoá đơn
là `0.6000000000000001`. Không ai gõ sai, không ai làm tròn nhầm — chỉ là kiểu
số đã chọn sai ngay từ dòng khai báo biến.

> **Học xong bài này bạn sẽ:** chọn đúng kiểu số cho từng việc, biết khi nào
> dùng `var`, và đọc được cảnh báo null của compiler thay vì tắt nó đi.
>
> **Cần biết trước:** chạy được một project console (bài trước).

## C# là ngôn ngữ statically typed

Kiểu của biến được xác định lúc compile. Gán sai kiểu là báo lỗi ngay, không
đợi tới lúc chạy — phần lớn bug kiểu dữ liệu bị chặn trước khi lên production.

```csharp
int soLuong = 5;
string ten = "Huy";
soLuong = "năm";   // lỗi compile, không phải runtime
```

## Thử ngay: bug tiền bạc kinh điển

Dán vào `Program.cs` rồi chạy `dotnet run`:

```csharp
double a = 0.1, b = 0.2;
Console.WriteLine(a + b);
Console.WriteLine(a + b == 0.3);

decimal x = 0.1m, y = 0.2m;
Console.WriteLine(x + y);
Console.WriteLine(x + y == 0.3m);
```

**Đoán trước khi chạy:** bốn dòng in ra gì? `0.1 + 0.2` có bằng `0.3` không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0.30000000000000004
False
0.3
True
```

`double` lưu số thực dưới dạng nhị phân, mà `0.1` không biểu diễn chính xác
được trong hệ nhị phân — y như `1/3` không viết hết được trong hệ thập phân.
`decimal` lưu theo hệ thập phân nên tiền bạc ra đúng.

</details>

## Chọn kiểu số

| Việc | Kiểu | Vì sao |
|---|---|---|
| Tiền, tỉ lệ phần trăm, số lượng hàng | `decimal` | chính xác theo hệ thập phân |
| Đo lường, toạ độ, tính khoa học | `double` | nhanh, đủ chính xác |
| Đếm, id, chỉ số | `int` | đủ tới hơn 2 tỉ |
| Id tự tăng của bảng lớn, mốc thời gian | `long` | `int` sẽ tràn |

Hậu tố `m` trong `0.1m` là bắt buộc với `decimal`, `f` cho `float`. Thiếu nó
thì compiler hiểu con số là `double`.

## var và kiểu tường minh

```csharp
var tong = 10;              // compiler suy ra int
var ten = "Huy";            // string
List<string> tags = new();  // target-typed new
```

`var` **không phải** kiểu động — kiểu vẫn cố định lúc compile, chỉ là bạn
không phải gõ ra. Dùng `var` khi kiểu đã hiện rõ ở vế phải; còn
`var ket = Tinh(x);` thì người đọc phải đi tra `Tinh` mới biết mình đang cầm
cái gì.

## Value type và reference type

```csharp
int a = 5;          // value type — giá trị ở trong biến
string s = "abc";   // reference type — giữ tham chiếu
```

- **Value type**: `int`, `double`, `bool`, `decimal`, `DateTime`, `struct`, `enum`. Gán là **chép giá trị**.
- **Reference type**: `string`, `class`, mảng, `record class`. Gán là **chép tham chiếu**, hai biến cùng trỏ một object.

Khác biệt này sinh ra cả một họ bug "sửa chỗ này sao chỗ kia đổi theo", nên có
hẳn một bài riêng ở chương sau.

## Nullable reference types

Project .NET mới bật sẵn `<Nullable>enable</Nullable>`. Khi đó `string` nghĩa
là **không bao giờ null**, muốn cho phép null phải viết `string?`:

```csharp
string? ten2 = null;        // hợp lệ
string ten1 = "Huy";        // không được null

int dai = ten2.Length;      // cảnh báo CS8602
int an = ten2?.Length ?? 0; // an toàn
```

`?.` là **null-conditional operator** (gặp null thì dừng, trả null), `??` là
**null-coalescing operator** (null thì lấy giá trị bên phải). Hai toán tử này
thay cho `if (x != null)` dài dòng.

## Dấu hiệu trong code của bạn

- `double` hay `float` đứng cạnh những cái tên như `price`, `amount`, `total` → đổi sang `decimal`.
- `int.Parse(Request.Query["page"])` → dữ liệu ngoài vào thì dùng `int.TryParse`, không thì một ký tự lạ là sập request.
- `var` mà nhìn vế phải không đoán ra kiểu → viết kiểu ra cho người đọc sau.
- Cảnh báo `CS8602` bị tắt hoặc dập bằng `!` → đó chính là `NullReferenceException` của tuần sau.

## Ghi nhớ

- `decimal` cho tiền, `double` cho đo lường. Số `decimal` phải có hậu tố `m`.
- `var` chỉ là cách viết gọn, kiểu vẫn cố định lúc compile.
- Value type chép **giá trị**, reference type chép **tham chiếu**.
- `int.Parse` ném exception khi chuỗi sai, `int.TryParse` trả `false` — dữ liệu người dùng thì luôn dùng `TryParse`.
- `const` cố định lúc compile, `readonly` gán một lần lúc chạy.

## Bước tiếp theo

Bài sau — **Toán tử và ép kiểu** — xem chuyện gì xảy ra khi hai số khác kiểu
gặp nhau, và vì sao `7 / 2` trong C# lại bằng `3`.

```quiz
[
  {
    "prompt": "Hệ thống tính tiền đang dùng double và thỉnh thoảng hoá đơn lệch vài đồng. Sửa thế nào?",
    "options": [
      "Làm tròn kết quả bằng Math.Round ở chỗ hiển thị",
      "Đổi kiểu sang decimal cho mọi phép tính tiền",
      "Đổi sang float cho nhẹ hơn",
      "Nhân 100 rồi lưu bằng double"
    ],
    "answer": 2,
    "explain": "Làm tròn lúc hiển thị chỉ giấu sai số, cộng dồn qua nhiều phép vẫn lệch. decimal lưu theo hệ thập phân nên tiền ra đúng ngay từ phép tính."
  },
  {
    "prompt": "Người dùng gõ \"abc\" vào ô số lượng, code chạy int.Parse(input). Chuyện gì xảy ra?",
    "options": [
      "Trả về 0",
      "Trả về null",
      "Ném FormatException, request lỗi 500",
      "Compiler chặn từ lúc build"
    ],
    "answer": 3,
    "explain": "Parse ném exception khi chuỗi không phải số. Với dữ liệu từ bên ngoài, dùng int.TryParse để tự xử lý trường hợp sai."
  },
  {
    "prompt": "Dòng này có vấn đề gì?",
    "code": "var ket = LayDuLieu();",
    "options": [
      "Sai cú pháp, var phải đi với giá trị hằng",
      "Chậm hơn vì kiểu chỉ biết lúc chạy",
      "Không sai, nhưng người đọc phải đi tra LayDuLieu mới biết kiểu",
      "var không dùng được với giá trị trả về của method"
    ],
    "answer": 3,
    "explain": "var vẫn là kiểu tĩnh, không chậm hơn. Vấn đề chỉ là người đọc: kiểu không hiện rõ ở vế phải thì nên viết kiểu ra."
  },
  {
    "prompt": "Project bật Nullable enable. Dòng string ten = null; sẽ ra sao?",
    "options": [
      "Lỗi compile, chương trình không build được",
      "Cảnh báo lúc compile, vẫn chạy được",
      "Không sao cả, string vốn cho phép null",
      "Ném NullReferenceException lúc chạy"
    ],
    "answer": 2,
    "explain": "Nullable reference types chỉ sinh cảnh báo (trừ khi project bật TreatWarningsAsErrors). Runtime không chặn, nên dữ liệu từ JSON hay database vẫn có thể lọt null vào."
  }
]
```
