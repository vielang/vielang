---
title: SRP là một lý do để đổi
minutes: 11
---

`OrderService.cs` dài hai nghìn dòng. Đầu file có `SmtpClient`, `PdfWriter`,
`AppDbContext` và một `HttpClient`.

Sprint nào cũng có ba người sửa nó. Tuần trước kế toán đổi cách làm tròn thuế,
và bản PDF gửi khách hỏng theo.

Không ai viết file này cho to. Nó to lên vì mỗi lần thêm việc, đây là chỗ gần
nhất để nhét vào.

> **Học xong bài này bạn sẽ:** đọc SRP theo nghĩa gốc thay vì nghĩa "một class
> làm một việc"; đếm được số lý do đổi của một file; và tách nó ra mà không sinh
> ra hai chục class rỗng.
>
> **Cần biết trước:** `class`, interface, composition (chương trước).

## SRP nói về lý do đổi, không nói về số việc

Câu gốc của Robert Martin: **một class chỉ nên có một lý do để thay đổi**.

Người ta hay đọc thành "một class làm một việc", rồi tách mọi thứ thành class
một method. Đó là cách hiểu khác, và nó dẫn tới một loại mớ khác.

| Cách đọc | Dẫn tới |
|---|---|
| "Một class làm một việc" | hai chục class một method, logic tán khắp nơi |
| "Một class một **lý do đổi**" | class gom những thứ luôn đổi cùng nhau |

Lý do đổi thường đi cùng một **người yêu cầu**. Đây là bốn lý do đang cùng nằm
trong `OrderService`:

| Ai yêu cầu đổi | Họ đổi gì |
|---|---|
| Kế toán | cách làm tròn thuế |
| Marketing | công thức khuyến mãi |
| Đối tác vận chuyển | format file gửi đi |
| Nhóm hạ tầng | đổi SMTP sang hàng đợi |

Bốn người, bốn nhịp thay đổi, một file. Đó là định nghĩa của rắc rối.

## Thử ngay: đếm số lý do đổi bằng git log

Không cần đọc hai nghìn dòng. Lịch sử commit đã nói hộ.

Mở terminal ở repo của bạn, chọn một file to nhất, rồi chạy lệnh này.

```bash
git log --format="%an" -- src/OrderService.cs \
  | sort | uniq -c | sort -rn
```

**Đoán trước khi chạy:** bao nhiêu người từng sửa file đó, và người nhiều nhất
chiếm bao nhiêu phần?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

Trên một repo thật, kết quả hay trông như dưới đây.

```text
     41 nguoi-a
     28 nguoi-b
     19 nguoi-c
      6 nguoi-d
```

Bốn người sửa gần như đều nhau. Nếu họ thuộc bốn nhóm khác nhau thì file ấy
đang có bốn lý do để đổi.

Còn lệnh dưới đây cho thấy file nào hay bị sửa cùng lúc với nó. Đó thường là
những mảnh cùng một trách nhiệm đã bị chia sai chỗ.

```bash
git log --format="%h" --name-only -- src/OrderService.cs \
  | sort | uniq -c | sort -rn | head -10
```

</details>

Hai lệnh này không nói cho bạn phải tách thế nào. Chúng chỉ chỉ đúng chỗ đáng
xem trước.

## Tách theo lý do đổi, không tách theo tầng kỹ thuật

Biết có bốn lý do rồi, còn phải quyết định đường cắt nằm ở đâu.

```csharp
// SAI — ba lý do đổi trong một class
class OrderService
{
    public decimal Tax(Order o) =>
        o.Total * 0.1m;

    public string ToCsv(Order o) =>
        $"{o.Id};{o.Total}";

    public void Send(string csv) =>
        Console.WriteLine(csv);
}
```

Đổi cách làm tròn thuế thì phải mở file này. Đổi dấu phân cách trong CSV cũng
mở đúng file này. Hai việc chẳng liên quan gì tới nhau.

```csharp
// ĐÚNG — mỗi class một lý do đổi
class TaxCalculator
{
    public decimal Tax(Order o) => o.Total * 0.1m;
}

class OrderCsvFormatter
{
    public string Format(Order o) =>
        $"{o.Id};{o.Total}";
}

class PartnerFeed(OrderCsvFormatter formatter)
{
    public void Send(Order o) =>
        Console.WriteLine(formatter.Format(o));
}
```

Ba class, ba lý do đổi, và mỗi cái test được riêng.

Để ý phần tách **không** theo tầng kỹ thuật. Không có `OrderHelper`, không có
`OrderUtils`. Tên mỗi class nói đúng việc nó chịu trách nhiệm.

## Tách quá tay cũng là một loại mớ

Nguyên tắc này bị lạm dụng nhiều hơn bị bỏ qua.

```csharp
// SAI — tách tới mức không còn gì để đọc
class OrderIdGetter
{
    public int Get(Order o) => o.Id;
}
```

Một class chỉ đọc một property không phải SRP. Nó là một tầng gián tiếp không
mua được gì cả.

| Dấu hiệu | Nên |
|---|---|
| Hai nhóm người sửa một file vì hai lý do | tách |
| Một method dài mà cả nó chỉ có một lý do đổi | tách **method**, giữ class |
| Class một method, chỉ gói một dòng | gộp lại |
| Ba class luôn phải sửa cùng lúc | chúng là một trách nhiệm, gộp lại |

Dòng cuối bảng là phép thử ngược rất hiệu quả. Nếu sửa tính năng nào cũng phải
mở đúng ba file ấy, thì ba file ấy vốn là một.

## Method dài chưa chắc là vi phạm SRP

Hai thứ hay bị gộp làm một, mà chúng khác nhau.

Một method bốn mươi dòng làm đúng một việc thì chỉ cần tách method cho dễ đọc.
Class vẫn nguyên.

Ngược lại, một class toàn method ba dòng vẫn vi phạm SRP. Chỉ cần những method
ấy phục vụ bốn nhóm người khác nhau.

Số dòng là dấu hiệu, không phải tiêu chí. Tiêu chí là **ai yêu cầu đổi**.

## Dấu hiệu trong code của bạn

- `git log` một file cho thấy bốn nhóm khác nhau cùng sửa → file đó có bốn lý do đổi.
- Tên class có `Manager`, `Helper`, `Utils`, `Processor` → cái tên không nói được trách nhiệm, vì trách nhiệm chưa rõ.
- Đầu file `using` cả SMTP, PDF, `DbContext` và HTTP → bốn hướng thay đổi đi qua một chỗ.
- Sửa một tính năng nào cũng phải mở đúng ba file giống nhau → ba file ấy là một trách nhiệm bị chia sai.
- Class chỉ có một method gói một dòng → tầng gián tiếp không mua được gì.

## Ghi nhớ

- SRP là **một lý do để đổi**, không phải "một class một việc".
- Lý do đổi thường gắn với một nhóm người yêu cầu.
- `git log` đếm hộ bạn số lý do đổi của một file.
- Tách theo trách nhiệm nghiệp vụ, đừng tách theo tầng kỹ thuật.
- Ba class luôn sửa cùng lúc thì vốn là một; gộp lại cũng là áp dụng SRP.

## Bước tiếp theo

Tách xong rồi thì câu hỏi kế là thêm tính năng mới vào đâu.

Bài sau, **OCP và LSP**, mở bằng một `switch` tính phí ship. Thêm một hãng vận
chuyển là sửa đúng cái method mà năm mươi test đang phụ thuộc vào.

```quiz
[
  {
    "prompt": "Câu nào diễn đạt SRP đúng theo nghĩa gốc?",
    "options": [
      "Một class chỉ nên có một lý do để thay đổi",
      "Một class chỉ nên có một method public",
      "Một class không nên dài quá hai trăm dòng",
      "Một class chỉ nên làm đúng một việc"
    ],
    "answer": 1,
    "explain": "Bản gốc nói về lý do đổi, tức về nguồn yêu cầu thay đổi. Đọc thành \"một việc\" hay đếm số dòng đều dẫn tới tách vụn, mà vẫn không giảm được chỗ va nhau."
  },
  {
    "prompt": "Trong repo của bạn có ba class mà sửa tính năng nào cũng phải mở cả ba. Điều đó nói lên gì?",
    "options": [
      "Thiết kế tốt, trách nhiệm đã được tách rõ",
      "Cần thêm một class thứ tư để phối hợp ba class kia",
      "Ba class đó vốn là một trách nhiệm, đang bị chia sai chỗ",
      "Cần viết thêm test cho cả ba"
    ],
    "answer": 3,
    "explain": "SRP gom những thứ đổi cùng nhau. Luôn đổi cùng nhau nghĩa là chúng có cùng một lý do đổi, nên gộp lại mới đúng nguyên tắc."
  },
  {
    "prompt": "Một method dài bốn mươi dòng, nhưng cả bốn mươi dòng chỉ phục vụ một yêu cầu của kế toán. Nên làm gì?",
    "options": [
      "Tách thành bốn class, mỗi class mười dòng",
      "Tách method cho dễ đọc, nhưng giữ nguyên class",
      "Để nguyên, vì SRP chỉ nói về class",
      "Chuyển sang abstract class rồi chia cho lớp con"
    ],
    "answer": 2,
    "explain": "Method dài là chuyện đọc hiểu, giải quyết bằng cách tách method. Còn SRP nói về lý do đổi, mà ở đây chỉ có một lý do nên không cần tách class."
  },
  {
    "prompt": "Bạn thấy class OrderIdGetter với đúng một method trả về o.Id. Nhận xét nào đúng?",
    "options": [
      "Đúng chuẩn SRP, một class một việc",
      "Nên đổi tên thành OrderIdService cho nhất quán",
      "Nên thêm interface IOrderIdGetter cho dễ test",
      "Đây là tầng gián tiếp không mua được gì, nên gộp lại"
    ],
    "answer": 4,
    "explain": "Không có lý do đổi nào riêng cho việc đọc một property. Thêm class hay thêm interface ở đây chỉ làm người đọc phải nhảy thêm một file."
  }
]
```
