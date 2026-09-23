---
title: class, struct hay record
minutes: 11
---

Hàm kiểm tra tiền thừa so hai số tiền bằng `==` và luôn trả về `false`, kể cả
khi hai bên rõ ràng cùng là 100.000 VND. Đổi sang `record` một dòng là hết —
vì `class` so sánh xem có **cùng một object** không, chứ không so nội dung.

> **Học xong bài này bạn sẽ:** chọn đúng giữa `class`, `record` và `struct` cho
> từng loại dữ liệu; biết khi nào `==` so nội dung và khi nào so danh tính;
> khai báo kiểu dữ liệu theo đúng thói quen .NET hiện đại.
>
> **Cần biết trước:** value type và reference type (bài trước).

## Thử ngay: hai object giống hệt nhau có bằng nhau không

```csharp
class TienC { public decimal So = 100; }
record TienR(decimal So);

var c1 = new TienC();
var c2 = new TienC();
var r1 = new TienR(100);
var r2 = new TienR(100);

Console.WriteLine($"{c1 == c2} và {r1 == r2}");
Console.WriteLine(r1);
```

**Đoán trước khi chạy:** hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
False và True
TienR { So = 100 }
```

`class` so sánh theo **danh tính**: hai object khác nhau thì `==` là `false`,
dù mọi field giống hệt. `record` so sánh theo **giá trị**, và `ToString()` cũng
được sinh sẵn nên in ra đọc được ngay — rất tiện khi ghi log.

</details>

## class — mặc định của mọi thứ có vòng đời

```csharp
public class Order
{
    public int Id { get; set; }
    public string KhachHang { get; set; } = "";
    public List<OrderItem> Items { get; set; } = new();

    public decimal Tong =>
        Items.Sum(i => i.Gia * i.SoLuong);
}
```

Reference type, có **identity**. Đúng cho những thứ có vòng đời và trạng thái:
đơn hàng, người dùng, service.

## record — dữ liệu, so sánh theo giá trị

```csharp
public record Money(decimal So, string DonVi);

var a = new Money(100, "VND");
var b = a with { So = 200 };   // bản sao, đổi 1 field
```

`record` vẫn là reference type (trừ `record struct`), nhưng compiler tự sinh
`Equals`, `GetHashCode`, `ToString` và toán tử `with`. Các property khai báo
kiểu rút gọn như trên là **init-only**: gán lúc khởi tạo rồi thôi.

Dùng `record` cho: DTO của API, message giữa các service, **value object**
trong domain (tiền, địa chỉ, khoảng thời gian), kết quả truy vấn.

Một lưu ý: `with` chỉ chép **nông**. List bên trong vẫn dùng chung — đúng cái
bẫy của bài trước.

## struct — nhỏ, bất biến, tạo rất nhiều

```csharp
public readonly record struct Diem(int X, int Y);
```

`struct` là value type: chép giá trị mỗi lần gán hay truyền đi. Chỉ dùng khi
thoả **đồng thời**: nhỏ (khoảng 16 byte trở xuống), immutable, và được tạo ra
rất nhiều. Ngoài các trường hợp đó, struct lớn còn chậm hơn class vì chép liên
tục.

## Property, field và init

```csharp
public class Product
{
    private readonly List<string> _tags = new();

    public required string Ten { get; init; }
    public decimal Gia { get; private set; }
    public IReadOnlyList<string> Tags => _tags;

    public void GiamGia(decimal phanTram) =>
        Gia -= Gia * phanTram / 100;
}

var p = new Product { Ten = "Bàn phím" };
```

- `required` — thiếu là **lỗi compile**, không phải lỗi lúc chạy.
- `init` — gán lúc khởi tạo rồi khoá lại.
- `private set` — đọc công khai, chỉ sửa được từ bên trong.
- Lộ ra ngoài bằng `IReadOnlyList<T>` để người khác không `Add` vào ruột object của bạn.

Public thì dùng **property**, không dùng field: property cho phép thêm kiểm
tra, đổi cách tính hay khoá quyền ghi mà không phá code đang gọi.

## Chọn cái nào

| Tình huống | Chọn |
|---|---|
| Thực thể có trạng thái, có id (Order, User, Service) | `class` |
| Dữ liệu chở đi, so sánh theo giá trị (DTO, value object) | `record` |
| Giá trị nhỏ, bất biến, tạo rất nhiều (Diem, Rgb) | `readonly record struct` |

## Dấu hiệu trong code của bạn

- DTO hoặc value object đang là `class`, và có ai đó tự viết `Equals`/`GetHashCode` bằng tay → đổi sang `record` là bỏ được cả đống code.
- So sánh hai object bằng `==` mà luôn ra `false` → đang so danh tính, không phải nội dung.
- `struct` có nhiều field hoặc có `set` → gần như luôn nên là `class` hoặc `record`.
- `public` field (không có `{ get; set; }`) → đổi sang property trước khi có người khác phụ thuộc vào nó.

## Ghi nhớ

- `class` so theo **tham chiếu**, `record` so theo **giá trị** — khác biệt hay bị hỏi nhất khi phỏng vấn.
- `with` chép nông: collection bên trong vẫn dùng chung.
- Mặc định cứ `class` hoặc `record`; chỉ chuyển sang `struct` khi đo đạc cho thấy cần.
- `required` bắt lỗi thiếu dữ liệu ngay lúc compile.

## Bước tiếp theo

Bài sau — **Làm việc với null** — xử lý cái giá trị vắng mặt đã sinh ra
exception phổ biến nhất lịch sử .NET, và cách để compiler bắt lỗi giúp bạn.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "record Tien(decimal So);\n\nvar a = new Tien(100);\nvar b = new Tien(100);\n\nConsole.WriteLine(a == b);",
    "options": ["True", "False", "Lỗi compile", "Tuỳ máy"],
    "answer": 1,
    "explain": "record so sánh theo giá trị: mọi field bằng nhau thì hai record bằng nhau. Nếu Tien là class thì kết quả sẽ là False."
  },
  {
    "prompt": "Bạn cần một kiểu chở dữ liệu từ API về: chỉ đọc, so sánh theo nội dung, ghi log dễ đọc. Chọn gì?",
    "options": ["class với property get/set", "record", "struct có set", "Dictionary<string, object>"],
    "answer": 2,
    "explain": "record sinh sẵn Equals, GetHashCode, ToString và with; property khai báo rút gọn là init-only nên dữ liệu không bị sửa lung tung."
  },
  {
    "prompt": "var b = a with { So = 200 }; — a có một List bên trong. b và a chia sẻ gì?",
    "options": [
      "Không chia sẻ gì, with chép sâu toàn bộ",
      "Chung chính List đó, vì with chỉ chép nông",
      "Chung mọi thứ, b chỉ là tên khác của a",
      "Lỗi compile vì record không dùng with với List"
    ],
    "answer": 2,
    "explain": "with tạo object mới và chép từng field. Field kiểu List là tham chiếu, nên bản sao trỏ đúng danh sách cũ."
  },
  {
    "prompt": "Khi nào struct là lựa chọn đúng?",
    "options": [
      "Khi kiểu đó có nhiều property và hay bị sửa",
      "Khi giá trị nhỏ, bất biến và được tạo ra rất nhiều",
      "Khi muốn truyền qua nhiều tầng mà không tốn bộ nhớ",
      "Khi cần so sánh theo giá trị"
    ],
    "answer": 2,
    "explain": "struct hợp với giá trị nhỏ và bất biến. Struct lớn còn chậm hơn class vì bị chép mỗi lần gán hay truyền đi; còn so sánh theo giá trị thì record đã làm được."
  }
]
```
