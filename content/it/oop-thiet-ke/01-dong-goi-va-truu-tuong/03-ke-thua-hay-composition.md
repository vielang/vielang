---
title: Kế thừa hay composition
minutes: 11
---

`BaseService` dài 800 dòng, mười hai class kế thừa nó.

Bạn sửa một method trong lớp cha cho đúng yêu cầu của module thanh toán. Rồi
ba module khác hỏng.

Ba module bạn chưa từng mở ra xem.

> **Học xong bài này bạn sẽ:** phân biệt "là một" với "có một"; biết cái bẫy
> gọi method `virtual` trong constructor; và chọn composition ở những chỗ
> người ta hay chọn kế thừa theo quán tính.
>
> **Cần biết trước:** `class`, interface (bài trước).

## Đọc to lên: "là một" thì kế thừa, "có một" thì composition

Ba module hỏng vì một lớp cha. Nhưng kế thừa không sai — sai là dùng nó cho
việc của composition.

```csharp
// Kế thừa — FullTimeEmployee LÀ MỘT Employee
class Employee { public string Name = ""; }
class FullTimeEmployee : Employee { }

// Composition — Order CÓ MỘT cách tính phí
class Order(IFeeCalculator calculator)
{
    public decimal Fee() => calculator.Calculate(this);
}
```

Phép thử nhanh là đọc to lên. "Nhân viên toàn thời gian **là một** nhân viên"
nghe xuôi tai.

"Đơn hàng **là một** cách tính phí" thì vô lý. Vậy chỗ đó phải là composition.

| | Kế thừa | Composition |
|---|---|---|
| Lớp con thấy `protected` | có | không |
| Đổi được lúc chạy | không | có |
| Phụ thuộc vào | cả nội bộ lớp cha | một hợp đồng nhỏ |
| Thay bằng bản giả khi test | khó | dễ |

Kế thừa tạo ràng buộc mạnh nhất giữa hai class. Composition thì chỉ ràng buộc
qua đúng những gì interface hứa.

## Thử ngay: constructor lớp cha chạy trước field lớp con

Ràng buộc mạnh nghĩa là gì, cụ thể? Đây là một chỗ mà cả hai class đều đúng mà
ghép lại thì sai.

```csharp
new Child();

class Parent
{
    public Parent() => PrintName();
    public virtual void PrintName() =>
        Console.WriteLine("Parent");
}

class Child : Parent
{
    private readonly string _name = "Child";
    public override void PrintName() =>
        Console.WriteLine(_name ?? "null");
}
```

**Đoán trước khi chạy:** `_name` được gán ngay lúc khai báo. Dòng in ra là
`Child` hay gì khác?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
null
```

Constructor của lớp cha chạy trước, và lúc ấy field của lớp con chưa được gán.

Method `virtual` gọi từ constructor lại nhảy xuống bản override của lớp con,
nơi mọi thứ còn rỗng.

Người ta gọi đây là **fragile base class**. Lớp cha đúng, lớp con đúng, ghép
lại thì sai, và không compiler nào cảnh báo bạn.

</details>

Quy tắc rút ra ngắn thôi: đừng gọi method `virtual` trong constructor.

Rộng hơn nữa, mỗi lần lớp cha gọi một method có thể bị override, nó đang phụ
thuộc vào code mà nó không kiểm soát.

## Kế thừa đúng chỗ chỉ còn vài trường hợp

Sau cái bẫy đó thì câu hỏi không phải "kế thừa có xấu không", mà là còn chỗ nào
nó đúng.

| Tình huống | Chọn |
|---|---|
| Framework yêu cầu (`ControllerBase`, `DbContext`) | kế thừa |
| "Là một" thật sự, lớp con chỉ thêm hành vi | kế thừa |
| Dùng lại tiện ích chung (log, mail, thuế) | composition |
| Cần đổi hành vi lúc chạy hoặc lúc test | composition |

Với mọi thứ khác, hãy bắt đầu bằng composition.

Lý do rất thực dụng. Chuyển từ composition sang kế thừa thì dễ, còn chiều
ngược lại thường phải mở lại cả mười hai class.

## Một lớp cha phình to nên tách thành phụ thuộc

Giờ tới `BaseService` 800 dòng ở đầu bài.

```csharp
// SAI — module nào cần gì lại nhét vào lớp cha
abstract class BaseService
{
    protected void WriteLog(string s) { }
    protected void SendMail(string s) { }
    protected void CheckPermission(string s) { }
    protected decimal Tax(decimal x) => x * 0.1m;
}

class PaymentService : BaseService { }
```

```csharp
// ĐÚNG — nhận đúng thứ mình cần
class PaymentService(
    ILogger<PaymentService> logger,
    ITaxCalculator tax)
{
    public decimal Total(decimal amount)
    {
        logger.LogInformation("Thuế {A}", amount);
        return amount + tax.Calculate(amount);
    }
}
```

Bản composition dài hơn vài dòng. Bù lại bạn được ba thứ.

Nhìn constructor là biết class cần gì. Sửa `ITaxCalculator` không làm module
khác hỏng. Và lúc test thì truyền vào một bản giả, khỏi phải dựng cả
`BaseService`.

## sealed là mặc định hợp lý, protected là API công khai

Còn khi bạn viết class mà người khác có thể muốn kế thừa, hãy quyết định trước
thay vì để ngỏ.

```csharp
public sealed class StandardFeeCalculator
    : IFeeCalculator { }
```

`sealed` nói rằng class này không thiết kế để kế thừa.

Đó là mặc định hợp lý. Cho phép kế thừa là một lời hứa: mọi method `virtual`
sẽ giữ nguyên cách gọi ở các phiên bản sau.

`protected` cũng là API công khai, chỉ là công khai với lớp con. Mỗi thành
viên `protected` bạn thêm vào là một thứ nữa phải giữ nguyên về sau.

## Composition đã ở khắp .NET mà bạn đang dùng

Bạn dùng composition rất nhiều rồi, có thể chỉ chưa gọi tên nó.

- `ILogger`, `IHttpClientFactory`, `IOptions<T>` nhận qua constructor.
- Middleware trong ASP.NET Core xếp chồng thành chuỗi, không kế thừa nhau.
- `Stream` bọc `Stream`: `GZipStream(fileStream)` thêm hành vi bằng cách bọc, đúng mẫu **Decorator** sẽ gặp ở chương pattern.

## Dấu hiệu trong code của bạn

- Lớp cha có tên `BaseSomething` với đủ thứ tiện ích không liên quan nhau → đó là túi đồ, không phải quan hệ "là một".
- Lớp con override một method chỉ để **vô hiệu hoá** nó (thân rỗng, hoặc ném `NotSupportedException`) → quan hệ "là một" sai ngay từ đầu.
- Constructor lớp cha gọi method `virtual` → fragile base class.
- Cây kế thừa sâu từ ba tầng trở lên → mỗi lần đọc code phải nhảy ba file mới biết method nào đang chạy.
- `protected` rải khắp lớp cha → bề mặt phải giữ nguyên ngày càng rộng.

## Ghi nhớ

- "Là một" thì kế thừa, "có một" thì composition — đọc to lên là biết.
- Không gọi method `virtual` trong constructor.
- Mặc định `sealed`; mở kế thừa là một lời hứa lâu dài.
- Override để vô hiệu hoá hành vi lớp cha nghĩa là quan hệ đó sai.
- Composition dễ test, dễ đổi, và dễ chuyển ngược lại hơn.

## Bước tiếp theo

Hết chương **Đóng gói và trừu tượng**. Bạn đã có đủ công cụ để giấu chi tiết và
dùng lại code cho an toàn.

Chương sau, **Đa hình trong thực tế**, dùng `virtual`, `abstract` và interface
để thay những chuỗi `if` phân loại theo kiểu.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "class Parent\n{\n    public Parent() => Print();\n    public virtual void Print() =>\n        Console.WriteLine(\"Parent\");\n}\n\nclass Child : Parent\n{\n    private readonly string _name = \"Child\";\n    public override void Print() =>\n        Console.WriteLine(_name ?? \"null\");\n}\n\nnew Child();",
    "options": [
      "Child",
      "Parent",
      "null",
      "Ném NullReferenceException"
    ],
    "answer": 3,
    "explain": "Constructor lớp cha chạy trước khi field của lớp con được gán, mà method virtual lại nhảy xuống bản override. Đừng gọi method virtual trong constructor."
  },
  {
    "prompt": "Quan hệ nào nên dùng composition thay vì kế thừa?",
    "options": [
      "Order có một cách tính phí vận chuyển",
      "PaymentController là một ControllerBase",
      "OrderException là một Exception",
      "EmailBackgroundJob là một BackgroundService"
    ],
    "answer": 1,
    "explain": "\"Có một\" là dấu hiệu của composition. Ba trường hợp còn lại là quan hệ \"là một\" thật sự, và đều do framework yêu cầu kế thừa."
  },
  {
    "prompt": "Lớp con override một method của lớp cha và để thân rỗng để nó không làm gì nữa. Điều đó nói lên gì?",
    "options": [
      "Thiết kế tốt, lớp con được tự do",
      "Cần thêm sealed cho lớp con",
      "Cần đánh dấu method là abstract",
      "Quan hệ \"là một\" sai — lớp con không thật sự là một lớp cha"
    ],
    "answer": 4,
    "explain": "Nếu lớp con phải vô hiệu hoá hành vi của lớp cha thì nó không thay thế được lớp cha. Đây chính là vi phạm nguyên tắc Liskov ở chương SOLID."
  },
  {
    "prompt": "Vì sao nên đặt sealed cho class không thiết kế để kế thừa?",
    "options": [
      "Chạy nhanh hơn đáng kể",
      "Tránh phải hứa giữ nguyên cách gọi các method virtual ở các phiên bản sau",
      "Bắt buộc theo quy ước .NET",
      "Để class không dùng được với interface"
    ],
    "answer": 2,
    "explain": "Mở cho kế thừa là một cam kết lâu dài về hành vi nội bộ. sealed nói rõ class này không nằm trong cam kết đó."
  }
]
```
