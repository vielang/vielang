---
title: class, struct hay record
minutes: 11
---

Hàm kiểm tra tiền thừa so hai số tiền bằng `==`, và luôn trả về `false`.

Kể cả khi hai bên rõ ràng cùng là 100.000 đồng. Đổi một chữ `class` thành
`record` là hết. Vì `class` so xem có **cùng một object** không, chứ không so
nội dung.

> **Học xong bài này bạn sẽ:** chọn đúng giữa `class`, `record` và `struct`;
> biết khi nào `==` so nội dung và khi nào so danh tính; khai báo kiểu theo
> đúng thói quen .NET hiện đại.
>
> **Cần biết trước:** value type và reference type (bài trước).

## Thử ngay: hai object giống hệt nhau có bằng nhau không

```csharp
var c1 = new MoneyC();
var c2 = new MoneyC();
var r1 = new MoneyR(100);
var r2 = new MoneyR(100);

Console.WriteLine($"{c1 == c2} và {r1 == r2}");
Console.WriteLine(r1);

class MoneyC { public decimal Amount = 100; }

record MoneyR(decimal Amount);
```

**Đoán trước khi chạy:** hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
False và True
MoneyR { Amount = 100 }
```

`class` so theo **danh tính**. Hai object khác nhau thì `==` là `false`, dù
mọi field giống hệt.

`record` so theo **giá trị**. Nó còn sinh sẵn `ToString()` nên in ra đọc được
ngay, rất tiện khi ghi log.

</details>

## Ba lựa chọn, ba loại dữ liệu

| Kiểu | So sánh theo | Dùng cho |
|---|---|---|
| `class` | danh tính (tham chiếu) | thực thể có trạng thái, có id |
| `record` | giá trị | DTO, value object, message |
| `readonly record struct` | giá trị, là value type | giá trị nhỏ, bất biến, tạo rất nhiều |

Phép thử nhanh: bỏ `Id` đi thì hai object còn phân biệt được với nhau không?

Đơn hàng thì còn. Hai đơn trùng từng ô dữ liệu vẫn là hai đơn khác nhau, và ai
cũng sẽ hỏi "đơn nào". Nên `class`.

Còn 100.000 đồng và 100.000 đồng thì không ai hỏi "số tiền nào". Nên `record`.

## class cho thứ có vòng đời

```csharp
public class Order
{
    public int Id { get; set; }
    public string Customer { get; set; } = "";
    public List<OrderLine> Lines { get; set; } = new();

    public decimal Total =>
        Lines.Sum(l => l.Price * l.Quantity);
}
```

Reference type, có **identity**. Đúng cho đơn hàng, người dùng, service — thứ
tồn tại theo thời gian và đổi trạng thái.

## record cho dữ liệu chở đi

```csharp
public record Money(decimal Amount, string Currency);

var a = new Money(100, "VND");
// bản sao, chỉ đổi một field
var b = a with { Amount = 200 };
```

Compiler tự sinh `Equals`, `GetHashCode`, `ToString` và toán tử `with`. Các
property khai báo kiểu rút gọn như trên là **init-only**: gán lúc khởi tạo rồi
khoá.

Dùng `record` cho DTO của API, message giữa các service, value object trong
domain, kết quả truy vấn.

Một lưu ý: `with` chỉ chép **nông**. List bên trong vẫn dùng chung, đúng cái
bẫy của bài trước.

## struct chỉ hợp khi nhỏ và bất biến

```csharp
public readonly record struct Point(int X, int Y);
```

`struct` là value type, nên chép giá trị mỗi lần gán hay truyền đi. Chỉ dùng
khi thoả **đồng thời** ba điều: nhỏ (khoảng 16 byte trở xuống), immutable, và
được tạo ra rất nhiều.

Ngoài các trường hợp đó, struct lớn còn chậm hơn class vì bị chép liên tục.
Mặc định cứ `class` hoặc `record`, chỉ chuyển sang `struct` khi đo đạc cho
thấy cần.

## required, init và private set

```csharp
public class Product
{
    private readonly List<string> _tags = new();

    public required string Name { get; init; }
    public decimal Price { get; private set; }
    public IReadOnlyList<string> Tags => _tags;

    public void ApplyDiscount(decimal percent)
        => Price -= Price * percent / 100;
}

var p = new Product { Name = "Bàn phím" };
```

| Khai báo | Tác dụng |
|---|---|
| `required` | thiếu là **lỗi compile**, không phải lỗi lúc chạy |
| `init` | gán lúc khởi tạo rồi khoá lại |
| `private set` | đọc công khai, chỉ sửa được từ bên trong |
| `IReadOnlyList<T>` | lộ ra để đọc, không cho `Add` |

Public thì dùng **property**, đừng dùng field. Property cho bạn chỗ thêm kiểm
tra, đổi cách tính hay khoá quyền ghi mà không phá code đang gọi.

## Dấu hiệu trong code của bạn

- DTO hoặc value object đang là `class`, và có ai đó tự viết `Equals` bằng tay → đổi sang `record` là bỏ được cả đống code.
- So sánh hai object bằng `==` mà luôn ra `false` → đang so danh tính, không phải nội dung.
- `struct` có nhiều field hoặc có `set` → gần như luôn nên là `class` hoặc `record`.
- `public` field không có `{ get; set; }` → đổi sang property trước khi có người phụ thuộc vào nó.

## Ghi nhớ

- `class` so theo **tham chiếu**, `record` so theo **giá trị**.
- `with` chép nông: collection bên trong vẫn dùng chung.
- `struct` chỉ khi nhỏ, bất biến và tạo rất nhiều.
- `required` bắt lỗi thiếu dữ liệu ngay lúc compile.

## Bước tiếp theo

Kiểu nào cũng có lúc rỗng. Mà rỗng trong C# nghĩa là `null`.

Bài sau, **Làm việc với null**, mở bằng một `NullReferenceException` trong log
production mà không ai biết giá trị null đi vào từ đâu.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "record Money(decimal Amount);\n\nvar a = new Money(100);\nvar b = new Money(100);\n\nConsole.WriteLine(a == b);",
    "options": ["True", "False", "Lỗi compile", "Tuỳ máy"],
    "answer": 1,
    "explain": "record so sánh theo giá trị: mọi field bằng nhau thì hai record bằng nhau. Nếu Money là class thì kết quả sẽ là False."
  },
  {
    "prompt": "Bạn cần một kiểu chở dữ liệu từ API về: chỉ đọc, so sánh theo nội dung, ghi log dễ đọc. Chọn gì?",
    "options": ["class với property get/set", "record", "struct có set", "Dictionary<string, object>"],
    "answer": 2,
    "explain": "record sinh sẵn Equals, GetHashCode, ToString và with; property khai báo rút gọn là init-only nên dữ liệu không bị sửa lung tung."
  },
  {
    "prompt": "a có một List bên trong. Sau dòng này, b và a chia sẻ gì?",
    "code": "var b = a with { Amount = 200 };",
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
