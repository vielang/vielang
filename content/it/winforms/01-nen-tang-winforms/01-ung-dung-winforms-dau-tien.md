---
title: Ứng dụng WinForms đầu tiên
minutes: 5
---

API ở khoá ASP.NET Core phục vụ khách mua hàng qua mạng. Nhân viên cửa hàng
thì cần một phần mềm trên máy tính để nhập hàng, xem đơn. Khoá này viết phần
mềm đó bằng WinForms, dùng chung database Oracle với API.

## Khái niệm

🪟 **WinForms (Windows Forms)**: thư viện của .NET để viết ứng dụng Windows có cửa sổ, nút bấm, ô nhập.

🖼️ **Form**: class đại diện cho một cửa sổ, mỗi cửa sổ trong app là một class kế thừa `Form`.

`MainForm : Form` là kế thừa như bài Kế thừa của khoá OOP: `MainForm` nhận
sẵn mọi thứ một cửa sổ cần, như tiêu đề, kích thước, nút đóng, và chỉ thêm
phần riêng.

## Ví dụ

Tạo project:

```bash
dotnet new winforms -o ShopDesk
cd ShopDesk
```

Template tạo sẵn `Form1.cs` và `Form1.Designer.cs` cho trình thiết kế kéo thả
của Visual Studio. VS Code không có trình thiết kế đó, nên khoá này xoá hai
file trên và viết giao diện bằng code. Thay toàn bộ `Program.cs` bằng:

```csharp
namespace ShopDesk;

static class Program
{
    [STAThread]
    static void Main()
    {
        ApplicationConfiguration.Initialize();
        Application.Run(new MainForm());
    }
}

class MainForm : Form
{
    public MainForm()
    {
        Text = "Quản lý cửa hàng";
        Width = 400;
        Height = 300;
    }
}
```

Chạy bằng `dotnet run`, một cửa sổ trống hiện ra với tiêu đề "Quản lý cửa
hàng".

- `Main` là nơi chương trình bắt đầu. Ở khoá C# Core, ta viết lệnh thẳng
  trong `Program.cs` và compiler tự bọc chúng vào `Main`. Ở đây ta tự viết
  `Main`, đặt trong `static class Program` chỉ chứa thành viên static.
- `[STAThread]` là attribute, như `[HttpGet]` ở khoá ASP.NET Core. Nó bật chế
  độ chạy mà cửa sổ Windows cần.
- `namespace ShopDesk;` đặt mọi class trong file vào namespace của project.
  Template sinh `ApplicationConfiguration` (bật font, độ nét mặc định) trong
  namespace này, nên phải giữ dòng này.
- `Text`, `Width`, `Height` là property kế thừa từ `Form`, gán trong
  constructor.
- `Application.Run` mở cửa sổ và chờ người dùng thao tác, giống `app.Run()`
  của Web API chờ request. Đóng cửa sổ thì `Run` kết thúc và chương trình
  thoát.

## Thử ngay

Trong `Main`, thêm một dòng ngay dưới `Application.Run(new MainForm());`:

```csharp
Application.Run(
    new MainForm { Text = "Cửa sổ thứ hai" });
```

**Đoán trước khi chạy:** hai cửa sổ hiện ra cùng lúc, hay cửa sổ thứ hai chỉ
hiện khi bạn đóng cửa sổ đầu?

<details>
<summary>Xem kết quả</summary>

```text
Lúc đầu chỉ có cửa sổ "Quản lý cửa hàng".
Đóng nó, cửa sổ "Cửa sổ thứ hai" mới hiện ra.
Đóng tiếp, chương trình thoát.
```

`Application.Run` dừng ở đó cho tới khi cửa sổ đóng, rồi mới chạy dòng tiếp
theo. Mở form thứ hai từ form chính là việc của bài Hộp thoại và form thứ hai.

</details>

## Lỗi hay gặp

**Quên `: Form`.** Thiếu kế thừa thì `MainForm` chỉ là class thường: không
có property `Text`, cũng không đưa được cho `Application.Run`.

```csharp
// SAI — lỗi compile: MainForm không có Text
class MainForm
{
    public MainForm()
    {
        Text = "Quản lý cửa hàng";
    }
}
```

```csharp
// ĐÚNG
class MainForm : Form
{
    public MainForm()
    {
        Text = "Quản lý cửa hàng";
    }
}
```

**Xoá dòng `namespace ShopDesk;`.** Compiler báo không tìm thấy
`ApplicationConfiguration`, vì template đặt nó trong namespace `ShopDesk`.

## Tóm tắt

- `dotnet new winforms` tạo project. Khoá này xoá file của trình thiết kế,
  viết giao diện bằng code.
- Mỗi cửa sổ là một class kế thừa `Form`, cấu hình trong constructor.
- `Main` có `[STAThread]`, gọi `Application.Run(new MainForm())`.
- `Application.Run` chờ tới khi cửa sổ đóng mới chạy tiếp.

```quiz
[
  {
    "prompt": "Muốn có cửa sổ đăng nhập riêng, ta khai báo nó thế nào?",
    "options": [
      "var LoginForm = new Form(); ngay trong Main",
      "class LoginForm : Form, cấu hình trong constructor",
      "interface LoginForm : Form",
      "static class LoginForm"
    ],
    "answer": 2,
    "explain": "Mỗi cửa sổ là một class kế thừa Form. Constructor gán tiêu đề, kích thước và thêm control."
  },
  {
    "prompt": "Main gọi Application.Run(new MainForm()) rồi tới một dòng ghi log. Dòng ghi log chạy lúc nào?",
    "options": [
      "Ngay khi cửa sổ vừa hiện",
      "Trước khi cửa sổ hiện",
      "Không bao giờ chạy",
      "Sau khi người dùng đóng cửa sổ"
    ],
    "answer": 4,
    "explain": "Application.Run chờ tới khi form chính đóng mới trả về, nên dòng sau nó chạy sau khi cửa sổ đóng."
  },
  {
    "prompt": "Trong MainForm : Form, vì sao gán được Text = \"Kho\" dù MainForm không khai báo property Text?",
    "options": [
      "Text là property MainForm kế thừa từ Form",
      "C# tự tạo property khi gán",
      "Text là biến toàn cục",
      "Phải có [STAThread] mới gán được"
    ],
    "answer": 1,
    "explain": "Class con nhận lại property của class cha. Form có sẵn Text là tiêu đề cửa sổ."
  }
]
```
