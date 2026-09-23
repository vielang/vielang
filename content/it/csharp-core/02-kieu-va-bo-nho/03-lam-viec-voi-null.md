---
title: Làm việc với null
minutes: 10
---

Log production báo `NullReferenceException` ở tầng service. Bạn mở đúng dòng
đó ra: mọi biến đều được gán tử tế, không chỗ nào viết `null` cả. Giá trị null
đi vào từ nơi khác, ba tầng gọi hàm trước đó, và stack trace không nói gì về
nơi ấy.

> **Học xong bài này bạn sẽ:** bật và đọc được cảnh báo nullable của compiler;
> dùng đúng `?.`, `??`, `is null`; và chặn null ngay ở biên thay vì đuổi theo
> nó qua nhiều tầng.
>
> **Cần biết trước:** reference type, method và tham số.

## Bật nullable reference types

```xml
<PropertyGroup>
  <Nullable>enable</Nullable>
</PropertyGroup>
```

Bật rồi thì `string` nghĩa là **không bao giờ null**, muốn cho phép null phải
viết `string?`. Project .NET mới đã bật sẵn.

```csharp
string? ten2 = null;        // hợp lệ
string ten1 = "Huy";        // không được null

int dai = ten2.Length;      // cảnh báo CS8602
int an = ten2?.Length ?? 0; // an toàn
```

## Thử ngay: cảnh báo chỉ là cảnh báo

```csharp
using System.Text.Json;

class Nguoi
{
    public string Ten { get; set; } = "";
}

var n = JsonSerializer.Deserialize<Nguoi>("{}");
Console.WriteLine(n!.Ten is null);
Console.WriteLine(n.Ten?.Length ?? -1);
```

**Đoán trước khi chạy:** `Ten` khai báo là `string` không null và có giá trị
mặc định `""`. Hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
False
0
```

Lần này may: JSON rỗng nên `Ten` giữ giá trị mặc định `""`. Nhưng đổi chuỗi
JSON thành `{"Ten": null}` mà chạy lại thì in ra `True` và `-1` — **null lọt
vào một biến khai báo là không null**, compiler không hề cảnh báo.

</details>

Bài học: nullable reference types là **kiểm tra lúc compile**, runtime không
chặn gì cả. Dữ liệu từ JSON, từ database, từ thư viện cũ vẫn đưa null vào
được. Vì vậy vẫn phải kiểm tra ở biên hệ thống.

## Nullable value type

```csharp
int? soLuong = null;          // Nullable<int>
if (soLuong.HasValue) { }

int thuc = soLuong ?? 0;
int chac = soLuong!.Value;    // ném nếu đang null
```

Với value type, `?` không chỉ là chú thích cho compiler mà đổi hẳn kiểu thành
`Nullable<T>` — có thật trong runtime.

## Các toán tử null

```csharp
var tp = user?.DiaChi?.ThanhPho;   // null-conditional
var hien = tp ?? "Chưa cập nhật";  // null-coalescing
cache ??= new Dictionary<string, string>();
var n = list?.Count ?? 0;
```

## Dấu `!` và khi nào được dùng

```csharp
// SAI — tắt cảnh báo chứ không kiểm tra gì
var don = await db.Orders.FindAsync(id);
var ten = don!.KhachHang;
```

```csharp
// ĐÚNG — kiểm tra thật, compiler theo được luồng
var don = await db.Orders.FindAsync(id);
if (don is null)
    return NotFound();

var ten = don.KhachHang;
```

`!` là **null-forgiving operator**: nó tắt cảnh báo, không kiểm tra gì. Mỗi lần
viết `!` là bạn nhận trách nhiệm; sai thì lại đúng cái exception ta đang tránh.

## Chặn ngay ở biên

```csharp
public class Invoice
{
    private readonly Customer _khach;

    public Invoice(Customer khach, decimal tien)
    {
        ArgumentNullException.ThrowIfNull(khach);
        if (tien <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(tien), "Số tiền phải lớn hơn 0");

        _khach = khach;
    }
}
```

**Fail fast**: chặn dữ liệu sai ngay lúc vào. `ArgumentNullException` ném ở
constructor dễ sửa hơn nhiều so với `NullReferenceException` nổ ba tầng sau
đó, khi chẳng còn manh mối nào về nơi null lọt vào — đúng tình huống ở đầu bài.

## Dấu hiệu trong code của bạn

- Dấu `!` rải rác để dập cảnh báo → mỗi cái là một `NullReferenceException` đang chờ.
- Model nhận từ JSON/API có property `string` không null nhưng không kiểm tra gì sau khi deserialize.
- Method `public` nhận tham số object mà không `ArgumentNullException.ThrowIfNull`.
- Method trả về `null` cho một danh sách → mọi nơi gọi đều phải nhớ kiểm tra; trả `[]` thì không ai phải nhớ gì.

## Ghi nhớ

- Nullable reference types là kiểm tra **lúc compile**; runtime vẫn cho null lọt vào.
- `is null` / `is not null` là cách kiểm tra chuẩn — không bị ảnh hưởng nếu kiểu đó nạp chồng `==`.
- `!` tắt cảnh báo chứ không kiểm tra; ưu tiên `if (x is null) return`.
- Đừng trả `null` cho danh sách: trả `Array.Empty<T>()` hoặc list rỗng.

## Bước tiếp theo

Bài sau — **enum và hằng số** — thay những chuỗi "paid", "PAID", "Paid" rải
khắp code bằng thứ compiler kiểm tra được.

```quiz
[
  {
    "prompt": "Project bật Nullable enable. Dòng nào dưới đây thật sự bảo vệ bạn lúc chạy?",
    "options": [
      "Khai báo string Ten thay vì string? Ten",
      "Thêm dấu ! sau biến để hết cảnh báo",
      "Kiểm tra if (x is null) trước khi dùng",
      "Bật TreatWarningsAsErrors"
    ],
    "answer": 3,
    "explain": "Nullable reference types chỉ tác dụng lúc compile. Runtime chỉ an toàn khi bạn kiểm tra thật, hoặc chặn ở biên bằng ArgumentNullException."
  },
  {
    "prompt": "Đoạn này in ra gì khi FindAsync không tìm thấy đơn hàng?",
    "code": "var don = await db.Orders.FindAsync(id);\nConsole.WriteLine(don!.KhachHang);",
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
    "prompt": "int? soLuong = null; có gì khác string? ten = null;?",
    "options": [
      "Không khác gì, chỉ là cú pháp",
      "int? đổi hẳn kiểu thành Nullable<int>, có thật lúc chạy; string? chỉ là chú thích cho compiler",
      "string? tốn thêm bộ nhớ còn int? thì không",
      "int? không dùng được với ??"
    ],
    "answer": 2,
    "explain": "Nullable value type là một kiểu thật (Nullable<T> với cờ HasValue). Nullable reference type chỉ là thông tin cho compiler, biến mất sau khi build."
  }
]
```
