---
title: Chương trình C# đầu tiên
minutes: 9
---

Ba dòng lệnh là bạn có một chương trình chạy thật. Nhưng giữa lúc bạn gõ
`dotnet run` và lúc chữ hiện ra màn hình, có hai bước dịch mà người đi phỏng
vấn backend gần như chắc chắn sẽ hỏi tới.

> **Học xong bài này bạn sẽ:** tạo và chạy được một project C#, đọc được cấu
> trúc một chương trình đầy đủ, và giải thích được code của mình biến thành gì
> trước khi máy chạy nó.
>
> **Cần biết trước:** dùng được terminal (cd, chạy lệnh). Chưa cần biết lập
> trình.

## Code của bạn đi qua hai bước dịch

```mermaid Từ code tới lúc chạy: compiler dịch sang IL, CLR dịch tiếp lúc chạy
flowchart TD
    A["Program.cs<br/>code bạn viết"] --> B["Compiler"]
    B --> C["IL trong file .dll<br/>chưa phải mã máy"]
    C --> D["CLR — máy ảo của .NET"]
    D --> E["Mã máy, chạy trên CPU"]
```

**Compiler** dịch code sang **IL** (Intermediate Language) và cất vào file
`.dll`. Tới lúc chạy, **CLR** (Common Language Runtime) mới dịch IL sang mã máy
của đúng CPU đang dùng. Nhờ bước giữa này mà cùng một file `.dll` chạy được
trên Windows, Linux hay máy Mac.

## Thử ngay: tạo và chạy

```bash
dotnet new console -o HelloBackend
cd HelloBackend
dotnet run
```

Lệnh `dotnet new console` tạo một **console application**: loại project đơn
giản nhất, chỉ có đầu vào và đầu ra bằng chữ.

Giờ mở `Program.cs`, xoá hết và dán đoạn này vào:

```csharp
Console.Write("Xin chào ");
Console.Write("backend");
Console.WriteLine("!");
Console.WriteLine("Dòng thứ hai");
```

**Đoán trước khi chạy:** bốn lệnh trên in ra mấy dòng?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Xin chào backend!
Dòng thứ hai
```

Hai dòng. `Console.Write` in xong **không xuống dòng**, còn `WriteLine` mới
xuống. Ba lệnh đầu cùng góp chữ vào một dòng.

</details>

## Chương trình đầy đủ trông thế nào

Từ .NET 6, C# cho phép viết **top-level statements**: không cần khai báo
`class` và hàm `Main` như đoạn trên. Compiler tự sinh chúng ra. Khi đọc code
cũ, bạn sẽ gặp dạng đầy đủ:

```csharp
namespace HelloBackend;

public class Program
{
    public static void Main(string[] args)
    {
        Console.WriteLine("Xin chào backend!");
    }
}
```

Hai cách chạy như nhau. Dạng đầy đủ nói rõ hơn về cấu trúc, nên phần còn lại
của khoá dùng dạng này khi cần bàn tới `class` hay `namespace`.

- `namespace` — cái tên dài đặt trước tên kiểu để không trùng với kiểu cùng tên của thư viện khác.
- `Main` — điểm bắt đầu; chương trình chạy từ đây.
- `static` — gọi được mà không cần tạo object, vì lúc đó chưa có object nào cả.

## Solution, project, assembly

Ba từ này xuất hiện trong mọi project .NET thật:

- **Project** (`.csproj`) — một đơn vị build. Build ra một **assembly**, tức là một file `.dll` (thư viện) hoặc `.exe` (chạy trực tiếp).
- **Solution** (`.sln`) — tập hợp nhiều project mở cùng nhau. Một API thật thường có `Api`, `Domain`, `Infrastructure`, `Tests` — bốn project trong một solution.
- **NuGet** — nơi lấy thư viện ngoài: `dotnet add package <tên>`.

## Lỗi hay gặp lần đầu

- `dotnet run` báo không tìm thấy project → bạn đang đứng sai thư mục, `cd` vào thư mục chứa file `.csproj`.
- Sửa code rồi mà chạy vẫn ra kết quả cũ → dùng `dotnet watch run`, nó tự build lại mỗi khi file đổi.
- `error CS1002: ; expected` → thiếu dấu chấm phẩy cuối câu lệnh; số dòng trong thông báo chỉ đúng chỗ.
- Gõ `dotnet` mà máy không hiểu → chưa cài .NET SDK, hoặc cài xong chưa mở lại terminal.

## Ghi nhớ

- `dotnet new console` tạo project, `dotnet run` build rồi chạy, `dotnet build` chỉ build.
- Compiler → **IL** trong file `.dll`; **CLR** dịch IL sang mã máy lúc chạy.
- **Top-level statements** là cách viết gọn; bên dưới vẫn là `class` và `Main`.
- Một **solution** chứa nhiều **project**, mỗi project build ra một **assembly**.

## Bước tiếp theo

Bài sau — **Kiểu dữ liệu và biến** — bắt đầu từ một bug tiền bạc kinh điển, và
giải thích vì sao chọn sai kiểu số thì phép cộng cũng ra sai.

```quiz
[
  {
    "prompt": "Bạn build project và được một file .dll. Bên trong file đó là gì?",
    "options": [
      "Mã máy của CPU đang dùng",
      "IL — mã trung gian, CLR dịch tiếp lúc chạy",
      "Chính file Program.cs đã nén lại",
      "Mã máy cho mọi loại CPU"
    ],
    "answer": 2,
    "explain": "Compiler dừng ở IL. CLR mới dịch IL sang mã máy lúc chạy, nên cùng một .dll chạy được trên nhiều hệ điều hành."
  },
  {
    "prompt": "File Program.cs của bạn chỉ có Console.WriteLine(\"Hi\"); và không có class nào. Vì sao chạy được?",
    "options": [
      "C# không cần class, mọi file đều chạy được",
      "Top-level statements: compiler tự sinh class và Main",
      "Vì đây là console application, loại project này khác",
      "Vì CLR bỏ qua bước tìm Main"
    ],
    "answer": 2,
    "explain": "Từ .NET 6, compiler tự sinh phần class và Main cho bạn. Chương trình vẫn bắt đầu từ Main như mọi khi."
  },
  {
    "prompt": "Bạn sửa code nhưng chạy lại vẫn thấy kết quả cũ. Cách xử lý hợp lý nhất?",
    "options": [
      "Xoá thư mục bin rồi tạo project mới",
      "Chạy dotnet watch run để tự build lại khi file đổi",
      "Khởi động lại máy",
      "Đổi từ dotnet run sang dotnet build"
    ],
    "answer": 2,
    "explain": "dotnet watch run theo dõi file và build lại mỗi khi bạn lưu — vòng lặp sửa/chạy nhanh hơn hẳn."
  },
  {
    "prompt": "Một API thật có 4 project: Api, Domain, Infrastructure, Tests. Bốn project này nằm trong cái gì?",
    "options": ["Một assembly", "Một solution", "Một namespace", "Một package NuGet"],
    "answer": 2,
    "explain": "Solution (.sln) gom nhiều project mở cùng nhau. Mỗi project build ra một assembly riêng."
  }
]
```
