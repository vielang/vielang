---
title: Toán tử và ép kiểu
minutes: 8
---

Phần lớn bug "số ra sai" trong code backend không nằm ở công thức, mà ở chỗ hai
số khác kiểu gặp nhau và compiler tự quyết thay bạn.

## Toán tử số học

```csharp
int a = 7, b = 2;

int q = a / b;        // 3  — chia hai int là chia lấy nguyên
int r = a % b;        // 1  — modulo, phần dư
double d = a / 2.0;   // 3.5 — có một vế là double thì kết quả là double
```

Chia hai `int` ra `int`: `7 / 2` bằng `3`, không phải `3.5`. Muốn ra số thực thì
một vế phải là số thực — đây là lỗi kinh điển khi tính trung bình hay phần trăm.

## Ép kiểu tường minh và ngầm định

```csharp
int small = 100;
long big = small;          // implicit — int lọt gọn vào long, không mất gì

double price = 19.99;
int rounded = (int)price;  // explicit — 19, CẮT phần thập phân chứ không làm tròn

int correct = (int)Math.Round(price);  // 20
```

Compiler tự chuyển (**implicit conversion**) khi chắc chắn không mất dữ liệu.
Ngược lại bạn phải tự ép (**explicit cast**) và tự chịu trách nhiệm: `(int)` cắt
cụt phần thập phân, muốn làm tròn phải gọi `Math.Round`.

## Tràn số và checked

```csharp
int max = int.MaxValue;
int overflow = max + 1;           // -2147483648, âm thầm quay vòng

checked
{
    int boom = max + 1;           // ném OverflowException
}
```

Mặc định C# **không** báo tràn số. Với tiền tệ, số lượng tồn kho hay id tự tăng,
hãy dùng kiểu đủ rộng (`long`, `decimal`) chứ đừng trông chờ vào may mắn.

## Đổi chuỗi sang số

```csharp
int ok = int.Parse("42");                  // 42
int bad = int.Parse("bốn hai");            // ném FormatException

if (int.TryParse(input, out int value))    // không ném, trả về false
{
    Console.WriteLine(value * 2);
}
```

Dữ liệu từ người dùng, từ query string hay từ file CSV thì luôn dùng `TryParse`.
`Parse` chỉ dành cho chuỗi mà bạn tự sinh ra và chắc chắn đúng định dạng.

## Toán tử logic và rút gọn

```csharp
if (user != null && user.IsActive) { }   // && ngắn mạch: user == null thì bỏ qua vế sau
bool both = Check(a) & Check(b);          // & luôn chạy CẢ HAI vế

int port = configPort ?? 8080;            // null thì lấy 8080
name ??= "Khách";                          // gán khi đang null
```

`&&` và `||` là **short-circuit**: vế phải chỉ chạy khi cần. Nhờ vậy mẫu
`x != null && x.Prop` mới an toàn. `&` và `|` thì chạy hết cả hai vế — hiếm khi
bạn muốn điều đó.

## Ghi nhớ

- `decimal` cho tiền, `double` cho đo lường khoa học. Đừng cộng tiền bằng `double`.
- `(int)x` cắt cụt, `Math.Round(x)` làm tròn, `Math.Floor`/`Math.Ceiling` làm tròn xuống/lên.
- So sánh chuỗi bằng `==` là so sánh nội dung, nhưng phân biệt hoa thường — cần bỏ qua thì dùng `string.Equals(a, b, StringComparison.OrdinalIgnoreCase)`.
