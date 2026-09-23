---
title: Task và async await
minutes: 11
---

Khi API của bạn ngồi chờ database trả lời, luồng đang làm gì?

Câu hỏi nghe hiền lành. Nó cũng là câu hay được hỏi lúc phỏng vấn, và trả lời
sai thì bạn có một API chịu được 200 request mỗi giây rồi đứng.

CPU mới 15%. Database thì nhàn. Máy chẳng bận gì cả.

> **Học xong bài này bạn sẽ:** hiểu `async`/`await` giải phóng luồng thế nào;
> viết method bất đồng bộ đúng từ controller xuống repository; chạy song song
> nhiều việc độc lập bằng `WhenAll`.
>
> **Cần biết trước:** method và giá trị trả về; LINQ với EF Core ở mức đã gặp.

## await trả luồng lại, chứ không đứng chờ

Máy rảnh mà vẫn không nhận thêm request. Chỗ hỏng nằm trong chữ *chờ*.

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

## Thử ngay: đo ba việc chờ I/O, tuần tự và song song

Một `await` trả luồng lại. Nhưng ba `await` liên tiếp thì có chạy cùng lúc
không?

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

Biết `await` làm gì rồi, còn phải biết chỗ nào làm nó mất tác dụng.

```csharp
// SAI — chặn luồng, mất sạch lợi ích của async
public List<Order> GetAll()
{
    return db.Orders.ToListAsync().Result;
}
```

```csharp
// ĐÚNG — async suốt đường từ controller xuống
public async Task<List<Order>> GetAllAsync()
{
    return await db.Orders.ToListAsync();
}
```

Controller `async`, service `async`, repository `async`. Cả chuỗi phải liền
mạch.

Chỉ một chỗ gọi `.Result` hay `.Wait()` là luồng bị chặn trở lại.

Mỗi luồng bị chặn là một luồng nằm chờ mà không phục vụ ai. Đủ nhiều thì ra
đúng cái API 200 request một giây với CPU 15% ở đầu bài.

## Task, Task&lt;T&gt; hay ValueTask tuỳ việc trả về gì

Method bất đồng bộ trả về một trong ba kiểu, và chọn đúng thì không phải nghĩ
lâu.

| Kiểu trả về | Dùng cho | Ghi chú |
|---|---|---|
| `Task` | việc không trả giá trị | như `void` nhưng bất đồng bộ |
| `Task<T>` | việc trả về một `T` | dạng hay gặp nhất |
| `ValueTask<T>` | đường chạy thường xong ngay | chỉ dùng khi đã đo và thấy cần |

```csharp
public async Task WriteLogAsync(string msg) =>
    await File.AppendAllTextAsync(path, msg);

public async Task<int> CountPaidAsync() =>
    await db.Orders.CountAsync(o => o.IsPaid);
```

Method đánh dấu `async` mà bên trong không có `await` nào thì compiler cảnh
báo ngay. Đó là dấu hiệu bạn gắn `async` thừa, hoặc quên mất một `await`.

## Hậu tố Async và CancellationToken là quy ước chung

Hai thứ nữa bạn sẽ thấy trong mọi chữ ký hàm bất đồng bộ của .NET.

```csharp
public Task<List<Order>> GetOrdersAsync(
    CancellationToken ct = default)
{
    return db.Orders.ToListAsync(ct);
}
```

Hậu tố `Async` là quy ước của cả .NET. Nhìn tên là người gọi biết phải `await`.

`CancellationToken` thì nhận vào rồi chuyền thẳng xuống dưới. Người dùng đóng
tab là truy vấn dừng theo.

Để ý method này không có `async` lẫn `await`. Nó chỉ trả thẳng một `Task` sẵn
có, nên thêm vào cũng chẳng để làm gì.

## async giúp việc chờ I/O, không giúp việc nặng CPU

Còn một hiểu nhầm phổ biến cần dẹp trước khi sang bài sau.

| Loại việc | async có giúp không | Nên dùng gì |
|---|---|---|
| Chờ database, HTTP, file | **có** | `async`/`await` |
| Nén ảnh, tính toán nặng | không | `Task.Run` hoặc hàng đợi nền |

`async` không tự tạo ra luồng mới. Với việc chờ I/O, không có luồng nào bị
chiếm trong suốt thời gian chờ cả.

Còn việc nặng CPU thì vẫn phải có ai đó ngồi tính. `async` không rút ngắn được
một giây nào của phép tính ấy.

## Dấu hiệu trong code của bạn

- `.Result`, `.Wait()`, `.GetAwaiter().GetResult()` trong code web → chặn luồng, và là nguồn của deadlock.
- Controller `async` nhưng repository vẫn gọi `ToList()` đồng bộ → chuỗi async đứt ở đó.
- Nhiều `await` liên tiếp cho các việc độc lập → gom bằng `Task.WhenAll`.
- Method trả `Task` mà thiếu hậu tố `Async` → người gọi khó nhận ra là phải `await`.
- `async void` ở bất cứ đâu ngoài event handler → exception không có đường bay lên, và hạ cả tiến trình.

## Ghi nhớ

- `await` trả luồng lại cho hệ thống, không chiếm chỗ trong lúc chờ.
- `async` làm máy chịu tải cao hơn, không làm một request nhanh hơn.
- Việc độc lập thì `Task.WhenAll`; đừng dùng chung một `DbContext` cho chúng.
- `async` phải đi suốt chuỗi gọi; một chỗ `.Result` là hỏng cả chuỗi.
- Việc nặng CPU thì `async` không giúp gì.

## Bước tiếp theo

Bạn đã biết `async` làm gì. Giờ tới lúc biết nó hỏng ở đâu.

Bài cuối của khoá, **Bẫy async thường gặp**, nói về bốn thứ làm app treo hoặc
nuốt lỗi trong im lặng: `async void`, quên `await`, `.Result`, và
fire-and-forget.

```quiz
[
  {
    "prompt": "Một action gọi ba service độc lập, mỗi service chờ HTTP khoảng 400ms. Action này mất bao lâu?",
    "code": "var rate = await rates.GetAsync(code);\nvar stock = await warehouse.GetAsync(id);\nvar ship = await shipping.QuoteAsync(id);",
    "options": [
      "~400ms",
      "Tuỳ số nhân CPU",
      "~1,2 giây",
      "~400ms, vì await chạy song song"
    ],
    "answer": 3,
    "explain": "Ba await liên tiếp là tuần tự: cái đầu xong thì cái sau mới bắt đầu, nên ba lần 400ms cộng lại. Ba service không phụ thuộc nhau, nên khởi động cả ba rồi await Task.WhenAll(...) là về lại ~400ms."
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
    "prompt": "Repository viết return db.Orders.ToListAsync().Result; trong một API. Vấn đề gì?",
    "options": [
      "Không có vấn đề, chỉ là cách viết khác",
      "Kết quả trả về sai kiểu",
      "Chặn luồng trong lúc chờ, mất lợi ích của async và có nguy cơ deadlock",
      "Chậm hơn vì phải tạo thêm Task"
    ],
    "answer": 3,
    "explain": "gọi .Result là đứng chờ ngay trên luồng hiện tại. Chuỗi async phải liền mạch từ controller xuống repository."
  },
  {
    "prompt": "Việc nào KHÔNG nên gom bằng Task.WhenAll?",
    "options": [
      "Ba truy vấn trên cùng một DbContext",
      "Đọc ba file khác nhau",
      "Gọi ba API bên ngoài khác nhau",
      "Gửi ba email qua ba HttpClient"
    ],
    "answer": 1,
    "explain": "DbContext không an toàn khi dùng song song. Cần chạy song song thì mỗi việc một context riêng, hoặc chạy tuần tự."
  }
]
```
