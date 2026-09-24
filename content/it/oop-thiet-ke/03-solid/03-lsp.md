---
title: LSP — Class con phải thay được class cha
minutes: 5
---

Vòng lặp hoàn tiền chạy ổn với mọi hình thức thanh toán, cho tới khi có thêm
thẻ quà tặng. Class này kế thừa `Payment` nhưng `Refund()` lại ném lỗi, vì thẻ
quà tặng không hoàn được. Chương trình sập ở chỗ không ai ngờ. LSP giúp tránh
loại lỗi này.

## Khái niệm

🔁 **LSP (Liskov Substitution Principle)**: object của class con phải dùng thay được cho object của class cha mà chương trình vẫn chạy đúng.

Nói cách khác, class con không được làm **ít hơn** những gì class cha đã hứa.
Class cha có `Refund()` thì mọi class con đều phải hoàn tiền được.

## Ví dụ

```csharp
// SAI — GiftCard kế thừa nhưng không làm được Refund
var payments = new List<Payment>
{
    new CardPayment(),
    new GiftCardPayment(),
};

foreach (Payment p in payments)
{
    p.Refund();   // sập ở GiftCardPayment
}

abstract class Payment
{
    public abstract void Refund();
}

class CardPayment : Payment
{
    public override void Refund() =>
        Console.WriteLine("Hoàn về thẻ");
}

class GiftCardPayment : Payment
{
    public override void Refund() =>
        throw new NotSupportedException(
            "Thẻ quà tặng không hoàn được");
}
```

- Vòng lặp tin rằng mọi `Payment` đều hoàn tiền được, vì class cha đã hứa
  như vậy.
- `GiftCardPayment` phá lời hứa đó, nên thay nó vào chỗ `Payment` là
  chương trình sập.
- Lỗi không nằm ở vòng lặp, mà ở chỗ `GiftCardPayment` không nên có
  `Refund()`.

## Sửa bằng interface

Chỉ class nào hoàn được mới nhận khả năng hoàn tiền, giống bài
**Interface hay abstract class**:

```csharp
var refunds = new List<IRefundable>
{
    new CardPayment(),
};

foreach (IRefundable r in refunds)
{
    r.Refund();
}

abstract class Payment
{
    public decimal Amount { get; set; }
}

interface IRefundable
{
    void Refund();
}

class CardPayment : Payment, IRefundable
{
    public void Refund() =>
        Console.WriteLine("Hoàn về thẻ");
}

// Không hứa hoàn tiền, nên không vào được list trên
class GiftCardPayment : Payment
{
}
```

## Thử ngay

Chép khối **SAI** ở phần Ví dụ vào `Program.cs` rồi chạy `dotnet run`.

**Đoán trước khi chạy:** dòng "Hoàn về thẻ" có được in ra trước khi chương
trình sập không?

<details>
<summary>Xem kết quả</summary>

```text
Hoàn về thẻ
Unhandled exception. System.NotSupportedException: Thẻ quà tặng không hoàn được
```

Có. Phần tử đầu chạy đúng, tới phần tử thứ hai thì sập. Lỗi kiểu này chỉ lộ
ra khi dữ liệu thật có thẻ quà tặng, nên rất khó phát hiện sớm.

</details>

## Lỗi hay gặp

**Override chỉ để ném exception.** Đây là dấu hiệu rõ nhất của vi phạm LSP.

```csharp
// SAI — nhận tại cửa hàng thì không có mã vận đơn
abstract class Shipment
{
    public abstract string Track();
}

class StorePickup : Shipment
{
    public override string Track() =>
        throw new NotSupportedException();
}
```

Nếu class con không làm được một việc của class cha, thì nó không phải là một
loại của class cha. Hãy dùng interface nhỏ hơn hoặc composition.

## Tóm tắt

- LSP: class con phải thay được class cha mà chương trình vẫn chạy đúng.
- Class con không được làm ít hơn những gì class cha đã hứa.
- Override chỉ để ném `NotSupportedException` là dấu hiệu vi phạm.
- Cách sửa: tách khả năng ra interface riêng, chỉ class làm được mới
  implement.

```quiz
[
  {
    "prompt": "Class Bird có method Fly(). Class Penguin : Bird override Fly() bằng cách ném exception. Vấn đề là gì?",
    "options": [
      "Không có vấn đề, override là để làm vậy",
      "Vi phạm LSP: code dùng Bird sẽ sập khi gặp Penguin",
      "Vi phạm SRP",
      "Chỉ là lỗi đặt tên"
    ],
    "answer": 2,
    "explain": "Code gọi bird.Fly() tin rằng mọi Bird đều bay được. Penguin phá lời hứa đó nên không thay được cho Bird."
  },
  {
    "prompt": "Cách sửa hợp lý cho ví dụ Bird và Penguin?",
    "options": [
      "Bắt mọi nơi gọi Fly() kiểm tra có phải Penguin không",
      "Để Fly() của Penguin không làm gì mà không báo",
      "Xoá class Penguin",
      "Tách interface IFlyable, chỉ loài biết bay mới implement"
    ],
    "answer": 4,
    "explain": "Bay là một khả năng, không phải chim nào cũng có. Tách ra interface thì chỉ loài bay được mới hứa bay."
  },
  {
    "prompt": "Dấu hiệu nào cho thấy có thể đang vi phạm LSP?",
    "options": [
      "Class con override method và ném NotSupportedException",
      "Class con thêm property mới",
      "Class con gọi base(...) trong constructor",
      "Class có nhiều method"
    ],
    "answer": 1,
    "explain": "Class con từ chối làm việc class cha đã hứa, nên thay nó vào chỗ class cha sẽ làm chương trình lỗi."
  }
]
```
