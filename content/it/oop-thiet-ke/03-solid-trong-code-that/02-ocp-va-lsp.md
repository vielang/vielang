---
title: OCP và LSP
minutes: 11
---

Phí ship tính trong một `switch` bốn nhánh. Hãng thứ năm ký hợp đồng, bạn thêm
nhánh thứ năm.

Build đỏ ba mươi chỗ. Không phải vì nhánh mới sai, mà vì năm mươi test đang gọi
đúng cái method ấy, và ba trong số đó so sánh kết quả từng đồng.

Tính năng mới lẽ ra chỉ là thêm. Ở đây nó thành sửa.

> **Học xong bài này bạn sẽ:** biết OCP nói "mở để mở rộng, đóng để sửa" nghĩa
> là gì trong code C#; nhận ra lúc nào lớp con phá hợp đồng của lớp cha; và
> biết cả hai nguyên tắc này đều có giá.
>
> **Cần biết trước:** `abstract`, `override`, đa hình (chương trước).

## OCP: thêm hành vi bằng cách thêm code, không sửa code cũ

**Open/Closed Principle**: một module nên **mở** để mở rộng và **đóng** với việc
sửa.

Nghe như nghịch lý, nhưng nó rất cụ thể. Thêm một hãng vận chuyển thì thêm một
file mới, còn những file đang chạy đúng thì không ai chạm vào.

| Cách thêm tính năng | Ảnh hưởng tới code đang chạy |
|---|---|
| Thêm nhánh vào `switch` | mọi test của method đó phải chạy lại |
| Thêm một class mới | không chạm gì, chỉ thêm một dòng đăng ký |

Đóng với việc sửa không có nghĩa là cấm sửa. Nó nghĩa là **sửa nghiệp vụ không
buộc bạn sửa cơ chế**.

```csharp
// SAI — hãng thứ năm là một lần sửa method cũ
decimal Fee(string carrier, decimal weight) =>
    carrier switch
    {
        "ghn" => weight * 12_000,
        "ghtk" => weight * 11_000,
        _ => throw new NotSupportedException(),
    };
```

```csharp
// ĐÚNG — hãng thứ năm là một file mới
interface IShippingRate
{
    string Carrier { get; }
    decimal Fee(decimal weight);
}

class GhnRate : IShippingRate
{
    public string Carrier => "ghn";
    public decimal Fee(decimal w) => w * 12_000;
}
```

Chỗ gọi tra theo `Carrier` rồi hỏi, và nó không cần biết có bao nhiêu hãng.

```csharp
decimal Fee(
    IEnumerable<IShippingRate> rates,
    string carrier,
    decimal weight) =>
    rates.First(r => r.Carrier == carrier)
         .Fee(weight);
```

## LSP: lớp con phải dùng được ở mọi chỗ lớp cha dùng được

**Liskov Substitution Principle** nói một câu: chỗ nào nhận lớp cha thì đưa lớp
con vào cũng phải chạy đúng.

Đa hình cho bạn cú pháp để làm việc đó. LSP là điều kiện để nó không thành bẫy.

| Lớp con làm gì | Có phá LSP |
|---|---|
| Trả kết quả khác nhưng vẫn đúng kiểu hợp đồng | không |
| Ném exception mà lớp cha không ném | **có** |
| Thắt chặt điều kiện đầu vào | **có** |
| Nới rộng điều kiện đầu vào | không |
| Bỏ trống thân method cho xong | **có** |

## Thử ngay: lớp con làm hợp đồng vỡ ở đâu

Dòng thứ hai của bảng nghe nặng nhất. Nhưng có một cách phá LSP tinh vi hơn,
mà đọc code thì không thấy.

```csharp
Rectangle r = new Square();
r.Width = 4;
r.Height = 5;

Console.WriteLine(r.Area());

class Rectangle
{
    public virtual int Width { get; set; }
    public virtual int Height { get; set; }
    public int Area() => Width * Height;
}

class Square : Rectangle
{
    public override int Width
    {
        get => base.Width;
        set { base.Width = value; base.Height = value; }
    }

    public override int Height
    {
        get => base.Height;
        set { base.Width = value; base.Height = value; }
    }
}
```

**Đoán trước khi chạy:** biến khai báo là `Rectangle`, gán rộng 4 và cao 5.
`Area()` in ra bao nhiêu?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
25
```

Không phải 20.

`Square` giữ cho hai cạnh luôn bằng nhau, nên gán `Height = 5` đã kéo `Width`
lên 5 theo. Bản thân `Square` không sai chút nào.

Sai là ở chỗ nó **hứa làm một `Rectangle`**. Mọi code đang cầm `Rectangle` đều
tin rằng đặt rộng rồi đặt cao thì rộng vẫn giữ nguyên.

Hình vuông là hình chữ nhật trong hình học. Nhưng `Square` không thay thế được
`Rectangle` trong code, vì `Rectangle` có một hành vi mà `Square` phá.

</details>

Bài học rút ra không phải "đừng kế thừa". Nó là: quan hệ "là một" phải đúng cả
về **hành vi**, không chỉ đúng về tên gọi.

## Hợp đồng gồm cả những gì không viết trong chữ ký

Chữ ký hàm nói kiểu tham số và kiểu trả về. Phần còn lại của hợp đồng thì nằm
trong đầu người gọi.

- Method này có thể ném exception gì.
- Truyền `null` vào thì sao.
- Gọi hai lần liền có ra cùng kết quả.
- Danh sách trả về có bao giờ là `null`.

```csharp
// SAI — lớp con thắt điều kiện đầu vào
class StrictWarehouse : Warehouse
{
    public override void Ship(Order o)
    {
        if (o.Total < 100_000)
            throw new NotSupportedException();

        base.Ship(o);
    }
}
```

Người gọi cầm `Warehouse` và gửi mọi đơn. Lớp con này từ chối đơn nhỏ, nên nó
làm vỡ code chưa từng biết nó tồn tại.

Muốn thêm điều kiện thì đặt nó ở chỗ **chọn** kho, đừng đặt trong lớp con.

## Cả hai nguyên tắc đều có giá, và cái giá là gián tiếp

Đây là chỗ SOLID hay bị áp dụng quá tay.

Bản `switch` bốn nhánh đọc trong ba giây. Bản interface bắt người đọc mở bốn
file mới thấy đủ bốn công thức.

| Tình huống | Chọn |
|---|---|
| Tập hãng vận chuyển còn mở, tháng nào cũng thêm | interface |
| Bốn nhánh và ba năm không đổi | giữ `switch` |
| Mỗi nhánh dài hơn mười dòng | interface |
| Mỗi nhánh là một phép nhân | giữ `switch` |

OCP đáng áp dụng ở chỗ bạn **biết** sẽ còn thêm nữa. Áp dụng ở mọi chỗ thì bạn
trả tiền gián tiếp cho một tương lai không tới.

## Dấu hiệu trong code của bạn

- Thêm một loại mới mà phải sửa `switch` ở nhiều file → OCP đang bị vi phạm đúng chỗ đáng sửa.
- Lớp con override rồi ném `NotSupportedException` → phá LSP, và người gọi không có cách nào biết trước.
- Lớp con kiểm tra thêm điều kiện đầu vào rồi từ chối → thắt hợp đồng, cũng là phá LSP.
- Người gọi phải `if (x is TypeB)` để né một lớp con → lớp con ấy không thay thế được lớp cha.
- Một interface có đúng một class implement, và tập loại không hề mở → gián tiếp không mua được gì.

## Ghi nhớ

- OCP: thêm hành vi bằng thêm file, không sửa file đang chạy đúng.
- Chỉ mở sẵn ở chỗ bạn biết sẽ còn thêm; chỗ khác thì `switch` bốn nhánh vẫn tốt.
- LSP: lớp con phải dùng được ở mọi chỗ lớp cha dùng được.
- Hợp đồng gồm cả exception, cả null, cả những gì chữ ký không nói.
- Thắt điều kiện đầu vào trong lớp con là phá LSP, dù code trông rất cẩn thận.

## Bước tiếp theo

Hai nguyên tắc vừa rồi đều dựa vào interface. Nhưng interface to thì lại sinh
ra vấn đề của nó.

Bài sau, **ISP và DIP**, mở bằng một class test phải viết mười hai method rỗng
chỉ để chạy được một phép kiểm tra.

```quiz
[
  {
    "prompt": "Bạn thêm hãng vận chuyển thứ năm. Với thiết kế nào thì không phải sửa file nào đang chạy đúng?",
    "options": [
      "Thêm nhánh vào switch, có throw ở cuối",
      "Thêm một class implement IShippingRate rồi đăng ký nó",
      "Thêm tham số bool vào method Fee",
      "Thêm một if ở đầu method Fee cho trường hợp mới"
    ],
    "answer": 2,
    "explain": "Thêm class là thêm file mới, code cũ không bị chạm. Ba cách kia đều sửa đúng method mà mọi test đang phụ thuộc vào."
  },
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "Rectangle r = new Square();\nr.Width = 4;\nr.Height = 5;\nConsole.WriteLine(r.Area());",
    "options": [
      "20",
      "16",
      "25",
      "Ném exception vì Square không cho đặt hai cạnh khác nhau"
    ],
    "answer": 3,
    "explain": "Square kéo hai cạnh bằng nhau, nên gán Height = 5 đã đổi luôn Width thành 5. Square không sai; sai là ở chỗ nó hứa làm một Rectangle mà lại phá một hành vi người gọi đang tin."
  },
  {
    "prompt": "Lớp con override method Ship rồi thêm điều kiện: đơn dưới 100.000 thì ném exception. Đây là vấn đề gì?",
    "options": [
      "Không phải vấn đề, thêm kiểm tra là cẩn thận",
      "Vi phạm OCP, vì đã sửa hành vi cũ",
      "Vi phạm SRP, vì lớp con làm hai việc",
      "Vi phạm LSP, vì lớp con thắt điều kiện đầu vào của lớp cha"
    ],
    "answer": 4,
    "explain": "Người gọi cầm kiểu cha và gửi mọi đơn. Một lớp con từ chối đơn nhỏ làm vỡ code chưa từng biết nó tồn tại. Điều kiện ấy nên nằm ở chỗ chọn kho."
  },
  {
    "prompt": "Một switch bốn nhánh, mỗi nhánh là một phép nhân, ba năm nay không đổi. Có nên tách thành bốn class implement interface?",
    "options": [
      "Không — tập loại không mở, gián tiếp thêm vào không mua được gì",
      "Có, vì OCP luôn đúng",
      "Có, nhưng chỉ cần hai class là đủ",
      "Không, vì switch chạy nhanh hơn gọi qua interface"
    ],
    "answer": 1,
    "explain": "OCP đáng áp dụng ở chỗ bạn biết sẽ còn thêm nữa. Ở đây bản switch đọc trong ba giây, còn bản interface bắt người đọc mở bốn file. Lý do loại bỏ không phải hiệu năng, mà là chi phí đọc hiểu."
  }
]
```
