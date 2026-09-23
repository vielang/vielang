---
title: Làm việc với null
minutes: 11
---

Log production chỉ có một dòng: `NullReferenceException` ở tầng service.

Bạn mở đúng dòng đó ra xem. Mọi biến đều được gán tử tế, không chỗ nào viết
`null` cả. Giá trị null đi vào từ nơi khác, ba tầng gọi hàm trước đó, và stack
trace không nói gì về nơi ấy.

> **Học xong bài này bạn sẽ:** bật và đọc được cảnh báo nullable của compiler;
> dùng đúng `?.`, `??`, `is null`; và chặn null ngay ở biên thay vì đuổi theo
> nó qua nhiều tầng.
>
> **Cần biết trước:** reference type, method và tham số.

## Nullable reference types: hàng rào ở mức compile

```xml
<PropertyGroup>
  <Nullable>enable</Nullable>
</PropertyGroup>
```

Bật rồi thì `string` nghĩa là **không bao giờ null**. Muốn cho phép null phải
viết `string?`. Project .NET mới đã bật sẵn.

```csharp
string? middleName = null;   // hợp lệ
string firstName = "Huy";    // không được null

int length = middleName.Length;       // cảnh báo CS8602
int safe = middleName?.Length ?? 0;   // an toàn
```

## Thử ngay: cảnh báo chỉ là cảnh báo

```csharp
using System.Text.Json;

class Person
{
    public string Name { get; set; } = "";
}

var p = JsonSerializer.Deserialize<Person>(
    """{"Name": null}""");

Console.WriteLine(p!.Name is null);
Console.WriteLine(p.Name?.Length ?? -1);
```

**Đoán trước khi chạy:** `Name` khai báo là `string` không null, lại có giá
trị mặc định `""`. Hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
True
-1
```

Null lọt thẳng vào một property khai báo là **không null**, và compiler không
hề cảnh báo.

Vì nullable reference types chỉ là kiểm tra **lúc compile**. Runtime không
chặn gì cả. Dữ liệu từ JSON, database hay thư viện cũ vẫn đưa null vào được.

</details>

## Nullable value type là một kiểu thật

```csharp
int? quantity = null;          // Nullable<int>
if (quantity.HasValue) { }

int actual = quantity ?? 0;
int risky = quantity!.Value;   // ném nếu đang null
```

| | `string?` | `int?` |
|---|---|---|
| Bản chất | chú thích cho compiler | kiểu thật `Nullable<int>` |
| Còn sau khi build | không | có |
| Runtime kiểm tra | không | có cờ `HasValue` |

Đây là lý do `int?` tốn thêm bộ nhớ còn `string?` thì không.

## Bốn toán tử làm việc với null

| Toán tử | Nghĩa |
|---|---|
| `?.` | gặp null thì dừng, trả về null |
| `??` | null thì lấy giá trị bên phải |
| `??=` | gán khi đang null |
| `is null` / `is not null` | cách kiểm tra chuẩn |

```csharp
var city = user?.Address?.City;
var display = city ?? "Chưa cập nhật";
cache ??= new Dictionary<string, string>();
```

Dùng `is null` thay cho `!= null`: nó không bị ảnh hưởng nếu kiểu đó nạp chồng
toán tử `==`.

Ba toán tử đầu bảng nối chuỗi được với nhau. `user?.Address?.City` gặp null ở
mắt xích nào cũng dừng ngay, trả về null thay vì nổ.

Nhưng đừng lạm dụng. Một chuỗi `?.` dài thường có nghĩa là bạn đang chấp nhận
dữ liệu thiếu, mà chưa quyết định phải làm gì với nó.

## Dấu ! tắt cảnh báo chứ không kiểm tra gì

```csharp
// SAI — chỉ dập cảnh báo
var order = await db.Orders.FindAsync(id);
var name = order!.Customer;
```

```csharp
// ĐÚNG — kiểm tra thật, compiler theo được luồng
var order = await db.Orders.FindAsync(id);
if (order is null)
    return NotFound();

var name = order.Customer;
```

`!` là **null-forgiving operator**. Mỗi lần viết nó là bạn nhận trách nhiệm
thay compiler. Sai thì lại đúng cái exception ta đang tránh.

## Chặn ngay ở biên, đừng đuổi theo null qua nhiều tầng

```csharp
public class Invoice
{
    private readonly Customer _customer;

    public Invoice(Customer customer, decimal amount)
    {
        ArgumentNullException.ThrowIfNull(customer);
        if (amount <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(amount));

        _customer = customer;
    }
}
```

**Fail fast**: chặn dữ liệu sai ngay lúc nó vào. `ArgumentNullException` ném ở
constructor dễ sửa hơn nhiều so với `NullReferenceException` nổ ba tầng sau
đó, khi chẳng còn manh mối nào.

Đó chính là khác biệt giữa bài này và cái log ở đầu bài.

## Dấu hiệu trong code của bạn

- Dấu `!` rải rác để dập cảnh báo → mỗi cái là một `NullReferenceException` đang chờ.
- Model nhận từ JSON có property `string` không null nhưng không kiểm tra gì sau khi deserialize.
- Method `public` nhận tham số object mà không `ArgumentNullException.ThrowIfNull`.
- Method trả về `null` cho một danh sách → mọi nơi gọi đều phải nhớ kiểm tra.

## Ghi nhớ

- Nullable reference types là kiểm tra **lúc compile**; runtime vẫn cho null lọt vào.
- `is null` và `is not null` là cách kiểm tra chuẩn.
- `!` tắt cảnh báo chứ không kiểm tra; ưu tiên `if (x is null) return`.
- Đừng trả `null` cho danh sách, trả `[]` để người gọi khỏi phải nhớ.

## Bước tiếp theo

Null là giá trị vắng mặt. Còn những giá trị có mặt thì sao — "paid", "PAID",
"Paid" nằm rải khắp code?

Bài sau, **enum và hằng số**, mở bằng một lần thêm trạng thái vào giữa enum.
Deploy xong, mọi đơn hàng cũ trong database đổi nghĩa.

```quiz
[
  {
    "prompt": "Project bật Nullable enable. Dòng nào dưới đây thật sự bảo vệ bạn lúc chạy?",
    "options": [
      "Khai báo string Name thay vì string? Name",
      "Thêm dấu ! sau biến để hết cảnh báo",
      "Kiểm tra if (x is null) trước khi dùng",
      "Bật TreatWarningsAsErrors"
    ],
    "answer": 3,
    "explain": "Nullable reference types chỉ tác dụng lúc compile. Runtime chỉ an toàn khi bạn kiểm tra thật, hoặc chặn ở biên bằng ArgumentNullException."
  },
  {
    "prompt": "Đoạn này in ra gì khi FindAsync không tìm thấy đơn hàng?",
    "code": "var order = await db.Orders.FindAsync(id);\nConsole.WriteLine(order!.Customer);",
    "options": [
      "Chuỗi rỗng",
      "null",
      "Ném NullReferenceException",
      "Lỗi compile vì thiếu kiểm tra"
    ],
    "answer": 3,
    "explain": "FindAsync trả null khi không có. Dấu ! chỉ tắt cảnh báo của compiler, còn lúc chạy thì vẫn chạm vào null."
  },
  {
    "prompt": "Method lấy danh sách đơn hàng, không có cái nào khớp. Nên trả về gì?",
    "options": [
      "null, để người gọi biết là không có",
      "Danh sách rỗng",
      "Ném exception",
      "Tuỳ, hai cách như nhau"
    ],
    "answer": 2,
    "explain": "Trả danh sách rỗng thì người gọi foreach hay Count đều chạy bình thường. Trả null là bắt mọi nơi gọi phải nhớ kiểm tra."
  },
  {
    "prompt": "int? quantity có gì khác string? name?",
    "options": [
      "Không khác gì, chỉ là cú pháp",
      "int? đổi hẳn kiểu thành Nullable<int>, có thật lúc chạy; string? chỉ là chú thích cho compiler",
      "string? tốn thêm bộ nhớ còn int? thì không",
      "int? không dùng được với ??"
    ],
    "answer": 2,
    "explain": "Nullable value type là một kiểu thật với cờ HasValue. Nullable reference type chỉ là thông tin cho compiler, biến mất sau khi build."
  }
]
```
