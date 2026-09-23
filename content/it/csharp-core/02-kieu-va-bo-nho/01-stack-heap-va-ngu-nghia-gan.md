---
title: Stack, heap và ngữ nghĩa gán
minutes: 9
---

Bài "Kiểu dữ liệu và biến" đã nói value type gán là chép giá trị, reference type
gán là chép tham chiếu. Bài này xem điều đó xảy ra ở đâu trong bộ nhớ, vì đó là
câu phỏng vấn gần như chắc chắn gặp.

## Hai vùng nhớ

- **Stack**: vùng nhớ theo từng lời gọi method, tự dọn khi method kết thúc. Nhanh, kích thước nhỏ.
- **Heap**: vùng nhớ chung cho object, do **GC** (Garbage Collector) dọn khi không còn ai tham chiếu tới.

```csharp
int x = 5;                 // giá trị 5 nằm trên stack
var order = new Order();   // object nằm trên HEAP, biến order trên stack giữ địa chỉ
```

Nói "value type luôn ở stack" là cách nói tắt và **không chính xác**: một `int`
là field của class thì nó nằm trong object đó, tức là trên heap. Điều luôn đúng
là: value type lưu **chính giá trị**, reference type lưu **tham chiếu**.

## Gán và truyền vào method

```csharp
var a = new Point(1, 1);   // struct — value type
var b = a;
b.X = 99;
Console.WriteLine(a.X);    // 1 — b là bản sao

var p = new Person { Name = "Huy" };   // class — reference type
var q = p;
q.Name = "Nam";
Console.WriteLine(p.Name); // Nam — p và q cùng một object
```

Đây là nguồn của loại bug "sửa chỗ này sao chỗ kia đổi theo". Khi một method nhận
vào object rồi sửa property của nó, người gọi **nhìn thấy** thay đổi đó.

```csharp
void Doi(Person person) => person.Name = "Nam";   // đổi được, vì cùng object
void ThayThe(Person person) => person = new();     // KHÔNG đổi gì bên ngoài
```

`person` bản thân nó là tham số được chép: gán lại tham số chỉ đổi bản chép cục
bộ. Muốn thay hẳn object của người gọi thì phải `return` object mới.

## Boxing

```csharp
int n = 42;
object boxed = n;       // boxing  — chép n vào một object trên heap
int back = (int)boxed;  // unboxing — chép ngược ra
```

**Boxing** là gói một value type vào object để nó đi được qua chỗ cần reference
type. Mỗi lần boxing là một lần cấp phát trên heap, nên trong vòng lặp nóng thì
đó là chi phí thật. Dùng **generic** (`List<int>` chứ không phải `ArrayList`) là
tránh được gần hết.

## GC nói ngắn gọn

GC chạy tự động, gom những object không còn ai tham chiếu. Bạn không gọi `delete`
bao giờ. Hai điều thực tế cần nhớ:

- Object còn bị tham chiếu thì **không bao giờ** bị dọn — event handler quên gỡ, `static` giữ danh sách là nguyên nhân phổ biến của **memory leak** trong .NET.
- Tài nguyên ngoài bộ nhớ (file, connection, socket) GC không dọn kịp thời — phải `Dispose`, xem chương về tài nguyên.

## Ghi nhớ

- Value type: chép **giá trị**. Reference type: chép **tham chiếu**. Mọi thứ còn lại suy ra từ đó.
- `string` là reference type nhưng **immutable**, nên dùng nó cảm giác như value type.
- Gán lại tham số bên trong method không ảnh hưởng người gọi, trừ khi có `ref`.
