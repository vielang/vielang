---
title: Những pattern .NET đã làm sẵn
minutes: 11
---

Một pull request thêm `EventBus` tự viết: 180 dòng, một `Dictionary` các handler,
một method `Publish`.

Reviewer hỏi một câu: "Cái này khác `IHostedService` với `Channel<T>` ở chỗ nào?"

Người viết không biết .NET đã có sẵn. Không phải vì họ kém — vì thứ đó nằm trong
framework dưới một cái tên khác hẳn.

> **Học xong bài này bạn sẽ:** nhận ra sáu pattern bạn đang dùng mỗi ngày mà chưa
> gọi tên; biết chỗ nào .NET đã dựng sẵn để khỏi viết lại; và có một phép thử
> trước khi tự viết pattern.
>
> **Cần biết trước:** ba bài trước của chương, `IEnumerable` và `async` (khoá C#
> Core).

## Sáu pattern bạn đã dùng mà chưa gọi tên

Bảng này là phần đáng tra lại nhất của cả chương.

| Pattern | Trong .NET nó tên là |
|---|---|
| Iterator | `IEnumerable<T>`, `yield return` |
| Observer | `event`, `IObservable<T>`, `IProgress<T>` |
| Chain of Responsibility | middleware, `DelegatingHandler` |
| Factory | DI container, `IHttpClientFactory` |
| Adapter | class bọc một thư viện ngoài |
| Command | một record request cộng handler xử lý nó |

Sáu dòng ấy nghĩa là một điều. Bạn đã dùng design pattern từ bài học C# thứ hai,
chỉ chưa ai gọi tên chúng.

Biết tên có ích khi đọc sách, đi phỏng vấn, và khi cần nói gọn trong review. Nó
không làm code tốt hơn.

## Iterator và Observer nằm ngay trong cú pháp

Bài "IEnumerable và hoãn thực thi" đã dạy trọn Iterator.

```csharp
IEnumerable<string> ReadLines(string path)
{
    using var reader = new StreamReader(path);
    string? line;
    while ((line = reader.ReadLine()) is not null)
        yield return line;
}
```

`yield return` là Iterator, và compiler sinh hộ bạn cả một state machine. Tự viết
class với `MoveNext` và `Current` là làm lại việc đó bằng tay.

Observer thì nằm ở `event`. Nó còn nằm ở một chỗ ít ai nghĩ tới.

```csharp
async Task Import(IProgress<int> progress)
{
    for (var i = 0; i < 100; i++)
    {
        await Task.Delay(1);
        progress.Report(i);
    }
}
```

`IProgress<T>` là Observer có sẵn cho việc báo tiến độ. Nó lo hộ cả phần đưa lời
gọi về đúng luồng.

## Chain of Responsibility chính là middleware

Mỗi mắt xích xử lý một phần rồi gọi mắt xích sau, hoặc dừng hẳn. Đó là định
nghĩa của pattern, và cũng là định nghĩa của middleware.

```csharp
app.Use(async (ctx, next) =>
{
    if (!ctx.Request.Headers.ContainsKey("X-Api-Key"))
    {
        ctx.Response.StatusCode = 401;
        return;                  // dừng chuỗi
    }

    await next();                // chuyển cho mắt xích sau
});
```

Gọi `next()` là đi tiếp. Không gọi là chuỗi dừng tại đây.

Vậy nên trước khi tự viết một chuỗi handler gọi tiếp nhau, hãy xem nó nằm ở đâu.

Request đi vào thì là middleware. Request HTTP đi ra thì là `DelegatingHandler`.

## Thử ngay: Command là một record cộng một handler

Pattern nghe nặng nhất trong bảng lại là pattern gọn nhất khi viết bằng C# hiện
đại.

```csharp
var handlers = new Dictionary<Type, Func<object, string>>
{
    [typeof(CancelOrder)] = c =>
        $"huỷ đơn {((CancelOrder)c).Id}",
    [typeof(ShipOrder)] = c =>
        $"giao đơn {((ShipOrder)c).Id}",
};

object cmd = new ShipOrder(7);
Console.WriteLine(handlers[cmd.GetType()](cmd));

record CancelOrder(int Id);
record ShipOrder(int Id);
```

**Đoán trước khi chạy:** biến `cmd` khai kiểu `object`, nhưng giữ một
`ShipOrder`. Dòng cuối in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
giao đơn 7
```

`cmd.GetType()` hỏi kiểu **thật** lúc chạy, không phải kiểu của biến. Nên nó tra
được đúng handler.

Đó là Command ở dạng gọn nhất. Một `record` mang dữ liệu của ý muốn, và một hàm
xử lý ý muốn ấy.

Thư viện như MediatR đóng gói đúng ý này, thêm phần tra handler tự động. Nhưng cơ
chế thì vẫn là hai thứ trên.

</details>

## Phép thử trước khi tự viết một pattern

Ba câu, hỏi theo thứ tự.

| Câu hỏi | Nếu có |
|---|---|
| .NET đã có sẵn thứ này chưa? | dùng cái có sẵn |
| Vấn đề đã xảy ra thật chưa, hay đang đề phòng? | đợi nó xảy ra |
| Người mới vào nhóm đọc có hiểu ngay không? | giữ cách đơn giản |

`EventBus` 180 dòng ở đầu bài trượt câu đầu tiên. `Channel<T>` cộng một
`BackgroundService` làm đúng việc đó, có sẵn trong framework, và người mới đọc
tài liệu Microsoft là hiểu.

Pattern là **từ vựng để gọi tên một giải pháp**, không phải mục tiêu để đạt. Code
tốt không phải code có nhiều pattern.

## Dấu hiệu trong code của bạn

- Tự viết class với `MoveNext` và `Current` → `yield return` làm hộ cả state machine.
- Tự viết `EventBus`, `Mediator` hay hàng đợi trong bộ nhớ → xem `Channel<T>`, `IHostedService` trước.
- Tự viết chuỗi handler gọi tiếp nhau → middleware hoặc `DelegatingHandler` đã là chuỗi ấy.
- Tự viết một `ServiceLocator` để lấy dependency → đó là DI container viết lại, và mất luôn phần compiler kiểm tra.
- Trong review có câu "đây là pattern gì" mà không ai trả lời được → cái tên đang không giúp gì, cân nhắc bỏ tầng đó.

## Ghi nhớ

- `IEnumerable` là Iterator, `event` và `IProgress<T>` là Observer.
- Middleware và `DelegatingHandler` là Chain of Responsibility.
- DI container đã là Factory; `ServiceLocator` tự viết thì mất phần compiler kiểm tra.
- Command gọn nhất trong C# là một `record` cộng một handler.
- Pattern là từ vựng để gọi tên giải pháp, không phải mục tiêu.

## Bước tiếp theo

Hết chương **Design pattern hay gặp**. Bốn pattern thật sự hay dùng, và sáu cái
framework đã làm sẵn.

Chương cuối của khoá là **Thiết kế để test được**. Nó mở bằng một test suite
chạy mười hai phút, và thỉnh thoảng đỏ mà không ai sửa gì.

Hai chuyện đó đều bắt nguồn từ thiết kế.

```quiz
[
  {
    "prompt": "Bạn định viết một class quản lý chuỗi handler, mỗi handler xử lý xong thì gọi handler tiếp theo. Trong ASP.NET Core, thứ đó đã có tên gì?",
    "options": [
      "Middleware, cho request đi vào",
      "Một Strategy đăng ký theo khoá",
      "Một Decorator bọc nhiều tầng",
      "IHostedService"
    ],
    "answer": 1,
    "explain": "Chuỗi mắt xích gọi tiếp nhau là Chain of Responsibility, và ASP.NET Core dựng sẵn nó thành middleware cho request đi vào. Request HTTP đi ra thì dùng DelegatingHandler."
  },
  {
    "prompt": "yield return trong một method trả về IEnumerable<T> tương ứng với pattern nào?",
    "options": [
      "Observer",
      "Builder",
      "Iterator",
      "Command"
    ],
    "answer": 3,
    "explain": "Iterator là duyệt một tập phần tử mà không lộ cấu trúc bên trong. yield return làm compiler sinh hộ cả state machine, nên tự viết MoveNext và Current là làm lại việc đó bằng tay."
  },
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "object cmd = new ShipOrder(7);\nConsole.WriteLine(cmd.GetType().Name);",
    "options": [
      "Object",
      "ShipOrder",
      "Lỗi compile vì cmd khai kiểu object",
      "Null"
    ],
    "answer": 2,
    "explain": "GetType() hỏi kiểu thật của object lúc chạy, không phải kiểu của biến. Đó cũng là cách một bảng handler tra được đúng hàm xử lý cho mỗi loại command."
  },
  {
    "prompt": "Câu hỏi nào nên hỏi ĐẦU TIÊN trước khi tự viết một pattern?",
    "options": [
      ".NET đã có sẵn thứ này chưa?",
      "Pattern này tên gì trong sách Gang of Four?",
      "Có bao nhiêu class sẽ phải thêm?",
      "Nhóm có ai từng dùng pattern này chưa?"
    ],
    "answer": 1,
    "explain": "Một EventBus 180 dòng trượt ngay câu này, vì Channel<T> cộng BackgroundService đã làm đúng việc đó. Hai câu tiếp theo là: vấn đề đã xảy ra thật chưa, và người mới đọc có hiểu ngay không."
  }
]
```
