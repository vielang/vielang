---
title: Interface là hợp đồng
minutes: 11
---

Dự án có một interface tên `IOrderManager` với mười bốn method.

Thêm tính năng nào cũng phải sửa nó. Ba class implement đều phải sửa theo, kể
cả hai class chẳng liên quan gì tới tính năng mới.

Đó không phải hợp đồng. Đó là một cái thùng.

> **Học xong bài này bạn sẽ:** viết interface theo **việc cần làm** chứ không
> theo lớp kỹ thuật; đặt tên interface để người đọc biết ngay nó hứa gì; và
> hiểu vì sao interface nhỏ làm code dễ test hơn hẳn.
>
> **Cần biết trước:** `class`, method, đóng gói (bài trước).

## Interface nói làm gì, class nói làm thế nào

**Interface** là danh sách những gì một kiểu hứa làm được, không kèm cách làm.
Nó chỉ có chữ ký method, không có thân.

Mười bốn method trong một interface là dấu hiệu của chuyện khác. Người viết chưa
hỏi interface ấy dùng để làm gì.

```csharp
public interface INotifier
{
    Task SendAsync(string to, string message);
}

public class EmailNotifier : INotifier
{
    public Task SendAsync(string to, string message) =>
        smtp.SendAsync(to, message);
}

public class SmsNotifier : INotifier
{
    public Task SendAsync(string to, string message) =>
        sms.SendAsync(to, message);
}
```

Class nào dùng `INotifier` chỉ biết một điều: gửi được thông báo.

Nó không biết bên dưới là SMTP, SMS, hay chỉ ghi ra file lúc chạy test. Đó
chính là **trừu tượng**: giữ lại phần cần biết, giấu phần còn lại.

## Thử ngay: đổi cách làm mà không sửa nơi dùng

Nói "giấu phần còn lại" thì trừu tượng. Chạy thử thì thấy ngay nó cho bạn cái
gì.

```csharp
new PlaceOrder(new EmailNotifier()).Run();
new PlaceOrder(new LogNotifier()).Run();

interface INotifier
{
    void Send(string message);
}

class EmailNotifier : INotifier
{
    public void Send(string m) =>
        Console.WriteLine($"mail: {m}");
}

class LogNotifier : INotifier
{
    public void Send(string m) =>
        Console.WriteLine($"log: {m}");
}

class PlaceOrder(INotifier notifier)
{
    public void Run() =>
        notifier.Send("Đã đặt hàng");
}
```

**Đoán trước khi chạy:** `PlaceOrder` chỉ có đúng một dòng thân hàm. Hai lần
gọi in ra giống nhau hay khác nhau?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
mail: Đã đặt hàng
log: Đã đặt hàng
```

Một dòng code, hai hành vi.

`PlaceOrder` không hề biết ai đang gửi. Nhờ vậy bạn thêm SMS hay push về sau
mà không phải mở lại file này.

Lúc viết test, bạn truyền vào một bản chỉ ghi nhớ nội dung trong bộ nhớ. Không
cần SMTP thật, không cần mạng.

</details>

## Đặt tên theo khả năng, không theo lớp kỹ thuật

Interface là một lời hứa, nên cái tên phải nói được nó hứa gì.

| Tên | Nó hứa gì | Nên là |
|---|---|---|
| `IOrderManager` | không rõ | tách theo từng việc |
| `IOrderHelper` | không rõ | tách theo từng việc |
| `IShippingFeeCalculator` | tính phí vận chuyển | ✔ |
| `IOrderReader` | đọc đơn hàng | ✔ |

```csharp
public interface IShippingFeeCalculator
{
    decimal Calculate(Order order);
}

public interface IOrderWriter
{
    Task<int> SaveAsync(
        Order order, CancellationToken ct);
}
```

`Manager`, `Helper`, `Utils`, `Processor` là những cái tên không nói gì cả.

Và đây là dấu hiệu hữu ích: khi một interface khó đặt tên cho cụ thể, thường
là vì nó đang ôm quá nhiều việc.

Quy ước .NET thì tên bắt đầu bằng `I`. Với interface mô tả khả năng, tên tính
từ cũng rất hợp: `IDisposable`, `IComparable`, `IEnumerable`.

## Nhiều interface nhỏ tốt hơn một interface to

Giờ tới `IOrderManager` mười bốn method ở đầu bài.

```csharp
// SAI — mọi nơi phải implement đủ 14 method
public interface IOrderManager
{
    Task<Order> GetAsync(int id);
    Task SaveAsync(Order order);
    Task CancelAsync(int id);
    decimal CalculateFee(Order order);
    Task SendMailAsync(Order order);
    Task ExportPdfAsync(Order order);
    // … thêm 8 method nữa
}
```

```csharp
// ĐÚNG — mỗi hợp đồng cho một việc
public interface IOrderReader
{
    Task<Order?> GetAsync(int id);
}

public interface IOrderWriter
{
    Task SaveAsync(Order order);
}
```

Chia nhỏ thì nơi chỉ cần đọc sẽ phụ thuộc đúng `IOrderReader`. Bạn sửa phần
lưu, nó không bị kéo theo.

Một class vẫn implement được nhiều interface cùng lúc. Chia nhỏ hợp đồng không
có nghĩa là phải sinh thêm class.

## Phụ thuộc nhận qua constructor, đừng new bên trong

Chia nhỏ hợp đồng rồi, còn câu hỏi ai đưa chúng vào cho class của bạn.

```csharp
// SAI — tự tạo bên trong, không đổi và không test
public class PlaceOrder
{
    private readonly EmailNotifier _mail = new();
}
```

```csharp
// ĐÚNG — nói rõ mình cần gì, ai cấp là việc bên ngoài
public class PlaceOrder(INotifier notifier)
{
    public Task RunAsync() =>
        notifier.SendAsync("admin", "Có đơn mới");
}

// Program.cs
builder.Services
    .AddScoped<INotifier, EmailNotifier>();
```

Constructor là **bản kê khai phụ thuộc**. Nhìn vào là biết class này cần những
gì để chạy được.

Đây chính là dependency injection. Chương SOLID sẽ nói kỹ vì sao nó lật ngược
chiều phụ thuộc.

## Chưa chạm ra ngoài thì chưa cần interface

Đến đây thì dễ thành tách interface cho mọi thứ. Nó cũng có giá của nó.

| Kiểu của bạn | Tách interface? |
|---|---|
| Gọi API bên ngoài, database, file | **nên** |
| Đọc đồng hồ, sinh số ngẫu nhiên | **nên**, để test được |
| Tính toán thuần từ tham số truyền vào | chưa cần |
| DTO, `record` chỉ chứa dữ liệu | không |

Interface có giá của nó. Thêm một file, và thêm một lớp gián tiếp khi ai đó
lần theo code.

Dấu hiệu rõ nhất để cần nó: chỗ đó chạm ra thế giới bên ngoài, hoặc bạn thật
sự đã có từ hai cách làm trở lên.

Tạo interface chỉ để "cho dễ test" một class không hề chạm ra ngoài là tự thêm
việc cho mình.

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

Interface cho bạn cách thay đổi hành vi từ bên ngoài. Còn dùng lại code thì
sao?

Bài sau, **Kế thừa hay composition**, so hai cách dùng lại. Và vì sao cách thứ
hai gần như luôn an toàn hơn.

```quiz
[
  {
    "prompt": "Interface nào đặt tên tốt nhất?",
    "options": [
      "IShippingFeeCalculator",
      "IOrderHelper",
      "IOrderManager",
      "IOrderUtils"
    ],
    "answer": 1,
    "explain": "Tên nói rõ hợp đồng hứa làm gì. Manager, Helper, Utils không hứa điều gì cụ thể, và thường là dấu hiệu interface đang ôm nhiều việc."
  },
  {
    "prompt": "Class PlaceOrder tự viết private readonly EmailNotifier _mail = new(); Hệ quả nào là nặng nhất?",
    "options": [
      "Tốn thêm bộ nhớ cho mỗi instance",
      "Vi phạm quy ước đặt tên",
      "Không thay được cách gửi và không test được nếu không có SMTP thật",
      "Gây memory leak"
    ],
    "answer": 3,
    "explain": "Tự new là khoá cứng vào một cách làm. Nhận INotifier qua constructor thì đổi sang SMS hay bản giả lúc test đều không phải sửa class này."
  },
  {
    "prompt": "Interface có 14 method, vài class implement phải ném NotImplementedException ở nửa số đó. Nên làm gì?",
    "options": [
      "Để nguyên, ném NotImplementedException là bình thường",
      "Chia thành nhiều interface nhỏ theo việc mà từng nơi cần",
      "Chuyển thành abstract class có sẵn thân rỗng",
      "Thêm default implementation cho mọi method"
    ],
    "answer": 2,
    "explain": "NotImplementedException nghĩa là class bị ép ký một hợp đồng nó không làm. Chia nhỏ để mỗi nơi chỉ phụ thuộc đúng phần nó dùng."
  },
  {
    "prompt": "Trường hợp nào CHƯA cần tạo interface?",
    "options": [
      "Lớp gọi API thanh toán bên ngoài",
      "Lớp đọc ghi database",
      "Lớp lấy thời gian hiện tại",
      "Lớp tính thuế thuần tuý từ số liệu truyền vào, chỉ có một cách tính"
    ],
    "answer": 4,
    "explain": "Không chạm ra ngoài, không có cách làm thứ hai thì interface chỉ thêm một lớp gián tiếp. Ba trường hợp còn lại đều chạm thế giới bên ngoài nên rất đáng tách hợp đồng."
  }
]
```
