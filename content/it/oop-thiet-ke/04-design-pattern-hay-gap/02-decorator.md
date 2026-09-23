---
title: Decorator
minutes: 11
---

Yêu cầu nghe rất nhỏ. API đối tác thỉnh thoảng lỗi mạng, nên thêm thử lại ba
lần.

Chỗ gọi nằm trong `PriceService`, mà class đó đang được bốn nơi dùng và có tám
test. Sửa nó là chạm vào cả tám.

Hôm sau lại có yêu cầu nữa: nhớ kết quả trong năm phút. Rồi: ghi log mỗi lần
gọi. Ba yêu cầu, ba lần sửa cùng một class.

> **Học xong bài này bạn sẽ:** thêm retry, cache và log vào một service mà không
> sửa class gốc; biết thứ tự bọc quyết định hành vi; và biết khi nào middleware
> hoặc `DelegatingHandler` mới là chỗ đúng.
>
> **Cần biết trước:** interface, composition, DI (hai chương trước).

## Decorator implement cùng interface, rồi bọc một bản khác

**Decorator** là một class implement đúng interface của thứ nó bọc, làm thêm
việc của mình, rồi gọi vào bản bên trong.

Người gọi vẫn chỉ thấy interface, nên không biết mình đang cầm mấy tầng.

| | Sửa class gốc | Bọc bằng Decorator |
|---|---|---|
| Class gốc và test của nó | phải sửa | không chạm |
| Tắt tính năng ở môi trường khác | thêm `if` | bỏ một dòng đăng ký |
| Ba yêu cầu mới | ba lần sửa một file | ba file mới, độc lập nhau |
| Đọc code | thấy hết trong một class | phải xem chỗ đăng ký DI |

Dòng cuối là cái giá, và nó có thật.

```csharp
interface IPriceApi
{
    decimal Get(string code);
}

class RetryPrice(IPriceApi inner) : IPriceApi
{
    public decimal Get(string code)
    {
        for (var i = 0; i < 3; i++)
        {
            try { return inner.Get(code); }
            catch (HttpRequestException) { }
        }

        throw new HttpRequestException("hết lượt");
    }
}
```

`RetryPrice` không biết bên trong là HTTP, là cache, hay một decorator khác.

Nó chỉ biết `IPriceApi`.

## Thử ngay: thứ tự bọc đổi hẳn hành vi

Hai decorator giống nhau, bọc theo hai thứ tự, cho ra hai kết quả khác nhau.

```csharp
IPriceApi a = new CachePrice(new LogPrice(new FlakyApi()));
IPriceApi b = new LogPrice(new CachePrice(new FlakyApi()));

Console.WriteLine("-- cache ngoài, log trong");
a.Get("SP01");
a.Get("SP01");

Console.WriteLine("-- log ngoài, cache trong");
b.Get("SP01");
b.Get("SP01");

interface IPriceApi
{
    decimal Get(string code);
}

class FlakyApi : IPriceApi
{
    public decimal Get(string code) => 100;
}

class LogPrice(IPriceApi inner) : IPriceApi
{
    public decimal Get(string code)
    {
        Console.WriteLine($"gọi {code}");
        return inner.Get(code);
    }
}

class CachePrice(IPriceApi inner) : IPriceApi
{
    private readonly Dictionary<string, decimal> _c = new();

    public decimal Get(string code)
    {
        if (_c.TryGetValue(code, out var hit)) return hit;
        return _c[code] = inner.Get(code);
    }
}
```

**Đoán trước khi chạy:** mỗi nhóm gọi `Get` hai lần với cùng mã. Mỗi nhóm in ra
mấy dòng `gọi SP01`?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
-- cache ngoài, log trong
gọi SP01
-- log ngoài, cache trong
gọi SP01
gọi SP01
```

Nhóm đầu in một dòng, nhóm sau in hai.

Cache ở ngoài thì lần gọi thứ hai dừng ngay tại cache, nên `LogPrice` không bao
giờ chạy. Log không thấy lần gọi thứ hai, dù nó có xảy ra.

Log ở ngoài thì mọi lần gọi đều được ghi, còn cache chỉ chặn phần đi ra mạng.

Cả hai đều chạy được. Nhưng chúng trả lời hai câu hỏi khác nhau, và bạn phải
chọn câu nào mình cần.

</details>

Quy tắc thực dụng: thứ bạn muốn **đo** thì đặt ngoài, thứ bạn muốn **tiết kiệm**
thì đặt trong.

## Đăng ký thứ tự bọc ở một chỗ duy nhất

Trong ASP.NET Core, chỗ bọc nằm ở `Program.cs`, không rải trong code nghiệp vụ.

```csharp
builder.Services.AddScoped<IPriceApi>(sp =>
    new LogPrice(
        new CachePrice(
            new HttpPriceApi())));
```

Đọc từ ngoài vào trong là đúng thứ tự chạy.

Muốn tắt cache ở môi trường dev thì bỏ một dòng. Không sửa class nào.

Thư viện như Scrutor cho bạn viết `Decorate<IPriceApi, LogPrice>()` gọn hơn.
Nhưng cơ chế bên dưới vẫn đúng như đoạn trên.

## .NET đã có chỗ dành riêng cho việc bọc

Trước khi tự viết decorator, hãy xem framework có sẵn chỗ chưa.

| Bạn muốn bọc | Dùng sẵn |
|---|---|
| Mọi request HTTP đi ra | `DelegatingHandler` của `HttpClient` |
| Mọi request HTTP đi vào | middleware của ASP.NET Core |
| Mọi lệnh gọi database | interceptor của EF Core |
| Một service cụ thể của bạn | Decorator tự viết |

Ba dòng đầu đã là Decorator, chỉ là framework dựng sẵn đường ống. Tự viết thêm
một tầng ở đó là làm hai lần một việc.

## Bọc quá nhiều tầng thì không ai đọc nổi stack trace

Pattern này dễ dùng tới mức dùng quá.

```csharp
// SAI — năm tầng cho một lời gọi
new MetricPrice(
    new LogPrice(
        new RetryPrice(
            new CachePrice(
                new HttpPriceApi()))));
```

Đoạn này chạy đúng.

Nhưng khi có lỗi, stack trace dài năm tầng, và không tầng nào nói được lỗi thật
nằm ở đâu.

Hai, ba tầng thì còn theo dõi được. Quá đó thì cân nhắc gộp, hoặc chuyển phần đo
đạc sang công cụ quan sát chuyên dụng.

## Dấu hiệu trong code của bạn

- Một class có `if (_cacheEnabled)` hoặc `if (_retryEnabled)` bên trong → hai hành vi đang bị trộn, tách ra thành decorator.
- Thêm log hay retry mà phải sửa class nghiệp vụ và cả test của nó → dấu hiệu cần bọc thay vì sửa.
- Tự viết một tầng retry quanh `HttpClient` → `DelegatingHandler` đã làm sẵn.
- Stack trace dài năm tầng decorator cho một lời gọi → gộp lại bớt.
- Không ai trong nhóm nói được thứ tự bọc hiện tại → thứ tự đang rải nhiều chỗ, gom về `Program.cs`.

## Ghi nhớ

- Decorator implement cùng interface với thứ nó bọc, nên người gọi không thấy gì khác.
- Thêm hành vi bằng file mới, không sửa class gốc và không sửa test của nó.
- Thứ tự bọc đổi hành vi: thứ cần đo đặt ngoài, thứ cần tiết kiệm đặt trong.
- Khai thứ tự ở một chỗ duy nhất, thường là `Program.cs`.
- HTTP đi ra thì dùng `DelegatingHandler`, đi vào thì dùng middleware.

## Bước tiếp theo

Hai bài vừa rồi đều bắt đầu từ một object đã có. Còn việc **dựng** object thì
sao, khi nó cần mười tham số mà chỉ ba cái bắt buộc?

Bài sau, **Factory và Builder**, mở bằng một constructor có mười một tham số, và
năm cái cuối luôn được truyền `null`.

```quiz
[
  {
    "prompt": "Bạn cần thêm retry cho PriceService mà không sửa class đó và tám test của nó. Cách nào?",
    "options": [
      "Thêm tham số bool enableRetry vào constructor",
      "Kế thừa PriceService rồi override method gọi API",
      "Viết một class implement cùng interface, bọc bản gốc rồi đăng ký nó",
      "Bọc mọi chỗ gọi PriceService trong try-catch với vòng lặp"
    ],
    "answer": 3,
    "explain": "Decorator implement cùng interface nên nơi gọi không thấy gì khác, và class gốc cùng test của nó không bị chạm. Kế thừa thì phải mở class gốc cho override, còn thêm tham số bool là trộn hai hành vi vào một class."
  },
  {
    "prompt": "Bạn bọc cache ở NGOÀI log. Gọi Get hai lần cùng một mã thì log ghi mấy lần?",
    "options": [
      "Hai lần, vì mỗi lệnh gọi đều đi qua log",
      "Một lần, vì lần thứ hai dừng ở cache nên không xuống tới log",
      "Không lần nào, vì cache chặn hết",
      "Tuỳ độ dài thời gian sống của cache"
    ],
    "answer": 2,
    "explain": "Tầng ngoài chạy trước. Cache ở ngoài trả kết quả luôn ở lần thứ hai, nên log bên trong không bao giờ được gọi — và log của bạn sẽ báo ít lệnh gọi hơn thực tế."
  },
  {
    "prompt": "Bạn muốn thêm header xác thực vào mọi request HTTP mà app gửi ra. Chỗ đúng là gì?",
    "options": [
      "Một DelegatingHandler đăng ký cùng AddHttpClient",
      "Một middleware của ASP.NET Core",
      "Một decorator quanh từng service gọi HTTP",
      "Sửa trực tiếp trong mỗi chỗ tạo HttpRequestMessage"
    ],
    "answer": 1,
    "explain": "Middleware xử lý request đi VÀO app. Request đi RA thì HttpClient đã có sẵn đường ống DelegatingHandler, nên tự viết decorator cho từng service là làm lại việc framework đã làm."
  },
  {
    "prompt": "Một lời gọi đi qua năm tầng decorator. Hệ quả đáng lo nhất là gì?",
    "options": [
      "Chậm hơn đáng kể vì năm lần gọi hàm",
      "Tốn bộ nhớ vì năm object",
      "Vi phạm nguyên tắc SRP",
      "Stack trace dài năm tầng, khó nói được lỗi thật nằm ở đâu"
    ],
    "answer": 4,
    "explain": "Năm lần gọi hàm không phải vấn đề hiệu năng. Vấn đề là lúc đọc log lỗi: mỗi tầng thêm một lớp, và không tầng nào chỉ ra chỗ hỏng thật."
  }
]
```
