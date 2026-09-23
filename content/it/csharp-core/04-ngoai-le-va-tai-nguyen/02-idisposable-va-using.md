---
title: IDisposable và using
minutes: 10
---

Service chạy êm vài giờ rồi bắt đầu trả lỗi
`SocketException: Only one usage of each socket address is normally permitted`.
Khởi động lại là hết, vài giờ sau lại thế. Nguyên nhân nằm ở một dòng trông
vô hại: `new HttpClient()` trong mỗi request.

> **Học xong bài này bạn sẽ:** biết thứ gì GC dọn được và thứ gì không; dùng
> `using` đúng chỗ; và tránh hai cái bẫy tài nguyên hay gặp nhất trong app
> .NET là `HttpClient` và `DbContext`.
>
> **Cần biết trước:** `try/finally` (bài trước), GC ở mức "dọn object không ai
> tham chiếu".

## GC không dọn mọi thứ

GC quản **bộ nhớ**. Còn file handle, connection tới database, socket, cổng
mạng — đó là tài nguyên của hệ điều hành, GC không biết chúng đắt tới mức nào
và không dọn kịp thời. Những kiểu nắm giữ chúng đều cài `IDisposable`, và bạn
phải gọi `Dispose` khi dùng xong.

## Thử ngay: using gọi Dispose lúc nào

```csharp
class TaiNguyen : IDisposable
{
    public TaiNguyen() => Console.WriteLine("mở");
    public void Dispose() => Console.WriteLine("đóng");
}

void Chay()
{
    using var tn = new TaiNguyen();
    Console.WriteLine("đang dùng");
    throw new Exception("hỏng giữa chừng");
}

try { Chay(); }
catch { Console.WriteLine("bắt được lỗi"); }
```

**Đoán trước khi chạy:** có lỗi ném ra giữa chừng — dòng "đóng" có được in
không, và in trước hay sau "bắt được lỗi"?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
mở
đang dùng
đóng
bắt được lỗi
```

`using` dọn **trước** khi lỗi bay lên tầng trên. Bên dưới, compiler dịch nó
thành `try/finally`, nên dù `return` sớm hay ném exception thì `Dispose` vẫn
chạy.

</details>

## Hai cách viết

```csharp
// using declaration — dọn ở cuối khối chứa nó
using var reader = new StreamReader(path);
var text = reader.ReadToEnd();

// using statement — dọn ở cuối khối ngoặc
using (var conn = new SqlConnection(chuoi))
{
    conn.Open();
}
```

Dạng `using var` gọn hơn và đủ dùng cho hầu hết trường hợp. Dạng có ngoặc hợp
khi bạn muốn tài nguyên đóng **sớm hơn** cuối method.

Với tài nguyên bất đồng bộ (stream mạng, `DbConnection`), dùng `await using` —
nó gọi `DisposeAsync` thay vì `Dispose`:

```csharp
await using var conn2 = new SqlConnection(chuoi);
await conn2.OpenAsync();
```

## Bẫy 1: HttpClient tạo mới mỗi lần

```csharp
// SAI — mỗi request một socket, đóng rồi vẫn bị giữ
using var http = new HttpClient();
var res = await http.GetAsync(url);
```

Nghe rất ngược: `HttpClient` cài `IDisposable`, nhưng tạo mới liên tục lại
chính là vấn đề. Socket sau khi đóng còn nằm ở trạng thái `TIME_WAIT` vài
phút, nên app bận sẽ cạn cổng — đúng lỗi ở đầu bài.

```csharp
// ĐÚNG — để DI quản, dùng lại kết nối
public class GiaService(HttpClient http)
{
    public Task<string> Lay(string url) =>
        http.GetStringAsync(url);
}

// Program.cs
builder.Services.AddHttpClient<GiaService>();
```

`IHttpClientFactory` (qua `AddHttpClient`) tái sử dụng kết nối bên dưới và tự
xoay vòng chúng theo định kỳ.

## Bẫy 2: DbContext sống quá lâu

`DbContext` cũng là `IDisposable`, nhưng trong ASP.NET Core bạn **không tự
`using`** nó: `AddDbContext` đăng ký theo vòng đời scoped, mỗi request một
context, và framework tự dispose khi request kết thúc.

Tự tạo `DbContext` bằng `new` rồi giữ trong một field `static` thì vừa rò
connection vừa giữ mọi thực thể đã tải trong bộ nhớ — hai loại leak cùng lúc.

## Khi nào viết Dispose của riêng mình

Chỉ khi class của bạn **nắm giữ** một tài nguyên cần dọn:

```csharp
public class BoDemFile : IDisposable
{
    private readonly StreamWriter _writer;

    public BoDemFile(string path) =>
        _writer = new StreamWriter(path);

    public void Dispose() => _writer.Dispose();
}
```

Class chỉ dùng các service khác (do DI cấp) thì **không** cần cài
`IDisposable` — dọn thứ mình không sở hữu là gây lỗi cho người khác.

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

Hết chương **Ngoại lệ và tài nguyên**. Chương cuối của khoá — **Bất đồng bộ** —
nói về `async`/`await`: vì sao một API chờ database lại không nên chiếm luồng,
và những cái bẫy khiến app treo cứng.

```quiz
[
  {
    "prompt": "Đoạn này in ra thứ tự nào?",
    "code": "void Chay()\n{\n    using var tn = new TaiNguyen();\n    throw new Exception(\"hỏng\");\n}\n\ntry { Chay(); }\ncatch { Console.WriteLine(\"bắt được lỗi\"); }",
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
