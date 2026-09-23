---
title: Rẽ nhánh và vòng lặp
minutes: 11
---

Job dọn dữ liệu chạy lúc hai giờ sáng. Việc của nó đơn giản: duyệt danh sách
đơn hàng, thấy đơn đã huỷ thì xoá đi.

Chạy thử với hai đơn còn hiệu lực, êm. Lên production gặp đơn đã huỷ đầu tiên,
job sập với `InvalidOperationException`.

> **Học xong bài này bạn sẽ:** viết rẽ nhánh bằng `switch expression` như code
> C# hiện đại; chọn đúng giữa `for` và `foreach`; và xoá phần tử trong lúc
> duyệt mà không làm sập chương trình.
>
> **Cần biết trước:** biến, toán tử so sánh, `List<T>`.

## Ba cách rẽ nhánh, chọn theo việc

| Cách viết | Hợp khi |
|---|---|
| `if` / `else if` | điều kiện phức tạp, mỗi nhánh làm nhiều việc |
| `switch expression` | chọn **một giá trị** theo một biến |
| Guard clause (`if … return`) | loại bỏ trường hợp sai rồi làm việc chính |

```csharp
string label = status switch
{
    OrderStatus.New       => "Mới tạo",
    OrderStatus.Paid      => "Đã thanh toán",
    OrderStatus.Cancelled => "Đã huỷ",
    _                     => "Không xác định",
};
```

`switch expression` (từ C# 8) trả về một giá trị nên không cần `break`. Dấu
`_` là **discard**, đóng vai `default`.

Nhưng để ý cái giá của `_`. Nó nhận mọi giá trị, kể cả giá trị bạn thêm vào
enum sáu tháng sau.

Bỏ `_` đi thì compiler buộc bạn liệt kê đủ, và thiếu một nhánh là nó cảnh báo
**CS8509** ngay lúc build. Chuỗi `if` thì im lặng trong cả hai trường hợp.

## Guard clause kéo logic chính ra khỏi ba tầng ngoặc

```csharp
// SAI — việc chính nằm sâu nhất
if (order != null)
{
    if (order.IsPaid)
    {
        if (order.Items.Count > 0)
            Ship(order);
    }
}
```

```csharp
// ĐÚNG — chặn sớm, việc chính ở mức ngoài cùng
if (order is null) return;
if (!order.IsPaid) return;
if (order.Items.Count == 0) return;

Ship(order);
```

Hai đoạn chạy như nhau. Nhưng đoạn dưới đọc được từ trên xuống. Thêm một điều
kiện nữa cũng không làm code thụt sâu thêm.

Còn `{ }`: luôn viết, kể cả khi thân `if` chỉ một dòng. Thêm dòng thứ hai mà
quên ngoặc là lỗi rất khó nhìn ra lúc review.

## Thử ngay: vì sao job dọn dữ liệu sập

```csharp
var codes = new List<string> { "a", "old", "b", "old" };

foreach (var code in codes)
{
    if (code == "old")
        codes.Remove(code);
}

Console.WriteLine(string.Join(",", codes));
```

**Đoán trước khi chạy:** in ra `a,b`, in ra thứ khác, hay không in được gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Unhandled exception. System.InvalidOperationException:
  Collection was modified; enumeration operation
  may not execute.
```

`foreach` giữ một con trỏ chạy trên danh sách gốc. Xoá phần tử là danh sách
đổi. Con trỏ mất chỗ đứng, và lần lặp sau ném lỗi.

Bản chạy thử không sập vì hai đơn mẫu đều còn hiệu lực — nhánh `Remove` chưa
từng chạy. Chỉ cần một đơn đã huỷ là lần lặp ngay sau đó ném lỗi, bất kể danh
sách dài mấy phần tử.

</details>

Hai cách sửa, chọn theo ý định:

```csharp
// Lọc ra danh sách mới — rõ ý, hay dùng nhất
codes = codes.Where(c => c != "old").ToList();

// Xoá tại chỗ, duyệt NGƯỢC từ cuối
for (int i = codes.Count - 1; i >= 0; i--)
{
    if (codes[i] == "old") codes.RemoveAt(i);
}
```

Duyệt ngược mới an toàn: xoá phần tử thứ `i` chỉ làm lệch những phần tử phía
sau, mà chúng thì đã xét xong rồi.

## Chọn vòng lặp theo thứ bạn cần

| Vòng lặp | Dùng khi |
|---|---|
| `foreach` | chỉ cần từng phần tử — mặc định nên dùng |
| `for` | cần chỉ số, hoặc cần duyệt ngược |
| `while` | chưa biết trước số lần lặp |
| `do … while` | phải chạy ít nhất một lần |

```csharp
foreach (var item in items)
    Console.WriteLine(item);

for (int i = 0; i < items.Count; i++)
    Console.WriteLine($"{i}: {items[i]}");

while (queue.Count > 0)
    Handle(queue.Dequeue());
```

`foreach` nói đúng ý định: làm gì đó với từng phần tử. Nó cũng không có chỗ
nào để gõ nhầm chỉ số.

Bên trong vòng lặp, hai từ khoá này giúp bạn khỏi phải lồng thêm `if`:

```csharp
foreach (var order in orders)
{
    if (order.IsDeleted) continue;  // bỏ qua, chạy tiếp
    if (order.IsFinal) break;      // thoát hẳn vòng lặp
    Process(order);
}
```

`continue` nhảy sang phần tử kế tiếp. `break` bỏ luôn cả vòng lặp. Dùng chúng
như guard clause: loại trường hợp không cần xử lý ra trước. Phần còn lại nằm
phẳng ở dưới.

Hai cách viết gọn nữa hay gặp trong code C# hiện đại:

```csharp
var last = items[^1];     // phần tử cuối
var first3 = items[..3];  // ba phần tử đầu
```

`^1` là "đếm ngược từ cuối", còn `..` là một khoảng. Không cần
`items.Count - 1` nữa, nên cũng hết chỗ sai chỉ số.

## Dấu hiệu trong code của bạn

- `Add`, `Remove`, `Clear` nằm trong thân một `foreach` duyệt chính collection đó → sẽ ném `InvalidOperationException`.
- `if` lồng từ ba tầng trở lên → đổi sang guard clause, hoặc tách method.
- `switch` dạng câu lệnh dài mà mỗi nhánh chỉ gán một giá trị → viết lại thành `switch expression`.
- `while (true)` mà không thấy `break` hay `CancellationToken` → service sẽ treo, không phải "có thể treo".

## Ghi nhớ

- Không thêm hay xoá phần tử của collection đang `foreach`.
- Muốn xoá: lọc ra danh sách mới, hoặc `for` duyệt ngược từ cuối.
- `switch expression` trả giá trị, không cần `break`, và được compiler nhắc khi thiếu nhánh.
- Guard clause giữ logic chính ở mức ngoài cùng.

## Bước tiếp theo

Những đoạn logic vừa viết rồi sẽ dài ra. Tới lúc gói chúng lại và đặt tên.

Bài sau, **Method và tham số**, mở bằng một câu đố. Sửa object trong method
thì bên ngoài thấy, còn gán lại thì không. Cùng một chữ ký hàm.

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
      "switch expression liệt kê đủ các nhánh, KHÔNG có nhánh _",
      "Chuỗi if - else if",
      "switch expression có nhánh _ nhận mọi giá trị còn lại"
    ],
    "answer": 2,
    "explain": "Compiler kiểm tra tính đầy đủ của switch expression và cảnh báo CS8509 khi còn giá trị chưa xử lý. Nhưng một nhánh _ là tự nhận hết phần còn lại, nên cảnh báo đó tắt luôn — phương án D im lặng đúng như if."
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
    "explain": "Guard clause đưa các trường hợp loại trừ lên đầu và return ngay, để logic chính nằm ở mức thụt lề ngoài cùng."
  }
]
```
