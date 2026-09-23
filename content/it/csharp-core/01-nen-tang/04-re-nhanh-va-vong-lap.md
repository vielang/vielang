---
title: Rẽ nhánh và vòng lặp
minutes: 10
---

Một job dọn dữ liệu chạy đêm: duyệt danh sách đơn hàng, thấy đơn đã huỷ thì
xoá khỏi danh sách. Chạy thử với hai đơn thì êm. Lên production gặp danh sách
thật, job sập ngay vòng lặp đầu tiên với `InvalidOperationException`.

> **Học xong bài này bạn sẽ:** viết rẽ nhánh bằng `switch expression` như code
> C# hiện đại, chọn đúng giữa `for` và `foreach`, và không còn sập khi cần xoá
> phần tử trong lúc duyệt.
>
> **Cần biết trước:** biến, toán tử so sánh, `List<T>` ở mức dùng được.

## if và guard clause

```csharp
if (score >= 80)
    level = "Giỏi";
else if (score >= 50)
    level = "Khá";
else
    level = "Trung bình";
```

Luôn viết `{ }` cho thân `if` kể cả khi chỉ có một dòng — thêm dòng thứ hai mà
quên ngoặc là lỗi rất khó nhìn ra lúc review.

Lồng quá hai tầng `if` thì đổi sang **guard clause**: kiểm tra trường hợp sai
rồi `return` sớm, phần còn lại của method khỏi thụt vào sâu.

```csharp
// SAI — logic chính nằm sâu trong ba tầng ngoặc
if (order != null)
{
    if (order.IsPaid)
    {
        if (order.Items.Count > 0)
            Giao(order);
    }
}
```

```csharp
// ĐÚNG — chặn sớm, logic chính nằm ở mức ngoài cùng
if (order is null) return;
if (!order.IsPaid) return;
if (order.Items.Count == 0) return;

Giao(order);
```

## switch expression

```csharp
string nhan = status switch
{
    OrderStatus.New       => "Mới tạo",
    OrderStatus.Paid      => "Đã thanh toán",
    OrderStatus.Cancelled => "Đã huỷ",
    _                     => "Không xác định",
};
```

Dạng **expression** (từ C# 8) trả về một giá trị, không cần `break`. Dấu `_` là
**discard**, đóng vai `default`. Thiếu nhánh nào đó thì compiler cảnh báo —
an toàn hơn hẳn `switch` dạng câu lệnh cũ, thứ bạn vẫn sẽ gặp trong code có sẵn.

## Thử ngay: vì sao job dọn dữ liệu sập

```csharp
var ds = new List<string> { "a", "huy", "b", "huy" };

foreach (var x in ds)
{
    if (x == "huy")
        ds.Remove(x);
}

Console.WriteLine(string.Join(",", ds));
```

**Đoán trước khi chạy:** in ra `a,b`, hay in ra thứ khác, hay không in được gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Unhandled exception. System.InvalidOperationException:
  Collection was modified; enumeration operation
  may not execute.
```

`foreach` giữ một con trỏ chạy trên danh sách gốc. Xoá phần tử là danh sách
đổi, con trỏ mất chỗ đứng, nên lần lặp kế tiếp ném lỗi.

</details>

Hai cách sửa, chọn theo ý định:

```csharp
// Lọc ra danh sách mới — rõ ý, hay dùng nhất
ds = ds.Where(x => x != "huy").ToList();

// Xoá tại chỗ, duyệt NGƯỢC từ cuối
for (int i = ds.Count - 1; i >= 0; i--)
{
    if (ds[i] == "huy") ds.RemoveAt(i);
}
```

Duyệt ngược mới an toàn: xoá phần tử thứ `i` không làm lệch những phần tử chưa
xét, vì chúng nằm phía trước.

## for, foreach, while

```csharp
for (int i = 0; i < items.Count; i++)  // cần chỉ số
    Console.WriteLine($"{i}: {items[i]}");

foreach (var item in items)        // chỉ cần phần tử
    Console.WriteLine(item);

while (hang.Count > 0)             // chưa biết mấy lần
    Xuly(hang.Dequeue());

do { lan++; } while (lan < 3);     // chạy ít nhất 1 lần
```

Mặc định dùng `foreach`: nó nói đúng ý định "làm gì đó với từng phần tử" và
không có chỗ để gõ nhầm chỉ số.

```csharp
foreach (var o in orders)
{
    if (o.IsDeleted) continue;   // bỏ qua, chạy tiếp
    if (o.IsFinal) break;        // thoát hẳn vòng lặp
    Xuly(o);
}

var cuoi = items[^1];      // phần tử cuối
var ba = items[..3];       // ba phần tử đầu
```

## Dấu hiệu trong code của bạn

- `Add`, `Remove`, `Clear` nằm trong thân một `foreach` duyệt chính collection đó → sẽ ném `InvalidOperationException`.
- `if` lồng từ ba tầng trở lên → đổi sang guard clause, hoặc tách method.
- `switch` dạng câu lệnh dài, mỗi nhánh chỉ gán một giá trị → viết lại thành `switch expression`.
- `while (true)` mà không thấy `break` hay `CancellationToken` → service sẽ treo, không phải "có thể treo".

## Ghi nhớ

- Không thêm/xoá phần tử của collection đang `foreach`; lọc ra danh sách mới, hoặc `for` duyệt ngược.
- `switch expression` trả giá trị, không cần `break`, và được compiler nhắc khi thiếu nhánh.
- Guard clause giữ logic chính ở mức ngoài cùng.
- `foreach` chạy trên mọi thứ cài `IEnumerable<T>` — mảng, `List<T>`, kết quả LINQ, dữ liệu đọc dần từ database.

## Bước tiếp theo

Bài sau — **Method và tham số** — gói những đoạn logic vừa viết thành method
đặt tên đàng hoàng, và giải thích vì sao sửa object trong method thì bên ngoài
thấy đổi, còn gán lại thì không.

```quiz
[
  {
    "prompt": "Code duyệt foreach trên một List và gọi list.Remove(x) bên trong. Chuyện gì xảy ra?",
    "options": [
      "Xoá được bình thường, vòng lặp bỏ qua phần tử đã xoá",
      "Ném InvalidOperationException ở lần lặp kế tiếp",
      "Vòng lặp chạy vô hạn",
      "Compiler báo lỗi lúc build"
    ],
    "answer": 2,
    "explain": "foreach giữ con trỏ trên danh sách gốc; danh sách đổi là con trỏ mất chỗ đứng. Lọc ra danh sách mới bằng Where, hoặc dùng for duyệt ngược."
  },
  {
    "prompt": "Bạn muốn xoá tại chỗ nhiều phần tử của một List bằng vòng for. Vì sao phải duyệt ngược từ cuối?",
    "options": [
      "Vì RemoveAt chỉ chạy được từ cuối danh sách",
      "Vì xoá phần tử i làm mọi phần tử sau nó lùi một chỗ, duyệt xuôi sẽ bỏ sót",
      "Vì duyệt ngược nhanh hơn",
      "Không bắt buộc, duyệt xuôi cũng đúng"
    ],
    "answer": 2,
    "explain": "Duyệt xuôi thì sau khi xoá phần tử i, phần tử i+1 lùi vào vị trí i mà vòng lặp đã đi qua — nó bị bỏ sót."
  },
  {
    "prompt": "Bạn thêm giá trị OrderStatus.Refunded vào enum. Cách viết nào giúp compiler nhắc bạn xử lý nhánh mới?",
    "options": [
      "switch dạng câu lệnh có default",
      "switch expression liệt kê đủ các nhánh",
      "Chuỗi if - else if",
      "Cả ba đều nhắc như nhau"
    ],
    "answer": 2,
    "explain": "switch expression được compiler kiểm tra tính đầy đủ và cảnh báo khi còn giá trị chưa xử lý; if và switch dạng câu lệnh thì im lặng."
  },
  {
    "prompt": "Method có ba tầng if lồng nhau, logic chính nằm trong cùng. Cách sửa gọn nhất?",
    "options": [
      "Đổi sang switch expression",
      "Gộp ba điều kiện bằng &&",
      "Dùng guard clause: kiểm tra trường hợp sai rồi return sớm",
      "Tách thành ba method nhỏ, mỗi method một tầng"
    ],
    "answer": 3,
    "explain": "Guard clause đưa các trường hợp loại trừ lên đầu và return ngay, để logic chính nằm ở mức thụt lề ngoài cùng, dễ đọc nhất."
  }
]
```
