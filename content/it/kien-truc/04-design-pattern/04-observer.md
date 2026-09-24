---
title: Observer
minutes: 5
---

Đặt hàng xong phải gửi email, cộng điểm thành viên, gửi SMS. Nếu
`OrderService` tự gọi cả ba, mỗi việc mới lại phải sửa `OrderService`, và nó
phải biết mọi phần khác của hệ thống. Với Observer, `OrderService` chỉ thông
báo "đã đặt hàng", việc nào cần thì tự đăng ký nghe.

## Khái niệm

📣 **Observer**: một object phát thông báo khi có việc xảy ra, các object khác đăng ký nghe và tự xử lý, bên phát không cần biết ai đang nghe.

C# có sẵn cơ chế này là event, đã gặp ở bài Sự kiện của khoá WinForms: nút là
bên phát event `Click`, handler gắn bằng `+=` là bên nghe.

## Ví dụ

```csharp
var service = new OrderService();
service.OrderPlaced += (sender, code) =>
    Console.WriteLine("Gửi email cho " + code);
service.OrderPlaced += (sender, code) =>
    Console.WriteLine("Cộng điểm cho " + code);

service.Place("DH1");

public class OrderService
{
    public event EventHandler<string>? OrderPlaced;

    public void Place(string code)
    {
        Console.WriteLine("Đã lưu " + code);
        OrderPlaced?.Invoke(this, code);
    }
}
```

- `event EventHandler<string>?` khai báo một event gửi kèm một `string`, ở
  đây là mã đơn. Handler nhận `sender` (bên phát) và giá trị đó.
- `OrderPlaced?.Invoke(this, code)` gọi lần lượt mọi handler. Chưa có handler
  nào thì event là `null`, và `?.` như bài null và nullable sẽ bỏ qua lời
  gọi.
- `OrderService` ở đây là bản rút gọn, chỉ giữ phần thông báo. Nó không biết
  gì về email hay điểm. Thêm việc gửi SMS là thêm một `+=` ở ngoài.

```mermaid OrderService phát một thông báo, nhiều bên cùng nghe
flowchart LR
    O["OrderService: OrderPlaced"] --> E["Gửi email"]
    O --> K["Cộng điểm"]
    O --> P["Gửi SMS"]
```

## Thử ngay

Chạy ví dụ, rồi thêm hai dòng sau vào cuối phần câu lệnh:

```csharp
var lonely = new OrderService();
lonely.Place("DH2");
```

**Đoán trước khi chạy:** toàn bộ chương trình in ra những dòng nào? Đơn DH2
không có ai nghe thì có lỗi không?

<details>
<summary>Xem kết quả</summary>

```text
Đã lưu DH1
Gửi email cho DH1
Cộng điểm cho DH1
Đã lưu DH2
```

Handler chạy theo thứ tự đăng ký. `lonely` không có handler nào, `?.` bỏ qua
lời gọi nên không lỗi.

</details>

## Lỗi hay gặp

**Gọi event mà không có `?.`.** Chưa ai đăng ký thì event là `null`, gọi thẳng
ném `NullReferenceException`.

```csharp
// SAI — không có handler là NullReferenceException
public class OrderService
{
    public event EventHandler<string>? OrderPlaced;

    public void Place(string code)
    {
        OrderPlaced(this, code);
    }
}
```

```csharp
// ĐÚNG — chỉ gọi khi có người nghe
public class OrderService
{
    public event EventHandler<string>? OrderPlaced;

    public void Place(string code)
    {
        OrderPlaced?.Invoke(this, code);
    }
}
```

## Tóm tắt

- Observer: bên phát thông báo, nhiều bên nghe tự phản ứng.
- Trong C#, dùng `event` để phát và `+=` để đăng ký.
- Bên phát không biết bên nghe, thêm việc mới không phải sửa bên phát.
- Phát event bằng `?.Invoke` để không lỗi khi chưa ai nghe.

```quiz
[
  {
    "prompt": "Thêm việc \"gửi SMS khi đặt hàng\" với Observer. Cần sửa OrderService không?",
    "options": [
      "Có, thêm lời gọi SmsSender vào Place",
      "Có, thêm một event mới",
      "Không, chỉ cần đăng ký thêm một handler cho OrderPlaced",
      "Không làm được"
    ],
    "answer": 3,
    "explain": "OrderService chỉ phát thông báo. Việc mới đăng ký nghe từ bên ngoài bằng +=."
  },
  {
    "prompt": "Ba handler đăng ký vào OrderPlaced theo thứ tự email, điểm, SMS. Chúng chạy theo thứ tự nào?",
    "options": [
      "Email, điểm, SMS",
      "Ngẫu nhiên",
      "SMS, điểm, email",
      "Chỉ handler cuối chạy"
    ],
    "answer": 1,
    "explain": "Event gọi các handler theo thứ tự đăng ký."
  },
  {
    "prompt": "OrderPlaced(this, code) ném NullReferenceException. Nguyên nhân?",
    "options": [
      "code bằng null",
      "OrderService chưa được new",
      "Sai kiểu EventHandler",
      "Chưa có handler nào đăng ký nên event là null"
    ],
    "answer": 4,
    "explain": "Event không có handler thì là null. Dùng OrderPlaced?.Invoke(this, code) để an toàn."
  }
]
```
