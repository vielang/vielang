---
title: Exception đúng cách
minutes: 11
---

Production lỗi, bạn mở log lên và thấy đúng một dòng: `Object reference not set
to an instance of an object`. Không stack trace, không id đơn hàng, không biết
file nào dòng nào. Ai đó đã `catch` rồi ghi `ex.Message` — và ném đi phần duy
nhất có ích.

> **Học xong bài này bạn sẽ:** biết khi nào nên `catch` và khi nào để lỗi bay
> lên; giữ nguyên stack trace khi ném lại; viết exception riêng cho lỗi nghiệp
> vụ; và không bao giờ nuốt lỗi nữa.
>
> **Cần biết trước:** method, `throw` ở mức đã thấy trong các bài trước.

## try, catch, finally

```csharp
try
{
    var text = File.ReadAllText(duongDan);
    Xuly(text);
}
catch (FileNotFoundException ex)
{
    logger.LogWarning(ex, "Không thấy {F}", duongDan);
}
catch (IOException ex)
{
    logger.LogError(ex, "Lỗi đọc {F}", duongDan);
    throw;
}
finally
{
    dongHo.Stop();   // luôn chạy, kể cả khi có lỗi
}
```

Bắt **kiểu cụ thể trước**, kiểu chung sau — compiler đọc từ trên xuống. Và chú
ý: khối `catch` đầu ghi log rồi thôi, khối thứ hai ghi log rồi `throw;` để lỗi
tiếp tục bay lên.

## Thử ngay: throw hay throw ex

```csharp
void Trong() =>
    throw new InvalidOperationException("hỏng");

void Giua()
{
    try { Trong(); }
    catch (Exception ex) { throw ex; }
}

try { Giua(); }
catch (Exception ex)
{
    Console.WriteLine(ex.StackTrace);
}
```

**Đoán trước khi chạy:** stack trace in ra có còn tên method `Trong` — nơi lỗi
thật sự xảy ra — hay không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
   at Program.<<Main>$>g__Giua|0_1()
   at Program.<Main>$(String[] args)
```

**Mất hẳn `Trong`.** `throw ex;` ném lại đúng object đó nhưng **ghi đè stack
trace** từ chỗ ném mới, nên nơi lỗi thật sự xảy ra biến mất khỏi log.

Đổi `throw ex;` thành `throw;` rồi chạy lại: dòng `at Program...Trong` xuất
hiện trở lại ở đầu danh sách.

</details>

Quy tắc: **luôn `throw;`**, không bao giờ `throw ex;`. Muốn thêm ngữ cảnh thì
bọc lại và giữ lỗi gốc làm inner exception:

```csharp
try { db.SaveChanges(); }
catch (SqlException ex)
{
    throw new DonHangException(
        $"Không lưu được đơn {id}", ex);
}
```

## Đừng nuốt lỗi

```csharp
// SAI — lỗi biến mất, không ai biết gì
try { Luu(don); }
catch { }

// SAI — giữ chữ, vứt ngữ cảnh
try { Luu(don); }
catch (Exception ex)
{
    logger.LogError(ex.Message);
}
```

```csharp
// ĐÚNG — truyền cả object exception cho logger
try { Luu(don); }
catch (Exception ex)
{
    logger.LogError(ex, "Lỗi lưu đơn {Id}", don.Id);
    throw;
}
```

Truyền `ex` làm tham số đầu tiên thì logger ghi cả stack trace lẫn inner
exception. `LogError(ex.Message)` chỉ ghi một dòng chữ — đúng tình huống ở đầu
bài.

## Khi nào nên catch

Chỉ bắt khi bạn **làm được gì đó** với lỗi:

- Thử lại (gọi mạng hỏng, deadlock database).
- Chuyển sang phương án khác (cache hỏng thì đọc thẳng database).
- Bổ sung ngữ cảnh rồi ném tiếp.
- Ở **biên hệ thống**: một chỗ duy nhất biến lỗi thành phản hồi HTTP.

Còn lại thì để lỗi bay lên. Trong ASP.NET Core, biên đó là một middleware duy
nhất, không phải `try/catch` rải trong từng controller.

## Exception của riêng bạn

```csharp
public class DonHangException : Exception
{
    public DonHangException(string message)
        : base(message) { }

    public DonHangException(
        string message, Exception inner)
        : base(message, inner) { }
}
```

Viết exception riêng cho **lỗi nghiệp vụ** ("đơn đã huỷ không sửa được"), để
tầng trên phân biệt được với lỗi kỹ thuật. Đừng đặt tên chung chung như
`AppException` cho mọi thứ.

## Đừng dùng exception làm luồng chạy

```csharp
// SAI — chậm và giấu ý định
try { so = int.Parse(input); }
catch { so = 0; }

// ĐÚNG
if (!int.TryParse(input, out var so2)) so2 = 0;
```

Ném và bắt exception tốn kém hơn một phép `if` rất nhiều, nên chỉ dành cho
tình huống **bất thường**, không dành cho trường hợp bạn biết chắc sẽ xảy ra.

## Dấu hiệu trong code của bạn

- `catch { }` hoặc `catch (Exception) { }` với thân rỗng → lỗi biến mất, không ai biết gì cả.
- `throw ex;` → đang xoá stack trace, đổi thành `throw;`.
- `logger.LogError(ex.Message)` → mất stack trace và inner exception; truyền cả `ex`.
- `try/catch` bao quanh mỗi action trong controller → gom về một middleware.
- `catch` để đổi lỗi thành `return null` → người gọi sẽ gặp `NullReferenceException` ở chỗ khác, khó lần hơn nhiều.

## Ghi nhớ

- `throw;` giữ stack trace, `throw ex;` xoá mất nó.
- Log thì truyền cả object `ex`, đừng chỉ truyền `ex.Message`.
- Chỉ `catch` khi làm được gì đó; còn lại để lỗi bay lên biên hệ thống.
- Exception dành cho tình huống bất thường, không dành cho luồng chạy bình thường.
- `finally` luôn chạy — chỗ để dọn dẹp.

## Bước tiếp theo

Bài sau — **IDisposable và using** — dọn những thứ GC không dọn giúp: file,
connection, socket. Và `finally` vừa gặp ở đây chính là thứ `using` viết gọn
lại.

```quiz
[
  {
    "prompt": "Đoạn này làm gì với stack trace của lỗi gốc?",
    "code": "try { Trong(); }\ncatch (Exception ex) { throw ex; }",
    "options": [
      "Giữ nguyên, ném lại y như cũ",
      "Ghi đè stack trace từ dòng throw, mất nơi lỗi thật sự xảy ra",
      "Xoá luôn cả message",
      "Bọc thành inner exception"
    ],
    "answer": 2,
    "explain": "throw ex; đặt lại điểm bắt đầu của stack trace. Dùng throw; để ném tiếp mà giữ nguyên, hoặc bọc lỗi gốc làm inner exception."
  },
  {
    "prompt": "Log production chỉ có một dòng message, không stack trace. Nguyên nhân thường gặp nhất?",
    "options": [
      "Logger cấu hình sai mức log",
      "Code ghi logger.LogError(ex.Message) thay vì logger.LogError(ex, ...)",
      "Exception không có stack trace",
      "Production tắt stack trace cho nhẹ"
    ],
    "answer": 2,
    "explain": "Truyền ex làm tham số đầu thì logger ghi cả stack trace lẫn inner exception. Truyền ex.Message là tự vứt đi phần có ích nhất."
  },
  {
    "prompt": "Trong ASP.NET Core, nên đặt try/catch biến lỗi thành phản hồi HTTP ở đâu?",
    "options": [
      "Trong mỗi action của controller",
      "Trong một middleware duy nhất ở biên",
      "Trong từng method của service",
      "Trong repository, gần database nhất"
    ],
    "answer": 2,
    "explain": "Một chỗ duy nhất biết cách biến lỗi thành mã HTTP. Rải try/catch khắp nơi thì mỗi chỗ xử lý một kiểu và dễ nuốt lỗi."
  },
  {
    "prompt": "Người dùng nhập chuỗi vào ô số. Cách nào đúng?",
    "options": [
      "try { int.Parse(s) } catch { dùng 0 }",
      "if (!int.TryParse(s, out var n)) n = 0;",
      "Ném ArgumentException rồi bắt ở controller",
      "Cả A và B như nhau"
    ],
    "answer": 2,
    "explain": "Người dùng gõ sai là chuyện bình thường, không phải tình huống bất thường. Exception tốn kém hơn một phép if rất nhiều."
  }
]
```
