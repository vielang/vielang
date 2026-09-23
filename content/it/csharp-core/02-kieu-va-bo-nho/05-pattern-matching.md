---
title: Pattern matching
minutes: 10
---

Bạn mở một file service trong dự án và thấy hai mươi dòng thế này: kiểm tra
kiểu, ép kiểu, gán vào biến mới, rồi mới dùng được. Cùng một việc đó, C# hiện
đại viết trong một dòng — và đọc code người khác thì bạn sẽ gặp dạng một dòng
nhiều hơn hẳn.

> **Học xong bài này bạn sẽ:** đọc được `switch expression` với property
> pattern trong code thật; thay chuỗi `if` ép kiểu bằng một biểu thức; biết
> pattern lồng tự xử lý null thế nào.
>
> **Cần biết trước:** `switch expression`, `record`, `enum`.

## Type pattern

```csharp
// Cách cũ
if (hinh is HinhTron)
{
    var tron = (HinhTron)hinh;
    Console.WriteLine(tron.BanKinh);
}

// Pattern matching: kiểm tra và gán một bước
if (hinh is HinhTron tron2)
    Console.WriteLine(tron2.BanKinh);

if (giaTri is not string chu)
    return;   // chu dùng được ở phần còn lại
```

## Thử ngay: property pattern

```csharp
record DiaChi(string ThanhPho);
record Don(decimal Tong, DiaChi? Noi);

decimal Phi(Don d) => d switch
{
    { Tong: > 1_000_000 } => 0,
    { Noi.ThanhPho: "Hà Nội" } => 15_000,
    _ => 30_000,
};

Console.WriteLine(Phi(new Don(2_000_000, null)));
Console.WriteLine(Phi(new Don(50_000, new("Hà Nội"))));
Console.WriteLine(Phi(new Don(50_000, null)));
```

**Đoán trước khi chạy:** ba dòng in ra gì? Chú ý dòng đầu và dòng cuối có
`Noi` là `null`.

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
0
15000
30000
```

Không dòng nào ném `NullReferenceException`. Pattern `{ Noi.ThanhPho: … }`
không khớp khi `Noi` là null — nó **rơi xuống nhánh dưới** thay vì nổ. Đây là
lý do pattern matching gọn hơn hẳn `if (d.Noi != null && d.Noi.ThanhPho == …)`.

</details>

## Relational và logical pattern

```csharp
string XepLoai(int diem) => diem switch
{
    < 0 or > 100 =>
        throw new ArgumentOutOfRangeException(),
    >= 80 => "Giỏi",
    >= 50 => "Khá",
    _ => "Trung bình",
};

bool laSo = c is >= '0' and <= '9';
```

`and`, `or`, `not` ghép các pattern lại. `x is not null` đọc xuôi hơn hẳn
`x != null`.

## List pattern

```csharp
var phan = dong.Split(':');

var mota = phan switch
{
    ["GET", var p] => $"Đọc {p}",
    ["POST", var p, ..] => $"Ghi {p}",
    [] => "Dòng trống",
    _ => "Không hiểu",
};
```

`..` là **slice pattern**: "còn lại bao nhiêu cũng được". Rất hợp khi phân
tích dòng log hay lệnh dạng chuỗi.

## Kết hợp với record

```csharp
abstract record ThanhToan;
record TienMat(decimal So) : ThanhToan;
record The(decimal So, string Duoi4) : ThanhToan;
record ChuyenKhoan(decimal So, string Bank) : ThanhToan;

string MoTa(ThanhToan t) => t switch
{
    TienMat { So: > 10_000_000 } =>
        "Tiền mặt, cần xác minh",
    TienMat m => $"Tiền mặt {m.So:N0}",
    The (var so, var duoi) =>
        $"Thẻ ****{duoi}, {so:N0}",
    ChuyenKhoan c => $"Chuyển khoản qua {c.Bank}",
};
```

`The (var so, var duoi)` là **positional pattern**, dùng được vì `record` tự
sinh sẵn `Deconstruct`. Kiểu cha là `abstract record` và các nhánh liệt kê đủ
thì compiler không còn cảnh báo thiếu nhánh.

## Dấu hiệu trong code của bạn

- Cặp `if (x is T)` rồi `(T)x` ngay dòng dưới → gộp thành `if (x is T t)`.
- Chuỗi `if` kiểm tra null rồi mới so property (`a != null && a.B == c`) → một property pattern là xong.
- `switch` dạng câu lệnh mà mỗi nhánh chỉ gán một giá trị → đổi sang `switch expression`.
- Chuỗi `if` phân loại theo kiểu con của một lớp cha → `switch expression` trên kiểu, và compiler sẽ nhắc khi có kiểu con mới.

## Ghi nhớ

- `is` vừa kiểm tra kiểu vừa gán biến — không còn ép kiểu hai lần.
- Pattern lồng tự xử lý null: không khớp thì rơi sang nhánh khác, không ném lỗi.
- `switch expression` trả về giá trị, mỗi nhánh là một biểu thức, không có `break`.
- `and`, `or`, `not` ghép pattern; `is not null` là cách viết chuẩn hiện nay.

## Bước tiếp theo

Hết chương **Kiểu và bộ nhớ**. Chương sau — **Collection và LINQ** — chọn đúng
cấu trúc dữ liệu cho từng việc, rồi xử lý chúng bằng LINQ; `switch expression`
và record vừa học sẽ đi cùng bạn suốt phần đó.

```quiz
[
  {
    "prompt": "Đoạn này in ra gì khi d.Noi là null?",
    "code": "decimal Phi(Don d) => d switch\n{\n    { Noi.ThanhPho: \"Hà Nội\" } => 15_000,\n    _ => 30_000,\n};",
    "options": [
      "Ném NullReferenceException",
      "15000",
      "30000",
      "Lỗi compile vì thiếu kiểm tra null"
    ],
    "answer": 3,
    "explain": "Pattern lồng không khớp khi một mắt xích là null — nó rơi xuống nhánh _ thay vì nổ. Đó là điểm mạnh so với chuỗi if kiểm tra null."
  },
  {
    "prompt": "Cách viết nào thay được hai dòng kiểm tra kiểu rồi ép kiểu?",
    "code": "if (hinh is HinhTron)\n{\n    var t = (HinhTron)hinh;\n}",
    "options": [
      "if (hinh as HinhTron)",
      "if (hinh is HinhTron t)",
      "if (hinh.GetType() == typeof(HinhTron))",
      "switch (hinh) { case HinhTron: break; }"
    ],
    "answer": 2,
    "explain": "Type pattern kiểm tra kiểu và gán biến trong một bước; biến t dùng được ngay trong thân if."
  },
  {
    "prompt": "Bạn thêm một record con mới kế thừa lớp cha abstract. Cách viết nào giúp compiler nhắc chỗ còn thiếu xử lý?",
    "options": [
      "Chuỗi if - else if theo kiểu",
      "switch expression trên kiểu, liệt kê đủ nhánh",
      "Dictionary ánh xạ kiểu sang hàm xử lý",
      "try - catch InvalidCastException"
    ],
    "answer": 2,
    "explain": "Compiler kiểm tra tính đầy đủ của switch expression và cảnh báo khi còn trường hợp chưa xử lý; if thì im lặng."
  },
  {
    "prompt": "phan là mảng [\"POST\", \"/orders\", \"1\"]. Pattern nào khớp?",
    "options": [
      "[\"POST\", var p]",
      "[\"POST\", var p, ..]",
      "[..]",
      "Cả B và C"
    ],
    "answer": 4,
    "explain": "[\"POST\", var p] đòi đúng 2 phần tử nên không khớp. [\"POST\", var p, ..] khớp vì .. nhận phần còn lại, và [..] khớp mọi mảng — nhánh nào viết trước thì thắng."
  }
]
```
