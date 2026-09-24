---
title: Array và List bên trong
minutes: 6
---

Ở khoá C# Core, array có số ô cố định, còn `List<T>` thêm bao nhiêu cũng được.
Thật ra bên trong `List<T>` vẫn là một array. Hiểu nó nới rộng thế nào sẽ giúp
bạn biết thao tác nào nhanh, thao tác nào chậm.

## Khái niệm

Array ở bài Array và List của khoá C# Core là dãy ô nằm liền nhau trong bộ
nhớ. Nhờ vậy lấy theo vị trí chỉ mất O(1), nhưng số ô cố định lúc tạo.

📏 **Capacity**: số ô của array bên trong `List<T>`, luôn lớn hơn hoặc bằng `Count` là số phần tử đang dùng.

Khi array bên trong đầy, `List<T>` tạo array mới gấp đôi, chép hết phần tử
sang, rồi bỏ array cũ.

## Ví dụ

Một bản `List` tối giản, chỉ chứa `string`:

```csharp
var cart = new MiniList();
cart.Add("Bút bi");
cart.Add("Vở");
cart.Add("Thước");
Console.WriteLine(cart.Count);      // 3
Console.WriteLine(cart.Capacity);   // 4

class MiniList
{
    private string[] _items = new string[2];

    public int Count { get; private set; }
    public int Capacity => _items.Length;

    public void Add(string item)
    {
        if (Count == _items.Length)
        {
            string[] bigger =
                new string[_items.Length * 2];
            for (int i = 0; i < Count; i++)
            {
                bigger[i] = _items[i];
            }
            _items = bigger;
        }
        _items[Count] = item;
        Count = Count + 1;
    }
}
```

- `new string[2]` tạo array 2 ô trống. `Count` đếm số ô đã dùng.
- `Capacity => _items.Length` là property chỉ đọc, viết gọn bằng `=>`.
- Thêm "Thước" khi 2 ô đã đầy: tạo array 4 ô, chép 2 phần tử cũ sang, rồi mới
  thêm.
- Chép là O(n), nhưng vì mỗi lần gấp đôi nên hiếm khi xảy ra. Tính trung bình,
  `Add` vẫn là O(1).

| Thao tác trên `List<T>` | Big-O | Vì sao |
|---|---|---|
| `list[i]`, `Add` | O(1) | nhảy thẳng tới ô, thêm vào cuối |
| `Insert(0, x)`, `RemoveAt(0)` | O(n) | dời mọi phần tử phía sau một ô |
| `Contains`, `IndexOf` | O(n) | so từng phần tử |

## Thử ngay

Xem `List<T>` thật của .NET nới rộng thế nào:

```csharp
var list = new List<string>();
Console.WriteLine(list.Capacity);
for (int i = 1; i <= 9; i++)
{
    list.Add("SP" + i);
    Console.Write(list.Capacity + " ");
}
```

**Đoán trước khi chạy:** list mới tạo có capacity bao nhiêu, và capacity
thay đổi ở những lần `Add` nào?

<details>
<summary>Xem kết quả</summary>

```text
0
4 4 4 4 8 8 8 8 16
```

List rỗng chưa cấp ô nào. Lần `Add` đầu cấp 4 ô. Tới phần tử thứ 5 và thứ 9,
array đầy nên gấp đôi thành 8 rồi 16.

</details>

## Lỗi hay gặp

**Lấy dần phần tử đầu bằng `RemoveAt(0)`.** Mỗi lần xoá đầu, cả list dời lên
một ô. Làm vậy cho cả list là O(n²).

```csharp
// SAI — mỗi RemoveAt(0) dời hết phần còn lại
List<string> orders = new List<string>
{
    "DH1", "DH2", "DH3"
};
while (orders.Count > 0)
{
    Console.WriteLine(orders[0]);
    orders.RemoveAt(0);
}
```

```csharp
// ĐÚNG — duyệt từ đầu tới cuối, không xoá
List<string> orders = new List<string>
{
    "DH1", "DH2", "DH3"
};
foreach (string order in orders)
{
    Console.WriteLine(order);
}
```

Cần vừa lấy ra ở đầu vừa thêm vào ở cuối thì dùng `Queue<T>`, ở bài Queue.

## Tóm tắt

- `List<T>` bên trong là array, đầy thì tạo array gấp đôi rồi chép sang.
- Lấy theo vị trí và `Add` vào cuối là O(1).
- Chèn hoặc xoá ở đầu là O(n), vì phải dời các phần tử phía sau.
- `Count` là số phần tử đang dùng, `Capacity` là số ô đã cấp.

```quiz
[
  {
    "prompt": "List<string> có Count = 8, Capacity = 8. Gọi Add thêm một phần tử. Capacity sau đó là bao nhiêu?",
    "options": [
      "9",
      "16",
      "8",
      "12"
    ],
    "answer": 2,
    "explain": "Array bên trong đã đầy, List tạo array mới gấp đôi là 16 ô rồi chép 8 phần tử sang."
  },
  {
    "prompt": "Thao tác nào trên List<T> là O(n)?",
    "options": [
      "list[5]",
      "list.Add(x)",
      "list.Count",
      "list.Insert(0, x)"
    ],
    "answer": 4,
    "explain": "Chèn vào đầu phải dời mọi phần tử phía sau lùi một ô."
  },
  {
    "prompt": "Vì sao Add vào cuối List vẫn được coi là O(1) dù đôi khi phải chép cả array?",
    "options": [
      "Vì chép array không tốn thời gian",
      "Vì List không bao giờ chép",
      "Vì mỗi lần nới là gấp đôi, nên lần chép hiếm dần; tính trung bình mỗi Add vẫn là O(1)",
      "Vì Big-O bỏ qua mọi vòng lặp"
    ],
    "answer": 3,
    "explain": "Gấp đôi nghĩa là sau mỗi lần chép n phần tử, phải thêm n lần nữa mới chép tiếp. Chia đều ra mỗi Add chỉ tốn thêm một hằng số."
  }
]
```
