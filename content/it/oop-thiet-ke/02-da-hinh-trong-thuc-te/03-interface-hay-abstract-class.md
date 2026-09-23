---
title: Interface hay abstract class
minutes: 11
---

Năm class đang implement `INotifier`, và cả năm cần thêm một method `Retry` y
như nhau.

Cách nhanh nhất trông rất rõ: đổi `INotifier` thành `abstract class NotifierBase`,
viết `Retry` một lần ở đó.

Bạn làm, rồi build đỏ hai chỗ. `EmailNotifier` đã kế thừa `BackgroundService`
rồi, mà C# chỉ cho kế thừa một class.

> **Học xong bài này bạn sẽ:** chọn giữa interface và abstract class bằng ba câu
> hỏi; biết default interface member giải quyết được gì và gây ra được gì; và
> biết khi nào cả hai đều sai.
>
> **Cần biết trước:** interface (chương trước), `abstract` và `override` (hai
> bài trước).

## Interface là hợp đồng, abstract class là bộ khung dùng chung

Cả hai đều cho bạn một kiểu cha để đa hình. Khác nhau ở chỗ chúng mang theo
những gì.

| | `interface` | `abstract class` |
|---|---|---|
| Một class dùng được mấy cái | **nhiều** | đúng một |
| Mang field, state | không | có |
| Có constructor | không | có |
| Thành viên `private`, `protected` | không | có |
| Có sẵn phần thân | chỉ qua default member (C# 8) | có, tự nhiên |
| Thêm thành viên mới | phá code người dùng, trừ khi có default | không phá |

Ba câu hỏi gỡ được gần hết mọi lần lưỡng lự:

| Câu hỏi | Nếu có |
|---|---|
| Kiểu này có thể cần nhiều vai cùng lúc? | `interface` |
| Các lớp con có **state** dùng chung không? | `abstract class` |
| Bạn chỉ muốn tránh chép code? | không phải cả hai — xem mục cuối |

## Thử ngay: default interface member không phải method của class

Từ C# 8, interface được phép có sẵn phần thân. Nhưng nó thuộc về ai?

```csharp
INotifier sms = new SmsNotifier();
sms.Send("hi");
sms.Retry("hi");

var direct = new SmsNotifier();
direct.Send("hi");

interface INotifier
{
    void Send(string message);

    void Retry(string message)
    {
        Console.WriteLine("thử lại...");
        Send(message);
    }
}

class SmsNotifier : INotifier
{
    public void Send(string message) =>
        Console.WriteLine($"SMS: {message}");
}
```

**Đoán trước khi chạy:** `direct` là một `SmsNotifier` thật. Thêm dòng
`direct.Retry("hi")` vào thì code còn build được không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
SMS: hi
thử lại...
SMS: hi
SMS: hi
```

Thêm `direct.Retry("hi")` là **lỗi compile**: `SmsNotifier` không có method nào
tên `Retry`.

Default member sống trong interface, không nhập vào class. Muốn gọi nó thì biến
phải có kiểu `INotifier`.

Đó là lý do người ta dùng nó để **thêm thành viên vào interface đã phát hành**
mà không phá code cũ, chứ không phải để thay abstract class.

</details>

## Một class nhận nhiều interface, nhưng chỉ một lớp cha

Đây là khác biệt quyết định nhiều lựa chọn thiết kế nhất.

```csharp
class EmailNotifier : BackgroundService,
    INotifier, IDisposable
{
    public void Send(string message) { }
    public void Dispose() { }
}
```

`EmailNotifier` đã dùng hết suất kế thừa cho `BackgroundService`. Nó vẫn nhận
thêm bao nhiêu interface cũng được.

Nên khi kiểu của bạn có thể cần nhiều vai — vừa gửi thông báo, vừa dọn tài
nguyên, vừa so sánh được — thì interface là cách duy nhất.

## Abstract class trả công khi có state và trình tự dùng chung

Interface không giữ được state. Có `readonly` field, có constructor kiểm tra dữ
liệu, thì phải là class.

```csharp
abstract class Exporter
{
    private readonly string _folder;

    protected Exporter(string folder)
    {
        ArgumentNullException.ThrowIfNull(folder);
        _folder = folder;
    }

    public void Run(string name)
    {
        Console.WriteLine($"Ghi vào {_folder}");
        Write(name);
    }

    protected abstract void Write(string name);
}
```

`Run` giữ trình tự, `Write` là phần mỗi lớp con tự lo. Constructor bảo đảm
không lớp con nào tồn tại với `_folder` rỗng.

Cả hai điều đó interface đều không làm được. Và `protected` thì interface cũng
không có.

## Cần dùng lại code thì đó là việc của composition

Đây là bẫy hay gặp nhất, và nó chính là chuyện ở đầu bài.

```csharp
// SAI — kế thừa chỉ để khỏi chép ba dòng
abstract class NotifierBase
{
    protected void Retry(string message) { }
}
```

Năm class kia không "là một" `NotifierBase`. Chúng chỉ tình cờ cần cùng một
đoạn code.

```csharp
// ĐÚNG — tách thành một thứ ai cũng dùng được
class RetryPolicy
{
    public void Run(Action work) => work();
}

class SmsNotifier(RetryPolicy retry) : INotifier
{
    public void Send(string message) =>
        retry.Run(() => Console.WriteLine(message));
}
```

Cách này không tiêu suất kế thừa, và `RetryPolicy` test được riêng. Bài "Kế thừa
hay composition" ở chương trước nói đúng chuyện này.

Phép thử một câu: bạn cần một **vai**, một **bộ khung**, hay chỉ cần **dùng lại
code**? Ba câu trả lời dẫn tới interface, abstract class, và composition.

## Dấu hiệu trong code của bạn

- Một `abstract class` chỉ có method `abstract`, không field, không constructor → đó là interface viết sai chỗ.
- Lớp cha `abstract` tên `BaseSomething` với các tiện ích không liên quan nhau → đang dùng kế thừa để chép code.
- Interface có `default` member chứa logic nghiệp vụ → logic nằm ở chỗ không test riêng được, không ai override được rõ ràng.
- Một class cần hai bộ khung cùng lúc → dấu hiệu cả hai bộ khung đều nên là interface cộng composition.
- Interface đã phát hành mà bạn muốn thêm method → dùng default member, hoặc tách một interface mới.

## Ghi nhớ

- Nhiều vai cùng lúc thì `interface`; state và trình tự dùng chung thì `abstract class`.
- Một class chỉ kế thừa được một lớp cha, còn interface thì bao nhiêu cũng được.
- Default interface member chỉ gọi được qua kiểu interface, không phải qua class.
- Nó sinh ra để thêm thành viên vào interface cũ mà không phá code người dùng.
- Chỉ muốn khỏi chép code thì cả hai đều sai — đó là việc của composition.

## Bước tiếp theo

Hết chương **Đa hình trong thực tế**. Bạn đã có đủ công cụ để một kiểu cha thay
được nhiều hành vi.

Chương sau, **SOLID trong code thật**, mở bằng một class tên `OrderService` dài
hai nghìn dòng — và năm nguyên tắc để nó không bao giờ dài đến thế nữa.

```quiz
[
  {
    "prompt": "EmailNotifier đã kế thừa BackgroundService. Bạn cần nó gửi được thông báo và dọn tài nguyên. Làm thế nào?",
    "options": [
      "Đổi BackgroundService thành interface",
      "Cho EmailNotifier implement INotifier và IDisposable",
      "Tạo một abstract class kế thừa BackgroundService rồi cho EmailNotifier kế thừa nó",
      "Gộp cả ba thành một abstract class duy nhất"
    ],
    "answer": 2,
    "explain": "Một class chỉ kế thừa được một lớp cha, nhưng nhận được bao nhiêu interface cũng được. Phương án C chạy được nhưng thêm một tầng kế thừa chỉ để giữ chỗ."
  },
  {
    "prompt": "Interface có default member Retry. Dòng nào KHÔNG build được?",
    "code": "// một dòng dưới đây không biên dịch được\nINotifier a = new SmsNotifier();\na.Retry(\"hi\");\n\nvar b = new SmsNotifier();\nb.Retry(\"hi\");",
    "options": [
      "Cả bốn dòng đều build được",
      "Dòng a.Retry, vì default member không gọi được",
      "Dòng b.Retry, vì SmsNotifier không có method Retry",
      "Dòng khai báo b, vì thiếu kiểu interface"
    ],
    "answer": 3,
    "explain": "Default member thuộc về interface, không nhập vào class. Gọi qua biến kiểu INotifier thì được; qua biến kiểu SmsNotifier thì compiler không tìm thấy method nào tên Retry."
  },
  {
    "prompt": "Ba class cần chung một đoạn code chờ rồi thử lại. Chúng không có quan hệ họ hàng gì với nhau. Nên làm gì?",
    "options": [
      "Tách thành một class RetryPolicy rồi nhận qua constructor",
      "Tạo abstract class RetryBase cho cả ba kế thừa",
      "Đưa đoạn code vào default member của một interface",
      "Chép đoạn code vào cả ba, ba dòng thì không sao"
    ],
    "answer": 1,
    "explain": "Chúng chỉ cần dùng lại code, không phải là một loại chung. Kế thừa ở đây tiêu mất suất kế thừa duy nhất, còn default member thì đặt logic vào chỗ khó test riêng."
  },
  {
    "prompt": "Bạn thấy một abstract class chỉ gồm các method abstract, không field, không constructor. Nhận xét nào đúng?",
    "options": [
      "Đúng chuẩn, abstract class luôn tốt hơn interface",
      "Cần thêm ít nhất một method có thân mới hợp lệ",
      "Nó nên là interface, vì đang tiêu suất kế thừa mà chẳng cho lại gì",
      "Không khác gì interface, nên đổi hay không đều được"
    ],
    "answer": 3,
    "explain": "Nó không mang state, không mang constructor, không mang phần thân dùng chung — tức là không dùng gì tới thứ chỉ class mới có. Đổi lại, nó chiếm suất kế thừa duy nhất của mọi lớp con."
  }
]
```
