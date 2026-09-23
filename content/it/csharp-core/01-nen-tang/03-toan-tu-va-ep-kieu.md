---
title: Toán tử và ép kiểu
minutes: 10
---

Báo cáo tỉ lệ đơn hàng thành công hiện `0%` suốt cả tuần, dù rõ ràng có 47
đơn thành công trên 100. Công thức đúng, dữ liệu đúng. Sai ở chỗ hai số đem
chia đều là `int`.

> **Học xong bài này bạn sẽ:** đọc một biểu thức số và biết kết quả ra kiểu
> gì; chọn đúng giữa ép kiểu, làm tròn và `TryParse`; nhận ra chỗ code có thể
> tràn số.
>
> **Cần biết trước:** `int`, `double`, `decimal` (bài trước).

## Thử ngay: phép chia làm hỏng báo cáo

```csharp
int thanhCong = 47, tong = 100;

Console.WriteLine(thanhCong / tong);
Console.WriteLine(thanhCong / tong * 100);
Console.WriteLine(thanhCong * 100 / tong);
Console.WriteLine((double)thanhCong / tong);
```

**Đoán trước khi chạy:** bốn dòng này in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0
0
47
0.47
```

Chia hai `int` là **chia lấy nguyên**: `47 / 100` bằng `0`, nhân bao nhiêu
cũng vẫn là `0`. Đổi thứ tự phép tính hoặc ép một vế sang số thực thì mới ra
kết quả mong đợi.

</details>

Quy tắc: kết quả mang kiểu của **vế rộng hơn**. `int / int` ra `int`;
`int / double` ra `double`.

## Ép kiểu ngầm định và tường minh

```csharp
int nho = 100;
long to = nho;             // implicit — không mất gì

double gia = 19.99;
int catCut = (int)gia;     // explicit — 19, CẮT phần lẻ
int lamTron = (int)Math.Round(gia);   // 20
```

Compiler tự chuyển khi chắc chắn không mất dữ liệu (**implicit conversion**).
Ngược lại bạn phải tự ép (**explicit cast**) và tự chịu trách nhiệm: `(int)`
cắt cụt chứ không làm tròn.

- `Math.Round(x)` — làm tròn về số gần nhất.
- `Math.Floor(x)` / `Math.Ceiling(x)` — làm tròn xuống / lên.

## Tràn số

```csharp
int max = int.MaxValue;
int tran = max + 1;   // -2147483648, âm thầm quay vòng

checked
{
    int no = max + 1;      // ném OverflowException
}
```

Mặc định C# **không** báo tràn. Với id tự tăng, số tiền cộng dồn hay số lượng
tồn kho, hãy dùng kiểu đủ rộng (`long`, `decimal`) chứ đừng trông vào may mắn.

## Đổi chuỗi sang số

```csharp
int ok = int.Parse("42");
int loi = int.Parse("bốn hai");   // FormatException

if (int.TryParse(input, out int so))
{
    Console.WriteLine(so * 2);
}
```

`Parse` chỉ dành cho chuỗi mà bạn tự sinh ra và chắc chắn đúng định dạng. Dữ
liệu từ người dùng, từ query string, từ file CSV thì luôn `TryParse`.

## Toán tử logic ngắn mạch

```csharp
if (user != null && user.IsActive) { }
bool ca2 = Kiem(a) & Kiem(b);   // chạy CẢ HAI vế

int port = cauHinh ?? 8080;     // null thì lấy 8080
ten ??= "Khách";                 // gán khi đang null
```

`&&` và `||` là **short-circuit**: vế phải chỉ chạy khi cần, nhờ vậy mẫu
`x != null && x.Prop` mới an toàn. `&` và `|` chạy hết cả hai vế — hiếm khi
bạn muốn thế.

## Dấu hiệu trong code của bạn

- Phép chia giữa hai biến `int` mà kết quả gán vào `double`/`decimal` → phần lẻ đã mất trước khi gán.
- `(int)` đứng trước một biến tiền hoặc biến tỉ lệ → đang cắt cụt, kiểm tra xem có định làm tròn không.
- `int.Parse` nhận dữ liệu từ request, file, hay biến môi trường → đổi sang `TryParse`.
- Cộng dồn vào một biến `int` trong vòng lặp chạy rất nhiều lần → cân nhắc `long`.

## Ghi nhớ

- Kết quả mang kiểu của vế rộng hơn; `int / int` luôn ra `int`.
- `(int)x` cắt cụt, `Math.Round(x)` mới làm tròn.
- C# không báo tràn số trừ khi bạn bọc trong `checked` — chọn kiểu đủ rộng ngay từ đầu.
- Dữ liệu ngoài vào thì `TryParse`, không `Parse`.
- So sánh chuỗi bỏ qua hoa thường: `string.Equals(a, b, StringComparison.OrdinalIgnoreCase)`.

## Bước tiếp theo

Bài sau — **Rẽ nhánh và vòng lặp** — dùng chính các toán tử này để điều khiển
luồng chạy, kèm `switch expression` mà code C# hiện đại dùng khắp nơi.

```quiz
[
  {
    "prompt": "tiLe bằng bao nhiêu?",
    "code": "int daXong = 47, tongSo = 100;\ndouble tiLe = daXong / tongSo;",
    "options": ["0.47", "0", "47", "Lỗi compile"],
    "answer": 2,
    "explain": "Phép chia thực hiện trước khi gán, mà hai vế đều là int nên ra 0. Ép kiểu một vế: (double)daXong / tongSo."
  },
  {
    "prompt": "x bằng mấy?",
    "code": "decimal gia = 19.99m;\nint x = (int)gia;",
    "options": ["20", "19", "19.99", "Lỗi compile vì phải dùng Math.Round"],
    "answer": 2,
    "explain": "Ép kiểu tường minh cắt cụt phần thập phân. Muốn 20 thì (int)Math.Round(gia)."
  },
  {
    "prompt": "Biến đếm kiểu int cộng dồn vượt quá int.MaxValue. Mặc định chuyện gì xảy ra?",
    "options": [
      "Ném OverflowException",
      "Giá trị quay vòng thành số âm, không báo gì",
      "Tự chuyển sang long",
      "Chương trình dừng"
    ],
    "answer": 2,
    "explain": "C# không kiểm tra tràn số nếu không bọc checked. Số âm bất ngờ trong báo cáo thường là dấu vết của chuyện này."
  },
  {
    "prompt": "Vì sao if (user != null && user.IsActive) an toàn?",
    "options": [
      "Vì C# tự kiểm tra null cho mọi phép truy cập",
      "Vì && ngắn mạch: user null thì vế phải không chạy",
      "Vì && kiểm tra cả hai vế rồi mới quyết định",
      "Không an toàn, phải dùng dấu &"
    ],
    "answer": 2,
    "explain": "&& chỉ chạy vế phải khi vế trái đúng. Đổi sang & là cả hai vế đều chạy và bạn nhận NullReferenceException."
  }
]
```
