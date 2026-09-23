---
title: Method và tham số
minutes: 11
---

Bạn truyền một object vào method, sửa property của nó, ra ngoài thấy đổi thật.
Lần sau cũng method đó, bạn gán hẳn object mới vào tham số — ra ngoài không đổi
gì cả. Cùng một chữ ký hàm, hai kết quả khác nhau.

> **Học xong bài này bạn sẽ:** biết method sửa được gì của người gọi và không
> sửa được gì; viết chữ ký hàm mà người đọc hiểu ngay không cần tra; chọn đúng
> giữa `out`, tuple và một kiểu trả về riêng.
>
> **Cần biết trước:** biến, kiểu dữ liệu, `class` ở mức biết `Person` có
> property `Name`.

## Khai báo và giá trị trả về

```csharp
public decimal TinhThanhTien(decimal gia, int soLuong)
{
    return gia * soLuong;
}

// Thân chỉ một biểu thức — viết gọn
public decimal TinhThue(decimal tien) => tien * 0.1m;

public void GhiLog(string mess)   // void: không trả gì
{
    Console.WriteLine($"[{DateTime.Now:HH:mm}] {mess}");
}
```

Tên method nên là **động từ** (`TinhThanhTien`, `SaveOrder`), tên biến là danh
từ. Quy ước .NET: method và property viết `PascalCase`, tham số và biến cục bộ
viết `camelCase`.

## Thử ngay: method sửa được gì của bạn

```csharp
public class Person { public string Name = "Huy"; }

void Doi(Person p) => p.Name = "Nam";
void ThayThe(Person p) => p = new Person();

var a = new Person();
Doi(a);
Console.WriteLine(a.Name);

var b = new Person();
ThayThe(b);
Console.WriteLine(b.Name);
```

**Đoán trước khi chạy:** hai dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Nam
Huy
```

`Doi` sửa **object mà cả hai bên cùng trỏ tới** nên bên ngoài thấy. `ThayThe`
chỉ gán lại **biến tham số** — bản chép cục bộ nằm trong method — còn biến `b`
bên ngoài vẫn trỏ object cũ.

</details>

Muốn thay hẳn object của người gọi thì `return` object mới, đừng gán vào tham
số. Cách này cũng dễ đọc hơn, vì chỗ gọi nhìn thấy rõ giá trị đang được thay.

## Tham số tuỳ chọn và named argument

```csharp
public string Format(
    decimal tien,
    string donVi = "VND",
    bool hienDonVi = true) =>
        hienDonVi ? $"{tien:N0} {donVi}" : $"{tien:N0}";

Format(150000);
Format(150000, hienDonVi: false);
```

**Named argument** làm chỗ gọi tự giải thích. Thấy `Save(order, true, false)`
thì không ai đoán được hai `bool` kia nghĩa gì; `Save(order, guiMail: true,
ghiDe: false)` thì rõ ngay.

## ref, out và in

```csharp
// out: method phải gán giá trị trước khi trả về
if (int.TryParse(input, out int soLuong)) { }

// ref: sửa được chính biến của người gọi
void GapDoi(ref int x) => x *= 2;

// in: truyền tham chiếu nhưng CHỈ ĐỌC
decimal Tong(in HoaDon hd) => hd.Tien + hd.Thue;
```

Thực tế `out` chủ yếu gặp ở mẫu `TryParse`/`TryGetValue`, còn `ref` thì hiếm.
Cần trả nhiều giá trị thì ưu tiên **tuple** hoặc một kiểu riêng:

```csharp
public (bool Ok, string? Loi) KiemTra(Order o)
{
    if (o.Items.Count == 0)
        return (false, "Đơn hàng trống");

    return (true, null);
}

var (ok, loi) = KiemTra(order);   // deconstruction
```

## Overload, params và local function

```csharp
public void Log(string message) { }
public void Log(string message, Exception ex) { }
public void Log(Exception ex) => Log(ex.Message, ex);
```

**Overload** là nhiều method cùng tên, khác danh sách tham số. Chỉ overload khi
các bản làm *cùng một việc*; khác việc thì đặt tên khác.

```csharp
public int Tong(params int[] so)  // gọi: Tong(1, 2, 3)
{
    int t = 0;
    foreach (var n in so) t += n;
    return t;
}

public string Chuan(string raw)
{
    return Sach(raw).ToUpperInvariant();

    // hàm phụ chỉ dùng trong method này
    static string Sach(string s) => s.Trim();
}
```

## Dấu hiệu trong code của bạn

- Method nhận tham số rồi **gán lại** chính tham số đó → người gọi không thấy gì, gần như luôn là hiểu nhầm.
- Chỗ gọi có hai `bool` trở lên đứng cạnh nhau → dùng named argument, hoặc đổi sang `enum`.
- Chữ ký hàm quá 4 tham số → gom thành một `record` tham số, vừa dễ đọc vừa dễ thêm về sau.
- Method dài hơn một màn hình → bên trong đang có vài việc khác nhau, tách ra.

## Ghi nhớ

- Sửa **property** của tham số thì người gọi thấy; **gán lại** tham số thì không.
- Một method làm một việc, tên là động từ.
- Cần nhiều giá trị trả về → tuple hoặc kiểu riêng, đừng rải `out`.
- Tham số `bool` trong lời gọi luôn mờ nghĩa — dùng `enum` hoặc named argument.

## Bước tiếp theo

Bài sau — **Chuỗi và định dạng** — đi vào thứ mọi backend dùng suốt ngày: nối
chuỗi, định dạng số và ngày, cùng cái bẫy culture chỉ lộ ra trên production.

```quiz
[
  {
    "prompt": "Sau khi gọi ThayThe(b), biến b ở ngoài ra sao?",
    "code": "void ThayThe(Person p) => p = new Person();\n\nvar b = new Person();\nThayThe(b);",
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
      "Trả về tuple (bool Ok, string? Loi)",
      "Trả về object rồi ép kiểu ở chỗ gọi",
      "Dùng biến static để truyền thông báo"
    ],
    "answer": 2,
    "explain": "Tuple (hoặc một record riêng) đọc xuôi và dùng deconstruction được. out chỉ nên giữ cho mẫu TryParse/TryGetValue."
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
