---
title: Stack, heap và ngữ nghĩa gán
minutes: 10
---

Bạn lấy giỏ hàng mẫu ra, gán cho khách A, thêm vài món. Khách B mở giỏ hàng
của mình thì thấy đúng mấy món đó. Không có dòng code nào gán nhầm — chỉ là cả
hai đang cầm chung một object.

> **Học xong bài này bạn sẽ:** nhìn một phép gán và biết ngay hai biến đang
> dùng chung hay tách rời; tránh được họ bug "sửa chỗ này chỗ kia đổi theo";
> giải thích được boxing khi phỏng vấn.
>
> **Cần biết trước:** value type và reference type ở mức đã gặp ở chương trước.

## Hai vùng nhớ

- **Stack**: vùng nhớ theo từng lời gọi method, tự dọn khi method kết thúc. Nhanh, nhỏ.
- **Heap**: vùng nhớ chung cho object, do **GC** (Garbage Collector) dọn khi không còn ai tham chiếu tới.

```csharp
int x = 5;              // giá trị 5 nằm trên stack
var don = new Order();  // object nằm trên HEAP,
                        // biến don giữ địa chỉ của nó
```

Nói "value type luôn ở stack" là cách nói tắt và **không chính xác**: một `int`
là field của class thì nó nằm trong object đó, tức là trên heap. Điều luôn đúng
là: value type lưu **chính giá trị**, reference type lưu **tham chiếu**.

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
struct Diem { public int X; }
class Nguoi { public string Ten = "Huy"; }

var a = new Diem { X = 1 };
var b = a;
b.X = 99;

var p = new Nguoi();
var q = p;
q.Ten = "Nam";

Console.WriteLine($"{a.X} và {p.Ten}");
```

**Đoán trước khi chạy:** dòng cuối in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
1 và Nam
```

`Diem` là **struct** (value type): `b = a` chép hẳn một bản, sửa `b` không đụng
tới `a`. `Nguoi` là **class** (reference type): `q = p` chỉ chép địa chỉ, hai
biến cùng trỏ một object nên sửa qua `q` thì `p` thấy ngay.

</details>

Đây chính là bug giỏ hàng ở đầu bài. Muốn mỗi khách một giỏ riêng thì phải tạo
object mới, không gán lại biến cũ:

```csharp
// SAI — hai khách dùng chung một giỏ
var gioA = gioMau;
var gioB = gioMau;
```

```csharp
// ĐÚNG — mỗi khách một object riêng
var gioA = new Gio(gioMau.Items.ToList());
var gioB = new Gio(gioMau.Items.ToList());
```

Chú ý cả `Items.ToList()`: chép danh sách chứ không dùng lại chính danh sách
của giỏ mẫu — nếu không thì vẫn chung nhau một tầng bên trong.

## Truyền vào method

```csharp
void Doi(Nguoi n) => n.Ten = "Nam";   // đổi được
void ThayThe(Nguoi n) => n = new();   // KHÔNG đổi gì
```

Tham số là một **bản chép của tham chiếu**. Sửa property thì cả hai bên cùng
thấy; gán lại tham số chỉ đổi bản chép cục bộ. Muốn thay hẳn object của người
gọi thì `return` object mới.

## Boxing

```csharp
int n = 42;
object hop = n;        // boxing — chép n lên heap
int lai = (int)hop;    // unboxing — chép ngược ra
```

**Boxing** là gói một value type vào object để nó đi được qua chỗ cần reference
type. Mỗi lần boxing là một lần cấp phát trên heap, nên trong vòng lặp nóng đó
là chi phí thật. Dùng **generic** (`List<int>` chứ không phải `ArrayList`) là
tránh được gần hết.

## GC nói ngắn gọn

GC chạy tự động, gom những object không còn ai tham chiếu. Bạn không bao giờ
gọi `delete`. Hai điều thực tế cần nhớ:

- Object còn bị tham chiếu thì **không bao giờ** bị dọn — biến `static` giữ danh sách, hay event handler quên gỡ, là nguyên nhân phổ biến của **memory leak** trong .NET.
- Tài nguyên ngoài bộ nhớ (file, connection, socket) GC không dọn kịp thời — phải `Dispose`, xem chương về tài nguyên.

## Dấu hiệu trong code của bạn

- Một `List<T>` hay object được gán cho nhiều nơi rồi nơi nào cũng sửa → tất cả đang dùng chung, sớm muộn cũng lệch dữ liệu.
- Method nhận collection rồi `Add`/`Remove` vào chính nó → người gọi bị sửa dữ liệu mà không biết; nhận `IReadOnlyList<T>` thì compiler chặn giúp.
- `static` giữ `List`/`Dictionary` mà chỉ thêm, không bao giờ xoá → memory leak.
- `ArrayList`, `Hashtable`, hay `object` làm tham số trong vòng lặp lớn → boxing.

## Ghi nhớ

- Value type chép **giá trị**, reference type chép **tham chiếu**. Mọi thứ khác suy ra từ đó.
- Muốn tách rời thì tạo object mới, và nhớ chép cả collection bên trong.
- Gán lại tham số trong method không ảnh hưởng người gọi.
- `string` là reference type nhưng **immutable** nên dùng như value type.

## Bước tiếp theo

Bài sau — **class, struct hay record** — chọn kiểu khai báo cho từng loại dữ
liệu, và giải thích vì sao hai object giống hệt nhau vẫn có thể `==` ra `false`.

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
    "code": "object hop = 42;",
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
