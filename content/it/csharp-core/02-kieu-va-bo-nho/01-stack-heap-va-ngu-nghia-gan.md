---
title: Stack, heap và ngữ nghĩa gán
minutes: 11
---

Bạn lấy giỏ hàng mẫu ra, gán cho khách A, thêm vài món.

Khách B mở giỏ của mình lên thì thấy đúng mấy món đó. Không dòng code nào gán
nhầm. Chỉ là hai người đang cầm chung một object.

> **Học xong bài này bạn sẽ:** nhìn một phép gán và biết ngay hai biến dùng
> chung hay tách rời; tránh được họ bug "sửa chỗ này chỗ kia đổi theo"; giải
> thích được boxing khi phỏng vấn.
>
> **Cần biết trước:** value type và reference type ở mức đã gặp ở chương trước.

## Stack và heap: hai vùng nhớ, hai cách dọn

| | Stack | Heap |
|---|---|---|
| Chứa gì | biến cục bộ, tham số | object do `new` tạo ra |
| Dọn khi nào | method kết thúc là tự dọn | GC dọn khi không còn ai tham chiếu |
| Tốc độ | rất nhanh | chậm hơn |
| Kích thước | nhỏ, có giới hạn | lớn |

```csharp
int x = 5;                // giá trị 5 nằm trên stack
var order = new Order();  // object nằm trên HEAP,
                          // biến order giữ địa chỉ
```

Nói "value type luôn ở stack" là cách nói tắt, và **không chính xác**. Một
`int` là field của class thì nó nằm trong object đó, tức là trên heap.

Điều luôn đúng chỉ có một câu: value type lưu **chính giá trị**, reference
type lưu **tham chiếu**.

```mermaid Hai biến cùng trỏ một object, còn int thì mỗi biến giữ một giá trị
flowchart TD
    subgraph stack["Stack — theo từng lời gọi method"]
        p["Person p"]
        q["Person q = p"]
        x["int x = 5"]
    end
    subgraph heap["Heap — GC dọn"]
        obj["Person { Name = &quot;Huy&quot; }"]
    end
    p -->|tham chiếu| obj
    q -->|cùng tham chiếu| obj
```

## Thử ngay: chép giá trị hay chép tham chiếu

```csharp
struct Point { public int X; }
class Person { public string Name = "Huy"; }

var a = new Point { X = 1 };
var b = a;
b.X = 99;

var p = new Person();
var q = p;
q.Name = "Nam";

Console.WriteLine($"{a.X} và {p.Name}");
```

**Đoán trước khi chạy:** dòng cuối in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
1 và Nam
```

`Point` là **struct**, tức value type. `b = a` chép hẳn một bản, nên sửa `b`
không đụng tới `a`.

`Person` là **class**, tức reference type. `q = p` chỉ chép địa chỉ. Hai biến
cùng trỏ một object, nên sửa qua `q` thì `p` thấy ngay.

</details>

Đó chính là bug giỏ hàng ở đầu bài. Muốn mỗi khách một giỏ riêng thì phải tạo
object mới:

```csharp
// SAI — hai khách dùng chung một giỏ
var cartA = sampleCart;
var cartB = sampleCart;
```

```csharp
// ĐÚNG — mỗi khách một object riêng
var cartA = new Cart(sampleCart.Items.ToList());
var cartB = new Cart(sampleCart.Items.ToList());
```

Chú ý cả `Items.ToList()`. Nếu dùng lại chính danh sách của giỏ mẫu thì hai
giỏ vẫn chung nhau một tầng bên trong.

## Truyền vào method: sửa được, nhưng không thay được

```csharp
void Rename(Person p) => p.Name = "Nam";   // đổi được
void Replace(Person p) => p = new();       // KHÔNG đổi
```

Tham số là một **bản chép của tham chiếu**. Sửa property thì cả hai bên cùng
thấy. Gán lại tham số chỉ đổi bản chép cục bộ.

Muốn thay hẳn object của người gọi thì `return` object mới.

## Boxing: gói value type lên heap

```csharp
int n = 42;
object boxed = n;        // boxing — chép n lên heap
int back = (int)boxed;   // unboxing — chép ngược ra
```

**Boxing** là gói một value type vào object, để nó đi được qua chỗ cần
reference type. Mỗi lần boxing là một lần cấp phát trên heap.

Trong vòng lặp nóng, đó là chi phí thật. Dùng **generic** (`List<int>` thay
cho `ArrayList`) là tránh được gần hết.

## GC dọn bộ nhớ, nhưng không dọn hộ bạn mọi thứ

GC chạy tự động và gom những object không còn ai tham chiếu. Bạn không bao giờ
gọi `delete`. Nhưng có hai điều nó không làm được:

| Tình huống | Vì sao GC chịu |
|---|---|
| `static` giữ `List` hay `Dictionary` mãi | vẫn còn tham chiếu, nên không phải rác |
| Event handler quên gỡ | object nghe sự kiện vẫn bị giữ |
| File, connection, socket | không phải bộ nhớ, phải `Dispose` |

Hai dòng đầu bảng là nguyên nhân phổ biến nhất của **memory leak** trong .NET.
Dòng cuối có hẳn một bài riêng ở chương Ngoại lệ và tài nguyên.

## Dấu hiệu trong code của bạn

- Một `List<T>` hay object được gán cho nhiều nơi rồi nơi nào cũng sửa → tất cả đang dùng chung, sớm muộn cũng lệch dữ liệu.
- Method nhận collection rồi `Add` hay `Remove` vào chính nó → người gọi bị sửa dữ liệu mà không biết.
- `static` giữ `List` hay `Dictionary` mà chỉ thêm, không bao giờ xoá → memory leak.
- `ArrayList`, `Hashtable`, hoặc tham số `object` trong vòng lặp lớn → boxing.

## Ghi nhớ

- Value type chép **giá trị**, reference type chép **tham chiếu**.
- Muốn tách rời thì tạo object mới, và nhớ chép cả collection bên trong.
- Gán lại tham số trong method không ảnh hưởng người gọi.
- `string` là reference type nhưng **immutable**, nên dùng như value type.

## Bước tiếp theo

Biết object nằm ở đâu rồi. Câu hỏi kế: khi khai báo kiểu mới thì chọn gì?

Bài sau, **class, struct hay record**, mở bằng một hàm so tiền luôn trả
`false`. Hai bên rõ ràng cùng là 100.000 đồng.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "var a = new List<int> { 1, 2 };\nvar b = a;\nb.Add(3);\n\nConsole.WriteLine(a.Count);",
    "options": ["2", "3", "0", "Lỗi lúc chạy"],
    "answer": 2,
    "explain": "List là reference type: b và a cùng trỏ một danh sách. Muốn tách rời thì var b = a.ToList()."
  },
  {
    "prompt": "Giỏ hàng mẫu được gán cho hai khách rồi mỗi khách thêm món riêng, cuối cùng hai giỏ giống hệt nhau. Nguyên nhân?",
    "options": [
      "Hai biến cùng trỏ một object giỏ hàng",
      "GC dọn nhầm một trong hai giỏ",
      "Giỏ hàng là struct nên bị chép giá trị",
      "Lỗi của database, không phải của code"
    ],
    "answer": 1,
    "explain": "Gán một reference type chỉ chép địa chỉ. Mỗi khách muốn giỏ riêng thì phải tạo object mới, và chép cả danh sách bên trong."
  },
  {
    "prompt": "Dòng này làm gì trong bộ nhớ?",
    "code": "object boxed = 42;",
    "options": [
      "Không có gì đặc biệt, 42 vẫn nằm trên stack",
      "Boxing: 42 được chép lên heap trong một object",
      "42 bị chuyển thành chuỗi \"42\"",
      "Lỗi compile vì int không gán cho object được"
    ],
    "answer": 2,
    "explain": "Boxing gói value type vào một object trên heap. Trong vòng lặp lớn, mỗi lần boxing là một lần cấp phát — dùng generic để tránh."
  },
  {
    "prompt": "Service có một static Dictionary làm cache, chỉ thêm chứ không bao giờ xoá. Hệ quả là gì?",
    "options": [
      "Không sao, GC sẽ tự dọn khi cần bộ nhớ",
      "Memory leak: còn tham chiếu thì GC không dọn được",
      "Cache tự hết hạn sau 20 phút",
      "Dictionary ném OutOfMemoryException khi đầy"
    ],
    "answer": 2,
    "explain": "GC chỉ dọn object không còn ai tham chiếu. Biến static sống suốt đời tiến trình nên mọi thứ nó giữ đều không dọn được."
  }
]
```
