---
title: Factory và Builder
minutes: 11
---

Constructor của `ReportRequest` có mười một tham số. Năm cái cuối luôn được
truyền `null`.

Chỗ gọi trông như một hàng số và `null` xếp cạnh nhau, không ai đọc ra cái nào
là cái gì. Đổi thứ tự hai tham số `string` là bug im lặng.

Không ai cố ý viết như vậy. Nó dài ra từng sprint, mỗi lần thêm một tuỳ chọn.

> **Học xong bài này bạn sẽ:** biết Factory giải quyết việc gì mà `new` không
> giải quyết được; dùng Builder cho object nhiều tuỳ chọn; và nhận ra hai chỗ
> hai pattern này bị dùng thừa.
>
> **Cần biết trước:** constructor, `required`, `init` (khoá C# Core), DI (chương
> trước).

## Factory quyết định tạo cái gì, Builder quyết định tạo thế nào

Hai pattern hay bị gộp làm một, nhưng chúng trả lời hai câu hỏi khác nhau.

| | Factory | Builder |
|---|---|---|
| Câu hỏi | tạo **kiểu nào** | dựng object có **nhiều tuỳ chọn** |
| Người gọi biết kiểu cụ thể | không | có |
| Số bước | một lời gọi | nhiều lời gọi rồi chốt |
| Hay gặp ở | `IHttpClientFactory`, DI container | query builder, cấu hình |

Trong C# hiện đại, cả hai đều bớt cần hơn trước. `required` và `init` đã giải
quyết phần lớn việc của Builder.

| Object của bạn | Nên dùng |
|---|---|
| Ba tham số, đều bắt buộc | constructor thường |
| Ba bắt buộc, hai tuỳ chọn | `required` cộng `init` |
| Ba bắt buộc, tám tuỳ chọn ghép nhau | Builder |
| Kiểu cụ thể phụ thuộc dữ liệu lúc chạy | Factory |

## Factory đáng có khi việc chọn kiểu là một quyết định

Bắt đầu từ dòng cuối bảng: kiểu cụ thể chỉ biết được lúc chạy.

```csharp
// SAI — nơi gọi phải biết mọi kiểu
IExporter exporter = format switch
{
    "csv" => new CsvExporter(),
    "pdf" => new PdfExporter(sp.GetRequiredService<IFontStore>()),
    _ => throw new NotSupportedException(),
};
```

Câu `switch` ấy sẽ bị chép sang chỗ thứ hai, rồi chỗ thứ ba. Mỗi chỗ lại tự lo
phần phụ thuộc của `PdfExporter`.

```csharp
// ĐÚNG — một chỗ biết cách chọn và cách dựng
interface IExporterFactory
{
    IExporter For(string format);
}

class ExporterFactory(IFontStore fonts) : IExporterFactory
{
    public IExporter For(string format) => format switch
    {
        "csv" => new CsvExporter(),
        "pdf" => new PdfExporter(fonts),
        _ => throw new NotSupportedException(format),
    };
}
```

Nơi gọi giờ chỉ cần `factory.For(format)`. Nó không biết `PdfExporter` cần font,
và không cần biết.

Để ý câu `switch` vẫn còn. Factory **không xoá** nó, chỉ dồn nó về một chỗ duy
nhất — và đó chính là giá trị.

## Thử ngay: Builder dựng dần rồi chốt một lần

Giờ tới cái constructor mười một tham số ở đầu bài.

```csharp
var q = new ReportQuery()
    .From(new DateTime(2026, 1, 1))
    .OnlyPaid()
    .Top(10)
    .Build();

Console.WriteLine(q);

class ReportQuery
{
    private DateTime? _from;
    private bool _paidOnly;
    private int _top = 100;

    public ReportQuery From(DateTime d)
    {
        _from = d;
        return this;
    }

    public ReportQuery OnlyPaid()
    {
        _paidOnly = true;
        return this;
    }

    public ReportQuery Top(int n)
    {
        _top = n;
        return this;
    }

    public string Build() =>
        $"từ {_from:dd/MM/yyyy}, paid={_paidOnly}, top={_top}";
}
```

**Đoán trước khi chạy:** mỗi method trả về `this`. Điều đó cho phép viết gì, và
dòng cuối in ra sao?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
từ 01/01/2026, paid=True, top=10
```

Trả về `this` là thứ cho phép nối các lời gọi thành một chuỗi. Tên gọi của nó là
**fluent interface**.

Bỏ `.Top(10)` đi thì `top` vẫn là 100, vì giá trị mặc định nằm ngay trong field.
Người gọi chỉ khai những gì khác mặc định.

So với constructor mười một tham số, chỗ gọi giờ tự giải thích. Và thêm một tuỳ
chọn mới là thêm một method, không phá chữ ký cũ.

</details>

## required và init đã thay Builder ở phần lớn trường hợp

Đừng viết Builder theo phản xạ. C# đã có cách gọn hơn cho trường hợp thường gặp.

```csharp
var req = new ReportRequest
{
    From = new DateTime(2026, 1, 1),
    PaidOnly = true,
};

class ReportRequest
{
    public required DateTime From { get; init; }
    public bool PaidOnly { get; init; }
    public int Top { get; init; } = 100;
}
```

Thiếu `From` là lỗi compile. Các tuỳ chọn có mặc định, và object không sửa được
sau khi tạo.

Builder chỉ hơn ở hai chỗ: khi việc dựng cần **kiểm tra chéo** giữa các tuỳ chọn,
và khi các bước phải theo thứ tự.

## Hai chỗ Factory bị dùng thừa

Pattern này bị viết theo phản xạ nhiều hơn theo nhu cầu.

```csharp
// SAI — factory chỉ gói một lời new
class OrderFactory
{
    public Order Create() => new Order();
}
```

Không có quyết định nào ở đây. Nó chỉ thêm một file và một tầng.

Chỗ thừa thứ hai là viết factory cho những gì DI container đã làm. Container
chính là một factory: bạn khai kiểu, nó dựng hộ cả cây phụ thuộc.

| Tình huống | Cần factory tự viết |
|---|---|
| Chọn kiểu theo dữ liệu lúc chạy | **có** |
| Dựng object cần tham số chỉ biết lúc chạy | **có** |
| Luôn dựng đúng một kiểu | không, dùng DI |
| Chỉ gói một lời `new` | không |

## Dấu hiệu trong code của bạn

- Constructor trên năm tham số, có cái luôn truyền `null` → đổi sang `required` cộng `init`, hoặc Builder.
- Cùng một `switch` chọn kiểu xuất hiện ở hai chỗ → dồn về một factory.
- Class tên `SomethingFactory` mà thân chỉ có `return new Something()` → bỏ đi, `new` trực tiếp.
- Builder mà không method nào kiểm tra gì, chỉ gán field → `required` với `init` làm cùng việc, ngắn hơn.
- Chỗ gọi có bốn tham số `bool` và `null` xếp cạnh nhau → chỗ gọi không tự giải thích được.

## Ghi nhớ

- Factory trả lời "tạo kiểu nào", Builder trả lời "dựng thế nào".
- Factory không xoá câu `switch`, nó dồn câu ấy về một chỗ.
- Builder trả về `this` để nối lời gọi, và giữ mặc định ngay trong field.
- `required` cộng `init` thay được Builder ở phần lớn trường hợp.
- DI container đã là một factory; đừng viết lại nó.

## Bước tiếp theo

Câu cuối mục trên gợi ra một câu hỏi lớn hơn: còn pattern nào nữa mà .NET đã làm
sẵn, trong khi người ta vẫn tự viết lại?

Bài cuối chương, **Những pattern .NET đã làm sẵn**, trả lời bằng một bảng — và
vài chỗ bạn đang dùng pattern mỗi ngày mà chưa biết tên nó.

```quiz
[
  {
    "prompt": "Constructor có mười một tham số, năm cái cuối luôn truyền null. Cách sửa gọn nhất trong C# hiện đại?",
    "options": [
      "Viết một Builder với mười một method",
      "Dùng required cho tham số bắt buộc và init cho tuỳ chọn có mặc định",
      "Thêm nhiều overload của constructor",
      "Gom mười một tham số vào một Dictionary"
    ],
    "answer": 2,
    "explain": "required bắt lỗi thiếu dữ liệu ngay lúc compile, còn init cho tuỳ chọn một giá trị mặc định. Builder chỉ hơn khi việc dựng cần kiểm tra chéo giữa các tuỳ chọn hoặc các bước phải theo thứ tự."
  },
  {
    "prompt": "Factory làm gì với câu switch chọn kiểu?",
    "options": [
      "Xoá hẳn nó, vì đa hình thay thế được",
      "Đổi nó thành một Dictionary tra kiểu",
      "Dồn nó về một chỗ duy nhất, thay vì để nó bị chép ra nhiều nơi",
      "Chuyển nó xuống lớp con của từng kiểu"
    ],
    "answer": 3,
    "explain": "Ở đâu đó vẫn phải có chỗ đọc chuỗi \"pdf\" rồi quyết định dựng cái gì. Giá trị của factory là câu switch ấy chỉ tồn tại một bản, và nó cũng là chỗ duy nhất biết PdfExporter cần font."
  },
  {
    "prompt": "Trong Builder, vì sao mỗi method trả về this?",
    "options": [
      "Để tiết kiệm bộ nhớ",
      "Để cho phép nối các lời gọi thành một chuỗi",
      "Để object trở thành immutable",
      "Vì C# bắt buộc method của Builder phải trả về chính nó"
    ],
    "answer": 2,
    "explain": "Trả về this cho phép viết .From(...).OnlyPaid().Top(10) liền mạch, gọi là fluent interface. Nó không liên quan gì tới bộ nhớ hay tính bất biến."
  },
  {
    "prompt": "Bạn thấy class OrderFactory với đúng một method: return new Order(). Nên làm gì?",
    "options": [
      "Bỏ nó đi và new trực tiếp, vì không có quyết định nào ở đây",
      "Thêm interface IOrderFactory cho dễ test",
      "Giữ lại, vì sau này có thể cần chọn kiểu khác",
      "Đổi tên thành OrderBuilder cho đúng pattern"
    ],
    "answer": 1,
    "explain": "Factory đáng có khi việc chọn kiểu hoặc cách dựng là một quyết định. Gói một lời new chỉ thêm một file để người đọc phải mở. Cần thay đổi thật thì lúc đó tách cũng chỉ mất vài phút."
  }
]
```
