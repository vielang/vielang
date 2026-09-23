---
title: Pattern matching
minutes: 11
---

Bạn mở một file service trong dự án và thấy hai mươi dòng lặp đi lặp lại: kiểm
tra kiểu, ép kiểu, gán vào biến mới, rồi mới dùng được.

Cùng việc đó, C# hiện đại viết trong một dòng. Và khi đọc code người khác, bạn
sẽ gặp dạng một dòng nhiều hơn hẳn.

> **Học xong bài này bạn sẽ:** đọc được `switch expression` với property
> pattern trong code thật; thay chuỗi `if` ép kiểu bằng một biểu thức; biết
> pattern lồng tự xử lý null thế nào.
>
> **Cần biết trước:** `switch expression`, `record`, `enum`.

## Năm loại pattern hay gặp

| Loại | Viết thế nào | Hỏi gì |
|---|---|---|
| Type | `x is Order o` | có phải kiểu này không |
| Property | `{ Total: > 1000 }` | property có giá trị thế nào |
| Relational | `> 80`, `< 0` | so sánh với một mốc |
| Logical | `and`, `or`, `not` | ghép các pattern lại |
| List | `["GET", var path]` | mảng có hình dạng thế nào |

Cả năm dùng được ở hai chỗ. Sau `is`, và trong nhánh của `switch expression`.

## Type pattern gộp kiểm tra với ép kiểu

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

Biến `c` chỉ tồn tại khi phép kiểm tra đúng. Không còn cảnh ép kiểu hai lần,
cũng không còn chỗ để ép nhầm kiểu.

## Thử ngay: property pattern tự xử lý null

```csharp
record Address(string City);
record Order(decimal Total, Address? ShipTo);

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

```csharp
string Grade(int score) => score switch
{
    < 0 or > 100 =>
        throw new ArgumentOutOfRangeException(),
    >= 80 => "Giỏi",
    >= 50 => "Khá",
    _ => "Trung bình",
};

bool isDigit = c is >= '0' and <= '9';
```

`and`, `or`, `not` ghép các pattern lại. `x is not null` đọc xuôi hơn hẳn
`x != null`, và cũng an toàn hơn nếu kiểu đó nạp chồng toán tử `==`.

Để ý nhánh đầu. Nó **ném lỗi** ngay trong một biểu thức, và đó là cú pháp
hợp lệ. Nhờ vậy bạn không phải tách ra một câu `if` riêng phía trên.

## List pattern đọc dữ liệu theo hình dạng

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

Thứ tự các nhánh quan trọng. Nhánh nào khớp trước thì thắng, nên đặt pattern
cụ thể lên trên.

## Kết hợp với record: thay cả cây if

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

Kiểu cha là `abstract record` và các nhánh liệt kê đủ, nên compiler không còn
cảnh báo thiếu nhánh. Thêm một loại thanh toán mới, compiler sẽ nhắc bạn ngay
tại đây.

## Dấu hiệu trong code của bạn

- Cặp `if (x is T)` rồi `(T)x` ngay dòng dưới → gộp thành `if (x is T t)`.
- Chuỗi `if` kiểm tra null rồi mới so property → một property pattern là xong.
- `switch` dạng câu lệnh mà mỗi nhánh chỉ gán một giá trị → đổi sang `switch expression`.
- Chuỗi `if` phân loại theo kiểu con của một lớp cha → `switch expression` trên kiểu, và compiler sẽ nhắc khi có kiểu con mới.

## Ghi nhớ

- `is` vừa kiểm tra kiểu vừa gán biến.
- Pattern lồng tự xử lý null: không khớp thì rơi sang nhánh khác, không ném lỗi.
- `switch expression` trả về giá trị, mỗi nhánh là một biểu thức.
- `and`, `or`, `not` ghép pattern; `is not null` là cách viết chuẩn hiện nay.

## Bước tiếp theo

Hết chương **Kiểu và bộ nhớ**. Bạn đã biết chọn kiểu và xử lý chúng gọn gàng.

Chương sau, **Collection và LINQ**, mở bằng một trang đồng bộ chạy 40 mili
giây ở máy dev. Với dữ liệu thật, nó mất gần một phút.

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
      "if (shape as Circle)",
      "if (shape is Circle c)",
      "if (shape.GetType() == typeof(Circle))",
      "switch (shape) { case Circle: break; }"
    ],
    "answer": 2,
    "explain": "Type pattern kiểm tra kiểu và gán biến trong một bước; biến c dùng được ngay trong thân if."
  },
  {
    "prompt": "Bạn thêm một record con mới kế thừa lớp cha abstract. Cách viết nào giúp compiler nhắc chỗ còn thiếu xử lý?",
    "options": [
      "Chuỗi if - else if theo kiểu",
      "switch expression trên kiểu, liệt kê đủ nhánh",
      "Dictionary ánh xạ kiểu sang hàm xử lý",
      "try - catch InvalidCastException"
    ],
    "answer": 2,
    "explain": "Compiler kiểm tra tính đầy đủ của switch expression và cảnh báo khi còn trường hợp chưa xử lý; if thì im lặng."
  },
  {
    "prompt": "parts là mảng [\"POST\", \"/orders\", \"1\"]. Pattern nào khớp?",
    "options": [
      "[\"POST\", var p]",
      "[\"POST\", var p, ..]",
      "[..]",
      "Cả B và C"
    ],
    "answer": 4,
    "explain": "[\"POST\", var p] đòi đúng 2 phần tử nên không khớp. [\"POST\", var p, ..] khớp vì .. nhận phần còn lại, và [..] khớp mọi mảng — nhánh nào viết trước thì thắng."
  }
]
```
