---
title: Đóng gói giữ object hợp lệ
minutes: 10
---

Khách gọi. Đơn giao hôm qua, sáng nay hiện lại là "mới".

Bạn mở code tìm thủ phạm. Năm chỗ cùng gán `order.Status = ...`: một trong
controller, một trong job chạy đêm, ba nằm rải ở các service. Đọc từng chỗ thì
chỗ nào cũng có lý. Ghép lại thì đơn hàng hỏng.

Dặn nhau cẩn thận hơn không chữa được chuyện này. Phải làm cho câu gán ấy
**không viết được nữa**.

> **Học xong bài này bạn sẽ:** dùng đúng từng access modifier; chọn giữa
> `set`, `init`, `private set`, `readonly`; và giữ object không bao giờ rơi
> vào trạng thái vô lý.
>
> **Cần biết trước:** `class`, property, constructor (C# Core).

## Đóng gói: dữ liệu và quy tắc nằm chung một chỗ

**Đóng gói (encapsulation)** là cho quy tắc sống cạnh dữ liệu mà nó ràng
buộc. Phần còn lại giấu đi.

Đưa câu "chỉ đơn đã thanh toán mới được giao" vào bên trong `Order` đi. Năm
chỗ kia lập tức hết đường phá.

Muốn vậy, mỗi thành viên của class phải trả lời hai câu: **ai nhìn thấy nó**,
và **ai gán được, lúc nào**.

## Access modifier quyết định ai nhìn thấy được gì

| Từ khoá | Nhìn thấy được từ |
|---|---|
| `public` | mọi nơi, kể cả project khác |
| `internal` | trong cùng **assembly** (cùng project) |
| `protected` | chính class đó và các lớp con |
| `private` | **chỉ bên trong** chính class đó |
| `protected internal` | cùng assembly, **hoặc** lớp con ở assembly khác |
| `private protected` | lớp con **và** phải cùng assembly |

```csharp
public class Order
{
    private decimal _total;      // chỉ class này thấy
    internal string Code = "";   // cả project thấy
    public string Customer = ""; // ai cũng thấy
}
```

Không ghi gì, C# chọn mức hẹp nhất: thành viên là `private`, class trong
namespace là `internal`. Mặc định ấy khôn hơn ta tưởng.

Đừng vội gõ `public` cho quen mắt. Mỗi chữ `public` là một lời hứa với người
ngoài. Mà lời hứa thì khó rút lại.

## set, init, private set: ai gán được và lúc nào

Câu hỏi thứ hai mới là câu quyết định sự cố đầu bài.

| Khai báo | Gán được khi nào |
|---|---|
| `public int Qty { get; set; }` | bất cứ đâu, bất cứ lúc nào |
| `public int Qty { get; init; }` | **chỉ lúc khởi tạo** object |
| `public int Qty { get; private set; }` | chỉ từ bên trong class |
| `public int Qty { get; }` | chỉ trong constructor |
| `private readonly int _qty;` | khai báo hoặc trong constructor |
| `const int Max = 3;` | cố định lúc compile |

Ba dòng giữa là thứ bạn dùng hằng ngày: cho đọc thoải mái, không cho ghi.
`public set` thì tiện thật. Tiện như trao chìa khoá nhà cho cả phố.

Một điều nữa: lộ ra ngoài thì luôn dùng **property**, đừng dùng field. Chỉ
property mới có chỗ đặt câu kiểm tra. Và đổi field thành property sau này là
phá vỡ mọi nơi đang gọi.

## Thử ngay: class toàn public set thì ai cũng phá được

```csharp
class Order
{
    public int Qty { get; set; }
    public decimal UnitPrice { get; set; }
    public string Status { get; set; } = "New";
    public decimal Total => Qty * UnitPrice;
}

var order = new Order { Qty = 2, UnitPrice = 100 };
order.Status = "Shipped";
order.Qty = -5;

Console.WriteLine($"{order.Status} / {order.Total}");
```

**Đoán trước khi chạy:** có gì chặn dòng `Qty = -5` trên một đơn đã giao không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Shipped / -500
```

Không có gì chặn, vì chẳng có gì để chặn. Class này là cái túi đựng property.

Quy tắc nghiệp vụ nằm hết ở người gọi. Ai mới vào dự án cũng phải tự nhớ đủ.
Một người quên là bạn có đơn hàng số lượng âm.

</details>

## Sửa lại: constructor kiểm tra, private set, method nghiệp vụ

```csharp
class Order
{
    public int Qty { get; private set; }
    public decimal UnitPrice { get; }

    public OrderStatus Status { get; private set; }
        = OrderStatus.New;

    public decimal Total => Qty * UnitPrice;

    public Order(int qty, decimal unitPrice)
    {
        if (qty <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(qty));

        Qty = qty;
        UnitPrice = unitPrice;
    }

    public void ChangeQty(int newQty)
    {
        if (Status != OrderStatus.New)
            throw new InvalidOperationException(
                "Đơn đã xử lý, không đổi số lượng");

        Qty = newQty > 0 ? newQty
            : throw new ArgumentOutOfRangeException(
                nameof(newQty));
    }

    public void Ship()
    {
        if (Status != OrderStatus.Paid)
            throw new InvalidOperationException(
                "Chưa thanh toán thì chưa giao được");

        Status = OrderStatus.Shipped;
    }
}
```

Constructor lo đầu vòng đời: tạo được `Order` nghĩa là nó hợp lệ. `private
set` lo phần giữa: chỉ lớp này đổi được trạng thái của mình. `ChangeQty` và
`Ship` là hai cánh cửa duy nhất, cửa nào cũng có người gác.

Còn một cái được nữa, nằm ở chỗ đọc. `order.Ship()` nói thẳng việc đang xảy
ra, và hứa rằng điều kiện đã được kiểm tra. `order.Status = "Shipped"` không
hứa gì hết. Muốn biết thì tự đi tìm — đúng việc bạn làm suốt sáng nay.

## init và readonly khoá dữ liệu lại ngay sau khi tạo

`private set` hợp với thứ còn đổi trong vòng đời. Còn địa chỉ giao hàng thì
tạo xong là xong, và `init` sinh ra cho đúng những thứ như vậy.

```csharp
class ShippingAddress
{
    public required string Street { get; init; }
    public required string City { get; init; }
}

var addr = new ShippingAddress
{
    Street = "12 Lê Lợi",
    City = "Hà Nội",
};

addr.City = "Đà Nẵng";   // lỗi compile: init-only
```

Cú pháp khởi tạo vẫn gọn như cũ. Qua dấu ngoặc đóng là object khoá lại.
`required` để compiler nhắc khi ai đó quên điền. Với field, `readonly` làm
đúng việc ấy.

## Bỏ set không khoá được nội dung collection

Đây là cái bẫy mà cả người đã quen đóng gói cũng vướng.

```csharp
// SAI — ai cũng Add được, bỏ qua mọi quy tắc
public List<OrderLine> Lines { get; } = new();
```

`{ get; }` chỉ chặn việc thay **cả danh sách**. Còn bên trong thì mở toang.
`Add`, `Clear`, `RemoveAt` chạy bình thường, mà class không hay biết gì.

```csharp
// ĐÚNG — thêm qua method, đọc qua kiểu chỉ đọc
private readonly List<OrderLine> _lines = new();

public IReadOnlyList<OrderLine> Lines => _lines;

public void AddLine(OrderLine line)
{
    if (Status != OrderStatus.New)
        throw new InvalidOperationException(
            "Đơn đã xử lý");

    _lines.Add(line);
}
```

`IReadOnlyList<T>` không có `Add`. Người gọi muốn thêm cũng không còn đường
nào ngoài `AddLine`.

## Anemic model: khi quy tắc rời khỏi class và tản đi khắp nơi

Quay lại sáng nay. Thủ phạm không phải năm dòng gán. Thủ phạm là quyết định
để `Order` trơ ra toàn property, rồi dồn hết logic vào một `OrderService` năm
trăm dòng. Kiểu thiết kế đó có tên: **anemic domain model**.

Vài tháng đầu nó chạy tốt. Rồi dự án có người thứ ba, người thứ tư. Mỗi người
mở thêm một đường sửa dữ liệu. Không ai đọc hết năm trăm dòng kia để biết quy
tắc đầy đủ là gì.

Dù vậy, đừng nhét hành vi vào mọi class. **DTO** chở dữ liệu qua API thì chỉ
nên có property, vì nó chẳng có gì để bảo vệ. Ranh giới rất gọn: có quy tắc
bất biến thì đóng gói, chỉ chở dữ liệu thì để trần.

## Dấu hiệu trong code của bạn

- `public set` cho trạng thái nghiệp vụ (`Status`, `Qty`, `Balance`) → `private set` cộng method.
- Cùng một câu `if` kiểm tra trước khi gán, lặp ở nhiều nơi → quy tắc đó thuộc về class.
- `public List<T>` trong domain model → `IReadOnlyList<T>` cộng method thêm bớt.
- Constructor rỗng rồi gán từng property → object dang dở giữa hai dòng code.

## Ghi nhớ

- Mặc định: thành viên là `private`, class trong namespace là `internal`.
- `set` ghi tự do, `init` chỉ lúc khởi tạo, `private set` chỉ từ bên trong.
- Bỏ `set` không khoá được nội dung collection.
- Đóng gói là để **object không thể sai**, không phải để giấu cho kín.

## Bước tiếp theo

`Order` giờ tự bảo vệ được mình. Nhưng nó vẫn phải gửi thông báo, lưu xuống
database, tính phí vận chuyển — toàn những việc cần người khác làm hộ. Bài sau,
**Interface là hợp đồng**, nói về cách nhờ vả mà không dính cứng vào một ai.

```quiz
[
  {
    "prompt": "Thành viên khai báo không ghi access modifier thì mặc định là gì?",
    "options": ["public", "private", "internal", "protected"],
    "answer": 2,
    "explain": "Thành viên của class mặc định là private. Riêng class khai báo trong namespace thì mặc định là internal."
  },
  {
    "prompt": "Bạn muốn property đọc được từ mọi nơi nhưng chỉ gán được LÚC KHỞI TẠO object. Khai báo thế nào?",
    "options": [
      "public int Qty { get; set; }",
      "public int Qty { get; init; }",
      "public int Qty { get; private set; }",
      "public readonly int Qty;"
    ],
    "answer": 2,
    "explain": "init cho phép gán trong cú pháp khởi tạo rồi khoá lại. private set thì class vẫn tự đổi được sau đó; readonly dành cho field, không phải property."
  },
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "class Order\n{\n    public List<string> Lines { get; } = new();\n}\n\nvar order = new Order();\norder.Lines.Add(\"Sách\");\n\nConsole.WriteLine(order.Lines.Count);",
    "options": [
      "0",
      "1",
      "Lỗi compile vì Lines chỉ có get",
      "Ném InvalidOperationException"
    ],
    "answer": 2,
    "explain": "Bỏ set chỉ chặn việc thay cả danh sách, không chặn Add vào danh sách đó. Muốn khoá nội dung thì lộ ra bằng IReadOnlyList."
  },
  {
    "prompt": "Thư viện nội bộ có một class chỉ dùng trong cùng project, không muốn project khác thấy. Dùng modifier nào?",
    "options": ["public", "internal", "protected", "private"],
    "answer": 2,
    "explain": "internal giới hạn tầm nhìn trong cùng assembly. protected dành cho lớp con, còn private thì ngay class khác trong cùng project cũng không thấy."
  }
]
```
