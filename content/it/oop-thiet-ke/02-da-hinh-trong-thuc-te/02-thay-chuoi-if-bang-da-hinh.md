---
title: Thay chuỗi if bằng đa hình
minutes: 11
---

Sếp muốn thêm thanh toán qua ví điện tử. Nghe như việc của một buổi chiều.

Bạn `grep` chữ `PaymentType` trong repo. Bốn chỗ: tính phí, dựng biên lai, kiểm
tra hạn mức, ghi log.

Bốn file. Bốn chuỗi `if`.

Ba chỗ bạn nhớ sửa. Chỗ thứ tư ra production rồi mới lộ.

> **Học xong bài này bạn sẽ:** nhận ra chuỗi `if` phân loại theo kiểu và thay
> nó bằng đa hình; biết khi nào `switch expression` mới là lựa chọn đúng; và
> biết cái giá phải trả của mỗi cách.
>
> **Cần biết trước:** `virtual`, `abstract`, `override` (bài trước); interface
> (chương trước).

## Chuỗi if theo kiểu là hành vi đang nằm sai chỗ

Dấu hiệu rất cụ thể: bạn hỏi một object "mày thuộc loại nào", rồi tự quyết định
thay nó.

```csharp
// SAI — mỗi loại mới là một lần sửa ở bốn file
decimal Fee(PaymentType type, decimal amount)
{
    if (type == PaymentType.Cash) return 0;
    if (type == PaymentType.Card) return amount * 0.02m;
    if (type == PaymentType.Transfer) return 5_000;
    throw new NotSupportedException();
}

enum PaymentType { Cash, Card, Transfer }
```

Hành vi "tính phí" thuộc về từng loại thanh toán.

Nhưng ở đây nó nằm trong một method bên ngoài, xa khỏi thứ nó mô tả.

Đa hình đảo chuyện đó lại. Mỗi loại tự biết phí của mình, và người gọi không
cần hỏi gì cả.

```csharp
// ĐÚNG — thêm loại mới là thêm một class
abstract class Payment
{
    public abstract decimal Fee(decimal amount);
}

class Cash : Payment
{
    public override decimal Fee(decimal a) => 0;
}

class Card : Payment
{
    public override decimal Fee(decimal a) =>
        a * 0.02m;
}
```

Chỗ gọi rút lại còn một dòng, và nó không còn biết có bao nhiêu loại thanh toán
trên đời.

```csharp
decimal total = amount + payment.Fee(amount);
```

## Thử ngay: thêm một loại mới và xem ai nhắc bạn

Câu hỏi đáng hỏi không phải "cách nào ngắn hơn", mà "quên thì ai nhắc".

```csharp
Payment[] all = { new Cash(), new Wallet() };

foreach (var p in all)
    Console.WriteLine(p.Fee(100_000));

abstract class Payment
{
    public abstract decimal Fee(decimal amount);
}

class Cash : Payment
{
    public override decimal Fee(decimal a) => 0;
}

class Wallet : Payment
{
    public override decimal Fee(decimal a) =>
        a * 0.01m;
}
```

**Đoán trước khi chạy:** `Wallet` là loại vừa thêm vào, và không ai sửa method
`Fee` nào ở ngoài cả. Đoạn này in ra gì, hay báo lỗi?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0
1000
```

Chạy đúng, không sửa gì ở chỗ gọi.

Giờ thử chiều ngược lại. Xoá dòng `Fee` trong `Wallet` đi rồi build lại: bạn
nhận **lỗi compile**, kèm đúng tên class còn thiếu.

Đó là khác biệt đáng giá nhất. Chuỗi `if` quên một nhánh thì im lặng rơi xuống
`throw` ở cuối, và chỉ lộ ra khi có khách thật bấm thanh toán.

</details>

## Đa hình cho hành vi, switch expression cho dữ liệu

Đây là chỗ dễ đi quá. Không phải mọi chuỗi `if` đều nên thành class.

| Tình huống | Chọn | Vì sao |
|---|---|---|
| Mỗi loại **làm** khác nhau | đa hình | hành vi ở cạnh dữ liệu của nó |
| Tập loại còn mở, sẽ thêm nữa | đa hình | thêm class, không sửa chỗ gọi |
| Chỉ tra một giá trị theo loại | `switch expression` | một dòng, không cần class |
| Tập loại đóng và ổn định | `switch expression` | compiler nhắc khi thiếu nhánh |
| Cần nhóm nhiều loại vào một nhánh | `switch expression` | đa hình không gộp được |

Ba dòng chữ thì không cần một cây kế thừa:

```csharp
string Label(OrderStatus status) => status switch
{
    OrderStatus.New => "Mới tạo",
    OrderStatus.Paid => "Đã thanh toán",
    OrderStatus.Cancelled => "Đã huỷ",
    _ => "Không xác định",
};
```

Phép thử nhanh: nếu mỗi nhánh chỉ **trả về một giá trị**, giữ `switch`. Nếu mỗi
nhánh **làm một việc khác nhau**, đó là lúc đa hình trả công.

## Không có sẵn lớp cha thì dùng Dictionary hàm

Có lúc bạn không sở hữu các kiểu ấy, hoặc chúng không cùng một họ. Lúc đó vẫn
tách được hành vi ra mà không cần kế thừa.

```csharp
var fees = new Dictionary<string, Func<decimal, decimal>>
{
    ["cash"] = a => 0,
    ["card"] = a => a * 0.02m,
    ["wallet"] = a => a * 0.01m,
};

decimal fee = fees["card"](100_000);
```

Thêm một hình thức là thêm một dòng vào bảng, không sửa chỗ gọi.

Đổi lại, bạn mất phần compiler kiểm tra. Gõ sai khoá thì nổ lúc chạy.

Cách này hợp khi các nhánh đều là công thức ngắn. Nhánh nào dài hơn ba dòng thì
đưa về class.

## Đa hình vẫn hỏng nếu lớp con làm trái lời hứa

Còn một điều kiện nữa, và nó không nằm trong cú pháp.

```csharp
// SAI — lớp con phá hợp đồng của lớp cha
class FreeCash : Cash
{
    public override decimal Fee(decimal a) =>
        throw new NotSupportedException();
}
```

Người gọi cầm một `Payment` và tin rằng gọi `Fee` thì ra một con số. Lớp này ném
lỗi, nên mọi chỗ đang chạy yên lành đều vỡ.

Đa hình chỉ có ích khi mọi lớp con **thay thế được** lớp cha. Nguyên tắc ấy tên
là Liskov, và chương SOLID sẽ nói kỹ.

## Dấu hiệu trong code của bạn

- Cùng một chuỗi `if` theo loại xuất hiện ở hai file trở lên → hành vi đang nằm sai chỗ, gom về class.
- `if (x is TypeA) … else if (x is TypeB)` rồi ép kiểu trong từng nhánh → đây là đa hình viết bằng tay.
- Một `enum` loại, cộng một `switch` trên nó ở mỗi tầng của hệ thống → thêm giá trị mới là đi sửa khắp nơi.
- Lớp con override rồi ném `NotSupportedException` → quan hệ kế thừa đó sai từ đầu.
- Chuỗi `if` mà mỗi nhánh chỉ `return` một chuỗi → để nguyên, `switch expression` đã đủ.

## Ghi nhớ

- Mỗi loại **làm** khác nhau thì dùng đa hình; chỉ **tra** một giá trị thì dùng `switch expression`.
- Thêm class không phải sửa chỗ gọi; thêm nhánh `if` thì phải tìm hết mọi chỗ.
- Quên một lớp con là lỗi compile, quên một nhánh `if` là bug lúc chạy.
- Không sở hữu các kiểu đó thì `Dictionary` hàm cũng tách được hành vi.
- Lớp con phải thay thế được lớp cha, nếu không đa hình thành bẫy.

## Bước tiếp theo

Hai bài vừa rồi đều dùng `abstract class` làm lớp cha. Nhưng chương trước lại
nói interface mới là hợp đồng.

Bài sau, **Interface hay abstract class**, trả lời câu hỏi đó bằng một bảng và
một chỗ hỏng mà chỉ default interface member mới gây ra được.

```quiz
[
  {
    "prompt": "Bạn thêm một loại thanh toán mới. Với thiết kế nào thì compiler nhắc bạn chỗ còn thiếu?",
    "options": [
      "Chuỗi if theo enum, có throw ở cuối",
      "switch expression có nhánh _ mặc định",
      "Một lớp con của abstract class, với method abstract chưa viết",
      "Dictionary ánh xạ enum sang hàm"
    ],
    "answer": 3,
    "explain": "Method abstract chưa viết là lỗi compile, kèm tên class còn thiếu. Ba cách kia đều có đường rơi mặc định, nên thiếu nhánh chỉ lộ ra lúc chạy."
  },
  {
    "prompt": "Đoạn này có nên đổi sang đa hình không?",
    "code": "string Label(OrderStatus s) => s switch\n{\n    OrderStatus.New => \"Mới tạo\",\n    OrderStatus.Paid => \"Đã thanh toán\",\n    _ => \"Không xác định\",\n};",
    "options": [
      "Không — mỗi nhánh chỉ trả một chuỗi, switch đã là cách gọn nhất",
      "Có — mọi switch theo loại đều nên thành class",
      "Có, nhưng chỉ khi có trên năm nhánh",
      "Không, vì OrderStatus là enum nên không dùng đa hình được"
    ],
    "answer": 1,
    "explain": "Đây là tra dữ liệu, không phải hành vi. Dựng ba class chỉ để trả ba chuỗi là thêm file mà không thêm giá trị. Đa hình trả công khi mỗi nhánh làm một việc khác nhau."
  },
  {
    "prompt": "Nhóm bạn dùng Dictionary<string, Func<...>> thay cho chuỗi if. Mất gì so với đa hình?",
    "options": [
      "Mất hiệu năng, vì lambda chậm hơn method",
      "Mất phần compiler kiểm tra: gõ sai khoá chỉ nổ lúc chạy",
      "Không mất gì, hai cách tương đương",
      "Mất khả năng thêm loại mới mà không sửa chỗ gọi"
    ],
    "answer": 2,
    "explain": "Khoá là chuỗi nên compiler không kiểm được. Bù lại cách này dùng được khi bạn không sở hữu các kiểu đó, và hợp với những nhánh chỉ là công thức ngắn."
  },
  {
    "prompt": "Một lớp con override method của lớp cha rồi ném NotSupportedException. Hệ quả nặng nhất là gì?",
    "options": [
      "Code chậm hơn vì phải tạo exception",
      "Vi phạm quy ước đặt tên",
      "Compiler cảnh báo lúc build",
      "Mọi chỗ đang cầm kiểu cha và tin vào hợp đồng của nó đều có thể vỡ"
    ],
    "answer": 4,
    "explain": "Người gọi chỉ thấy kiểu cha và tin rằng method ấy trả về một giá trị. Một lớp con ném lỗi làm hỏng đúng cái tính thay thế được mà đa hình dựa vào."
  }
]
```
