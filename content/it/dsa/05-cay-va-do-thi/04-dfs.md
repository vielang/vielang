---
title: DFS
minutes: 5
---

Bão làm hỏng vài con đường, công ty cần biết từ kho còn tới được những điểm
nào. Câu hỏi này không cần số bước ít nhất, chỉ cần đi được tới hay không. DFS
trả lời việc đó bằng cách đi sâu hết một nhánh rồi mới quay lại, dùng đúng đệ
quy của chương 1.

## Khái niệm

🤿 **DFS (depth-first search, tìm theo chiều sâu)**: từ một đỉnh, đi tiếp sang một đỉnh kề chưa thăm, cứ thế đi sâu tới khi hết đường, rồi quay lui thử nhánh khác.

BFS dùng queue, còn DFS dùng stack. Viết DFS bằng đệ quy thì call stack chính
là stack đó, như bài Đệ quy. Cả hai đều O(số đỉnh + số cạnh).

## Ví dụ

Dùng lại `roads` và `AddRoad` của bài trước:

```csharp
var visited = new HashSet<string>();
Dfs("Kho", visited);
Console.WriteLine(visited.Count);   // 5

void Dfs(string current, HashSet<string> seen)
{
    seen.Add(current);
    foreach (string next in roads[current])
    {
        if (!seen.Contains(next))
        {
            Dfs(next, seen);
        }
    }
}
```

- `seen` là `HashSet` như chương 3, đánh dấu điểm đã thăm với O(1).
- Mỗi lần gặp điểm kề chưa thăm là gọi đệ quy đi sâu vào đó ngay.
- Hết điểm kề chưa thăm thì method `return`, tức quay lui về điểm trước.
- Sau khi chạy, `seen` chứa mọi điểm tới được từ kho.

## Thử ngay

Thêm `Console.WriteLine("Thăm " + current);` ngay dưới dòng `seen.Add(current);`
rồi chạy lại.

**Đoán trước khi chạy:** thứ tự thăm có giống BFS ở bài trước (Kho, A, B, C,
D) không?

<details>
<summary>Xem kết quả</summary>

```text
Thăm Kho
Thăm A
Thăm C
Thăm B
Thăm D
5
```

Không giống. Từ A, DFS đi thẳng sang C rồi sang B, hết đường mới quay lui
về C để sang D.

BFS thì thăm A và B (cùng cách kho 1 bước) trước C. DFS vẫn tới đủ 5 điểm,
nhưng thứ tự thăm không cho biết số bước ít nhất.

</details>

## Lỗi hay gặp

**Đệ quy trên đồ thị rất sâu.** Một chuỗi hàng trăm nghìn điểm nối liền nhau
làm call stack đầy, chương trình sập với `Stack overflow.` Khi đó thay call
stack bằng `Stack<T>` tự quản lý.

```csharp
// SAI — đồ thị rất sâu: đệ quy làm tràn call stack
Dfs("Kho", visited);
```

```csharp
// ĐÚNG — dùng Stack<T>, không phụ thuộc call stack
var seen = new HashSet<string>();
var stack = new Stack<string>();
stack.Push("Kho");
while (stack.Count > 0)
{
    string current = stack.Pop();
    if (seen.Add(current))
    {
        foreach (string next in roads[current])
        {
            stack.Push(next);
        }
    }
}
```

`seen.Add` trả `false` nếu điểm đã thăm (bài HashSet và bài toán đếm), nên
điểm đó bị bỏ qua.

## Tóm tắt

- DFS đi sâu hết một nhánh rồi quay lui, dùng stack hoặc đệ quy.
- BFS dùng queue, cho số bước ít nhất. DFS không cho số bước ít nhất.
- DFS hợp để hỏi "tới được không", "có những vùng nào nối với nhau".
- Đồ thị rất sâu thì dùng `Stack<T>` thay cho đệ quy.

```quiz
[
  {
    "prompt": "Cần tìm đường qua ít điểm nhất từ kho tới khách. Nên dùng gì?",
    "options": [
      "DFS",
      "Sắp xếp chèn",
      "Tìm nhị phân",
      "BFS"
    ],
    "answer": 4,
    "explain": "BFS thăm theo từng lớp khoảng cách, nên lần đầu tới khách là đường ít điểm nhất. DFS không bảo đảm điều đó."
  },
  {
    "prompt": "DFS viết bằng đệ quy dùng stack nào để nhớ đường quay lui?",
    "options": [
      "Call stack",
      "Queue<T>",
      "HashSet<T>",
      "Không cần stack"
    ],
    "answer": 1,
    "explain": "Mỗi lời gọi đệ quy chồng lên call stack. Return là quay lui về điểm trước."
  },
  {
    "prompt": "Sau khi chạy DFS từ kho, visited có 5 điểm trong khi đồ thị có 8 điểm. Điều đó nghĩa là gì?",
    "options": [
      "DFS dừng sớm khi gặp vòng",
      "Có 3 điểm không nối tới kho",
      "Có 3 điểm bị thăm hai lần",
      "BFS sẽ thăm đủ 8 điểm"
    ],
    "answer": 2,
    "explain": "DFS thăm mọi điểm tới được. Điểm không có trong visited là không có đường nối từ kho, nên BFS cũng không tới được."
  }
]
```
