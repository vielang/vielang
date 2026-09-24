---
title: Đóng gói
minutes: 5
---

Ở khoá C# Core bạn đã viết class có property và constructor. Khoá này học
cách tổ chức nhiều class cho dễ sửa.

Tồn kho của một sản phẩm bỗng thành số âm. Tìm trong code thì thấy năm chỗ
khác nhau cùng gán thẳng `product.Stock = ...`, và một chỗ quên kiểm tra. Đóng
gói giải quyết việc này bằng cách chỉ cho sửa dữ liệu qua đúng một cửa.

## Khái niệm

🔒 **Đóng gói (encapsulation)**: giấu dữ liệu bên trong class, chỉ cho thay đổi qua method của class để object luôn hợp lệ.

🚪 **Access modifier**: từ khoá quy định ai được dùng một thành viên của class.

| Từ khoá | Ai dùng được |
|---|---|
| `public` | mọi nơi |
| `private` | chỉ code bên trong class đó |

Thành viên không ghi gì thì mặc định là `private`.

## Ví dụ

```csharp
var pen = new Product("Bút bi");
pen.Restock(100);
pen.Sell(30);
Console.WriteLine(pen.Stock);   // 70

class Product
{
    public string Name { get; }
    public int Stock { get; private set; }

    public Product(string name)
    {
        Name = name;
    }

    public void Restock(int quantity)
    {
        Stock = Stock + quantity;
    }

    public void Sell(int quantity)
    {
        if (quantity > Stock)
        {
            throw new InvalidOperationException(
                "Không đủ hàng");
        }
        Stock = Stock - quantity;
    }
}
```

- `{ get; private set; }`: bên ngoài đọc được `Stock`, nhưng chỉ code trong
  `Product` mới gán được. Khác `{ get; }` ở bài Property và constructor
  (chỉ gán trong constructor), `private set` cho method của class gán lại
  bao nhiêu lần cũng được.
- Muốn đổi tồn kho thì phải gọi `Restock` hoặc `Sell`. Quy tắc "không bán quá
  số đang có" nằm đúng một chỗ, trong `Sell`.
- Không nơi nào khác trong chương trình làm `Stock` âm được nữa.

## Thử ngay

Chép ví dụ trên vào `Program.cs`, thay các dòng đầu bằng:

```csharp
var pen = new Product("Bút bi");
pen.Restock(5);

try
{
    pen.Sell(3);
    pen.Sell(10);
}
catch (InvalidOperationException ex)
{
    Console.WriteLine(ex.Message);
}

Console.WriteLine(pen.Stock);
```

**Đoán trước khi chạy:** lệnh bán 10 cái bị chặn. Vậy `Stock` cuối cùng là
5, 2 hay -8?

<details>
<summary>Xem kết quả</summary>

```text
Không đủ hàng
2
```

Là 2. Bán 3 cái thành công. Lệnh bán 10 bị `Sell` chặn trước khi trừ, nên
`Stock` không bao giờ âm.

</details>

## Lỗi hay gặp

**Để `set` công khai.** Nơi nào cũng gán thẳng được, và quy tắc trong `Sell`
bị bỏ qua.

```csharp
// SAI — nơi khác gán thẳng, bỏ qua mọi kiểm tra
var cup = new Cup();
cup.Stock = -5;

class Cup
{
    public int Stock { get; set; }
}
```

```csharp
// ĐÚNG — chỉ đổi được qua method của class
var cup = new Cup();
cup.Restock(5);

class Cup
{
    public int Stock { get; private set; }

    public void Restock(int quantity)
    {
        if (quantity <= 0)
        {
            throw new ArgumentException(
                "Số lượng phải lớn hơn 0");
        }
        Stock = Stock + quantity;
    }
}
```

## Tóm tắt

- Đóng gói: dữ liệu để `private`, chỉ đổi qua method của class.
- `public` dùng được ở mọi nơi, `private` chỉ dùng được trong class.
- `{ get; private set; }` cho bên ngoài đọc mà không cho ghi.
- Quy tắc kiểm tra nằm trong method của class, không rải ở nơi gọi.

```quiz
[
  {
    "prompt": "Class Account có public decimal Balance { get; private set; }. Dòng account.Balance = 1000m; ở Program.cs thì sao?",
    "options": [
      "Chạy được, Balance thành 1000",
      "Lỗi khi chạy",
      "Lỗi compile vì set là private",
      "Chạy được nhưng Balance không đổi"
    ],
    "answer": 3,
    "explain": "private set nghĩa là chỉ code bên trong Account mới gán được Balance. Bên ngoài chỉ đọc."
  },
  {
    "prompt": "Quy tắc \"giảm giá không quá 50%\" nên đặt ở đâu?",
    "options": [
      "Trong method ApplyDiscount của class Product",
      "Ở mọi nơi gọi, trước khi gán Price",
      "Trong file ghi chú cho team",
      "Không cần kiểm tra"
    ],
    "answer": 1,
    "explain": "Đặt quy tắc trong method của class thì chỉ có một chỗ phải viết và không nơi nào bỏ qua được."
  },
  {
    "prompt": "Một field khai báo int _count; mà không ghi access modifier. Nó là gì?",
    "options": [
      "public",
      "Tuỳ project",
      "Không hợp lệ, bắt buộc phải ghi",
      "private"
    ],
    "answer": 4,
    "explain": "Thành viên trong class không ghi access modifier thì mặc định là private."
  }
]
```
