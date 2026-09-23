---
title: Chương trình C# đầu tiên
minutes: 10
---

Cài SDK. Gõ ba dòng lệnh. Ba mươi giây sau, màn hình hiện chữ.

Dễ đến mức người ta bỏ qua câu hỏi đáng hỏi nhất: trong ba mươi giây ấy, máy
đã làm gì với code của bạn?

Buổi phỏng vấn backend nào cũng hỏi câu đó. Không phải để làm khó. Biết code
biến thành gì trước khi chạy thì bạn đọc được lỗi, đọc được log, và hiểu vì
sao một file chạy được trên cả Windows lẫn Linux.

> **Học xong bài này bạn sẽ:** tạo và chạy được một project C#; đọc được cấu
> trúc một chương trình đầy đủ; giải thích được code của mình biến thành gì
> trước khi máy chạy nó.
>
> **Cần biết trước:** dùng được terminal. Chưa cần biết lập trình.

## Bốn lệnh dotnet bạn sẽ gõ mỗi ngày

| Lệnh | Làm gì |
|---|---|
| `dotnet new console -o Ten` | tạo project mới trong thư mục `Ten` |
| `dotnet build` | dịch code, **không** chạy |
| `dotnet run` | dịch rồi chạy luôn |
| `dotnet watch run` | chạy lại mỗi khi bạn lưu file |

`console application` là loại project đơn giản nhất. Đầu vào và đầu ra đều
bằng chữ, không giao diện. Cả khoá dùng nó, vì nó không che mất thứ đang học.

## Code đi qua hai bước dịch trước khi chạy

```mermaid Từ code tới lúc chạy: compiler dịch sang IL, CLR dịch tiếp lúc chạy
flowchart TD
    A["Program.cs<br/>code bạn viết"] --> B["Compiler"]
    B --> C["IL trong file .dll<br/>chưa phải mã máy"]
    C --> D["CLR — máy ảo của .NET"]
    D --> E["Mã máy, chạy trên CPU"]
```

**Compiler** dịch code sang **IL** (Intermediate Language), rồi cất vào file
`.dll`. IL chưa phải mã máy. Nó là thứ tiếng trung gian, không CPU nào hiểu
trực tiếp.

Tới lúc chạy, **CLR** (Common Language Runtime) mới dịch IL sang mã máy của
đúng con CPU đang có. Đó là lý do một file `.dll` chạy được trên Windows,
Linux và máy Mac. Bạn build một lần. Mỗi máy tự lo phần còn lại.

## Thử ngay: Write và WriteLine khác nhau ở đâu

```bash
dotnet new console -o HelloBackend
cd HelloBackend
```

Mở `Program.cs`, xoá sạch, dán đoạn này vào rồi `dotnet run`:

```csharp
Console.Write("Xin chào ");
Console.Write("backend");
Console.WriteLine("!");
Console.WriteLine("Dòng thứ hai");
```

**Đoán trước khi chạy:** bốn lệnh in ra mấy dòng?

<details>
<summary>Đoán xong rồi — xem kết quả</summary>

```text
Xin chào backend!
Dòng thứ hai
```

Hai dòng. `Write` in xong thì dừng tại chỗ. `WriteLine` mới xuống dòng. Ba
lệnh đầu cùng góp chữ vào một dòng.

Chi tiết nhỏ này theo bạn suốt nghề. Log dính liền nhau hay xuống dòng lung
tung đều từ đây mà ra.

</details>

## Top-level statements chỉ là cách viết gọn

Đoạn vừa chạy không có `class`, không có `Main`. Từ .NET 6, C# cho phép viết
thẳng như vậy. Tên gọi của nó là **top-level statements**, và compiler sẽ tự
sinh phần còn thiếu.

Code cũ thì viết đủ. Bạn sẽ gặp dạng này rất nhiều:

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

| Thành phần | Vai trò |
|---|---|
| `namespace` | họ của kiểu, để không trùng tên với thư viện khác |
| `class Program` | nơi chứa code; mọi thứ trong C# đều nằm trong một kiểu |
| `Main` | điểm bắt đầu, chương trình chạy từ đây |
| `static` | gọi được mà chưa cần tạo object nào |

Hai cách chạy như nhau. Khoá này dùng dạng đầy đủ mỗi khi cần bàn tới `class`
hay `namespace`.

## Project, solution, assembly: ba từ gặp mỗi ngày

| Từ | Là gì |
|---|---|
| **Project** (`.csproj`) | một đơn vị build |
| **Assembly** | kết quả build: một file `.dll` hoặc `.exe` |
| **Solution** (`.sln`) | tập hợp nhiều project mở cùng nhau |
| **NuGet** | kho thư viện ngoài, lấy về bằng `dotnet add package` |

Một API thật hiếm khi chỉ có một project. Thường là bốn: `Api`, `Domain`,
`Infrastructure`, `Tests`. Bốn project ấy nằm chung một solution, và mỗi
project build ra một assembly riêng.

## Lỗi hay gặp lần đầu

- `dotnet run` báo không tìm thấy project → bạn đang đứng sai thư mục, `cd` vào chỗ có file `.csproj`.
- Sửa code mà kết quả vẫn như cũ → dùng `dotnet watch run`, nó tự build lại khi bạn lưu.
- `error CS1002: ; expected` → thiếu dấu chấm phẩy; số dòng trong thông báo chỉ đúng chỗ.
- Gõ `dotnet` mà máy không hiểu → chưa cài SDK, hoặc cài xong chưa mở lại terminal.

## Ghi nhớ

- `dotnet run` dịch rồi chạy; `dotnet build` chỉ dịch; `dotnet watch run` chạy lại khi file đổi.
- Compiler ra **IL** trong file `.dll`, **CLR** dịch IL sang mã máy lúc chạy.
- **Top-level statements** là cách viết gọn; bên dưới vẫn là `class` và `Main`.
- Một **solution** chứa nhiều **project**, mỗi project build ra một **assembly**.

## Bước tiếp theo

Chương trình đầu tiên đã chạy. Nhưng nó mới in ra chữ, chưa tính toán gì.

Bài sau, **Kiểu dữ liệu và biến**, mở bằng một hoá đơn in ra
`0.6000000000000001`. Chọn sai kiểu số thì phép cộng cũng ra sai.

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
    "prompt": "File Program.cs chỉ có Console.WriteLine(\"Hi\"); và không có class nào. Vì sao chạy được?",
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
    "explain": "dotnet watch run theo dõi file và build lại mỗi khi bạn lưu — vòng lặp sửa rồi chạy nhanh hơn hẳn."
  },
  {
    "prompt": "Một API thật có 4 project: Api, Domain, Infrastructure, Tests. Bốn project này nằm trong cái gì?",
    "options": ["Một assembly", "Một solution", "Một namespace", "Một package NuGet"],
    "answer": 2,
    "explain": "Solution (.sln) gom nhiều project mở cùng nhau. Mỗi project build ra một assembly riêng."
  }
]
```
