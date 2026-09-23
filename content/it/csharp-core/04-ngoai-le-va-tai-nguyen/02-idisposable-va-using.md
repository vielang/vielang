---
title: IDisposable và using
minutes: 10
---

Service chạy êm vài giờ rồi bắt đầu trả lỗi.

```text
SocketException: Only one usage of each socket
address is normally permitted
```

Khởi động lại là hết. Vài giờ sau lại thế.

Nguyên nhân nằm ở một dòng trông vô hại: `new HttpClient()` trong mỗi request.

> **Học xong bài này bạn sẽ:** biết thứ gì GC dọn được và thứ gì không; dùng
> `using` đúng chỗ; và tránh hai cái bẫy tài nguyên hay gặp nhất trong app
> .NET là `HttpClient` và `DbContext`.
>
> **Cần biết trước:** `try/finally` (bài trước), GC ở mức "dọn object không ai
> tham chiếu".

## GC dọn bộ nhớ, không dọn tài nguyên hệ điều hành

| Thứ bạn tạo ra | GC dọn giúp | Ai chịu trách nhiệm |
|---|---|---|
| `List`, `string`, object thường | có | GC, khi không còn ai tham chiếu |
| File handle (`FileStream`) | không | bạn, qua `Dispose` |
| Connection tới database | không | bạn, hoặc framework |
| Socket mạng (`HttpClient`) | không | dùng lại, đừng tạo mới |

GC quản bộ nhớ, và nó làm việc đó rất tốt. Nhưng file handle, connection,
socket thì thuộc về hệ điều hành.

GC không biết chúng đắt tới mức nào, nên cũng không vội dọn. Những kiểu nắm
giữ chúng đều cài `IDisposable`, và bạn phải gọi `Dispose` khi dùng xong.

## Thử ngay: using dọn trước khi lỗi bay lên

```csharp
class Resource : IDisposable
{
    public Resource() => Console.WriteLine("mở");
    public void Dispose() =>
        Console.WriteLine("đóng");
}

void Run()
{
    using var res = new Resource();
    Console.WriteLine("đang dùng");
    throw new Exception("hỏng giữa chừng");
}

try { Run(); }
catch { Console.WriteLine("bắt được lỗi"); }
```

**Đoán trước khi chạy:** có lỗi ném ra giữa chừng. Dòng "đóng" có được in
không, và in trước hay sau "bắt được lỗi"?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
mở
đang dùng
đóng
bắt được lỗi
```

`using` dọn **trước** khi lỗi bay lên tầng trên.

Bên dưới, compiler dịch nó thành `try/finally`. Nên dù bạn `return` sớm hay
ném exception thì `Dispose` vẫn chạy.

</details>

## Ba cách viết using, khác nhau ở thời điểm dọn

| Viết | Dọn lúc nào | Dùng khi |
|---|---|---|
| `using var x = …` | cuối khối chứa nó | mặc định |
| `using (var x = …) { }` | cuối cặp ngoặc | cần đóng sớm hơn |
| `await using var x = …` | cuối khối, gọi `DisposeAsync` | tài nguyên bất đồng bộ |

```csharp
using var reader = new StreamReader(path);
var text = reader.ReadToEnd();

using (var conn = new SqlConnection(cs))
{
    conn.Open();
}

await using var db = new SqlConnection(cs);
await db.OpenAsync();
```

Dạng `using var` gọn hơn và đủ dùng cho hầu hết trường hợp. Dạng có ngoặc hợp
khi bạn muốn tài nguyên đóng sớm hơn cuối method.

## HttpClient phải dùng lại, không tạo mới mỗi request

```csharp
// SAI — mỗi request một socket mới
using var http = new HttpClient();
var res = await http.GetAsync(url);
```

Nghe rất ngược. `HttpClient` cài `IDisposable` hẳn hoi, vậy mà `using` nó lại
chính là vấn đề.

Socket sau khi đóng còn nằm ở trạng thái `TIME_WAIT` thêm vài phút. App bận
thì cạn cổng, và đó là lỗi ở đầu bài.

```csharp
// ĐÚNG — để DI quản, dùng lại kết nối
public class PriceService(HttpClient http)
{
    public Task<string> Get(string url) =>
        http.GetStringAsync(url);
}

// Program.cs
builder.Services.AddHttpClient<PriceService>();
```

`IHttpClientFactory`, mà `AddHttpClient` dựng lên cho bạn, tái sử dụng kết nối
bên dưới và tự xoay vòng chúng theo định kỳ.

## DbContext để framework quản theo từng request

`DbContext` cũng là `IDisposable`. Nhưng trong ASP.NET Core bạn **không tự
`using`** nó.

`AddDbContext` đăng ký theo vòng đời scoped. Mỗi request một context, và
framework tự dispose khi request kết thúc.

Tự `new` một `DbContext` rồi giữ trong field `static` thì hai request dùng
chung một context — mà `DbContext` không an toàn khi dùng song song.

Change tracker cũng phình mãi, vì mọi thực thể đã tải đều nằm lại đó. Một lỗi
chạy sai, một lỗi ăn dần bộ nhớ.

## Chỉ tự viết Dispose khi bạn sở hữu tài nguyên

```csharp
public class FileBuffer : IDisposable
{
    private readonly StreamWriter _writer;

    public FileBuffer(string path) =>
        _writer = new StreamWriter(path);

    public void Dispose() => _writer.Dispose();
}
```

Class này tự tạo ra `StreamWriter`, nên nó phải tự dọn.

Còn class chỉ nhận service từ DI rồi dùng thì **không** cần cài `IDisposable`.
Dọn thứ mình không sở hữu là gây lỗi cho người khác đang dùng chung.

## Dấu hiệu trong code của bạn

- `new HttpClient()` trong method xử lý request → chuyển sang `AddHttpClient` và nhận qua constructor.
- `new StreamReader`, `new SqlConnection`, `new FileStream` mà không có `using` → rò tài nguyên.
- `DbContext` được `new` bằng tay, hoặc giữ trong field `static`.
- Class cài `IDisposable` nhưng bên trong chỉ gọi `Dispose` của service do DI cấp → đang dọn đồ của người khác.
- Lỗi `Too many open files`, `SocketException`, hay connection pool cạn trong log → gần như luôn là bài này.

## Ghi nhớ

- GC dọn **bộ nhớ**, không dọn file handle, connection, socket.
- `using` là `try/finally` viết gọn: dọn cả khi `return` sớm lẫn khi ném lỗi.
- Tài nguyên bất đồng bộ dùng `await using`.
- `HttpClient` thì dùng lại qua DI, đừng tạo mới mỗi request.
- Chỉ dọn thứ bạn tự tạo ra.

## Bước tiếp theo

Hết chương **Ngoại lệ và tài nguyên**. Code của bạn giờ không nuốt lỗi, cũng
không rò tài nguyên nữa.

Chương cuối của khoá là **Bất đồng bộ**. Vì sao một API ngồi chờ database lại
không nên chiếm luồng, và những cái bẫy khiến cả app treo cứng.

```quiz
[
  {
    "prompt": "Đoạn này in ra thứ tự nào?",
    "code": "void Run()\n{\n    using var res = new Resource();\n    throw new Exception(\"hỏng\");\n}\n\ntry { Run(); }\ncatch { Console.WriteLine(\"bắt được lỗi\"); }",
    "options": [
      "\"bắt được lỗi\" rồi mới \"đóng\"",
      "\"đóng\" rồi mới \"bắt được lỗi\"",
      "Chỉ in \"bắt được lỗi\", Dispose không chạy",
      "Chỉ in \"đóng\", lỗi bị nuốt"
    ],
    "answer": 2,
    "explain": "using dịch thành try/finally nên Dispose chạy ngay khi rời khỏi khối, trước khi exception bay lên tầng trên."
  },
  {
    "prompt": "Service tạo new HttpClient() cho mỗi request và bọc trong using. Hệ quả?",
    "options": [
      "Đúng chuẩn, vì HttpClient là IDisposable",
      "Cạn cổng mạng: socket đóng vẫn nằm ở TIME_WAIT vài phút",
      "Rò bộ nhớ vì GC không dọn được HttpClient",
      "Chậm vì mỗi lần phải phân giải DNS lại"
    ],
    "answer": 2,
    "explain": "Đây là nghịch lý quen thuộc của HttpClient: nó nên được dùng lại. AddHttpClient / IHttpClientFactory lo phần tái sử dụng và xoay vòng kết nối."
  },
  {
    "prompt": "Trong ASP.NET Core, DbContext nên được quản lý thế nào?",
    "options": [
      "using var db = new AppDbContext() trong mỗi method",
      "Đăng ký AddDbContext, nhận qua constructor, framework tự dispose theo request",
      "Một field static dùng chung cho cả app",
      "Tạo mới rồi để GC dọn"
    ],
    "answer": 2,
    "explain": "AddDbContext đăng ký vòng đời scoped: mỗi request một context và tự dispose khi request kết thúc. Giữ static thì vừa rò connection vừa giữ hết thực thể đã tải."
  },
  {
    "prompt": "Class của bạn nhận IEmailSender qua constructor (do DI cấp) và dùng nó. Có nên cài IDisposable để dispose nó không?",
    "options": [
      "Có, dọn sớm được chừng nào tốt chừng ấy",
      "Không — bạn không tạo ra nó, dispose là gây lỗi cho nơi khác đang dùng chung",
      "Có, nhưng chỉ khi class là singleton",
      "Không quan trọng, GC sẽ lo"
    ],
    "answer": 2,
    "explain": "Chỉ dọn thứ mình sở hữu. Service do DI cấp có vòng đời riêng; dispose nó sớm sẽ làm hỏng các nơi khác đang dùng cùng instance."
  }
]
```
