---
title: enum
minutes: 4
---

Đơn hàng chỉ có vài trạng thái: mới, đã thanh toán, đang giao, đã huỷ. Nếu
lưu trạng thái bằng chuỗi, gõ nhầm `"paid"` thành `"piad"` thì compiler cũng
không phát hiện. enum giới hạn giá trị vào đúng danh sách cho phép.

## Khái niệm

🎚️ **enum**: kiểu gồm một danh sách giá trị có tên cố định, biến kiểu enum chỉ nhận được một trong các giá trị đó.

## Ví dụ

```csharp
OrderStatus status = OrderStatus.Paid;

if (status == OrderStatus.Paid)
{
    Console.WriteLine("Chuẩn bị giao hàng");
}

Console.WriteLine(status);        // Paid
Console.WriteLine((int)status);   // 1

enum OrderStatus
{
    New,
    Paid,
    Shipping,
    Cancelled
}
```

- Khai báo `enum OrderStatus { ... }`, dùng giá trị qua tên enum:
  `OrderStatus.Paid`.
- Gõ sai tên như `OrderStatus.Piad` là lỗi compile ngay.
- In ra thì được tên (`Paid`). Mỗi giá trị còn có một số nguyên đi kèm, đánh
  số từ 0: `New` là 0, `Paid` là 1.
- enum là value type.

## Thử ngay

Chép khai báo `OrderStatus` ở ví dụ trên, thay các dòng đầu bằng:

```csharp
OrderStatus status = OrderStatus.Shipping;

switch (status)
{
    case OrderStatus.New:
        Console.WriteLine("Chờ thanh toán");
        break;
    case OrderStatus.Paid:
        Console.WriteLine("Chuẩn bị giao");
        break;
    case OrderStatus.Shipping:
        Console.WriteLine("Đang giao");
        break;
    default:
        Console.WriteLine("Đã huỷ");
        break;
}

Console.WriteLine((int)status);
```

**Đoán trước khi chạy:** dòng cuối in ra `Shipping` hay một con số?

<details>
<summary>Xem kết quả</summary>

```text
Đang giao
2
```

Ép sang `int` thì được số thứ tự: `New` 0, `Paid` 1, `Shipping` 2.

</details>

## Lỗi hay gặp

**Dùng chuỗi cho giá trị cố định.** Gõ nhầm vẫn chạy, chỉ là chạy sai.

```csharp
// SAI — gõ nhầm "piad", điều kiện sai mà không ai biết
string status = "piad";
if (status == "paid")
{
    Console.WriteLine("Chuẩn bị giao hàng");
}
```

```csharp
// ĐÚNG — gõ nhầm tên enum sẽ bị compiler bắt
OrderStatus status = OrderStatus.Paid;
if (status == OrderStatus.Paid)
{
    Console.WriteLine("Chuẩn bị giao hàng");
}
```

**Tên enum đặt số nhiều.** Một biến chỉ giữ một trạng thái, nên tên enum là
số ít.

```csharp
// SAI — đọc thành "một đơn có nhiều trạng thái"
enum OrderStatuses { New, Paid }
```

```csharp
// ĐÚNG
enum OrderStatus { New, Paid }
```

## Tóm tắt

- enum là danh sách giá trị có tên cố định, như trạng thái đơn hàng.
- Dùng qua tên enum: `OrderStatus.Paid`. Gõ sai tên là lỗi compile.
- Mỗi giá trị có số nguyên đi kèm, mặc định đánh từ 0.
- Dùng enum thay cho chuỗi khi giá trị nằm trong một danh sách cố định.

```quiz
[
  {
    "prompt": "enum Size { Small, Medium, Large } Console.WriteLine((int)Size.Large); In ra gì?",
    "options": [
      "Large",
      "3",
      "2",
      "1"
    ],
    "answer": 3,
    "explain": "Số đánh từ 0: Small 0, Medium 1, Large 2."
  },
  {
    "prompt": "Trường nào nên dùng enum?",
    "options": [
      "Phương thức thanh toán: tiền mặt, thẻ, ví điện tử",
      "Tên sản phẩm",
      "Ghi chú của khách",
      "Số điện thoại"
    ],
    "answer": 1,
    "explain": "Phương thức thanh toán chỉ có vài giá trị cố định. Ba trường còn lại có thể là bất kỳ chuỗi nào."
  },
  {
    "prompt": "Vì sao enum an toàn hơn chuỗi khi lưu trạng thái đơn hàng?",
    "options": [
      "enum chạy nhanh hơn string",
      "enum tốn ít bộ nhớ hơn",
      "enum in ra đẹp hơn",
      "Gõ sai tên giá trị enum là lỗi compile, gõ sai chuỗi thì không"
    ],
    "answer": 4,
    "explain": "Compiler chỉ cho dùng các giá trị đã khai báo trong enum, nên lỗi gõ nhầm bị phát hiện ngay lúc build."
  }
]
```
