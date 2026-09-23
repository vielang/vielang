---
title: Bẫy async thường gặp
minutes: 11
---

Một job nền gửi email chạy `async void`. Hôm SMTP lỗi, job ném exception —
và không có gì trong log cả. Tệ hơn: tiến trình chết hẳn, container khởi động
lại, không ai hiểu vì sao. Cùng đoạn code đó viết `async Task` thì lỗi đã nằm
gọn trong log.

> **Học xong bài này bạn sẽ:** tránh ba cái bẫy async làm app treo hoặc nuốt
> lỗi; chuyền `CancellationToken` cho đúng; và biết `ConfigureAwait` dùng ở
> đâu.
>
> **Cần biết trước:** `async`/`await` và `Task` (bài trước).

## Bẫy 1: async void

```csharp
// SAI — không ai await được, lỗi không bắt được
public async void GuiMail(string to)
{
    await smtp.SendAsync(to);
}
```

```csharp
// ĐÚNG — trả Task để người gọi await và bắt lỗi
public async Task GuiMailAsync(string to)
{
    await smtp.SendAsync(to);
}
```

`async void` không trả về gì để `await`, nên exception bên trong **không bay
lên người gọi** — nó rơi thẳng vào runtime và hạ luôn tiến trình. Ngoại lệ duy
nhất được phép dùng `async void` là event handler của UI, vì chữ ký hàm bắt
buộc thế.

## Thử ngay: quên await

```csharp
async Task Nem()
{
    await Task.Delay(50);
    throw new Exception("hỏng");
}

try
{
    Nem();                 // KHÔNG await
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

**Không bắt được gì cả.** `Nem()` không có `await` nên nó chỉ trả về một
`Task` rồi đi tiếp; exception nằm im trong `Task` đó, và vì không ai hỏi tới
nên nó biến mất lặng lẽ.

Thêm `await` vào trước `Nem()` rồi chạy lại: dòng `bắt được: hỏng` xuất hiện.

</details>

Compiler có cảnh báo CS4014 cho trường hợp này. Đừng tắt nó.

## Bẫy 2: .Result và .Wait()

```csharp
// SAI — chặn luồng, và treo cứng trong vài môi trường
var don = LayDonAsync(id).Result;
```

Ngoài chuyện chặn luồng đã nói ở bài trước, `.Result` còn gây **deadlock**
trong các môi trường có SynchronizationContext (WinForms, WPF, ASP.NET cũ):
luồng chính đứng chờ task, mà task lại cần chính luồng đó để chạy tiếp. Cả hai
chờ nhau vĩnh viễn.

ASP.NET Core hiện đại không còn SynchronizationContext nên ít treo hơn, nhưng
chặn luồng vẫn là chặn luồng. Quy tắc: **async suốt đường**, không `.Result`.

## Bẫy 3: quên CancellationToken

```csharp
// SAI — người dùng đóng tab, truy vấn vẫn chạy tiếp
[HttpGet]
public async Task<IActionResult> Tim(string q)
{
    var kq = await db.Orders
        .Where(o => o.Ma.Contains(q))
        .ToListAsync();

    return Ok(kq);
}
```

```csharp
// ĐÚNG — huỷ được, chuyền token xuống tận database
[HttpGet]
public async Task<IActionResult> Tim(
    string q, CancellationToken ct)
{
    var kq = await db.Orders
        .Where(o => o.Ma.Contains(q))
        .ToListAsync(ct);

    return Ok(kq);
}
```

ASP.NET Core tự cấp `CancellationToken` cho action và huỷ nó khi client ngắt
kết nối. Chuyền nó xuống mọi lời gọi bất đồng bộ thì truy vấn nặng sẽ dừng
thay vì chạy tiếp cho một người đã bỏ đi.

Trong job nền, `ct` đến từ host và được huỷ khi ứng dụng tắt — nhờ vậy
container dừng gọn thay vì bị giết cứng.

## ConfigureAwait(false)

```csharp
await smtp.SendAsync(to).ConfigureAwait(false);
```

Nói với runtime: "chạy tiếp ở luồng nào cũng được". Cần trong **thư viện dùng
chung**, vì bạn không biết người dùng thư viện chạy trong môi trường nào. Code
ứng dụng ASP.NET Core thì **không cần** — ở đó không có
SynchronizationContext để quay về.

## Chạy nền cho đúng

```csharp
// SAI — fire-and-forget, lỗi biến mất
_ = GuiMailAsync(to);
```

Muốn làm việc gì đó sau khi trả response, hãy dùng thứ được thiết kế cho việc
ấy: `IHostedService`/`BackgroundService`, hàng đợi (Channel, RabbitMQ), hoặc
một job runner. Chúng có chỗ ghi log, có cơ chế thử lại, và tắt máy cũng
không mất việc.

## Dấu hiệu trong code của bạn

- `async void` ngoài event handler → đổi sang `async Task`.
- Cảnh báo CS4014 (gọi method async mà không `await`) bị bỏ qua hoặc tắt.
- `.Result` / `.Wait()` / `.GetAwaiter().GetResult()` trong controller, service hay repository.
- Action hay method async không nhận `CancellationToken`, hoặc nhận rồi không chuyền xuống.
- `_ = SomethingAsync();` để "chạy nền" → không có log, không có thử lại, mất việc khi tắt máy.

## Ghi nhớ

- `async void` chỉ dành cho event handler; còn lại luôn `async Task`.
- Quên `await` là exception biến mất lặng lẽ — tôn trọng cảnh báo CS4014.
- Không `.Result`, không `.Wait()`; async suốt đường.
- Chuyền `CancellationToken` từ controller xuống tận truy vấn.
- `ConfigureAwait(false)` cho thư viện, không cần cho app ASP.NET Core.

## Bước tiếp theo

Bạn vừa học hết khoá **C# Core**: cú pháp, kiểu dữ liệu và bộ nhớ, collection
và LINQ, xử lý lỗi và tài nguyên, bất đồng bộ. Đây là nền để đi tiếp sang
**OOP và design principles**, rồi **ASP.NET Core** và **SQL/Oracle** — những
thứ tạo nên công việc hằng ngày của một backend developer.

```quiz
[
  {
    "prompt": "Job nền viết public async void GuiMail(). SMTP lỗi thì chuyện gì xảy ra?",
    "options": [
      "Lỗi được ghi log như bình thường",
      "Exception không bay lên người gọi được, rơi vào runtime và có thể hạ cả tiến trình",
      "Task bị huỷ, các job khác vẫn chạy",
      "Compiler chặn từ lúc build"
    ],
    "answer": 2,
    "explain": "async void không có Task để ai đó await, nên không có đường cho exception đi lên. Luôn dùng async Task trừ event handler."
  },
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "try\n{\n    Nem();   // async Task, ném lỗi sau 50ms\n    await Task.Delay(200);\n}\ncatch (Exception ex)\n{\n    Console.WriteLine(\"bắt được\");\n}",
    "options": [
      "\"bắt được\"",
      "Không in gì từ catch — lỗi nằm im trong Task không ai await",
      "Chương trình dừng ngay ở Nem()",
      "Lỗi compile"
    ],
    "answer": 2,
    "explain": "Thiếu await nên Nem() chỉ trả về một Task; exception nằm trong Task đó và biến mất lặng lẽ. Compiler có cảnh báo CS4014 cho đúng trường hợp này."
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
      "_ = GuiMailAsync(to); rồi return luôn",
      "Đưa việc vào hàng đợi hoặc BackgroundService",
      "GuiMailAsync(to).Wait(); trước khi return",
      "Gọi async void cho khỏi phải await"
    ],
    "answer": 2,
    "explain": "Fire-and-forget không có log, không thử lại và mất việc khi tắt máy. Hàng đợi hoặc BackgroundService được thiết kế đúng cho việc này."
  }
]
```
