---
title: Pattern matching
minutes: 8
---

**Pattern matching** là cách hỏi "giá trị này có dạng thế nào" thay vì phải ép
kiểu rồi kiểm tra từng bước. Code C# hiện đại dùng nó khắp nơi, nên đọc được nó
là điều kiện để đọc được code người khác.

## Type pattern

```csharp
// Cách cũ
if (shape is Circle)
{
    var circle = (Circle)shape;
    Console.WriteLine(circle.Radius);
}

// Pattern matching: kiểm tra và gán trong một bước
if (shape is Circle circle)
    Console.WriteLine(circle.Radius);

if (value is not string text)
    return;                      // text dùng được ở phần còn lại của method
```

## switch expression với pattern

```csharp
decimal PhiVanChuyen(Order order) => order switch
{
    { Total: > 1_000_000 }                 => 0,
    { Address.City: "Hà Nội" or "TP.HCM" } => 15_000,
    { Items.Count: 0 }                     => throw new ArgumentException("Đơn hàng trống"),
    _                                      => 30_000,
};
```

`{ Total: > 1_000_000 }` là **property pattern**: đọc thẳng property và so sánh.
`Address.City` là pattern lồng — không cần tự kiểm tra `Address` có null hay
không, pattern không khớp thì rơi xuống nhánh dưới.

## Relational và logical pattern

```csharp
string XepLoai(int score) => score switch
{
    < 0 or > 100 => throw new ArgumentOutOfRangeException(nameof(score)),
    >= 80        => "Giỏi",
    >= 50        => "Khá",
    _            => "Trung bình",
};

bool laKyTuSo = c is >= '0' and <= '9';
```

`and`, `or`, `not` ghép các pattern lại. `not null` đọc xuôi hơn hẳn `!= null`.

## List pattern

```csharp
var parts = line.Split(':');

var mota = parts switch
{
    ["GET", var path]        => $"Đọc {path}",
    ["POST", var path, ..]   => $"Ghi {path}",
    []                       => "Dòng trống",
    _                        => "Không hiểu",
};
```

`..` là **slice pattern**: "còn lại bao nhiêu cũng được". Rất hợp khi phân tích
dòng log hay lệnh dạng chuỗi.

## Kết hợp với record

```csharp
public abstract record Payment;
public record Cash(decimal Amount) : Payment;
public record Card(decimal Amount, string Last4) : Payment;
public record Transfer(decimal Amount, string Bank) : Payment;

string MoTa(Payment payment) => payment switch
{
    Cash { Amount: > 10_000_000 } => "Tiền mặt, cần xác minh",
    Cash c                        => $"Tiền mặt {c.Amount:N0}",
    Card (var amount, var last4)  => $"Thẻ ****{last4}, {amount:N0}",
    Transfer t                    => $"Chuyển khoản qua {t.Bank}",
};
```

`Card (var amount, var last4)` là **positional pattern**, dùng được vì `record`
tự sinh sẵn `Deconstruct`. Kiểu cha đánh dấu `abstract record` và các nhánh liệt
kê đủ thì compiler không còn cảnh báo thiếu nhánh.

## Ghi nhớ

- `is` vừa kiểm tra kiểu vừa gán biến — không còn cần ép kiểu hai lần.
- `switch expression` trả về giá trị, mỗi nhánh là một biểu thức, không có `break`.
- Pattern lồng tự xử lý null: không khớp thì rơi sang nhánh khác, không ném lỗi.
