---
title: Null Object và Strategy
minutes: 11
---

`grep -c "_audit is not null" src` trả về 41.

Bốn mươi mốt chỗ hỏi cùng một câu: có bộ ghi log kiểm toán hay không.

Ở môi trường dev thì không có. Nên ai cũng phải kiểm.

Quên một chỗ là `NullReferenceException`. Và không ai nhớ hết bốn mươi mốt chỗ.

> **Học xong bài này bạn sẽ:** xoá những chuỗi `if null` bằng Null Object; nhận
> ra Strategy chính là thứ bạn đã dùng ở chương trước; và biết khi nào Null
> Object là lựa chọn sai.
>
> **Cần biết trước:** interface, đa hình, dependency injection (hai chương
> trước).

## Null Object: một object không làm gì, thay cho null

**Null Object** là một class implement đúng hợp đồng, nhưng thân method để
trống. Người gọi không cần biết nó tồn tại.

| | Trả về `null` | Trả về Null Object |
|---|---|---|
| Nơi gọi phải kiểm tra | **có**, mọi chỗ | không |
| Quên kiểm tra thì | `NullReferenceException` | chạy bình thường |
| Đọc code | thấy `if` trước mỗi lần dùng | thấy đúng việc chính |
| Khi "không có" là lỗi | phù hợp | **giấu mất lỗi** |

Dòng cuối bảng là giới hạn của pattern này, và mục cuối bài sẽ quay lại.

```csharp
interface IAuditLog
{
    void Write(string action);
}

class NoopAuditLog : IAuditLog
{
    public void Write(string action) { }
}
```

Mười dòng đó xoá được cả bốn mươi mốt câu `if`.

Ở môi trường dev bạn đăng ký `NoopAuditLog`. Ở production đăng ký bản ghi thật.

```csharp
builder.Services.AddScoped<IAuditLog, NoopAuditLog>();
```

## Thử ngay: bỏ hết if null mà hành vi không đổi

Cách chắc nhất để tin: cho cả hai bản chạy qua cùng một chỗ gọi.

```csharp
Run(new NoopAuditLog());
Run(new ConsoleAuditLog());

void Run(IAuditLog audit)
{
    audit.Write("tạo đơn");
    Console.WriteLine("đã tạo đơn");
}

interface IAuditLog
{
    void Write(string action);
}

class NoopAuditLog : IAuditLog
{
    public void Write(string action) { }
}

class ConsoleAuditLog : IAuditLog
{
    public void Write(string action) =>
        Console.WriteLine($"[audit] {action}");
}
```

**Đoán trước khi chạy:** `Run` không có câu `if` nào. Bốn dòng in ra theo thứ tự
nào?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
đã tạo đơn
[audit] tạo đơn
đã tạo đơn
```

Ba dòng, không phải bốn. Lần gọi đầu, `Write` chạy và không làm gì cả.

Để ý `Run` không hề biết có hai bản.

Nó cũng không có chỗ nào để quên kiểm tra null. Không còn gì để kiểm nữa.

Đó là toàn bộ giá trị của pattern này: nó **xoá một loại lỗi**, chứ không chỉ
làm code ngắn hơn.

</details>

## Strategy là tên của thứ bạn đã dùng ở chương trước

Bài "Thay chuỗi if bằng đa hình" đã dạy trọn Strategy. Chỉ là chưa gọi tên nó.

**Strategy** là tách một thuật toán ra thành object riêng, để đổi được nó mà
không sửa nơi gọi.

| Bạn đã thấy ở | Tên pattern |
|---|---|
| `IShippingRate` cho từng hãng vận chuyển | Strategy |
| `abstract Payment.Fee` cho từng hình thức | Strategy |
| `Run` giữ trình tự, lớp con điền `Render` | Template Method |
| `GZipStream` bọc `FileStream` | Decorator |

Biết tên có ích ở hai chỗ. Đọc tài liệu, và nói với đồng nghiệp trong ba chữ
thay vì ba câu.

Trong .NET, Strategy hay xuất hiện dưới dạng nhiều class cùng implement một
interface, rồi DI chọn hộ:

```csharp
builder.Services
    .AddKeyedScoped<IShippingRate, GhnRate>("ghn");

builder.Services
    .AddKeyedScoped<IShippingRate, GhtkRate>("ghtk");
```

Đôi khi cả class cũng không cần.

Một `Func<decimal, decimal>` truyền vào constructor cũng là Strategy. Viết gọn
hơn nhiều.

## Null Object sai chỗ khi "không có" nghĩa là lỗi

Đây là chỗ pattern này gây hại nếu dùng máy móc.

```csharp
// SAI — giấu mất một lỗi cấu hình
class NoopPaymentGateway : IPaymentGateway
{
    public void Charge(decimal amount) { }
}
```

Không có bộ ghi log kiểm toán thì hệ thống vẫn đúng.

Không có cổng thanh toán thì đơn hàng coi như đã trả tiền, mà thật ra chưa.

| "Không có" nghĩa là | Nên |
|---|---|
| Không cần làm gì, vẫn đúng | Null Object |
| Cấu hình còn thiếu | ném exception lúc khởi động |
| Dữ liệu không tồn tại | trả `null`, người gọi tự xử |

Phép thử một câu. Bản "không làm gì" chạy suốt một tháng mà không ai phát hiện
thì Null Object đúng.

Còn nếu nó làm mất tiền thật, đó là chỗ phải nổ to.

## Dấu hiệu trong code của bạn

- Cùng một câu `if (x is null)` trước mỗi lần gọi, ở nhiều file → ứng viên của Null Object.
- Một interface mà nơi gọi luôn phải hỏi "có không" trước khi dùng → hợp đồng đang thiếu bản mặc định.
- Class tên `Noop`, `Null`, `Empty` mà bọc một cổng thanh toán hay một lần ghi database → đang giấu lỗi cấu hình.
- Bạn viết `if (type == ...)` để chọn thuật toán → đó là Strategy viết bằng tay.
- Một interface có ba class implement, chọn theo khoá chuỗi → dùng keyed service của DI, đừng tự viết bảng tra.

## Ghi nhớ

- Null Object implement đủ hợp đồng, thân method để trống.
- Nó xoá cả một loại lỗi, không chỉ làm code ngắn hơn.
- Chỉ dùng khi "không có" vẫn là trạng thái đúng; cấu hình thiếu thì phải nổ.
- Strategy là tách thuật toán thành object, và bạn đã dùng nó ở chương trước.
- Một `Func<>` truyền vào constructor cũng là Strategy, khỏi cần class.

## Bước tiếp theo

Null Object thay hẳn một hành vi bằng hành vi rỗng. Còn muốn **thêm** vào hành
vi đang có thì sao?

Bài sau, **Decorator**, mở bằng một yêu cầu nghe rất nhỏ: thêm retry cho lời gọi
API đối tác, mà không được sửa class đang gọi.

```quiz
[
  {
    "prompt": "Bốn mươi chỗ trong code đều kiểm tra if (_audit is not null) trước khi ghi log. Cách gọn nhất để xoá cả bốn mươi?",
    "options": [
      "Gom bốn mươi chỗ vào một method helper có kiểm tra null",
      "Đăng ký một class NoopAuditLog implement IAuditLog với thân rỗng",
      "Đổi _audit thành kiểu nullable rồi dùng toán tử ?.",
      "Bật Nullable enable để compiler tự xử lý"
    ],
    "answer": 2,
    "explain": "Null Object làm nơi gọi không còn gì để kiểm tra. Ba cách kia vẫn để lại phép kiểm tra, chỉ đổi chỗ hoặc đổi cú pháp — và vẫn quên được."
  },
  {
    "prompt": "Đoạn này in ra mấy dòng?",
    "code": "Run(new NoopAuditLog());\n\nvoid Run(IAuditLog audit)\n{\n    audit.Write(\"tạo đơn\");\n    Console.WriteLine(\"đã tạo đơn\");\n}",
    "options": [
      "Không dòng nào, vì NoopAuditLog không làm gì",
      "Hai dòng",
      "Một dòng",
      "Ném NullReferenceException"
    ],
    "answer": 3,
    "explain": "Write chạy bình thường nhưng thân rỗng nên không in gì; dòng Console.WriteLine vẫn chạy. Null Object là một object thật, không phải null."
  },
  {
    "prompt": "Trường hợp nào KHÔNG nên dùng Null Object?",
    "options": [
      "Bộ ghi log kiểm toán chưa bật ở môi trường dev",
      "Bộ gửi thông báo chưa cấu hình ở máy local",
      "Cổng thanh toán chưa cấu hình ở production",
      "Bộ đếm số liệu thống kê chưa bật"
    ],
    "answer": 3,
    "explain": "Một cổng thanh toán không làm gì sẽ khiến đơn hàng coi như đã trả tiền. Thiếu cấu hình ở đây phải ném exception lúc khởi động, không được im lặng."
  },
  {
    "prompt": "Bạn đã viết IShippingRate với một class cho mỗi hãng vận chuyển ở chương trước. Đó là pattern nào?",
    "options": [
      "Strategy",
      "Decorator",
      "Template Method",
      "Null Object"
    ],
    "answer": 1,
    "explain": "Strategy là tách thuật toán thành object riêng để đổi được mà không sửa nơi gọi. Template Method thì lớp cha giữ trình tự; Decorator thì bọc thêm hành vi quanh một object có sẵn."
  }
]
```
