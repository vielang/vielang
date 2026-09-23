---
title: Kế thừa hay composition
minutes: 11
---

`BaseService` dài 800 dòng, mười hai class kế thừa nó. Bạn sửa một method
trong lớp cha cho đúng yêu cầu của module thanh toán, rồi ba module khác hỏng
— những module bạn chưa từng mở ra xem.

> **Học xong bài này bạn sẽ:** phân biệt "là một" với "có một"; biết cái bẫy
> gọi method `virtual` trong constructor; và chọn composition ở những chỗ
> người ta hay chọn kế thừa theo quán tính.
>
> **Cần biết trước:** `class`, interface (bài trước).

## Hai cách dùng lại code

```csharp
// Kế thừa — "NhanVienToanThoiGian LÀ MỘT NhanVien"
class NhanVien { public string Ten = ""; }
class NhanVienToanThoiGian : NhanVien { }

// Composition — "DonHang CÓ MỘT cách tính phí"
class DonHang(ITinhPhi tinhPhi)
{
    public decimal Phi() => tinhPhi.Tinh(this);
}
```

Phép thử nhanh: đọc to lên. "Nhân viên toàn thời gian **là một** nhân viên" —
xuôi tai. "Đơn hàng **là một** cách tính phí" — vô lý, nên chỗ đó phải là
composition.

Kế thừa tạo ràng buộc mạnh nhất giữa hai class: lớp con thấy cả `protected`,
phụ thuộc vào thứ tự gọi trong lớp cha, và **không đổi được lúc chạy**.
Composition thì chỉ ràng buộc qua một hợp đồng nhỏ.

## Thử ngay: cái bẫy constructor

```csharp
class Cha
{
    public Cha() => InDanhTinh();
    public virtual void InDanhTinh() =>
        Console.WriteLine("Cha");
}

class Con : Cha
{
    private readonly string _ten = "Con";
    public override void InDanhTinh() =>
        Console.WriteLine($"Con: {_ten ?? "null"}");
}

new Con();
```

**Đoán trước khi chạy:** `_ten` được gán ngay khi khai báo. Dòng in ra là
`Con: Con` hay gì khác?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Con: null
```

Constructor của **lớp cha chạy trước**, mà lúc đó field của lớp con chưa được
gán. Method `virtual` gọi từ constructor nhảy xuống bản override của lớp con,
nơi mọi thứ còn rỗng.

Đây là **fragile base class**: lớp cha đúng, lớp con đúng, ghép lại thì sai —
và không compiler nào cảnh báo.

</details>

Quy tắc rút ra: **không gọi method `virtual` trong constructor**. Rộng hơn:
mỗi lần lớp cha gọi một method có thể bị override, nó đang phụ thuộc vào code
mà nó không kiểm soát.

## Khi nào kế thừa là đúng

- Framework yêu cầu: `ControllerBase`, `DbContext`, `Exception`, `BackgroundService`.
- Quan hệ "là một" thật sự và ổn định, lớp con **chỉ thêm** chứ không bóp méo hành vi lớp cha.
- Bạn kiểm soát cả cha lẫn con, và cả hai nằm cùng một module.

Với những thứ khác, hãy bắt đầu bằng composition. Đổi từ composition sang kế
thừa dễ hơn nhiều so với chiều ngược lại.

## Viết lại một lớp cha phình to

```csharp
// SAI — mỗi module cần thêm gì lại nhét vào lớp cha
abstract class BaseService
{
    protected void GhiLog(string s) { }
    protected void GuiMail(string s) { }
    protected void KiemTraQuyen(string s) { }
    protected decimal TinhThue(decimal x) => x * 0.1m;
}

class ThanhToanService : BaseService { }
```

```csharp
// ĐÚNG — nhận đúng thứ mình cần
class ThanhToanService(
    ILogger<ThanhToanService> logger,
    ITinhThue thue)
{
    public decimal Tinh(decimal goc)
    {
        logger.LogInformation("Tính thuế {Goc}", goc);
        return goc + thue.Tinh(goc);
    }
}
```

Bản composition dài dòng hơn vài dòng, nhưng: nhìn constructor là biết class
cần gì; sửa `ITinhThue` không ảnh hưởng module khác; và lúc test thì truyền
vào bản giả, không phải dựng cả `BaseService`.

## sealed, protected và những gì lộ ra

```csharp
public sealed class TinhPhiTieuChuan : ITinhPhi { }
```

`sealed` nói "class này không thiết kế để kế thừa". Đây là mặc định hợp lý:
cho phép kế thừa là một lời hứa rằng mọi method `virtual` sẽ giữ nguyên cách
gọi ở các phiên bản sau.

`protected` là một loại API công khai — với lớp con. Mỗi thành viên
`protected` bạn thêm vào là một thứ nữa phải giữ nguyên về sau.

## Composition trong .NET hằng ngày

Bạn đã dùng composition rất nhiều mà có thể chưa gọi tên nó:

- `ILogger`, `IHttpClientFactory`, `IOptions<T>` nhận qua constructor.
- Middleware trong ASP.NET Core: xếp chồng các mắt xích, không phải kế thừa nhau.
- `Stream` bọc `Stream`: `GZipStream(fileStream)` — thêm hành vi bằng cách bọc, đúng mẫu **Decorator** sẽ gặp ở chương pattern.

## Dấu hiệu trong code của bạn

- Lớp cha có tên `BaseSomething` với đủ thứ tiện ích không liên quan nhau → đó là túi đồ, không phải quan hệ "là một".
- Lớp con override một method chỉ để **vô hiệu hoá** nó (thân rỗng, hoặc ném `NotSupportedException`) → quan hệ "là một" sai ngay từ đầu.
- Constructor lớp cha gọi method `virtual` → fragile base class.
- Cây kế thừa sâu từ ba tầng trở lên → mỗi lần đọc code phải nhảy ba file mới biết method nào đang chạy.
- `protected` rải khắp lớp cha → bề mặt phải giữ nguyên ngày càng rộng.

## Ghi nhớ

- "Là một" thì kế thừa, "có một" thì composition — đọc to lên là biết.
- Không gọi method `virtual` trong constructor.
- Mặc định `sealed`; mở kế thừa là một lời hứa lâu dài.
- Override để vô hiệu hoá hành vi lớp cha nghĩa là quan hệ đó sai.
- Composition dễ test, dễ đổi, và dễ chuyển ngược lại hơn.

## Bước tiếp theo

Hết chương **Đóng gói và trừu tượng**. Chương sau — **Đa hình trong thực tế** —
dùng `virtual`, `abstract` và interface để thay những chuỗi `if` phân loại
theo kiểu, và chọn giữa interface với abstract class.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "class Cha\n{\n    public Cha() => In();\n    public virtual void In() =>\n        Console.WriteLine(\"Cha\");\n}\n\nclass Con : Cha\n{\n    private readonly string _ten = \"Con\";\n    public override void In() =>\n        Console.WriteLine(_ten ?? \"null\");\n}\n\nnew Con();",
    "options": ["Con", "Cha", "null", "Ném NullReferenceException"],
    "answer": 3,
    "explain": "Constructor lớp cha chạy trước khi field của lớp con được gán, mà method virtual lại nhảy xuống bản override. Đừng gọi method virtual trong constructor."
  },
  {
    "prompt": "Quan hệ nào nên dùng composition thay vì kế thừa?",
    "options": [
      "ThanhToanController là một ControllerBase",
      "DonHang có một cách tính phí vận chuyển",
      "DonHangException là một Exception",
      "EmailBackgroundJob là một BackgroundService"
    ],
    "answer": 2,
    "explain": "\"Có một\" là dấu hiệu của composition. Ba trường hợp còn lại là quan hệ \"là một\" thật sự, và đều do framework yêu cầu kế thừa."
  },
  {
    "prompt": "Lớp con override một method của lớp cha và để thân rỗng để nó không làm gì nữa. Điều đó nói lên gì?",
    "options": [
      "Thiết kế tốt, lớp con được tự do",
      "Quan hệ \"là một\" sai — lớp con không thật sự là một lớp cha",
      "Cần đánh dấu method là abstract",
      "Cần thêm sealed cho lớp con"
    ],
    "answer": 2,
    "explain": "Nếu lớp con phải vô hiệu hoá hành vi của lớp cha thì nó không thay thế được lớp cha. Đây chính là vi phạm nguyên tắc Liskov ở chương SOLID."
  },
  {
    "prompt": "Vì sao nên đặt sealed cho class không thiết kế để kế thừa?",
    "options": [
      "Chạy nhanh hơn đáng kể",
      "Tránh phải hứa giữ nguyên cách gọi các method virtual ở các phiên bản sau",
      "Bắt buộc theo quy ước .NET",
      "Để class không dùng được với interface"
    ],
    "answer": 2,
    "explain": "Mở cho kế thừa là một cam kết lâu dài về hành vi nội bộ. sealed nói rõ class này không nằm trong cam kết đó."
  }
]
```
