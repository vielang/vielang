---
title: Interface
minutes: 5
---

Đặt hàng xong, hệ thống gửi email cho khách. Tháng sau sếp muốn gửi thêm SMS.
Nếu code đặt hàng gắn chặt với email thì phải sửa chính code đó. Interface
giúp code đặt hàng chỉ cần biết "có ai đó gửi được thông báo", còn gửi bằng gì thì tuỳ.

## Khái niệm

📜 **Interface**: danh sách method và property mà một class phải có, không kèm cách làm.

🤝 **Implement**: class ghi `: TênInterface` thì cam kết viết đủ mọi thành viên của interface đó, thiếu một cái là lỗi compile.

## Ví dụ

```csharp
var order = new PlaceOrder(new EmailNotifier());
order.Run();

interface INotifier
{
    void Send(string message);
}

class EmailNotifier : INotifier
{
    public void Send(string message) =>
        Console.WriteLine($"Email: {message}");
}

class PlaceOrder
{
    private readonly INotifier _notifier;

    public PlaceOrder(INotifier notifier)
    {
        _notifier = notifier;
    }

    public void Run() => _notifier.Send("Đã đặt hàng");
}
```

- `interface INotifier` chỉ khai báo `Send`, không có thân. Tên interface
  bắt đầu bằng chữ `I`.
- `EmailNotifier : INotifier` implement interface, nên phải có method `Send`
  và để `public`.
- `PlaceOrder` chỉ biết `INotifier`, không biết email. Nó nhận notifier qua
  constructor.
- `readonly` nghĩa là field chỉ gán được lúc khai báo hoặc trong
  constructor, sau đó không đổi (giống property `{ get; }` ở bài Property
  và constructor). Field `private` theo quy ước .NET bắt đầu bằng `_`.
- Không `new INotifier()` được, nhưng biến kiểu `INotifier` thì giữ được bất
  kỳ class nào implement nó.
- Một class implement được nhiều interface:
  `class Shop : INotifier, IPrinter`.

## Thử ngay

Chép ví dụ trên vào `Program.cs`. Thêm class dưới đây vào cuối file, rồi thay
hai dòng đầu bằng đoạn gọi mới:

```csharp
class SmsNotifier : INotifier
{
    public void Send(string message) =>
        Console.WriteLine($"SMS: {message}");
}
```

```csharp
new PlaceOrder(new EmailNotifier()).Run();
new PlaceOrder(new SmsNotifier()).Run();
```

**Đoán trước khi chạy:** `PlaceOrder` không đổi một dòng nào. Hai lần chạy in
ra giống hay khác nhau?

<details>
<summary>Xem kết quả</summary>

```text
Email: Đã đặt hàng
SMS: Đã đặt hàng
```

Khác nhau. `PlaceOrder` gọi `Send` của object được truyền vào. Thêm cách gửi
mới chỉ cần viết class mới, không sửa `PlaceOrder`.

</details>

## Lỗi hay gặp

**Implement thiếu thành viên.** Class đã ghi `: INotifier` thì phải viết đủ.

```csharp
// SAI — lỗi compile: thiếu method Send
class PushNotifier : INotifier
{
}
```

```csharp
// ĐÚNG
class PushNotifier : INotifier
{
    public void Send(string message) =>
        Console.WriteLine($"Push: {message}");
}
```

**Tạo object từ interface.** Interface không có code để chạy.

```csharp
// SAI — lỗi compile: không new được interface
INotifier notifier = new INotifier();
```

```csharp
// ĐÚNG — new một class implement interface đó
INotifier notifier = new EmailNotifier();
```

## Tóm tắt

- Interface liệt kê method và property mà class phải có, không kèm cách làm.
- Class `: TênInterface` phải implement đủ mọi thành viên, để `public`.
- Code chỉ phụ thuộc vào interface thì đổi được cách làm mà không phải
  sửa code đó.
- Không `new` được interface. Một class implement được nhiều interface.

```quiz
[
  {
    "prompt": "interface IPrinter { void Print(string text); } Class nào implement IPrinter đúng?",
    "options": [
      "class A : IPrinter { }",
      "class A : IPrinter { void Print(string text) { } }",
      "class A : IPrinter { public void Print(string text) { } }",
      "class A : IPrinter { public void Print() { } }"
    ],
    "answer": 3,
    "explain": "Phải có đúng method Print(string) và để public. Thiếu method, thiếu public hay sai tham số đều là lỗi compile."
  },
  {
    "prompt": "Class ReportService nhận IExporter qua constructor. Muốn xuất thêm định dạng Excel, cần làm gì?",
    "options": [
      "Viết class ExcelExporter : IExporter rồi truyền vào ReportService",
      "Sửa ReportService, thêm if cho Excel",
      "Sửa interface IExporter, thêm method ExportExcel",
      "Tạo ReportService mới cho Excel"
    ],
    "answer": 1,
    "explain": "ReportService chỉ phụ thuộc vào IExporter, nên thêm cách xuất mới chỉ cần thêm một class implement nó."
  },
  {
    "prompt": "Dòng nào KHÔNG biên dịch được, biết EmailNotifier implement INotifier?",
    "options": [
      "INotifier n = new EmailNotifier();",
      "EmailNotifier e = new EmailNotifier();",
      "var list = new List<INotifier> { new EmailNotifier() };",
      "var n = new INotifier();"
    ],
    "answer": 4,
    "explain": "Không tạo object từ interface được. Biến hay list kiểu INotifier thì hợp lệ, miễn object bên trong là class implement nó."
  }
]
```
