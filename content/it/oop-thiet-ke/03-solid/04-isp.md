---
title: ISP — Interface nhỏ, đúng việc
minutes: 5
---

Interface `IOrderStore` có bốn method: đọc, lưu, xoá, xuất PDF. Trang báo cáo
chỉ cần đọc, nhưng class của nó vẫn phải implement cả bốn, và ba method còn lại
thành `throw new NotImplementedException()`. ISP tránh việc ép class nhận những
thứ nó không dùng.

## Khái niệm

✂️ **ISP (Interface Segregation Principle)**: không bắt class phụ thuộc vào những method nó không dùng, nên chia interface lớn thành nhiều interface nhỏ theo từng việc.

## Ví dụ

```csharp
var repo = new OrderRepository();
var page = new ReportPage(repo);
page.Show();

interface IOrderReader
{
    string Get(int id);
}

interface IOrderWriter
{
    void Save(string order);
}

class OrderRepository : IOrderReader, IOrderWriter
{
    public string Get(int id) => $"Đơn #{id}";
    public void Save(string order) =>
        Console.WriteLine($"Lưu {order}");
}

class ReportPage
{
    private readonly IOrderReader _reader;

    public ReportPage(IOrderReader reader)
    {
        _reader = reader;
    }

    public void Show() =>
        Console.WriteLine(_reader.Get(1));
}
```

- `IOrderReader` chỉ đọc, `IOrderWriter` chỉ ghi. Mỗi interface một việc.
- `OrderRepository` làm được cả hai nên implement cả hai.
- `ReportPage` chỉ cần đọc nên chỉ nhận `IOrderReader`. Sửa phần ghi không
  ảnh hưởng tới nó.

## Thử ngay

Chép ví dụ trên vào `Program.cs`, thêm dòng sau vào cuối phần gọi ở đầu file:

```csharp
IOrderWriter writer = repo;
writer.Save("Đơn #2");
```

**Đoán trước khi chạy:** cùng một object `repo`, vừa truyền cho
`ReportPage` như một `IOrderReader`, vừa gán vào biến `IOrderWriter`. Có
được không?

<details>
<summary>Xem kết quả</summary>

```text
Đơn #1
Lưu Đơn #2
```

Được. Một object implement nhiều interface thì dùng được ở vai nào cũng được.
Mỗi nơi chỉ thấy đúng phần nó cần.

</details>

## Lỗi hay gặp

**Interface to, class buộc phải viết cho có.** `NotImplementedException` là
dấu hiệu rõ nhất.

```csharp
// SAI — trang báo cáo bị ép implement cả ghi và xoá
interface IOrderStore
{
    string Get(int id);
    void Save(string order);
    void Delete(int id);
}

class ReportSource : IOrderStore
{
    public string Get(int id) => $"Đơn #{id}";
    public void Save(string order) =>
        throw new NotImplementedException();
    public void Delete(int id) =>
        throw new NotImplementedException();
}
```

Cách sửa: tách thành `IOrderReader` và `IOrderWriter`, để `ReportSource` chỉ
implement `IOrderReader`.

## Tóm tắt

- ISP: không bắt class phụ thuộc vào method nó không dùng.
- Chia interface lớn thành nhiều interface nhỏ theo từng việc.
- Một class implement được nhiều interface nhỏ khi nó làm được nhiều việc.
- `NotImplementedException` trong class implement là dấu hiệu interface quá
  to.

```quiz
[
  {
    "prompt": "IPrinter có Print(), Scan(), Fax(). Máy in rẻ chỉ in được, nên Scan và Fax ném NotImplementedException. Theo ISP nên làm gì?",
    "options": [
      "Giữ nguyên, ném exception là đủ",
      "Bỏ class máy in rẻ",
      "Tách thành IPrint, IScan, IFax",
      "Thêm method CanScan() vào IPrinter"
    ],
    "answer": 3,
    "explain": "Máy in rẻ chỉ nên implement IPrint. Máy đa năng thì implement cả ba interface nhỏ."
  },
  {
    "prompt": "Class Dashboard chỉ đọc dữ liệu đơn hàng. Constructor của nó nên nhận kiểu nào?",
    "options": [
      "IOrderReader",
      "IOrderWriter",
      "IOrderStore có đủ đọc, ghi, xoá",
      "OrderRepository"
    ],
    "answer": 1,
    "explain": "Dashboard chỉ cần đọc, nên chỉ phụ thuộc vào interface đọc. Nhận thứ lớn hơn là tự kéo thêm phụ thuộc không cần."
  },
  {
    "prompt": "Một class implement cả IOrderReader và IOrderWriter. Điều nào đúng?",
    "options": [
      "Không hợp lệ, class chỉ implement được một interface",
      "Phải chọn một trong hai",
      "Chỉ hợp lệ nếu hai interface có cùng method",
      "Hợp lệ, dùng được ở cả hai vai"
    ],
    "answer": 4,
    "explain": "Một class implement được nhiều interface. Nơi cần đọc thấy nó là IOrderReader, nơi cần ghi thấy nó là IOrderWriter."
  }
]
```
