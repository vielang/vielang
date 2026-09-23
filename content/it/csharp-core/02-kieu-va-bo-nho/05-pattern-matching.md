---
title: Pattern matching
minutes: 11
---

Bạn mở một file service trong dự án, và thấy hai mươi dòng lặp đi lặp lại.

Kiểm tra kiểu. Ép kiểu. Gán vào biến mới. Rồi mới dùng được.

Cùng việc đó, C# hiện đại viết trong một dòng. Và khi đọc code người khác, bạn
sẽ gặp dạng một dòng nhiều hơn hẳn.

> **Học xong bài này bạn sẽ:** đọc được `switch expression` với property
> pattern trong code thật; thay chuỗi `if` ép kiểu bằng một biểu thức; biết
> pattern lồng tự xử lý null thế nào.
>
> **Cần biết trước:** `switch expression`, `record`, `enum`.

## Năm loại pattern hay gặp

Hai mươi dòng kia rút được vì C# có một bộ công cụ riêng cho việc "hỏi xem cái
này là gì".

| Loại | Viết thế nào | Hỏi gì |
|---|---|---|
| Type | `x is Order o` | có phải kiểu này không |
| Property | `{ Total: > 1000 }` | property có giá trị thế nào |
| Relational | `> 80`, `< 0` | so sánh với một mốc |
| Logical | `and`, `or`, `not` | ghép các pattern lại |
| List | `["GET", var path]` | mảng có hình dạng thế nào |

Cả năm dùng được ở hai chỗ. Sau `is`, và trong nhánh của `switch expression`.

## Type pattern gộp kiểm tra với ép kiểu

Bắt đầu từ loại thay thế trực tiếp cho hai mươi dòng ở đầu bài.

```csharp
// Cách cũ
if (shape is Circle)
{
    var circle = (Circle)shape;
    Console.WriteLine(circle.Radius);
}

// Pattern matching: một bước
if (shape is Circle c)
    Console.WriteLine(c.Radius);

if (value is not string text)
    return;   // text dùng được ở phần còn lại
```

Biến `c` chỉ tồn tại khi phép kiểm tra đúng.

Không còn cảnh ép kiểu hai lần. Cũng không còn chỗ để ép nhầm kiểu.

## Thử ngay: property pattern tự xử lý null

Loại thứ hai có một tính chất mà đọc thì không tin, phải chạy mới tin.

```csharp
decimal Fee(Order o) => o switch
{
    { Total: > 1_000_000 } => 0,
    { ShipTo.City: "Hà Nội" } => 15_000,
    _ => 30_000,
};

Console.WriteLine(Fee(new Order(2_000_000, null)));
var hanoi = new Address("Hà Nội");
Console.WriteLine(Fee(new Order(50_000, hanoi)));
Console.WriteLine(Fee(new Order(50_000, null)));

record Address(string City);

record Order(decimal Total, Address? ShipTo);
```

**Đoán trước khi chạy:** ba dòng in ra gì? Chú ý dòng đầu và dòng cuối có
`ShipTo` là `null`.

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0
15000
30000
```

Không dòng nào ném `NullReferenceException`.

Pattern `{ ShipTo.City: … }` không khớp khi `ShipTo` là null. Nó **rơi xuống
nhánh dưới** thay vì nổ. Đó là lý do nó gọn hơn hẳn
`if (o.ShipTo != null && o.ShipTo.City == …)`.

</details>

## Relational và logical pattern gộp nhiều điều kiện

So với một mốc, rồi ghép nhiều phép so lại. Đây là chỗ `switch` thay được cả
một thang điều kiện.

```csharp
string Grade(int score) => score switch
{
    < 0 or > 100 =>
        throw new ArgumentOutOfRangeException(),
    >= 80 => "Giỏi",
    >= 50 => "Khá",
    _ => "Trung bình",
};

char ch = line[0];
bool isDigit = ch is >= '0' and <= '9';
```

`and`, `or`, `not` ghép các pattern lại.

`x is not null` đọc xuôi hơn hẳn `x != null`. Nó cũng an toàn hơn, nếu kiểu đó
nạp chồng toán tử `==`.

Để ý nhánh đầu. Nó **ném lỗi** ngay trong một biểu thức, và đó là cú pháp
hợp lệ. Nhờ vậy bạn không phải tách ra một câu `if` riêng phía trên.

## List pattern đọc dữ liệu theo hình dạng

Loại cuối hỏi về hình dạng của một mảng, không phải giá trị của một phần tử.

```csharp
var parts = line.Split(':');

var description = parts switch
{
    ["GET", var path] => $"Đọc {path}",
    ["POST", var path, ..] => $"Ghi {path}",
    [] => "Dòng trống",
    _ => "Không hiểu",
};
```

`..` là **slice pattern**, nghĩa là "còn lại bao nhiêu cũng được". Rất hợp khi
phân tích dòng log hay lệnh dạng chuỗi.

Thứ tự các nhánh quan trọng. Nhánh nào khớp trước thì thắng.

Nên đặt pattern cụ thể lên trên, pattern rộng xuống dưới.

## Kết hợp với record: thay cả cây if

Năm loại pattern gộp lại thì làm được thứ đáng làm nhất: xoá hẳn một cây `if`
phân loại theo kiểu.

```csharp
abstract record Payment;
record Cash(decimal Amount) : Payment;
record Card(decimal Amount, string Last4) : Payment;
record Transfer(decimal Amount, string Bank) : Payment;

string Describe(Payment p) => p switch
{
    Cash { Amount: > 10_000_000 } =>
        "Tiền mặt, cần xác minh",
    Cash cash => $"Tiền mặt {cash.Amount:N0}",
    Card (var amount, var last4) =>
        $"Thẻ ****{last4}, {amount:N0}",
    Transfer t => $"Chuyển khoản qua {t.Bank}",
};
```

`Card (var amount, var last4)` là **positional pattern**. Nó chạy được vì
`record` tự sinh sẵn `Deconstruct` cho bạn.

Nhưng C# không coi cây kế thừa này là đóng. `switch` vẫn báo **CS8509**: còn
`null`, và còn kiểu con chưa ai viết ra.

Thêm `_ => throw new NotSupportedException(p.GetType().Name)` là hết cảnh báo.

Đổi lại, hôm có loại thanh toán mới thì nhánh đó nổ ngay lần chạy đầu. Đúng một
chỗ, và nói đúng tên kiểu còn thiếu.

## Dấu hiệu trong code của bạn

- Cặp `if (x is T)` rồi `(T)x` ngay dòng dưới → gộp thành `if (x is T t)`.
- Chuỗi `if` kiểm tra null rồi mới so property → một property pattern là xong.
- `switch` dạng câu lệnh mà mỗi nhánh chỉ gán một giá trị → đổi sang `switch expression`.
- Chuỗi `if` phân loại theo kiểu con của một lớp cha → `switch expression` trên kiểu, thêm nhánh `_ => throw` để kiểu con mới nổ đúng một chỗ thay vì rơi vào im lặng.

## Ghi nhớ

- `is` vừa kiểm tra kiểu vừa gán biến.
- Pattern lồng tự xử lý null: không khớp thì rơi sang nhánh khác, không ném lỗi.
- `switch expression` trả về giá trị, mỗi nhánh là một biểu thức.
- `and`, `or`, `not` ghép pattern; `is not null` là cách viết chuẩn hiện nay.

## Bước tiếp theo

Hết chương **Kiểu và bộ nhớ**. Bạn đã biết chọn kiểu và xử lý chúng gọn gàng.

Chương sau, **Collection và LINQ**, mở bằng một trang đồng bộ chạy 40 mili giây
ở máy dev. Với 200.000 bản ghi thật, nó mất gần một phút.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì khi o.ShipTo là null?",
    "code": "decimal Fee(Order o) => o switch\n{\n    { ShipTo.City: \"Hà Nội\" } => 15_000,\n    _ => 30_000,\n};",
    "options": [
      "Ném NullReferenceException",
      "15000",
      "30000",
      "Lỗi compile vì thiếu kiểm tra null"
    ],
    "answer": 3,
    "explain": "Pattern lồng không khớp khi một mắt xích là null — nó rơi xuống nhánh _ thay vì nổ. Đó là điểm mạnh so với chuỗi if kiểm tra null."
  },
  {
    "prompt": "Cách viết nào thay được hai dòng kiểm tra kiểu rồi ép kiểu?",
    "code": "if (shape is Circle)\n{\n    var c = (Circle)shape;\n}",
    "options": [
      "if (shape is Circle c)",
      "if (shape as Circle)",
      "if (shape.GetType() == typeof(Circle))",
      "switch (shape) { case Circle: break; }"
    ],
    "answer": 1,
    "explain": "Type pattern kiểm tra kiểu và gán biến trong một bước; biến c dùng được ngay trong thân if."
  },
  {
    "prompt": "switch trên kiểu Payment có nhánh _ => throw. Nửa năm sau đồng nghiệp thêm record Momo : Payment. Chuyện gì xảy ra?",
    "options": [
      "Compiler chặn ngay lúc build, vì switch không còn đầy đủ",
      "Trả về null, không ai biết gì",
      "Momo lặng lẽ rơi vào nhánh của Cash",
      "Chạy tới một khoản Momo mới nổ, đúng một chỗ và nói rõ tên kiểu còn thiếu"
    ],
    "answer": 4,
    "explain": "C# không coi cây kế thừa là đóng, nên compiler không thể biết Momo cần xử lý ở đâu — nó không chặn được lúc build. Nhánh _ => throw là cách biến chỗ thiếu thành một lỗi nói rõ tên kiểu, thay vì một hành vi sai âm thầm."
  },
  {
    "prompt": "parts là mảng [\"POST\", \"/orders\", \"1\"]. Pattern nào khớp?",
    "options": [
      "[\"POST\", var p]",
      "Cả [\"POST\", var p, ..] và [..]",
      "[..]",
      "[\"POST\", var p, ..]"
    ],
    "answer": 2,
    "explain": "[\"POST\", var p] đòi đúng 2 phần tử nên không khớp. [\"POST\", var p, ..] khớp vì .. nhận phần còn lại, và [..] khớp mọi mảng — nhánh nào viết trước thì thắng."
  }
]
```
