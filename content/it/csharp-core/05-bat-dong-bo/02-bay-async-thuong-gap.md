---
title: Bẫy async thường gặp
minutes: 11
---

Một job nền gửi email chạy `async void`. Hôm SMTP lỗi, job ném exception, và
trong log không có gì cả.

Tệ hơn nữa: tiến trình chết hẳn, container khởi động lại, không ai hiểu vì
sao.

Cùng đoạn code ấy trả `async Task`, rồi để `BackgroundService` await nó, thì lỗi
đã nằm gọn trong log.

> **Học xong bài này bạn sẽ:** tránh bốn cái bẫy async làm app treo hoặc nuốt
> lỗi; chuyền `CancellationToken` cho đúng; và biết `ConfigureAwait` dùng ở
> đâu.
>
> **Cần biết trước:** `async`/`await` và `Task` (bài trước). Vài ví dụ dùng
> controller và `BackgroundService` của ASP.NET Core, nhưng cái bẫy thì giống
> nhau ở mọi loại app.

## Bốn cái bẫy, và thứ thay thế chúng

Job gửi mail kia mắc cái bẫy thứ nhất. Ba cái còn lại cũng đều làm lỗi biến
mất, mỗi cái theo một đường.

| Bẫy | Hậu quả | Thay bằng |
|---|---|---|
| `async void` | exception hạ cả tiến trình | `async Task` |
| Quên `await` | lỗi biến mất lặng lẽ | `await DoAsync()` |
| `.Result`, `.Wait()` | chặn luồng, có nơi treo cứng | async suốt đường |
| `_ = DoAsync()` | không log, tắt máy là mất việc | hàng đợi hoặc `BackgroundService` |

Bốn dòng này là bốn cái bẫy. Hai mục cuối bài nói tiếp hai thứ đi kèm với
chúng: `CancellationToken` và `ConfigureAwait`.

## async void nuốt exception và hạ cả tiến trình

Bắt đầu từ dòng đầu bảng, vì nó là cái bẫy làm chết tiến trình.

```csharp
// SAI — không ai await được, lỗi không bắt được
public async void SendMail(string to)
{
    await smtp.SendAsync(to);
}
```

```csharp
// ĐÚNG — trả Task để người gọi await và bắt lỗi
public async Task SendMailAsync(string to)
{
    await smtp.SendAsync(to);
}
```

`async void` không trả về gì để `await`. Exception bên trong vì thế không có
đường bay lên người gọi.

Nó rơi thẳng vào runtime và hạ luôn tiến trình. Ngoại lệ duy nhất được phép là
event handler của UI, vì chữ ký hàm bắt buộc phải thế.

## Thử ngay: gọi một method async mà không await

Cái bẫy thứ hai không cần `async void`. Chỉ cần bạn quên một chữ.

```csharp
async Task Throw()
{
    await Task.Delay(50);
    throw new Exception("hỏng");
}

try
{
    Throw();               // KHÔNG await
    Console.WriteLine("qua được try");
    await Task.Delay(200);
}
catch (Exception ex)
{
    Console.WriteLine($"bắt được: {ex.Message}");
}

Console.WriteLine("hết");
```

**Đoán trước khi chạy:** khối `catch` có bắt được lỗi không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
qua được try
hết
```

Không bắt được gì cả.

`Throw()` thiếu `await` nên nó chỉ trả về một `Task` rồi đi tiếp. Exception
nằm im trong `Task` đó, và vì không ai hỏi tới nên nó biến mất.

Thêm `await` vào trước `Throw()` rồi chạy lại. Dòng `bắt được: hỏng` xuất hiện
ngay.

</details>

Compiler có cảnh báo CS4014 cho đúng trường hợp này. Đừng tắt nó.

## .Result chặn luồng, và có nơi còn treo cứng

Hai bẫy trên làm mất lỗi. Cái thứ ba thì làm mất luôn cả app.

```csharp
// SAI — chặn luồng, và treo cứng ở vài môi trường
var order = GetOrderAsync(id).Result;
```

Chặn luồng thì bài trước đã nói. Nhưng `.Result` còn gây **deadlock** ở những
môi trường có SynchronizationContext, như WinForms, WPF hay ASP.NET cũ.

Luồng chính đứng chờ task. Task lại cần chính luồng đó để chạy tiếp. Hai bên
chờ nhau vĩnh viễn.

ASP.NET Core hiện đại bỏ SynchronizationContext nên ít treo hơn. Nhưng chặn
luồng vẫn là chặn luồng.

## CancellationToken phải chuyền xuống tận truy vấn

Hết bốn cái bẫy. Còn hai thứ đi kèm với chúng, và đây là thứ nhất.

```csharp
// SAI — người dùng đóng tab, truy vấn vẫn chạy
[HttpGet]
public async Task<IActionResult> Search(string q)
{
    var rows = await db.Orders
        .Where(o => o.Code.Contains(q))
        .ToListAsync();

    return Ok(rows);
}
```

```csharp
// ĐÚNG — token đi xuống tận database
[HttpGet]
public async Task<IActionResult> Search(
    string q, CancellationToken ct)
{
    var rows = await db.Orders
        .Where(o => o.Code.Contains(q))
        .ToListAsync(ct);

    return Ok(rows);
}
```

ASP.NET Core tự cấp token cho action và huỷ nó khi client ngắt kết nối. Bạn
chỉ cần khai vào tham số.

Chuyền nó xuống mọi lời gọi bất đồng bộ thì truy vấn nặng sẽ dừng, thay vì
chạy tiếp cho một người đã bỏ đi.

Job nền cũng có token như vậy, chỉ khác nguồn: nó đến từ host, và bị huỷ lúc
ứng dụng nhận lệnh tắt.

Nhận rồi chuyền xuống thì job kịp dừng trong thời gian ân hạn, thay vì bị cắt
giữa việc.

## ConfigureAwait(false) dành cho thư viện, không cho app

Thứ thứ hai bạn sẽ gặp trong code thư viện, và hay bị dán vào chỗ không cần.

| Bạn đang viết | Cần `ConfigureAwait(false)` |
|---|---|
| Thư viện dùng chung, NuGet package | **có** |
| App ASP.NET Core | không |
| App WinForms, WPF | có, ở tầng không chạm UI |

```csharp
// trong một NuGet package dùng chung
await transport.SendAsync(msg)
    .ConfigureAwait(false);
```

Câu này nói với runtime rằng chạy tiếp ở luồng nào cũng được. Thư viện cần nó
vì bạn không biết người dùng thư viện chạy trong môi trường nào.

Còn app ASP.NET Core thì không có SynchronizationContext để quay về, nên thêm
vào không đổi được gì, chỉ dài dòng thêm.

## Chạy nền cần hàng đợi, không phải fire-and-forget

Quay lại dòng cuối bảng bẫy, vì nó là chỗ người ta hay nghĩ mình đang làm đúng.

```csharp
// SAI — lỗi biến mất, tắt máy là mất việc
_ = SendMailAsync(to);
```

Muốn làm gì đó sau khi đã trả response, hãy dùng thứ được thiết kế cho việc
ấy.

`BackgroundService`, một hàng đợi như Channel hay RabbitMQ, hoặc một job
runner. Chúng có chỗ ghi log, có cơ chế thử lại, và tắt máy cũng không mất
việc.

## Dấu hiệu trong code của bạn

- `async void` ngoài event handler → đổi sang `async Task`.
- Cảnh báo CS4014 (gọi method async mà không `await`) bị bỏ qua hoặc tắt.
- `.Result` / `.Wait()` / `.GetAwaiter().GetResult()` trong controller, service hay repository.
- Action hay method async không nhận `CancellationToken`, hoặc nhận rồi không chuyền xuống.
- `_ = SomethingAsync();` để "chạy nền" → không có log, không có thử lại, mất việc khi tắt máy.

## Ghi nhớ

- Không có `Task` thì không có đường cho exception đi lên — đó là toàn bộ lý do của `async Task`.
- Lỗi nằm trong một `Task` không ai `await` thì im lặng biến mất, kể cả khi đã `async Task`.
- `.Result` chặn luồng ở mọi nơi, và treo cứng ở nơi có SynchronizationContext.
- Token chỉ có tác dụng ở lời gọi cuối cùng; chuyền thiếu một tầng là mất.
- `ConfigureAwait(false)` cho thư viện, không cần cho app ASP.NET Core.

## Bước tiếp theo

Bạn vừa học hết khoá **C# Core**. Cú pháp, kiểu dữ liệu và bộ nhớ, collection
và LINQ, xử lý lỗi và tài nguyên, bất đồng bộ.

Đây là nền để đi tiếp sang **OOP và thiết kế**, rồi **ASP.NET Core** và
**SQL**. Những thứ làm nên công việc hằng ngày của một backend developer.

```quiz
[
  {
    "prompt": "Job nền viết public async void SendMail(). SMTP lỗi thì chuyện gì xảy ra?",
    "options": [
      "Exception không bay lên người gọi được, rơi vào runtime và có thể hạ cả tiến trình",
      "Lỗi được ghi log như bình thường",
      "Task bị huỷ, các job khác vẫn chạy",
      "Compiler chặn từ lúc build"
    ],
    "answer": 1,
    "explain": "async void không có Task để ai đó await, nên không có đường cho exception đi lên. Luôn dùng async Task trừ event handler."
  },
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "try\n{\n    Throw();   // async Task, ném lỗi sau 50ms\n    await Task.Delay(200);\n}\ncatch\n{\n    Console.WriteLine(\"bắt được\");\n}",
    "options": [
      "\"bắt được\"",
      "Chương trình dừng ngay ở Throw()",
      "Không in gì từ catch — lỗi nằm im trong Task không ai await",
      "Lỗi compile"
    ],
    "answer": 3,
    "explain": "Thiếu await nên Throw() chỉ trả về một Task; exception nằm trong Task đó và biến mất lặng lẽ. Compiler có cảnh báo CS4014 cho đúng trường hợp này."
  },
  {
    "prompt": "Người dùng đóng tab khi truy vấn tìm kiếm đang chạy. Làm sao để truy vấn dừng theo?",
    "options": [
      "Không làm được gì, phải đợi truy vấn xong",
      "Nhận CancellationToken ở action và chuyền xuống ToListAsync(ct)",
      "Gọi Thread.Abort trong middleware",
      "Đặt timeout ngắn cho mọi truy vấn"
    ],
    "answer": 2,
    "explain": "ASP.NET Core huỷ token khi client ngắt kết nối. Token chỉ có tác dụng nếu bạn chuyền nó xuống tận lời gọi bất đồng bộ dưới cùng."
  },
  {
    "prompt": "Bạn muốn gửi email sau khi trả response cho người dùng. Cách nào đúng?",
    "options": [
      "_ = SendMailAsync(to); rồi return luôn",
      "Gọi async void cho khỏi phải await",
      "SendMailAsync(to).Wait(); trước khi return",
      "Đưa việc vào hàng đợi hoặc BackgroundService"
    ],
    "answer": 4,
    "explain": "Fire-and-forget không có log, không thử lại và mất việc khi tắt máy. Hàng đợi hoặc BackgroundService được thiết kế đúng cho việc này."
  }
]
```
