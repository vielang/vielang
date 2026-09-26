---
title: Câu hỏi phỏng vấn C# và .NET thường gặp (kèm đáp án)
summary: Các câu hỏi C#, OOP và ASP.NET Core hay gặp ở vòng fresher và junior, kèm đáp án ngắn đủ ý.
updated: 2026-09-27
---

Đáp án dưới đây là phần cốt lõi cần nói được. Khi trả lời, nói ý chính trước
rồi thêm một ví dụ từ dự án của mình.

## Chuẩn bị trước buổi phỏng vấn

- [ ] Đọc kỹ mô tả công việc, đánh dấu công nghệ họ yêu cầu
- [ ] Trả lời thành tiếng các câu trong bài này, không nhìn đáp án
- [ ] Mở lại dự án cá nhân, nhớ lại vì sao chọn từng cách làm
- [ ] Chuẩn bị phần giới thiệu bản thân khoảng 1–2 phút
- [ ] Chuẩn bị 2–3 câu hỏi cho nhà tuyển dụng
- [ ] Phỏng vấn online: thử trước camera, micro, chia sẻ màn hình

## C# cơ bản

**Value type và reference type khác nhau thế nào?**

Biến value type (`int`, `bool`, `struct`, `enum`) chứa trực tiếp giá trị; gán
sang biến khác là sao chép giá trị. Biến reference type (`class`, `string`,
array) chứa tham chiếu tới object; gán sang biến khác thì cả hai cùng trỏ
một object.

```csharp
var a = new Product { Name = "Pen" };
var b = a;
b.Name = "Book";
Console.WriteLine(a.Name); // Book — a và b cùng một object
```

**`string` là reference type, sao gán xong sửa không ảnh hưởng nhau?**

`string` là immutable: mọi thao tác "sửa" đều tạo string mới. Nối chuỗi
nhiều lần trong vòng lặp nên dùng `StringBuilder`.

**`==` và `Equals` khác nhau thế nào?**

Với reference type, mặc định cả hai so sánh tham chiếu. Class có thể
override `Equals` (và overload `==`) để so sánh theo giá trị; `string` và
`record` đã làm sẵn việc đó.

**`const` và `readonly` khác nhau thế nào?**

`const` là hằng lúc biên dịch, phải gán ngay khi khai báo. `readonly` được
gán khi khai báo hoặc trong constructor, sau đó không đổi được.

**Nullable reference type là gì?**

Khi bật `<Nullable>enable</Nullable>`, `string` nghĩa là không được null,
`string?` là có thể null. Compiler cảnh báo khi bạn dùng biến có thể null
mà chưa kiểm tra.

## OOP

**Nêu bốn tính chất của OOP.**

- Đóng gói: giấu dữ liệu bên trong, chỉ cho sửa qua method/property.
- Kế thừa: class con nhận lại thành viên của class cha.
- Đa hình: cùng một lời gọi, mỗi class con thực hiện theo cách riêng.
- Trừu tượng: chỉ lộ ra những gì cần dùng, giấu chi tiết cài đặt.

**Abstract class và interface khác nhau thế nào?**

Một class kế thừa được một abstract class nhưng implement được nhiều
interface. Abstract class có thể chứa field và constructor. Dùng interface
để mô tả "làm được gì", abstract class khi các class con dùng chung code.

**SOLID là gì?** Năm nguyên tắc thiết kế: Single responsibility, Open/closed,
Liskov substitution, Interface segregation, Dependency inversion. Nên chuẩn
bị ví dụ cho ít nhất S và D. Xem khoá [OOP và thiết kế](/it/oop-thiet-ke).

## Collection và LINQ

**`IEnumerable` và `IQueryable` khác nhau thế nào?**

`IEnumerable` lọc dữ liệu trong bộ nhớ. `IQueryable` (EF Core) giữ biểu thức
truy vấn để dịch sang SQL, phần lọc chạy ở database.

**Deferred execution là gì?**

Câu LINQ chỉ chạy khi được duyệt (`foreach`, `ToList()`, `Count()`…), không
chạy lúc khai báo. Duyệt hai lần là chạy hai lần.

**Khi nào dùng `List`, khi nào dùng `Dictionary`?**

`List` khi cần giữ thứ tự và duyệt tuần tự. `Dictionary` khi cần tìm theo
khoá, trung bình O(1) thay vì O(n).

## Async và bộ nhớ

**`async`/`await` giải quyết vấn đề gì?**

Không giữ thread trong lúc chờ I/O (gọi database, gọi HTTP). Trong web API,
thread rảnh được dùng để phục vụ request khác.

**Vì sao tránh `.Result` và `.Wait()`?**

Chúng chặn thread cho tới khi task xong, mất lợi ích của async và có thể gây
deadlock ở một số môi trường. Dùng `await` xuyên suốt.

**Garbage collector làm gì?**

Tự thu hồi bộ nhớ của object không còn được tham chiếu. Tài nguyên không
phải bộ nhớ (kết nối database, file) phải giải phóng bằng `Dispose`, thường
qua `using`.

## ASP.NET Core

**Dependency injection là gì, ba lifetime khác nhau thế nào?**

Class nhận phụ thuộc qua constructor thay vì tự `new`. Container tạo và
truyền vào.

- `Transient`: mỗi lần yêu cầu tạo mới.
- `Scoped`: một instance cho mỗi request. `DbContext` mặc định là scoped.
- `Singleton`: một instance cho cả ứng dụng.

**Middleware là gì?**

Các bước xử lý request xếp thành chuỗi (pipeline). Mỗi middleware xử lý rồi
gọi middleware tiếp theo hoặc trả response luôn. Thứ tự đăng ký quyết định
thứ tự chạy.

**Trả về status code nào khi tạo mới thành công, khi không tìm thấy, khi dữ
liệu gửi lên sai?** `201 Created`, `404 Not Found`, `400 Bad Request`.

**JWT hoạt động thế nào?**

Sau khi đăng nhập, server ký một token chứa thông tin user. Client gửi token
trong header `Authorization: Bearer <token>` ở mỗi request; server kiểm chữ
ký mà không cần tra session.

## Nguồn

- [Microsoft Learn — Value types (C# reference)](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-types)
- [Microsoft Learn — Asynchronous programming with async and await](https://learn.microsoft.com/en-us/dotnet/csharp/asynchronous-programming/)
- [Microsoft Learn — Fundamentals of garbage collection](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals)
- [Microsoft Learn — Dependency injection in ASP.NET Core](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/dependency-injection)
