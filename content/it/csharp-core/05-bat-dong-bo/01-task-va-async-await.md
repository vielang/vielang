---
title: Task và async await
minutes: 11
---

API của bạn chịu được 200 request mỗi giây rồi đứng.

CPU chỉ 15%. Database cũng nhàn. Máy chẳng bận gì cả.

Nó chỉ đang có vài trăm luồng **đứng chờ** database trả lời, và không còn luồng
nào rảnh để nhận request mới.

> **Học xong bài này bạn sẽ:** hiểu `async`/`await` giải phóng luồng thế nào;
> viết method bất đồng bộ đúng từ controller xuống repository; chạy song song
> nhiều việc độc lập bằng `WhenAll`.
>
> **Cần biết trước:** method và giá trị trả về; LINQ với EF Core ở mức đã gặp.

## await trả luồng lại, chứ không đứng chờ

```csharp
public async Task<Order?> GetOrder(int id)
{
    var order = await db.Orders.FindAsync(id);
    return order;
}
```

Chạy tới `await`, method trả luồng lại cho hệ thống và đăng ký một lời nhắn:
xong thì gọi tôi.

Luồng đó quay về phục vụ request khác. Lúc database trả lời, một luồng rảnh
nào đó sẽ chạy tiếp phần sau `await`.

Đây là lý do `async` giúp máy chịu tải cao hơn. Nó không làm một request nhanh
hơn, nó làm **số request chạy cùng lúc** nhiều hơn.

## Thử ngay: ba việc tuần tự mất 3 giây, song song mất 1

```csharp
using System.Diagnostics;

async Task<int> Slow(int id)
{
    await Task.Delay(1000);
    return id;
}

var sw = Stopwatch.StartNew();

var a = await Slow(1);
var b = await Slow(2);
var c = await Slow(3);
Console.WriteLine($"Lần lượt: {sw.Elapsed}");

sw.Restart();
var results = await Task.WhenAll(
    Slow(1), Slow(2), Slow(3));
Console.WriteLine($"Song song: {sw.Elapsed}");
```

**Đoán trước khi chạy:** mỗi việc mất 1 giây. Hai con số in ra là bao nhiêu?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Lần lượt: 00:00:03.0131
Song song: 00:00:01.0062
```

Ba `await` liên tiếp chạy tuần tự. Cái đầu phải xong thì cái sau mới bắt đầu.

`Task.WhenAll` khởi động cả ba trước rồi mới chờ, nên tổng thời gian chỉ bằng
việc lâu nhất.

</details>

Chỉ gom bằng `WhenAll` khi các việc thật sự **độc lập**.

Ba câu truy vấn trên cùng một `DbContext` thì không được. `DbContext` không an
toàn khi dùng song song.

## async phải đi suốt chuỗi gọi, đứt một chỗ là hỏng

```csharp
// SAI — chặn luồng, mất sạch lợi ích của async
public Order Get(int id)
{
    return db.Orders.FindAsync(id).Result;
}
```

```csharp
// ĐÚNG — async suốt đường từ controller xuống
public async Task<Order?> Get(int id)
{
    return await db.Orders.FindAsync(id);
}
```

Controller `async`, service `async`, repository `async`. Cả chuỗi phải liền
mạch.

Chỉ một chỗ gọi `.Result` hay `.Wait()` là luồng bị chặn trở lại. Tệ hơn nữa
là deadlock, và bài sau sẽ cho bạn thấy nó xảy ra thế nào.

## Task, Task&lt;T&gt; hay ValueTask tuỳ việc trả về gì

| Kiểu trả về | Dùng cho | Ghi chú |
|---|---|---|
| `Task` | việc không trả giá trị | như `void` nhưng bất đồng bộ |
| `Task<T>` | việc trả về một `T` | dạng hay gặp nhất |
| `ValueTask<T>` | đường chạy thường xong ngay | chỉ dùng khi đã đo và thấy cần |

```csharp
public async Task WriteLog(string message) { }
public async Task<int> Count() => 1;
```

Method đánh dấu `async` mà bên trong không có `await` nào thì compiler cảnh
báo ngay. Đó là dấu hiệu bạn gắn `async` thừa, hoặc quên mất một `await`.

## Hậu tố Async và CancellationToken là quy ước chung

```csharp
public Task<List<Order>> GetOrdersAsync(
    CancellationToken ct = default)
{
    return db.Orders.ToListAsync(ct);
}
```

Hậu tố `Async` là quy ước của cả .NET. Nhìn tên là người gọi biết phải `await`.

`CancellationToken` thì nhận vào rồi chuyền thẳng xuống dưới. Bài sau nói kỹ
vì sao nó quan trọng.

Để ý method này không có `async` lẫn `await`. Nó chỉ trả thẳng một `Task` sẵn
có, nên thêm vào cũng chẳng để làm gì.

## async giúp việc chờ I/O, không giúp việc nặng CPU

| Loại việc | async có giúp không | Nên dùng gì |
|---|---|---|
| Chờ database, HTTP, file | **có** | `async`/`await` |
| Nén ảnh, tính toán nặng | không | `Task.Run` hoặc hàng đợi nền |

`async` không tự tạo ra luồng mới. Với việc chờ I/O, không có luồng nào bị
chiếm trong suốt thời gian chờ cả.

Còn việc nặng CPU thì vẫn phải có ai đó ngồi tính. `async` không làm phép màu
ở đây được.

## Dấu hiệu trong code của bạn

- `.Result`, `.Wait()`, `.GetAwaiter().GetResult()` trong code web → chặn luồng, và là nguồn của deadlock.
- Controller `async` nhưng repository vẫn gọi `ToList()` đồng bộ → chuỗi async đứt ở đó.
- Nhiều `await` liên tiếp cho các việc độc lập → gom bằng `Task.WhenAll`.
- Method trả `Task` mà thiếu hậu tố `Async` → người gọi khó nhận ra là phải `await`.
- `async void` ở bất cứ đâu ngoài event handler → bài sau nói vì sao đây là cái bẫy nặng nhất.

## Ghi nhớ

- `await` trả luồng lại cho hệ thống, không chiếm chỗ trong lúc chờ.
- `async` làm máy chịu tải cao hơn, không làm một request nhanh hơn.
- Việc độc lập thì `Task.WhenAll`; đừng dùng chung một `DbContext` cho chúng.
- `async` phải đi suốt chuỗi gọi; một chỗ `.Result` là hỏng cả chuỗi.
- Việc nặng CPU thì `async` không giúp gì.

## Bước tiếp theo

Bạn đã biết `async` làm gì. Giờ tới lúc biết nó hỏng ở đâu.

Bài cuối của khoá, **Bẫy async thường gặp**, nói về ba thứ làm app treo hoặc
nuốt lỗi trong im lặng: `async void`, `.Result`, và quên `CancellationToken`.

```quiz
[
  {
    "prompt": "Ba việc độc lập, mỗi việc chờ I/O 1 giây. Đoạn này mất bao lâu?",
    "code": "var a = await Slow(1);\nvar b = await Slow(2);\nvar c = await Slow(3);",
    "options": ["~1 giây", "~3 giây", "~9 giây", "Tuỳ số nhân CPU"],
    "answer": 2,
    "explain": "await chờ xong việc này mới bắt đầu việc sau. Muốn ~1 giây thì khởi động cả ba rồi await Task.WhenAll(...)."
  },
  {
    "prompt": "API dùng async/await đúng cách. Điều nào ĐÚNG?",
    "options": [
      "Mỗi request chạy nhanh hơn hẳn",
      "Máy chủ phục vụ được nhiều request cùng lúc hơn, vì luồng không bị chiếm khi chờ I/O",
      "Mỗi request được cấp một luồng riêng",
      "CPU được dùng triệt để hơn cho việc tính toán"
    ],
    "answer": 2,
    "explain": "async không rút ngắn thời gian chờ database; nó trả luồng lại để phục vụ request khác trong lúc chờ."
  },
  {
    "prompt": "Repository viết return db.Orders.FindAsync(id).Result; trong một API. Vấn đề gì?",
    "options": [
      "Không có vấn đề, chỉ là cách viết khác",
      "Chặn luồng trong lúc chờ, mất lợi ích của async và có nguy cơ deadlock",
      "Kết quả trả về sai kiểu",
      "Chậm hơn vì phải tạo thêm Task"
    ],
    "answer": 2,
    "explain": "gọi .Result là đứng chờ ngay trên luồng hiện tại. Chuỗi async phải liền mạch từ controller xuống repository."
  },
  {
    "prompt": "Việc nào KHÔNG nên gom bằng Task.WhenAll?",
    "options": [
      "Gọi ba API bên ngoài khác nhau",
      "Đọc ba file khác nhau",
      "Ba truy vấn trên cùng một DbContext",
      "Gửi ba email qua ba HttpClient"
    ],
    "answer": 3,
    "explain": "DbContext không an toàn khi dùng song song. Cần chạy song song thì mỗi việc một context riêng, hoặc chạy tuần tự."
  }
]
```
