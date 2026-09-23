---
title: virtual, abstract và override
minutes: 11
---

Reviewer để lại đúng một dòng: "Sao bản export PDF lại ra file CSV?"

Bạn mở code. `PdfExporter` kế thừa `Exporter`, và nó có method `Export` hẳn
hoi, nội dung viết đúng.

Chỉ có điều lúc chạy, cái chạy là `Export` của lớp cha.

> **Học xong bài này bạn sẽ:** biết `virtual` và `override` làm gì ở tầng dưới;
> phân biệt override với method hiding; và chọn đúng giữa `abstract` với
> `virtual` khi thiết kế một lớp cha.
>
> **Cần biết trước:** `class`, kế thừa và `sealed` (bài trước).

## virtual mở cửa, override thay hành vi, abstract bắt buộc

**Đa hình (polymorphism)** là gọi một method qua biến kiểu cha, mà chạy đúng
bản của kiểu con đang nằm trong biến đó.

Bảy từ khoá dưới đây là toàn bộ công cụ để làm việc ấy.

| Từ khoá | Nghĩa |
|---|---|
| `virtual` | lớp cha cho phép lớp con thay hành vi |
| `override` | lớp con thay hành vi của một method `virtual` |
| `abstract` trên method | chỉ có chữ ký, lớp con **buộc** phải viết |
| `abstract` trên class | không `new` trực tiếp được |
| `sealed override` | override lần này là lần cuối, con cháu không đổi nữa |
| `new` trên method | khai báo một method khác trùng tên, **không** phải đa hình |
| `base.Export()` | gọi bản của lớp cha từ trong lớp con |

Cột cuối của bảng dưới đây là chỗ cả bài này quy về.

| Lớp cha viết | Lớp con viết | `Exporter e = new PdfExporter(); e.Export();` |
|---|---|---|
| `void Export()` | `new void Export()` | chạy bản của **Exporter** |
| `virtual void Export()` | `override void Export()` | chạy bản của **PdfExporter** |
| `abstract void Export()` | `override void Export()` | chạy bản của **PdfExporter** |

## Thử ngay: lớp con có method mà vẫn không được gọi

Dòng đầu bảng nghe vô lý nhất, nên thử nó trước.

```csharp
Exporter csv = new CsvExporter();
csv.Export();

Exporter pdf = new PdfExporter();
pdf.Export();

class Exporter
{
    public void Export() =>
        Console.WriteLine("CSV");
}

class CsvExporter : Exporter { }

class PdfExporter : Exporter
{
    public new void Export() =>
        Console.WriteLine("PDF");
}
```

**Đoán trước khi chạy:** biến `pdf` đang giữ một `PdfExporter`, mà class đó có
`Export` riêng. Dòng thứ hai in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
CSV
CSV
```

Cả hai đều in `CSV`.

`Export` của lớp cha không phải `virtual`, nên C# không hỏi object thuộc kiểu
gì. Nó chọn method ngay lúc compile, theo kiểu của **biến** — mà `pdf` khai báo
là `Exporter`.

Chữ `new` chỉ nói "tôi biết tên này đã có, đừng cảnh báo tôi". Nó **che** method
cũ chứ không thay method cũ.

</details>

Sửa bằng cách đổi hai chữ:

```csharp
Exporter pdf = new PdfExporter();
pdf.Export();          // PDF

class Exporter
{
    public virtual void Export() =>
        Console.WriteLine("CSV");
}

class PdfExporter : Exporter
{
    public override void Export() =>
        Console.WriteLine("PDF");
}
```

`virtual` ở cha, `override` ở con. Lúc này C# không quyết định lúc compile nữa.

Nó tra bảng method của object thật vào lúc chạy, rồi mới gọi. Cơ chế đó tên là
**virtual dispatch**, và nó là toàn bộ phần "phép thuật" của đa hình.

## abstract bắt lớp con phải viết, virtual thì chỉ mời

`virtual` cho lớp con quyền đổi. `abstract` thì không cho quyền, mà giao việc.

```csharp
abstract class Report
{
    public abstract void Render();

    public void Run()
    {
        Console.WriteLine("Bắt đầu");
        Render();
    }
}

class PdfReport : Report
{
    public override void Render() =>
        Console.WriteLine("PDF");
}
```

Method `abstract` không có thân. Class chứa nó cũng phải `abstract`, và không ai
`new Report()` được nữa.

Quên viết `Render` trong lớp con là **lỗi compile**, không phải lỗi lúc chạy.
Đó chính là điểm mạnh của nó.

| | `abstract` | `virtual` |
|---|---|---|
| Có thân ở lớp cha | không | có |
| Lớp con bắt buộc viết | **có** | không |
| Quên viết thì | lỗi compile | dùng bản của cha |
| Dùng khi | không có mặc định nào hợp lý | có mặc định, nhưng cho phép đổi |

Để ý `Run` trong ví dụ. Nó gọi `Render` mà không biết bản nào sẽ chạy.

Đó là **template method**: lớp cha giữ trình tự, lớp con điền phần khác nhau.
Bạn sẽ gặp lại nó ở chương design pattern.

## base gọi lại phần của lớp cha thay vì thay hẳn

Đôi khi lớp con không muốn thay, chỉ muốn thêm.

```csharp
class AuditReport : PdfReport
{
    public override void Render()
    {
        base.Render();
        Console.WriteLine("Đã ghi log");
    }
}
```

`base.Render()` gọi bản của lớp cha, rồi lớp con làm tiếp phần của mình.

Nhưng để ý một điều. `PdfReport.Render` phải là `virtual` hoặc `override` thì
`AuditReport` mới override được — và trong ví dụ trên nó là `override`, nên
vẫn còn mở.

Muốn đóng lại thì có `sealed override`:

```csharp
class FinalReport : PdfReport
{
    public sealed override void Render() =>
        Console.WriteLine("chốt, không đổi nữa");
}
```

Từ đây trở xuống không ai override `Render` được nữa. Dùng nó khi hành vi ấy là
cam kết, không phải chỗ để mở rộng.

## Dấu hiệu trong code của bạn

- Lớp con có method trùng tên lớp cha mà không có `override` → compiler cảnh báo **CS0108**, và đó là bug ở đầu bài đang chờ.
- Thấy `new` trên một method → gần như luôn là hiểu nhầm; thứ người viết muốn là `virtual` cộng `override`.
- Lớp cha `virtual` mà không lớp con nào override → chưa cần `virtual`, cứ để thường.
- `abstract` trên method có thân rỗng `{ }` → đó là `virtual`, không phải `abstract`; thân rỗng nghĩa là "không làm gì" chứ không phải "chưa viết".
- Gọi method `virtual` trong constructor → cái bẫy fragile base class ở bài trước.

## Ghi nhớ

- `virtual` ở cha cộng `override` ở con mới có đa hình; thiếu một trong hai là không.
- Không `virtual` thì method được chọn lúc compile, theo kiểu của **biến**.
- `new` che method cũ, không thay method cũ.
- `abstract` giao việc cho lớp con, và quên làm là lỗi compile.
- `sealed override` chốt hành vi lại, không cho con cháu đổi nữa.

## Bước tiếp theo

Bạn đã có công cụ. Câu hỏi kế là dùng nó vào việc gì.

Bài sau, **Thay chuỗi if theo kiểu bằng đa hình**, mở bằng một lần `grep`: thêm
một hình thức thanh toán mới, mà phải sửa bốn chỗ ở bốn file khác nhau.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "class Base\n{\n    public void Hi() =>\n        Console.WriteLine(\"Base\");\n}\n\nclass Child : Base\n{\n    public new void Hi() =>\n        Console.WriteLine(\"Child\");\n}\n\nBase x = new Child();\nx.Hi();",
    "options": [
      "Base",
      "Child",
      "Lỗi compile vì trùng tên method",
      "Cả hai dòng, Base rồi Child"
    ],
    "answer": 1,
    "explain": "Hi không phải virtual, nên C# chọn method theo kiểu của biến x lúc compile — mà x khai báo là Base. Chữ new chỉ tắt cảnh báo trùng tên; muốn chạy bản của Child thì phải virtual cộng override."
  },
  {
    "prompt": "Lớp cha có method tính phí, và bạn muốn compiler báo lỗi nếu lớp con quên viết bản riêng. Khai báo thế nào?",
    "options": [
      "public virtual decimal Fee() => 0;",
      "public decimal Fee() => throw new NotImplementedException();",
      "public abstract decimal Fee();",
      "public virtual decimal Fee() => throw new NotImplementedException();"
    ],
    "answer": 3,
    "explain": "abstract không có thân, nên lớp con buộc phải override, và quên là lỗi compile. Ba phương án kia đều để lớp con lặng lẽ dùng bản của cha, rồi nổ hoặc trả 0 lúc chạy."
  },
  {
    "prompt": "Lớp con override một method và muốn giữ nguyên hành vi cũ, chỉ thêm một dòng ghi log. Viết gì trong thân method?",
    "options": [
      "Chép lại code của lớp cha rồi thêm dòng log",
      "this.Render() rồi thêm dòng log",
      "Bỏ override, dùng new để không mất bản cũ",
      "base.Render() rồi thêm dòng log"
    ],
    "answer": 4,
    "explain": "base.X() gọi đúng bản của lớp cha. Gọi this.Render() là gọi lại chính nó, tức đệ quy vô hạn."
  },
  {
    "prompt": "Trong review, bạn thấy một lớp cha khai báo mười method virtual mà không lớp con nào override. Nhận xét nào đúng?",
    "options": [
      "Tốt, lớp con sau này muốn đổi gì cũng được",
      "Mỗi virtual là một lời hứa phải giữ về sau, nên chưa cần thì đừng mở",
      "Không ảnh hưởng gì, virtual chỉ là cú pháp",
      "Phải đổi hết sang abstract cho rõ ý"
    ],
    "answer": 2,
    "explain": "Mở virtual là hứa rằng cách gọi method ấy sẽ giữ nguyên ở các phiên bản sau, vì có thể đã có người override. Chưa có ai cần đổi thì cứ để method thường, sau này mở vẫn được — còn đóng lại thì không."
  }
]
```
