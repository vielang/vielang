---
title: SRP — Một class, một trách nhiệm
minutes: 5
---

Class `OrderService` vừa tính tiền, vừa lưu file, vừa gửi email, nên đổi
thuế, đổi nơi lưu hay đổi mẫu email đều phải sửa nó. Sửa chỗ này dễ làm hỏng
chỗ kia. SRP là nguyên tắc đầu tiên của SOLID, giúp tránh
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

**Đoán trước khi chạy:** lần chạy đầu và lần chạy sau khi đổi thuế in ra số
tiền bao nhiêu?

<details>
<summary>Xem kết quả</summary>

```text
Gửi an@shop.vn: 220000đ
```

Lần đầu in `220000đ`, sau khi đổi sang 8% in `216000đ`. Việc đổi thuế chỉ
chạm vào `OrderCalculator`, còn `OrderNotifier` giữ nguyên.

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
        Console.WriteLine($"Lưu file: {total}");

    public void SendEmail(string email) =>
        Console.WriteLine($"Gửi {email}");
}
```

```csharp
// ĐÚNG — mỗi class một lý do để thay đổi
class OrderCalculator
{
    public decimal Total(decimal subtotal) =>
        subtotal + subtotal * 10 / 100;
}

class OrderFileStore
{
    public void Save(decimal total) =>
        Console.WriteLine($"Lưu file: {total}");
}

class OrderNotifier
{
    public void Send(string email) =>
        Console.WriteLine($"Gửi {email}");
}
```

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
      "3",
      "1",
      "2",
      "0"
    ],
    "answer": 1,
    "explain": "Đổi cách tính, đổi mẫu PDF, đổi nơi lưu là ba lý do khác nhau, thường do ba nhóm người khác nhau yêu cầu."
  },
  {
    "prompt": "Cách tách nào hợp lý nhất cho class Invoice ở câu trên?",
    "options": [
      "Tách mỗi method ra một project",
      "Tách ba class: tính, in PDF, lưu",
      "Giữ nguyên, thêm comment phân chia",
      "Đổi mọi method thành static"
    ],
    "answer": 2,
    "explain": "Mỗi class ứng với đúng một lý do thay đổi. Sửa cách in PDF không đụng tới cách tính hay cách lưu."
  },
  {
    "prompt": "Câu nào mô tả đúng SRP?",
    "options": [
      "Mỗi class chỉ được có một method",
      "Mỗi method chỉ được có một dòng",
      "Mỗi class chỉ có một lý do thay đổi",
      "Mỗi class chỉ được gọi từ một nơi"
    ],
    "answer": 3,
    "explain": "SRP nói về lý do thay đổi, không phải số method. Một class có nhiều method vẫn đúng SRP nếu chúng phục vụ cùng một trách nhiệm."
  }
]
```
