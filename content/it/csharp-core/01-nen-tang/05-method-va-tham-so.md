---
title: Method và tham số
minutes: 9
---

**Method** là đơn vị công việc nhỏ nhất mà bạn đặt tên. Đặt tên đúng và chọn
tham số đúng quyết định code đọc được hay không, nhiều hơn mọi mẹo cú pháp.

## Khai báo và giá trị trả về

```csharp
public decimal TinhThanhTien(decimal donGia, int soLuong)
{
    return donGia * soLuong;
}

// Thân chỉ một biểu thức — viết gọn bằng expression-bodied member
public decimal TinhThue(decimal tien) => tien * 0.1m;

public void GhiLog(string message)   // void: không trả về gì
{
    Console.WriteLine($"[{DateTime.Now:HH:mm:ss}] {message}");
}
```

Tên method nên là **động từ** (`TinhThanhTien`, `SaveOrder`), tên biến là danh
từ. Quy ước .NET: method và property viết `PascalCase`, tham số và biến cục bộ
viết `camelCase`.

## Tham số tuỳ chọn và tham số gọi theo tên

```csharp
public string Format(decimal amount, string currency = "VND", bool showSymbol = true)
    => showSymbol ? $"{amount:N0} {currency}" : $"{amount:N0}";

Format(150000);                              // dùng mặc định
Format(150000, showSymbol: false);           // named argument: bỏ qua currency
```

**Named argument** làm chỗ gọi tự giải thích. Thấy `Save(order, true, false)` thì
không ai đoán được hai `bool` kia nghĩa gì; `Save(order, sendMail: true,
overwrite: false)` thì rõ ngay.

## ref, out và in

```csharp
// out: method chịu trách nhiệm gán giá trị trước khi trả về
if (int.TryParse(input, out int soLuong)) { }

// ref: truyền tham chiếu của chính biến, sửa được giá trị bên ngoài
void TangGapDoi(ref int x) => x *= 2;

// in: truyền tham chiếu nhưng CHỈ ĐỌC — tránh chép struct lớn
decimal Tong(in HoaDon hoaDon) => hoaDon.Tien + hoaDon.Thue;
```

Thực tế `out` chủ yếu gặp ở mẫu `TryParse`/`TryGetValue`, `ref` thì hiếm. Cần trả
về nhiều giá trị thì ưu tiên **tuple** hoặc một kiểu riêng, dễ đọc hơn nhiều:

```csharp
public (bool Success, string? Error) Validate(Order order)
{
    if (order.Items.Count == 0) return (false, "Đơn hàng trống");
    return (true, null);
}

var (success, error) = Validate(order);   // deconstruction
```

## Overload

```csharp
public void Log(string message) { }
public void Log(string message, Exception ex) { }
public void Log(Exception ex) => Log(ex.Message, ex);
```

**Overload** là nhiều method cùng tên, khác danh sách tham số. Compiler chọn bản
phù hợp lúc compile. Chỉ overload khi các bản làm *cùng một việc*; khác việc thì
đặt tên khác.

## params và local function

```csharp
public int Tong(params int[] numbers)      // gọi: Tong(1, 2, 3)
{
    int total = 0;
    foreach (var n in numbers) total += n;
    return total;
}

public string Chuan(string raw)
{
    return Clean(raw).ToUpperInvariant();

    // local function: hàm phụ chỉ dùng trong method này
    static string Clean(string s) => s.Trim().Replace("  ", " ");
}
```

## Ghi nhớ

- Một method nên làm **một việc**; dài quá một màn hình là dấu hiệu nên tách.
- Quá 3–4 tham số thì gom lại thành một class/record tham số, dễ đọc và dễ mở rộng.
- Tham số `bool` trong lời gọi luôn mờ nghĩa — dùng `enum` hoặc named argument.
