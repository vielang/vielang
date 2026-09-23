---
title: Interface là hợp đồng
minutes: 11
---

Dự án có một interface tên `IOrderManager` với mười bốn method. Thêm tính năng
nào cũng phải sửa nó, và ba class implement đều phải sửa theo — kể cả hai class
chẳng liên quan gì tới tính năng mới. Đó không phải hợp đồng, đó là một cái
thùng.

> **Học xong bài này bạn sẽ:** viết interface theo **việc cần làm** chứ không
> theo lớp kỹ thuật; đặt tên interface để người đọc biết ngay nó hứa gì; và
> hiểu vì sao interface nhỏ làm code dễ test hơn hẳn.
>
> **Cần biết trước:** `class`, method, đóng gói (bài trước).

## Interface trả lời "làm gì", class trả lời "làm thế nào"

```csharp
public interface IGuiThongBao
{
    Task GuiAsync(string den, string noiDung);
}

public class GuiMail : IGuiThongBao
{
    public Task GuiAsync(string den, string noiDung) =>
        smtp.SendAsync(den, noiDung);
}

public class GuiSms : IGuiThongBao
{
    public Task GuiAsync(string den, string noiDung) =>
        sms.SendAsync(den, noiDung);
}
```

Class nào dùng `IGuiThongBao` chỉ biết "có thể gửi thông báo", không biết bên
dưới là SMTP, SMS hay ghi ra file lúc chạy test. Đó là **trừu tượng**: giữ lại
phần cần biết, giấu phần còn lại.

## Thử ngay: đổi cách làm mà không sửa nơi dùng

```csharp
interface IGuiThongBao
{
    void Gui(string noiDung);
}

class GuiMail : IGuiThongBao
{
    public void Gui(string n) =>
        Console.WriteLine($"mail: {n}");
}

class GuiLog : IGuiThongBao
{
    public void Gui(string n) =>
        Console.WriteLine($"log: {n}");
}

class DatHang
{
    private readonly IGuiThongBao _tb;
    public DatHang(IGuiThongBao tb) => _tb = tb;

    public void Dat() => _tb.Gui("Đã đặt hàng");
}

new DatHang(new GuiMail()).Dat();
new DatHang(new GuiLog()).Dat();
```

**Đoán trước khi chạy:** class `DatHang` có một dòng `Dat()` duy nhất. Hai lần
gọi in ra giống nhau hay khác nhau?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
mail: Đã đặt hàng
log: Đã đặt hàng
```

Cùng một dòng code trong `DatHang` cho ra hai hành vi. `DatHang` **không hề
biết** ai đang gửi — và đó chính là điều làm nó thay được cách gửi (thêm SMS,
thêm push) mà không phải mở lại file này.

Lúc viết test, bạn truyền vào một bản ghi nhớ nội dung trong bộ nhớ, không cần
SMTP thật. Chi tiết ở chương "Thiết kế để test được".

</details>

## Đặt tên theo khả năng, không theo kỹ thuật

```csharp
// SAI — tên không hứa điều gì cụ thể
public interface IOrderManager { }
public interface IOrderHelper { }
public interface IOrderService { }
```

```csharp
// ĐÚNG — đọc tên là biết nó làm được gì
public interface ITinhPhiVanChuyen
{
    decimal Tinh(DonHang don);
}

public interface ILuuDonHang
{
    Task<int> LuuAsync(DonHang d, CancellationToken ct);
}
```

`Manager`, `Helper`, `Utils`, `Processor` là những cái tên không nói gì. Khi
một interface khó đặt tên cho cụ thể, thường là vì nó đang ôm quá nhiều việc.

Quy ước .NET: tên interface bắt đầu bằng `I`. Với interface mô tả **khả năng**,
tên tính từ cũng rất hợp: `IDisposable`, `IComparable`, `IEnumerable`.

## Interface nhỏ, chia theo người dùng

```csharp
// SAI — mọi nơi phải implement đủ 14 method
public interface IOrderManager
{
    Task<DonHang> LayAsync(int id);
    Task LuuAsync(DonHang d);
    Task HuyAsync(int id);
    decimal TinhPhi(DonHang d);
    Task GuiMailAsync(DonHang d);
    Task XuatPdfAsync(DonHang d);
    // … thêm 8 method nữa
}
```

```csharp
// ĐÚNG — mỗi hợp đồng cho một việc
public interface IDocDonHang
{
    Task<DonHang?> LayAsync(int id);
}

public interface ILuuDonHang
{
    Task LuuAsync(DonHang d);
}

public interface ITinhPhiVanChuyen
{
    decimal Tinh(DonHang d);
}
```

Chia nhỏ thì nơi chỉ cần đọc sẽ phụ thuộc đúng `IDocDonHang`, và khi bạn sửa
phần lưu thì nó không bị kéo theo. Một class vẫn có thể implement nhiều
interface cùng lúc — chia nhỏ không có nghĩa là phải có nhiều class.

## Phụ thuộc vào interface, nhận qua constructor

```csharp
// SAI — tự tạo bên trong, không đổi và không test được
public class DatHang
{
    private readonly GuiMail _mail = new();
}
```

```csharp
// ĐÚNG — nói rõ mình cần gì, ai cấp là việc bên ngoài
public class DatHang(IGuiThongBao thongBao)
{
    public Task DatAsync() =>
        thongBao.GuiAsync("admin", "Có đơn mới");
}

// Program.cs
builder.Services
    .AddScoped<IGuiThongBao, GuiMail>();
```

Constructor là **bản kê khai phụ thuộc**: nhìn vào là biết class này cần
những gì để chạy. Đây chính là dependency injection, và chương SOLID sẽ nói
kỹ vì sao nó lật ngược chiều phụ thuộc.

## Khi nào thì chưa cần interface

Interface có giá của nó: thêm một file, thêm một lớp gián tiếp khi đọc code.
Chưa cần vội nếu:

- Chỉ có **một** cách làm và chưa thấy cách thứ hai nào trong tầm nhìn.
- Kiểu đó là dữ liệu thuần (DTO, record) — không có hành vi để trừu tượng.
- Bạn tạo interface chỉ vì "để test", trong khi class đó không hề chạm ra ngoài (không gọi mạng, không đọc file, không đụng thời gian).

Dấu hiệu rõ nhất để **cần** interface: chỗ đó chạm ra thế giới bên ngoài
(database, HTTP, file, đồng hồ, hàng đợi), hoặc bạn thật sự có từ hai cách
làm trở lên.

## Dấu hiệu trong code của bạn

- Interface có tên `Manager`, `Helper`, `Utils`, `Processor` → nó đang ôm nhiều việc, khó đặt tên cụ thể.
- Interface trên mười method, và các class implement có method ném `NotImplementedException` → chia nhỏ theo người dùng.
- Interface chỉ có đúng một class implement, mà class ấy không chạm ra ngoài → có thể chưa cần interface.
- Class tự `new` các service bên trong → không thay được, không test được; nhận qua constructor.
- Interface lộ chi tiết kỹ thuật trong chữ ký (`SqlConnection`, `HttpResponseMessage`) → hợp đồng đang rò rỉ cách làm.

## Ghi nhớ

- Interface nói **làm gì**, class nói **làm thế nào**.
- Đặt tên theo khả năng; tránh `Manager`/`Helper`.
- Nhiều interface nhỏ tốt hơn một interface to; một class implement được nhiều cái.
- Phụ thuộc nhận qua constructor, đừng `new` bên trong.
- Chưa có cách làm thứ hai và không chạm ra ngoài thì chưa cần interface.

## Bước tiếp theo

Bài sau — **Kế thừa hay composition** — hai cách dùng lại code, và vì sao cách
thứ hai gần như luôn là lựa chọn an toàn hơn.

```quiz
[
  {
    "prompt": "Interface nào đặt tên tốt nhất?",
    "options": [
      "IOrderManager",
      "IOrderHelper",
      "ITinhPhiVanChuyen",
      "IOrderUtils"
    ],
    "answer": 3,
    "explain": "Tên nói rõ hợp đồng hứa làm gì. Manager, Helper, Utils không hứa điều gì cụ thể, và thường là dấu hiệu interface đang ôm nhiều việc."
  },
  {
    "prompt": "Class DatHang tự viết private readonly GuiMail _mail = new(); Hệ quả nào là nặng nhất?",
    "options": [
      "Tốn thêm bộ nhớ cho mỗi instance",
      "Không thay được cách gửi và không test được nếu không có SMTP thật",
      "Vi phạm quy ước đặt tên",
      "Gây memory leak"
    ],
    "answer": 2,
    "explain": "Tự new là khoá cứng vào một cách làm. Nhận IGuiThongBao qua constructor thì đổi sang SMS hay bản giả lúc test đều không phải sửa class này."
  },
  {
    "prompt": "Interface có 14 method, vài class implement phải ném NotImplementedException ở nửa số đó. Nên làm gì?",
    "options": [
      "Để nguyên, ném NotImplementedException là bình thường",
      "Chuyển thành abstract class có sẵn thân rỗng",
      "Chia thành nhiều interface nhỏ theo việc mà từng nơi cần",
      "Thêm default implementation cho mọi method"
    ],
    "answer": 3,
    "explain": "NotImplementedException nghĩa là class bị ép ký một hợp đồng nó không làm. Chia nhỏ để mỗi nơi chỉ phụ thuộc đúng phần nó dùng."
  },
  {
    "prompt": "Trường hợp nào CHƯA cần tạo interface?",
    "options": [
      "Lớp gọi API thanh toán bên ngoài",
      "Lớp đọc ghi database",
      "Lớp tính thuế thuần tuý từ số liệu truyền vào, chỉ có một cách tính",
      "Lớp lấy thời gian hiện tại"
    ],
    "answer": 3,
    "explain": "Không chạm ra ngoài, không có cách làm thứ hai thì interface chỉ thêm một lớp gián tiếp. Ba trường hợp còn lại đều chạm thế giới bên ngoài nên rất đáng tách hợp đồng."
  }
]
```
