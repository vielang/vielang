---
title: Queue
minutes: 5
---

Đơn hàng đến trước phải được đóng gói trước. Stack lấy phần tử mới nhất, nên
dùng cho đơn hàng thì khách đặt sớm nhất lại chờ lâu nhất. Queue làm ngược
lại: ai đến trước được phục vụ trước.

## Khái niệm

🎟️ **Queue (hàng đợi)**: danh sách thêm vào ở cuối và lấy ra ở đầu, phần tử vào trước được lấy ra trước (FIFO: first in, first out).

| Thao tác | Việc làm | Big-O |
|---|---|---|
| `Enqueue(x)` | thêm `x` vào cuối hàng | O(1) trung bình |
| `Dequeue()` | lấy và bỏ phần tử ở đầu hàng | O(1) |
| `Peek()` | xem phần tử ở đầu hàng, không bỏ | O(1) |

## Ví dụ

```csharp
var orders = new Queue<string>();
orders.Enqueue("DH1");
orders.Enqueue("DH2");
orders.Enqueue("DH3");

Console.WriteLine(orders.Dequeue());   // DH1
Console.WriteLine(orders.Peek());      // DH2
Console.WriteLine(orders.Count);       // 2
```

- `Enqueue` xếp đơn vào cuối hàng, `Dequeue` lấy đơn ở đầu hàng.
- `DH1` vào trước nên ra trước. `Peek` thấy `DH2` đang đứng đầu.

## Bên trong Queue

Tự viết queue bằng `List<T>` với `Add` ở cuối và `RemoveAt(0)` ở đầu thì
`Dequeue` thành O(n), như bài Array và List bên trong. `Queue<T>` không dời
chỗ mà dùng array vòng tròn: giữ hai chỉ số đầu và cuối, lấy ra chỉ là tăng
chỉ số đầu. Hết ô ở cuối array thì quay lại dùng các ô trống ở đầu.

## Thử ngay

Xử lý đơn trong khi vẫn có đơn mới đến:

```csharp
var orders = new Queue<string>();
orders.Enqueue("DH1");
orders.Enqueue("DH2");

Console.WriteLine("Đóng gói " + orders.Dequeue());
orders.Enqueue("DH3");
orders.Enqueue("DH4");
Console.WriteLine("Đóng gói " + orders.Dequeue());

while (orders.Count > 0)
{
    Console.WriteLine("Còn chờ " + orders.Dequeue());
}
```

**Đoán trước khi chạy:** thứ tự các đơn được in ra là gì?

<details>
<summary>Xem kết quả</summary>

```text
Đóng gói DH1
Đóng gói DH2
Còn chờ DH3
Còn chờ DH4
```

Đơn mới đến lúc nào cũng xếp cuối hàng, nên không đơn nào chen lên trước đơn
đến sớm hơn.

</details>

## Lỗi hay gặp

**Dùng `List<T>` làm hàng đợi.** `RemoveAt(0)` dời cả list mỗi lần lấy đơn,
hàng dài thì chậm hẳn.

```csharp
// SAI — mỗi lần lấy đơn là O(n)
var orders = new List<string> { "DH1", "DH2" };
string next = orders[0];
orders.RemoveAt(0);
```

```csharp
// ĐÚNG — Dequeue là O(1)
var orders = new Queue<string>();
orders.Enqueue("DH1");
orders.Enqueue("DH2");
string next = orders.Dequeue();
```

Giống `Stack<T>`, gọi `Dequeue` khi hàng rỗng sẽ ném
`InvalidOperationException` (`Queue empty.`), nên kiểm tra `Count` trước.

## Tóm tắt

- Queue thêm ở cuối, lấy ở đầu: vào trước ra trước (FIFO).
- `Enqueue`, `Dequeue`, `Peek` đều O(1) nhờ array vòng tròn.
- Dùng cho đơn hàng chờ xử lý, và cho tìm kiếm theo chiều rộng (BFS) ở chương 5.
- Đừng dùng `List<T>` với `RemoveAt(0)` làm hàng đợi.

```quiz
[
  {
    "prompt": "Enqueue lần lượt A, B, C rồi Dequeue một lần. Peek lúc này trả về gì?",
    "options": [
      "A",
      "B",
      "C",
      "Hàng rỗng"
    ],
    "answer": 2,
    "explain": "Dequeue lấy A là phần tử vào trước. B đứng đầu hàng nên Peek thấy B."
  },
  {
    "prompt": "Tổng đài xử lý cuộc gọi theo thứ tự gọi đến. Nên dùng cấu trúc nào?",
    "options": [
      "Stack",
      "Linked list không giữ node cuối",
      "Queue",
      "Array cố định"
    ],
    "answer": 3,
    "explain": "Ai gọi trước được nghe trước, đúng kiểu vào trước ra trước của queue."
  },
  {
    "prompt": "Vì sao Dequeue của Queue<T> là O(1) còn List.RemoveAt(0) là O(n)?",
    "options": [
      "Queue<T> lưu phần tử trong Dictionary",
      "Queue<T> dùng linked list bên trong",
      "RemoveAt(0) phải tìm phần tử trước",
      "Queue<T> chỉ tăng chỉ số đầu"
    ],
    "answer": 4,
    "explain": "Array vòng tròn giữ chỉ số đầu và cuối. Lấy ra chỉ đổi chỉ số, không dời các phần tử còn lại như RemoveAt(0)."
  }
]
```
