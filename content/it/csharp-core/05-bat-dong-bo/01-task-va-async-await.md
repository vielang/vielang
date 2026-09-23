---
title: Task và async await
minutes: 11
---

API của bạn chịu được 200 request mỗi giây rồi đứng. CPU chỉ 15%, database
cũng nhàn. Máy chẳng bận gì cả — nó chỉ đang có vài trăm luồng **đứng chờ**
database trả lời, và không còn luồng nào rảnh để nhận request mới.

> **Học xong bài này bạn sẽ:** hiểu `async`/`await` giải phóng luồng thế nào;
> viết method bất đồng bộ đúng từ controller xuống repository; chạy song song
> nhiều việc độc lập bằng `WhenAll`.
>
> **Cần biết trước:** method và giá trị trả về; LINQ với EF Core ở mức đã gặp.

## await không phải là chờ

```csharp
public async Task<Order?> LayDon(int id)
{
    var don = await db.Orders.FindAsync(id);
    return don;
}
```

Khi chạy tới `await`, method **trả luồng lại** cho hệ thống và đăng ký "xong
thì gọi tôi". Luồng đó quay về phục vụ request khác. Lúc database trả lời, một
luồng rảnh nào đó chạy tiếp phần sau `await`.

Đây là lý do `async` giúp máy chủ chịu tải cao hơn: nó không làm một request
nhanh hơn, nó làm **số request chạy cùng lúc** nhiều hơn.

## Thử ngay: bao lâu cho ba việc

```csharp
async Task<int> Cham(int id)
{
    await Task.Delay(1000);
    return id;
}

var dh = Stopwatch.StartNew();

var a = await Cham(1);
var b = await Cham(2);
var c = await Cham(3);
Console.WriteLine($"Lần lượt: {dh.Elapsed}");

dh.Restart();
var ketQua = await Task.WhenAll(
    Cham(1), Cham(2), Cham(3));
Console.WriteLine($"Song song: {dh.Elapsed}");
```

**Đoán trước khi chạy:** mỗi việc mất 1 giây. Hai con số in ra là bao nhiêu?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Lần lượt: 00:00:03.0131
Song song: 00:00:01.0062
```

Ba `await` liên tiếp chạy **tuần tự**: `await` đầu phải xong mới tới cái sau.
`Task.WhenAll` khởi động cả ba rồi mới chờ, nên tổng thời gian bằng việc lâu
nhất.

</details>

Chỉ dùng `WhenAll` khi các việc **độc lập** với nhau. Ba câu truy vấn trên
cùng một `DbContext` thì **không** được: `DbContext` không an toàn khi dùng
song song.

## Viết async cho đúng

```csharp
// SAI — chặn luồng, mất sạch lợi ích của async
public Order Lay(int id)
{
    return db.Orders.FindAsync(id).Result;
}
```

```csharp
// ĐÚNG — async suốt đường từ controller xuống
public async Task<Order?> Lay(int id)
{
    return await db.Orders.FindAsync(id);
}
```

`async` phải đi **suốt chuỗi gọi**: controller `async`, service `async`,
repository `async`. Chỉ một chỗ gọi `.Result` hay `.Wait()` là cả chuỗi mất
tác dụng, và tệ hơn — xem bài sau về deadlock.

## Task, Task&lt;T&gt; và ValueTask

- `Task` — việc không trả giá trị (như `void` nhưng bất đồng bộ).
- `Task<T>` — việc trả về `T`.
- `ValueTask<T>` — dành cho đường chạy thường xong ngay (đọc cache); chỉ dùng khi đã đo và thấy cần.

```csharp
public async Task GhiLog(string s) { }
public async Task<int> Dem() => 1;
```

Method `async` mà không có `await` nào bên trong thì compiler cảnh báo — đó là
dấu hiệu bạn đánh dấu `async` thừa, hoặc quên `await`.

## Đặt tên và quy ước

```csharp
public Task<List<Order>> LayDonAsync(
    CancellationToken ct = default)
{
    return db.Orders.ToListAsync(ct);
}
```

- Hậu tố `Async` cho method trả `Task` — quy ước chung của .NET.
- Nhận `CancellationToken` và chuyền nó xuống dưới; bài sau nói kỹ.
- Method chỉ trả thẳng một `Task` thì **không cần** `async`/`await` — trả luôn `Task` cho gọn.

## Async không phải đa luồng

`async` không tự tạo luồng mới. Với việc **chờ I/O** (database, HTTP, file),
không có luồng nào bị chiếm trong lúc chờ cả. Còn việc **nặng CPU** (tính
toán, nén ảnh) thì `async` không giúp gì — thứ bạn cần là
`Task.Run` để đẩy sang luồng khác, hoặc xử lý nền bằng hàng đợi.

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

Bài cuối của khoá — **Bẫy async thường gặp** — ba thứ làm app treo hoặc nuốt
lỗi trong im lặng: `async void`, `.Result`, và quên `CancellationToken`.

```quiz
[
  {
    "prompt": "Ba việc độc lập, mỗi việc chờ I/O 1 giây. Đoạn này mất bao lâu?",
    "code": "var a = await Cham(1);\nvar b = await Cham(2);\nvar c = await Cham(3);",
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
