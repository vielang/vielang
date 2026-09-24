---
title: Linked list
minutes: 5
---

Chèn một đơn gấp vào đầu `List<T>` thì mọi đơn phía sau phải dời chỗ, tốn
O(n). Linked list không xếp phần tử liền nhau, nên chèn vào đầu chỉ mất một
bước. Đổi lại, muốn tới phần tử thứ 1000 thì phải đi qua 999 phần tử trước.

## Khái niệm

⛓️ **Linked list**: danh sách mà mỗi phần tử (gọi là node) giữ giá trị và tham chiếu tới node kế tiếp, node cuối trỏ tới `null`.

`Next` là một tham chiếu (bài Value type và reference type của khoá C# Core):
nó không chứa node kế tiếp, mà chỉ trỏ tới node đó.

## Ví dụ

```csharp
var first = new Node("DH1");
first.Next = new Node("DH2");
first.Next.Next = new Node("DH3");

Node? current = first;
while (current != null)
{
    Console.WriteLine(current.Value);
    current = current.Next;
}

class Node
{
    public string Value { get; }
    public Node? Next { get; set; }

    public Node(string value)
    {
        Value = value;
    }
}
```

- Mỗi `Node` giữ `Value` và `Next`. `Next` của `DH3` là `null`, đánh dấu hết
  danh sách.
- Duyệt bằng cách đi theo `Next` từ node đầu, tới khi gặp `null`.
- `Node?` cho phép biến chứa `null`, như `string?` ở bài null và nullable
  của khoá C# Core.

| Thao tác | `List<T>` | Linked list |
|---|---|---|
| Lấy phần tử thứ `i` | O(1) | O(n), đi từ đầu |
| Thêm vào đầu | O(n), dời cả list | O(1), đổi một tham chiếu |
| Thêm vào cuối | O(1) trung bình | O(1) nếu giữ node cuối |

.NET có sẵn `LinkedList<T>` với `AddFirst`, `AddLast`, `RemoveFirst`,
`RemoveLast`, đều O(1).

## Thử ngay

Thêm đơn gấp `DH0` vào đầu danh sách. Đặt ba dòng này ngay trên dòng
`Node? current = first;`:

```csharp
var urgent = new Node("DH0");
urgent.Next = first;
first = urgent;
```

**Đoán trước khi chạy:** danh sách in ra theo thứ tự nào? Có node nào phải
dời chỗ không?

<details>
<summary>Xem kết quả</summary>

```text
DH0
DH1
DH2
DH3
```

`DH0` đứng đầu, mà không node nào phải dời. Chỉ có hai tham chiếu đổi:
`urgent.Next` trỏ tới `DH1`, và `first` trỏ tới `urgent`.

</details>

## Lỗi hay gặp

**Lấy phần tử theo vị trí trên `LinkedList<T>`.** Linked list không có ô
đánh số, nên không có `[i]`.

```csharp
// SAI — lỗi compile: LinkedList không có [i]
var orders = new LinkedList<string>();
orders.AddLast("DH1");
orders.AddLast("DH2");
Console.WriteLine(orders[1]);
```

```csharp
// ĐÚNG — cần lấy theo vị trí thì dùng List<T>
var orders = new List<string>();
orders.Add("DH1");
orders.Add("DH2");
Console.WriteLine(orders[1]);
```

## Tóm tắt

- Linked list gồm các node, mỗi node giữ giá trị và tham chiếu tới node kế.
- Thêm, bớt ở đầu là O(1). Lấy phần tử thứ `i` là O(n).
- .NET có `LinkedList<T>`. Không có `[i]`.
- Trong thực tế, `List<T>` thường nhanh hơn vì các ô nằm liền nhau. Chỉ dùng
  linked list khi thêm bớt ở đầu hoặc giữa thật nhiều.

```quiz
[
  {
    "prompt": "Danh sách chờ giao hàng liên tục có đơn gấp chen lên đầu, và hiếm khi cần lấy đơn thứ i. Cấu trúc nào hợp hơn?",
    "options": [
      "List<T>, thêm bằng Insert(0, x)",
      "Array, chép sang array mới mỗi lần",
      "List<T>, Add rồi sắp xếp lại",
      "Linked list, thêm bằng AddFirst"
    ],
    "answer": 4,
    "explain": "AddFirst chỉ đổi một tham chiếu, là O(1). Insert(0, x), chép array hay sắp xếp lại đều phải đụng tới mọi phần tử."
  },
  {
    "prompt": "Linked list có 1.000 node. Lấy node thứ 1.000 tốn khoảng bao nhiêu bước?",
    "options": [
      "1.000 bước",
      "1 bước",
      "10 bước",
      "Không lấy được"
    ],
    "answer": 1,
    "explain": "Node không đánh số nên phải đi lần lượt từ node đầu, tức O(n)."
  },
  {
    "prompt": "Node cuối của linked list có Next bằng gì?",
    "options": [
      "Chính nó",
      "null",
      "Node đầu",
      "Số 0"
    ],
    "answer": 2,
    "explain": "Next bằng null báo hiệu không còn node nào phía sau. Vòng duyệt dừng khi gặp null."
  }
]
```
