---
title: SRP — Một class, một trách nhiệm
minutes: 5
---

Class `OrderService` vừa tính tiền, vừa lưu file, vừa gửi email. Kế toán đổi
thuế thì sửa nó, đổi nơi lưu cũng sửa nó, đổi mẫu email lại sửa nó. Sửa
chỗ này dễ làm hỏng chỗ kia. SRP là nguyên tắc đầu tiên của SOLID, giúp tránh
đúng chuyện này.

## Khái niệm

🎯 **SRP (Single Responsibility Principle)**: mỗi class chỉ nên có một lý do để thay đổi.

"Một lý do để thay đổi" thường ứng với một nhóm người yêu cầu sửa: kế toán lo
cách tính tiền, bộ phận vận hành lo nơi lưu dữ liệu, marketing lo nội dung
email.

## Ví dụ

```csharp
var calculator = new OrderCalculator();
var notifier = new OrderNotifier();

decimal total = calculator.Total(200000m);
notifier.Send("an@shop.vn", total);

class OrderCalculator
{
    public decimal Total(decimal subtotal) =>
        subtotal + subtotal * 10 / 100;
}

class OrderNotifier
{
    public void Send(string email, decimal total) =>
        Console.WriteLine($"Gửi {email}: {total}đ");
}
```

- `OrderCalculator` chỉ tính tiền. Đổi thuế thì chỉ sửa class này.
- `OrderNotifier` chỉ gửi thông báo. Đổi nội dung thông báo thì chỉ sửa
  class này.
- Mỗi class nhỏ, đọc nhanh, và sửa một class không đụng tới class kia.

## Thử ngay

Chép ví dụ trên vào `Program.cs` rồi chạy `dotnet run`. Sau đó đổi thuế từ
10% sang 8% trong `OrderCalculator` và chạy lại.

**Đoán trước khi chạy:** lần chạy đầu in ra gì, và muốn đổi thuế thì phải mở
mấy class?

<details>
<summary>Xem kết quả</summary>

```text
Gửi an@shop.vn: 220000đ
```

Chỉ phải mở một class là `OrderCalculator`. `OrderNotifier` không biết gì về
thuế nên không bị ảnh hưởng. Sau khi đổi sang 8%, dòng in ra là `216000đ`.

</details>

## Lỗi hay gặp

**Một class ôm mọi việc.** Ba lý do để sửa nằm chung một chỗ.

```csharp
// SAI — tính tiền, lưu file, gửi email chung một class
class OrderService
{
    public decimal Total(decimal subtotal) =>
        subtotal + subtotal * 10 / 100;

    public void SaveToFile(decimal total) =>
        File.WriteAllText("order.txt", $"{total}");

    public void SendEmail(string email) =>
        Console.WriteLine($"Gửi {email}");
}
```

Cách sửa: tách thành `OrderCalculator`, `OrderFileStore`, `OrderNotifier`,
mỗi class một việc như ở ví dụ trên.

**Tách quá vụn.** Mỗi class chỉ có một method một dòng, phải đọc qua năm
class mới hiểu một luồng, thì code còn khó đọc hơn. SRP chia theo **lý do
thay đổi**, không phải theo số method.

## Tóm tắt

- SRP: mỗi class chỉ có một lý do để thay đổi.
- Lý do thay đổi thường ứng với một nhóm người hay một phần nghiệp vụ.
- Class làm nhiều việc thì sửa một việc dễ làm hỏng việc khác.
- Chia theo lý do thay đổi, đừng chia vụn cho có.

```quiz
[
  {
    "prompt": "Class Invoice có các method: Calculate(), PrintPdf(), SaveToDatabase(). Class này có mấy lý do để thay đổi?",
    "options": [
      "1",
      "2",
      "3",
      "0"
    ],
    "answer": 3,
    "explain": "Đổi cách tính, đổi mẫu PDF, đổi nơi lưu là ba lý do khác nhau, thường do ba nhóm người khác nhau yêu cầu."
  },
  {
    "prompt": "Cách tách nào hợp lý nhất cho class Invoice ở câu trên?",
    "options": [
      "InvoiceCalculator, InvoicePdfPrinter, InvoiceRepository",
      "Mỗi method một project riêng",
      "Giữ nguyên, thêm comment phân chia",
      "Chuyển hết thành static method"
    ],
    "answer": 1,
    "explain": "Mỗi class ứng với đúng một lý do thay đổi. Sửa cách in PDF không đụng tới cách tính hay cách lưu."
  },
  {
    "prompt": "Câu nào mô tả đúng SRP?",
    "options": [
      "Mỗi class chỉ được có một method",
      "Mỗi method chỉ được có một dòng",
      "Mỗi project chỉ có một class",
      "Mỗi class chỉ nên có một lý do để thay đổi"
    ],
    "answer": 4,
    "explain": "SRP nói về lý do thay đổi, không phải số method. Một class có nhiều method vẫn đúng SRP nếu chúng phục vụ cùng một trách nhiệm."
  }
]
```
