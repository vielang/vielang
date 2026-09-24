---
title: OCP — Mở rộng mà không sửa code cũ
minutes: 5
---

Cửa hàng có giảm giá cho khách thường và khách thân thiết. Mỗi lần thêm loại
khách mới, bạn lại mở method tính giảm giá ra thêm một nhánh `if`, và mỗi lần
sửa code đang chạy là một lần có thể làm hỏng. OCP
chỉ cách thêm tính năng mà không đụng vào code cũ.

## Khái niệm

🔓 **OCP (Open/Closed Principle)**: class nên mở cho việc mở rộng nhưng đóng với việc sửa đổi.

Nghĩa là thêm tính năng bằng cách viết code mới thay vì sửa code đang chạy.
Cách làm thường gặp: đưa phần hay thay đổi ra sau một interface. Mỗi biến thể
mới là một class mới implement interface đó.

## Ví dụ

```csharp
var checkout = new Checkout();
Console.WriteLine(
    checkout.Pay(100000m, new RegularDiscount()));
Console.WriteLine(
    checkout.Pay(100000m, new LoyalDiscount()));

interface IDiscount
{
    decimal Apply(decimal amount);
}

class RegularDiscount : IDiscount
{
    public decimal Apply(decimal amount) => amount;
}

class LoyalDiscount : IDiscount
{
    public decimal Apply(decimal amount) =>
        amount - amount * 10 / 100;
}

class Checkout
{
    public decimal Pay(
        decimal amount, IDiscount discount) =>
        discount.Apply(amount);
}
```

```text
100000
90000
```

- `Checkout` chỉ biết `IDiscount`, không biết có bao nhiêu loại giảm giá.
- Thêm loại giảm giá mới là thêm class mới. `Checkout` không phải sửa.
- Đây chính là đa hình và interface đã học ở các chương trước.

## Thử ngay

Chép ví dụ trên vào `Program.cs`. Thêm class dưới đây vào cuối file, rồi in
thêm `checkout.Pay(100000m, new VipDiscount())` ngay sau hai lệnh in ở đầu
file:

```csharp
class VipDiscount : IDiscount
{
    public decimal Apply(decimal amount) =>
        amount - amount * 20 / 100;
}
```

**Đoán trước khi chạy:** dòng thứ ba in ra số nào?

<details>
<summary>Xem kết quả</summary>

```text
100000
90000
80000
```

Dòng thứ ba là 80000, tức 100000 giảm 20%. `Checkout` không sửa dòng nào,
tính năng mới nằm gọn trong class `VipDiscount`.

</details>

## Lỗi hay gặp

**Chuỗi `if` phình ra theo thời gian.** Mỗi loại khách mới là một lần sửa
method cũ.

```csharp
// SAI — thêm loại khách là phải sửa method này
decimal Pay(decimal amount, string type)
{
    if (type == "regular")
    {
        return amount;
    }
    else if (type == "loyal")
    {
        return amount - amount * 10 / 100;
    }
    return amount;
}
```

```csharp
// ĐÚNG — loại khách mới là một class IDiscount mới
decimal Pay(decimal amount, IDiscount discount)
{
    return discount.Apply(amount);
}
```

**Làm trước cho mọi khả năng.** Mới có một cách tính đã tạo interface để dành
cho "sau này" thì chỉ tốn thêm file. Hãy áp dụng OCP khi thấy phần đó **thật
sự** hay thay đổi.

## Tóm tắt

- OCP: thêm tính năng bằng code mới, không sửa code đang chạy.
- Đưa phần hay thay đổi ra sau interface, mỗi biến thể là một class.
- Chuỗi `if`/`switch` theo loại, phình ra mỗi lần có loại mới, là dấu hiệu
  nên dùng OCP.
- Chỉ áp dụng cho phần thật sự hay thay đổi.

```quiz
[
  {
    "prompt": "Method ExportReport(string format) có switch cho \"pdf\", \"excel\", \"csv\". Tháng nào cũng thêm định dạng mới. Theo OCP nên làm gì?",
    "options": [
      "Thêm case mới vào switch mỗi tháng",
      "Viết lại ExportReport từ đầu mỗi lần",
      "Gộp mọi định dạng vào một method dài",
      "Tách IExporter, mỗi định dạng một class"
    ],
    "answer": 4,
    "explain": "Định dạng là phần hay thay đổi. Đưa nó ra sau interface thì thêm định dạng mới chỉ là thêm class mới."
  },
  {
    "prompt": "\"Đóng với việc sửa đổi\" trong OCP nghĩa là gì?",
    "options": [
      "Tính năng mới không phải sửa code cũ",
      "Không bao giờ được sửa bug nữa",
      "Mọi thành viên phải để private",
      "Không class nào được kế thừa nó"
    ],
    "answer": 1,
    "explain": "OCP không cấm sửa bug. Nó nói tính năng mới nên đến từ code mới, để code cũ đã chạy ổn không bị đụng vào."
  },
  {
    "prompt": "Cửa hàng chỉ có đúng một cách tính phí ship, và chưa có kế hoạch đổi. Có nên tạo interface IShipping ngay không?",
    "options": [
      "Có, OCP bắt buộc class nào cũng có interface",
      "Chưa cần, đợi có cách tính thứ hai",
      "Có, để sau này thêm cách tính khỏi sửa code",
      "Có, và viết sẵn vài class cho sau này"
    ],
    "answer": 2,
    "explain": "OCP áp dụng cho phần thật sự hay thay đổi. Tạo interface quá sớm chỉ thêm file; khi xuất hiện cách tính thứ hai thì tách vẫn kịp."
  }
]
```
