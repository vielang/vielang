---
title: Exception đúng cách
minutes: 11
---

Production lỗi lúc hai giờ sáng. Bạn mở log lên, và thấy đúng một dòng.

```text
Object reference not set to an instance of an object
```

Không stack trace. Không id đơn hàng. Không biết file nào, dòng nào.

Ai đó đã `catch` rồi ghi `ex.Message`, và ném đi mất phần duy nhất có ích.

> **Học xong bài này bạn sẽ:** biết khi nào nên `catch` và khi nào để lỗi bay
> lên; giữ nguyên stack trace khi ném lại; viết exception riêng cho lỗi nghiệp
> vụ; và không bao giờ nuốt lỗi nữa.
>
> **Cần biết trước:** method, `throw` ở mức đã thấy trong các bài trước.

## Bắt kiểu cụ thể trước, kiểu chung sau

```csharp
try
{
    var text = File.ReadAllText(path);
    Process(text);
}
catch (FileNotFoundException ex)
{
    logger.LogWarning(ex, "Không thấy {F}", path);
}
catch (IOException ex)
{
    logger.LogError(ex, "Lỗi đọc {F}", path);
    throw;
}
finally
{
    timer.Stop();   // luôn chạy, kể cả khi lỗi
}
```

Các khối `catch` được xét từ trên xuống. Đặt `IOException` lên trước thì khối
`FileNotFoundException` thành khối chết, và compiler chặn hẳn bằng lỗi
**CS0160** — vì khối cha đã bắt trọn phần của khối con.

Hai khối ở đây xử lý khác nhau. Khối đầu ghi log rồi thôi. Khối sau ghi log
xong còn `throw;` để lỗi tiếp tục bay lên trên.

## Thử ngay: throw ex xoá mất nơi lỗi xảy ra

```csharp
void Inner() =>
    throw new InvalidOperationException("hỏng");

void Middle()
{
    try { Inner(); }
    catch (Exception ex) { throw ex; }
}

try { Middle(); }
catch (Exception ex)
{
    Console.WriteLine(ex.StackTrace);
}
```

**Đoán trước khi chạy:** stack trace in ra còn tên `Inner`, nơi lỗi thật sự
xảy ra, hay không?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
   at Program.<<Main>$>g__Middle|0_1()
   at Program.<Main>$(String[] args)
```

`Inner` biến mất. `throw ex;` ném lại đúng object cũ, nhưng nó **ghi đè stack
trace** từ chỗ ném mới.

Đổi một chữ, bỏ `ex` đi, rồi chạy lại. Dòng `at Program...Inner` xuất hiện trở
lại ngay đầu danh sách.

</details>

## Ba cách ném lại, chỉ hai cách giữ được nơi lỗi

| Viết | Stack trace gốc | Dùng khi |
|---|---|---|
| `throw;` | giữ nguyên | ghi log xong, để lỗi bay tiếp |
| `throw new X(msg, ex)` | giữ trong inner | cần thêm ngữ cảnh nghiệp vụ |
| `throw ex;` | **mất** | không bao giờ |

```csharp
try { db.SaveChanges(); }
catch (SqlException ex)
{
    throw new OrderException(
        $"Không lưu được đơn {id}", ex);
}
```

Bọc lại thì lỗi gốc vẫn nằm nguyên trong `InnerException`. Bạn được cả hai:
ngữ cảnh nghiệp vụ ở lớp ngoài, chi tiết kỹ thuật ở lớp trong.

## Log phải nhận cả object exception, không chỉ message

```csharp
// SAI — lỗi biến mất, không ai biết gì
try { Save(order); }
catch { }
```

```csharp
// SAI — giữ chữ, vứt ngữ cảnh
try { Save(order); }
catch (Exception ex)
{
    logger.LogError(ex.Message);
}
```

```csharp
// ĐÚNG — truyền cả object exception
try { Save(order); }
catch (Exception ex)
{
    logger.LogError(ex, "Lỗi đơn {Id}", order.Id);
    throw;
}
```

Truyền `ex` làm tham số đầu tiên thì logger ghi cả stack trace lẫn inner
exception cho bạn.

`LogError(ex.Message)` chỉ để lại một dòng chữ. Đúng tình huống hai giờ sáng ở
đầu bài.

## Chỉ catch khi bạn làm được gì đó với lỗi

| Tình huống | Nên làm |
|---|---|
| Gọi mạng hỏng, deadlock database | bắt rồi thử lại |
| Cache hỏng | bắt, quay sang đọc thẳng database |
| Cần thêm id đơn hay tên file vào lỗi | bọc lại rồi ném tiếp |
| Biên hệ thống | biến lỗi thành phản hồi HTTP |
| Mọi trường hợp còn lại | **không bắt**, để lỗi bay lên |

Trong ASP.NET Core, biên ấy là một middleware duy nhất. Không phải `try/catch`
rải đều trong từng controller.

Một chỗ duy nhất biết cách biến lỗi thành mã HTTP thì dễ sửa và dễ test. Rải
ra khắp nơi thì mỗi chỗ xử lý một kiểu, và thế nào cũng có chỗ nuốt mất lỗi.

## Exception riêng tách lỗi nghiệp vụ khỏi lỗi kỹ thuật

```csharp
public class OrderException : Exception
{
    public OrderException(string message)
        : base(message) { }

    public OrderException(
        string message, Exception inner)
        : base(message, inner) { }
}
```

"Đơn đã huỷ thì không sửa được" là lỗi nghiệp vụ. `SqlException` là lỗi kỹ
thuật. Tầng trên cần phân biệt được hai thứ ấy.

Một cái trả 400 và nói cho người dùng biết vì sao. Cái kia trả 500 và gọi bạn
dậy lúc hai giờ sáng.

Đừng gom tất cả vào một `AppException` chung chung.

## Exception dành cho bất thường, không dành cho luồng chạy

```csharp
// SAI — chậm và giấu mất ý định
try { number = int.Parse(input); }
catch { number = 0; }
```

```csharp
// ĐÚNG
if (!int.TryParse(input, out var number))
    number = 0;
```

Ném rồi bắt một exception tốn kém hơn phép `if` rất nhiều lần. Nhưng cái giá
thật nằm ở chỗ khác.

Người đọc thấy `try` là hiểu rằng chỗ này có thể hỏng bất thường. Dùng nó cho
việc người dùng gõ sai một ô số là nói dối người đọc.

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
- `finally` luôn chạy — đó là chỗ để dọn dẹp.

## Bước tiếp theo

`finally` vừa gặp ở đây là chỗ dọn dẹp thủ công. Nhưng viết tay thì có ngày
quên.

Bài sau, **IDisposable và using**, dọn những thứ GC không dọn giúp: file,
connection, socket. Và `using` chính là `finally` viết gọn lại.

```quiz
[
  {
    "prompt": "Đoạn này làm gì với stack trace của lỗi gốc?",
    "code": "try { Inner(); }\ncatch (Exception ex) { throw ex; }",
    "options": [
      "Ghi đè stack trace từ dòng throw, mất nơi lỗi thật sự xảy ra",
      "Giữ nguyên, ném lại y như cũ",
      "Xoá luôn cả message",
      "Bọc thành inner exception"
    ],
    "answer": 1,
    "explain": "throw ex; đặt lại điểm bắt đầu của stack trace. Dùng throw; để ném tiếp mà giữ nguyên, hoặc bọc lỗi gốc làm inner exception."
  },
  {
    "prompt": "Log production chỉ có một dòng message, không stack trace. Nguyên nhân thường gặp nhất?",
    "options": [
      "Logger cấu hình sai mức log",
      "Exception không có stack trace",
      "Code ghi logger.LogError(ex.Message) thay vì logger.LogError(ex, ...)",
      "Production tắt stack trace cho nhẹ"
    ],
    "answer": 3,
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
      "Parse hay TryParse đều như nhau, chỉ khác cách viết",
      "Ném ArgumentException rồi bắt ở controller",
      "if (!int.TryParse(s, out var n)) n = 0;"
    ],
    "answer": 4,
    "explain": "Người dùng gõ sai là chuyện bình thường, không phải tình huống bất thường. Exception tốn kém hơn một phép if rất nhiều."
  }
]
```
