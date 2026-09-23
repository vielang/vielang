---
title: Method và tham số
minutes: 11
---

Bạn truyền một object vào method rồi sửa property của nó. Ra ngoài, thay đổi
còn nguyên.

Lần sau cũng method ấy, bạn gán hẳn object mới vào tham số. Ra ngoài không đổi
gì cả. Cùng một chữ ký hàm, hai kết quả khác hẳn nhau.

> **Học xong bài này bạn sẽ:** biết method sửa được gì của người gọi; viết chữ
> ký hàm mà người đọc hiểu ngay không cần tra; chọn đúng giữa `out`, tuple và
> một kiểu trả về riêng.
>
> **Cần biết trước:** biến, kiểu dữ liệu, `class` ở mức biết object có
> property.

## Khai báo, và ba quy ước đặt tên

```csharp
public decimal CalculateTotal(decimal price, int qty)
{
    return price * qty;
}

// Thân chỉ một biểu thức — viết gọn
public decimal CalculateTax(decimal amount)
    => amount * 0.1m;

public void Log(string message)   // void: không trả gì
{
    Console.WriteLine(message);
}
```

| Thành phần | Quy ước .NET |
|---|---|
| Method, property | `PascalCase`, tên là **động từ** cho method |
| Tham số, biến cục bộ | `camelCase` |
| Field private | `_camelCase` |

Tên method nói việc, tên biến nói vật. `CalculateTotal` thì rõ, còn
`ProcessData` thì chẳng nói gì — "xử lý" là xử lý cái gì?

## Thử ngay: method sửa được gì của bạn

```csharp
void Rename(Person p) => p.Name = "Nam";
void Replace(Person p) => p = new Person();

var a = new Person();
Rename(a);
Console.WriteLine(a.Name);

var b = new Person();
Replace(b);
Console.WriteLine(b.Name);

public class Person { public string Name = "Huy"; }
```

**Đoán trước khi chạy:** hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Nam
Huy
```

`Rename` sửa **object mà cả hai bên cùng trỏ tới**, nên bên ngoài thấy ngay.

`Replace` thì chỉ gán lại **biến tham số**. Biến ấy là bản chép nằm trong
method. Còn `b` ở ngoài vẫn trỏ object cũ.

</details>

Muốn thay hẳn object của người gọi thì `return` object mới. Cách đó cũng dễ
đọc hơn, vì chỗ gọi nhìn thấy rõ giá trị đang được thay.

## Named argument làm chỗ gọi tự giải thích

```csharp
public string Format(
    decimal amount,
    string currency = "VND",
    bool showSymbol = true)
        => showSymbol ? $"{amount:N0} {currency}"
                      : $"{amount:N0}";

Format(150000);
Format(150000, showSymbol: false);
```

Nhìn `Save(order, true, false)` thì không ai đoán được hai `bool` kia nghĩa
gì. Viết `Save(order, sendMail: true, overwrite: false)` là hết phải đoán.

Tham số tuỳ chọn thì đặt ở cuối, và chỉ dùng cho giá trị mặc định thật sự hợp
lý với đa số trường hợp.

## ref, out, in: ba cách truyền tham chiếu

| Từ khoá | Ai gán giá trị | Gặp ở đâu |
|---|---|---|
| `out` | method **phải** gán trước khi trả về | `TryParse`, `TryGetValue` |
| `ref` | cả hai bên, sửa thẳng biến của người gọi | hiếm |
| `in` | chỉ đọc, tránh chép struct lớn | tối ưu hiệu năng |

```csharp
if (int.TryParse(input, out int qty)) { }

void Double(ref int x) => x *= 2;

decimal Total(in Invoice invoice)
    => invoice.Amount + invoice.Tax;
```

Cần trả về nhiều giá trị thì đừng rải `out`. Dùng **tuple** hoặc một kiểu
riêng, đọc xuôi hơn nhiều:

```csharp
public (bool Ok, string? Error) Validate(Order order)
{
    if (order.Lines.Count == 0)
        return (false, "Đơn hàng trống");

    return (true, null);
}

var (ok, error) = Validate(order);   // deconstruction
```

## Overload, params và local function

```csharp
public void Log(string message) { }
public void Log(string message, Exception ex) { }
public void Log(Exception ex) => Log(ex.Message, ex);

public int Sum(params int[] numbers)   // Sum(1, 2, 3)
{
    int total = 0;
    foreach (var n in numbers) total += n;
    return total;
}

public string Normalize(string raw)
{
    return Clean(raw).ToUpperInvariant();

    // hàm phụ chỉ dùng trong method này
    static string Clean(string s) => s.Trim();
}
```

**Overload** là nhiều method cùng tên, khác danh sách tham số. Compiler chọn
bản phù hợp lúc compile.

Chỉ overload khi các bản làm *cùng một việc*. Khác việc thì đặt tên khác, đừng
bắt người đọc tự đoán.

`params` cho phép gọi `Sum(1, 2, 3)` mà không cần gõ `new[]`. Tiện, nhưng chỉ
đặt nó ở tham số cuối cùng, và mỗi method chỉ một cái.

**Local function** là hàm phụ nằm hẳn bên trong method. Nó không lọt ra ngoài
class, nên người đọc biết ngay phạm vi dùng của nó chỉ có bấy nhiêu. Trước đây
người ta phải viết một `private` method riêng, và rồi ai cũng ngại xoá vì
không chắc còn ai gọi.

## Dấu hiệu trong code của bạn

- Method nhận tham số rồi **gán lại** chính tham số đó → người gọi không thấy gì, gần như luôn là hiểu nhầm.
- Chỗ gọi có hai `bool` trở lên đứng cạnh nhau → dùng named argument, hoặc đổi sang `enum`.
- Chữ ký hàm quá bốn tham số → gom thành một `record` tham số.
- Method dài hơn một màn hình → bên trong đang có vài việc khác nhau, tách ra.

## Ghi nhớ

- Sửa **property** của tham số thì người gọi thấy; **gán lại** tham số thì không.
- Một method làm một việc, tên là động từ.
- Nhiều giá trị trả về → tuple hoặc kiểu riêng, đừng rải `out`.
- Tham số `bool` ở chỗ gọi luôn mờ nghĩa.

## Bước tiếp theo

Method đã gọn gàng. Nhưng thứ chúng xử lý nhiều nhất là chuỗi.

Bài sau, **Chuỗi và định dạng**, mở bằng một API gửi số tiền sang đối tác.
Máy dev gửi `1.5`, server gửi `1,5`, và đối tác đọc thành mười lăm.

```quiz
[
  {
    "prompt": "Sau khi gọi Replace(b), biến b ở ngoài ra sao?",
    "code": "void Replace(Person p) => p = new Person();\n\nvar b = new Person();\nReplace(b);",
    "options": [
      "Trỏ tới object mới vừa tạo",
      "Vẫn trỏ object cũ, không đổi gì",
      "Thành null",
      "Lỗi compile vì không được gán lại tham số"
    ],
    "answer": 2,
    "explain": "Tham số là một bản chép của tham chiếu. Gán lại nó chỉ đổi bản chép cục bộ; muốn thay object của người gọi thì phải return object mới."
  },
  {
    "prompt": "Reviewer đọc Save(order, true, false) và không hiểu hai bool. Cách sửa tốt nhất?",
    "options": [
      "Thêm comment giải thích ở chỗ gọi",
      "Đổi sang named argument, hoặc thay bool bằng enum",
      "Đổi thứ tự tham số cho dễ nhớ",
      "Tách thành hai method Save khác nhau"
    ],
    "answer": 2,
    "explain": "Named argument làm chỗ gọi tự giải thích mà không đổi chữ ký; enum còn rõ hơn vì tên giá trị mang nghĩa."
  },
  {
    "prompt": "Method cần trả về cả kết quả kiểm tra lẫn thông báo lỗi. Cách nào hợp lý nhất trong C# hiện đại?",
    "options": [
      "Hai tham số out",
      "Trả về tuple (bool Ok, string? Error)",
      "Trả về object rồi ép kiểu ở chỗ gọi",
      "Dùng biến static để truyền thông báo"
    ],
    "answer": 2,
    "explain": "Tuple hoặc một record riêng đọc xuôi và dùng deconstruction được. out chỉ nên giữ cho mẫu TryParse và TryGetValue."
  },
  {
    "prompt": "Ba method cùng tên Log nhưng khác danh sách tham số. Compiler chọn bản nào lúc nào?",
    "options": [
      "Lúc chạy, theo kiểu thật của đối số",
      "Lúc compile, theo kiểu của đối số ở chỗ gọi",
      "Luôn chọn bản có ít tham số nhất",
      "Phải chỉ định rõ bằng tên đầy đủ"
    ],
    "answer": 2,
    "explain": "Overload resolution xảy ra lúc compile dựa trên kiểu tĩnh của đối số — đó cũng là lý do overload nhập nhằng bị báo lỗi ngay khi build."
  }
]
```
