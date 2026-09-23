---
title: Đóng gói giữ object hợp lệ
minutes: 11
---

Một đơn hàng đã giao bỗng quay về trạng thái "mới". Bạn grep cả dự án và thấy
năm chỗ khác nhau cùng gán `don.TrangThai = ...`, mỗi chỗ một người viết, chỗ
nào cũng đúng theo ý tác giả của nó. Không ai sai, mà đơn hàng thì hỏng.

> **Học xong bài này bạn sẽ:** giữ object không bao giờ rơi vào trạng thái vô
> lý; thay `set` công khai bằng method mang tên nghiệp vụ; và nhận ra kiểu
> class "chỉ có property" đang đẩy hết trách nhiệm cho người gọi.
>
> **Cần biết trước:** `class`, property, `init`/`private set` (C# Core).

## Đóng gói không phải là đặt private cho có

Đóng gói là giữ **quy tắc bất biến** (invariant) nằm chung một chỗ với dữ liệu
mà nó ràng buộc. "Số lượng không âm", "đơn đã giao thì không sửa được", "tổng
tiền luôn bằng tổng các dòng hàng" — những câu đó phải sống bên trong class,
không phải nằm rải trong mười controller.

## Thử ngay: class ai cũng sửa được

```csharp
class DonHang
{
    public int SoLuong { get; set; }
    public decimal DonGia { get; set; }
    public string TrangThai { get; set; } = "Moi";
    public decimal Tong => SoLuong * DonGia;
}

var d = new DonHang { SoLuong = 2, DonGia = 100 };
d.TrangThai = "DaGiao";
d.SoLuong = -5;

Console.WriteLine($"{d.TrangThai} / {d.Tong}");
```

**Đoán trước khi chạy:** compiler hay runtime có chặn dòng `SoLuong = -5` trên
một đơn đã giao không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
DaGiao / -500
```

Không ai chặn cả. Class này chỉ là một túi đựng property, nên **mọi quy tắc
nghiệp vụ đều nằm ở người gọi** — và chỉ cần một chỗ quên kiểm tra là dữ liệu
hỏng. Đơn đã giao vẫn sửa được số lượng, tổng tiền thành số âm.

</details>

## Viết lại: object tự bảo vệ mình

```csharp
class DonHang
{
    public int SoLuong { get; private set; }
    public decimal DonGia { get; }
    public TrangThai TrangThai { get; private set; }
        = TrangThai.Moi;

    public decimal Tong => SoLuong * DonGia;

    public DonHang(int soLuong, decimal donGia)
    {
        if (soLuong <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(soLuong));
        if (donGia < 0)
            throw new ArgumentOutOfRangeException(
                nameof(donGia));

        SoLuong = soLuong;
        DonGia = donGia;
    }

    public void DoiSoLuong(int moi)
    {
        if (TrangThai != TrangThai.Moi)
            throw new InvalidOperationException(
                "Đơn đã xử lý, không đổi số lượng");
        if (moi <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(moi));

        SoLuong = moi;
    }

    public void Giao()
    {
        if (TrangThai != TrangThai.DaThanhToan)
            throw new InvalidOperationException(
                "Chưa thanh toán thì chưa giao được");

        TrangThai = TrangThai.DaGiao;
    }
}
```

Ba thay đổi, mỗi cái giải quyết một việc:

- **Constructor kiểm tra** — không tồn tại một `DonHang` sai ngay từ lúc sinh ra.
- **`private set`** — chỉ class tự đổi trạng thái của mình.
- **Method mang tên nghiệp vụ** (`Giao`, `DoiSoLuong`) — quy tắc nằm cạnh dữ liệu, và tên method nói đúng việc đang xảy ra.

Đọc `don.Giao()` thì biết ngay ý định. Đọc `don.TrangThai = "DaGiao"` thì
không biết người viết đã kiểm tra gì chưa.

## Đừng lộ collection bên trong

```csharp
// SAI — ai cũng Add được, bỏ qua mọi quy tắc
public List<DongHang> Items { get; } = new();
```

```csharp
// ĐÚNG — thêm qua method, đọc qua kiểu chỉ đọc
private readonly List<DongHang> _items = new();

public IReadOnlyList<DongHang> Items => _items;

public void ThemHang(DongHang hang)
{
    if (TrangThai != TrangThai.Moi)
        throw new InvalidOperationException(
            "Đơn đã xử lý");

    _items.Add(hang);
}
```

Trả thẳng `List<T>` ra ngoài là trao chìa khoá cho người lạ: họ `Add`,
`Clear`, `RemoveAt` được mà class của bạn không hề biết.

## Anemic model: dấu hiệu mất đóng gói

Class chỉ có property, còn mọi logic nằm trong `OrderService` dài 500 dòng —
kiểu thiết kế này có tên: **anemic domain model**. Nó không sai về cú pháp,
nhưng hậu quả là quy tắc nghiệp vụ bị chép ra nhiều nơi, và chỉ cần một nơi
quên là dữ liệu hỏng.

Không phải class nào cũng cần hành vi. **DTO** — thứ chở dữ liệu qua API — thì
đúng là chỉ nên có property, vì nó không có quy tắc nào để bảo vệ. Ranh giới:
*có quy tắc bất biến thì đóng gói; chỉ chở dữ liệu thì để trần*.

## Dấu hiệu trong code của bạn

- Property `public set` cho trạng thái nghiệp vụ (`TrangThai`, `SoLuong`, `SoDu`) → đổi sang `private set` cộng method.
- Cùng một câu `if` kiểm tra trước khi gán, lặp lại ở nhiều nơi → quy tắc đó thuộc về class, không thuộc về người gọi.
- `public List<T>` trong domain model → đổi sang `IReadOnlyList<T>` + method thêm/bớt.
- Constructor rỗng rồi gán từng property sau đó → object tồn tại ở trạng thái dang dở giữa hai dòng code.
- Class chỉ có property + một service dài 500 dòng thao tác lên nó → anemic model.

## Ghi nhớ

- Đóng gói là để **object không thể sai**, không phải để giấu cho kín.
- Kiểm tra ở constructor: không có object nào sinh ra đã sai.
- Method mang tên nghiệp vụ nói rõ ý định hơn phép gán property.
- Đừng trả `List<T>` ra ngoài; `IReadOnlyList<T>` + method.
- DTO thì để trần, domain model thì đóng gói.

## Bước tiếp theo

Bài sau — **Interface là hợp đồng** — tách "làm gì" khỏi "làm thế nào", và vì
sao một interface 14 method là dấu hiệu thiết kế sai.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì?",
    "code": "var d = new DonHang { SoLuong = 2, DonGia = 100 };\nd.TrangThai = \"DaGiao\";\nd.SoLuong = -5;\n\nConsole.WriteLine($\"{d.TrangThai} / {d.Tong}\");",
    "options": [
      "Ném InvalidOperationException",
      "DaGiao / -500",
      "DaGiao / 200",
      "Lỗi compile vì không được gán SoLuong"
    ],
    "answer": 2,
    "explain": "Class chỉ có property public set nên không có gì chặn giá trị vô lý. Mọi quy tắc bị đẩy sang người gọi, và chỉ cần một chỗ quên là dữ liệu hỏng."
  },
  {
    "prompt": "Cách nào thể hiện rõ ý định và giữ được quy tắc nghiệp vụ?",
    "options": [
      "don.TrangThai = TrangThai.DaGiao;",
      "don.Giao();",
      "orderService.CapNhatTrangThai(don, 3);",
      "don.SetTrangThai(TrangThai.DaGiao);"
    ],
    "answer": 2,
    "explain": "Method mang tên nghiệp vụ vừa nói đúng việc đang xảy ra, vừa là chỗ duy nhất kiểm tra điều kiện trước khi đổi trạng thái."
  },
  {
    "prompt": "Domain model có public List<DongHang> Items { get; }. Vấn đề là gì?",
    "options": [
      "Không có vấn đề vì đã bỏ set",
      "Người ngoài vẫn Add/Clear/RemoveAt được mà class không biết",
      "List không dùng được trong domain model",
      "Sẽ gây memory leak"
    ],
    "answer": 2,
    "explain": "Bỏ set chỉ chặn việc thay cả danh sách; nội dung bên trong vẫn sửa thoải mái. Lộ ra bằng IReadOnlyList và thêm bớt qua method."
  },
  {
    "prompt": "Class nào KHÔNG cần đóng gói hành vi?",
    "options": [
      "Đơn hàng có quy tắc chuyển trạng thái",
      "Tài khoản có số dư không được âm",
      "DTO chở dữ liệu từ API về",
      "Giỏ hàng có giới hạn số món"
    ],
    "answer": 3,
    "explain": "DTO không có quy tắc bất biến nào để bảo vệ, chỉ chở dữ liệu. Ba kiểu còn lại đều có invariant nên logic phải nằm cùng dữ liệu."
  }
]
```
