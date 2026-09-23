---
title: Toán tử và ép kiểu
minutes: 11
---

Báo cáo tỉ lệ đơn thành công hiện `0%`. Suốt cả tuần.

Bạn mở database kiểm tra: 47 đơn thành công trên 100. Công thức trong code
cũng đúng, chia số này cho số kia. Sai ở chỗ không ai ngờ — cả hai số đều là
`int`.

> **Học xong bài này bạn sẽ:** nhìn một biểu thức và biết kết quả ra kiểu gì;
> chọn đúng giữa ép kiểu, làm tròn và `TryParse`; nhận ra chỗ code có thể tràn
> số.
>
> **Cần biết trước:** `int`, `double`, `decimal` (bài trước).

## Thử ngay: phép chia làm hỏng báo cáo

Báo cáo kia gói lại được trong bốn dòng. Chạy thử trước, đọc giải thích sau.

```csharp
int success = 47, total = 100;

Console.WriteLine(success / total);
Console.WriteLine(success / total * 100);
Console.WriteLine(success * 100 / total);
Console.WriteLine((double)success / total);
```

**Đoán trước khi chạy:** bốn dòng in ra gì?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0
0
47
0.47
```

Chia hai `int` là **chia lấy nguyên**. `47 / 100` bằng `0`, và nhân bao nhiêu
cũng vẫn là `0`. Dòng thứ ba đổi thứ tự phép tính nên cứu được, dòng thứ tư ép
một vế sang số thực.

Cả tuần không ai phát hiện, vì `0%` trông vẫn như một con số hợp lệ.

</details>

## Kết quả mang kiểu của vế rộng hơn

Một phép chia có thể ra bốn kết quả khác nhau, tuỳ kiểu của hai vế.

| Biểu thức | Kết quả | Kiểu |
|---|---|---|
| `7 / 2` | `3` | `int` |
| `7 / 2.0` | `3.5` | `double` |
| `7 % 2` | `1` | `int` |
| `7m / 2` | `3.5` | `decimal` |

Quy tắc chỉ có một câu: chỉ cần **một vế** là số thực, kết quả thành số thực.
Còn hai `int` gặp nhau thì phần lẻ bị vứt. Không làm tròn, không cảnh báo.

## Compiler tự ép khi không mất gì, còn lại là việc của bạn

Đi từ kiểu hẹp sang kiểu rộng thì compiler làm giúp. Ngược lại thì nó bắt bạn
nói rõ, vì chiều đó có mất dữ liệu.

```csharp
int small = 100;
long big = small;          // implicit — không mất gì

double price = 19.99;
int cut = (int)price;      // explicit — 19, CẮT phần lẻ
int rounded = (int)Math.Round(price);   // 20
```

Compiler tự chuyển khi chắc chắn không mất dữ liệu, gọi là **implicit
conversion**. Ngược lại thì bạn phải tự ép, và tự chịu trách nhiệm.

| Muốn | Dùng | `19.99` ra |
|---|---|---|
| Cắt phần lẻ | `(int)x` | `19` |
| Làm tròn gần nhất | `Math.Round(x)` | `20` |
| Làm tròn xuống | `Math.Floor(x)` | `19` |
| Làm tròn lên | `Math.Ceiling(x)` | `20` |

## C# không báo tràn số, trừ khi bạn yêu cầu

Cắt phần lẻ thì còn thấy được. Tràn số thì im lặng hoàn toàn.

```csharp
int max = int.MaxValue;
int over = max + 1;   // -2147483648, âm thầm quay vòng

checked
{
    int boom = max + 1;   // ném OverflowException
}
```

Số tràn thì quay vòng thành số âm. Không lỗi, không cảnh báo, chỉ còn một con
số vô lý nằm trong báo cáo tháng sau.

Với id tự tăng, tiền cộng dồn hay số lượng tồn kho, hãy chọn kiểu đủ rộng ngay
từ đầu: `long` hoặc `decimal`.

## Parse ném lỗi, TryParse thì không

Đổi số sang số là chuyện của compiler. Đổi chuỗi sang số thì phải tự lo, vì
chuỗi có thể là bất cứ thứ gì.

```csharp
int ok = int.Parse("42");
int fail = int.Parse("bốn hai");   // FormatException

if (int.TryParse(input, out int value))
{
    Console.WriteLine(value * 2);
}
```

| Hàm | Chuỗi sai thì | Dùng cho |
|---|---|---|
| `int.Parse` | ném `FormatException` | chuỗi bạn tự sinh ra |
| `int.TryParse` | trả `false` | dữ liệu từ người dùng, file, API |

Query string, form, CSV đều là dữ liệu ngoài. Ở đó luôn dùng `TryParse`.

## && và || chỉ chạy vế phải khi cần

Còn một toán tử nữa hay bị gõ thiếu một ký tự, và hậu quả thì không nhỏ.

```csharp
if (user != null && user.IsActive) { }

// Bớt một ký tự: cả hai vế cùng chạy
if (user != null & user.IsActive) { }
```

`&&` và `||` là **short-circuit**: vế trái sai thì vế phải không chạy. Nhờ vậy
mẫu `x != null && x.Prop` mới an toàn.

Dòng thứ hai thì `user.IsActive` vẫn chạy dù `user` đang null, nên bạn nhận
`NullReferenceException` ngay tại chỗ `if`. Hiếm khi ai muốn thế.

## Dấu hiệu trong code của bạn

- Phép chia giữa hai biến `int` mà kết quả gán vào `double` hay `decimal` → phần lẻ đã mất trước khi gán.
- `(int)` đứng trước biến tiền hoặc biến tỉ lệ → đang cắt cụt, kiểm tra xem có định làm tròn không.
- `int.Parse` nhận dữ liệu từ request, file hay biến môi trường → đổi sang `TryParse`.
- Cộng dồn vào biến `int` trong vòng lặp chạy rất nhiều lần → cân nhắc `long`.

## Ghi nhớ

- Một vế là số thực thì kết quả là số thực; hai `int` thì phần lẻ bị vứt.
- `(int)x` cắt cụt, `Math.Round(x)` mới làm tròn.
- Tràn số im lặng, trừ khi bọc trong `checked`. Chọn kiểu đủ rộng từ đầu.
- Dữ liệu ngoài vào thì `TryParse`, không `Parse`.
- `&&` ngắn mạch, `&` thì không.

## Bước tiếp theo

Biểu thức đã tính đúng. Giờ tới lúc dùng nó để rẽ hướng.

Bài sau, **Rẽ nhánh và vòng lặp**, mở bằng một job dọn dữ liệu chạy đêm. Chạy
thử hai đơn thì êm, gặp dữ liệu thật thì sập ngay vòng lặp đầu tiên.

```quiz
[
  {
    "prompt": "rate bằng bao nhiêu?",
    "code": "int success = 47, total = 100;\ndouble rate = success / total;",
    "options": [
      "0",
      "0.47",
      "47",
      "Lỗi compile"
    ],
    "answer": 1,
    "explain": "Phép chia thực hiện trước khi gán, mà hai vế đều là int nên ra 0. Ép kiểu một vế: (double)success / total."
  },
  {
    "prompt": "x bằng mấy?",
    "code": "decimal price = 19.99m;\nint x = (int)price;",
    "options": [
      "20",
      "19.99",
      "19",
      "Lỗi compile vì phải dùng Math.Round"
    ],
    "answer": 3,
    "explain": "Ép kiểu tường minh cắt cụt phần thập phân. Muốn 20 thì (int)Math.Round(price)."
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
      "Không an toàn, phải dùng dấu &",
      "Vì && kiểm tra cả hai vế rồi mới quyết định",
      "Vì && ngắn mạch: user null thì vế phải không chạy"
    ],
    "answer": 4,
    "explain": "&& chỉ chạy vế phải khi vế trái đúng. Đổi sang & là cả hai vế đều chạy và bạn nhận NullReferenceException."
  }
]
```
