---
title: Chương trình C# đầu tiên
minutes: 6
---

C# là ngôn ngữ chạy trên nền **.NET**. Code bạn viết được **compiler** dịch sang
**IL** (Intermediate Language), rồi **CLR** (Common Language Runtime) dịch tiếp
sang mã máy lúc chạy.

## Tạo project

```bash
dotnet new console -o HelloBackend
cd HelloBackend
dotnet run
```

Lệnh `dotnet new console` tạo một **console application**: loại project đơn giản
nhất, chỉ có đầu vào và đầu ra bằng chữ.

## Chương trình tối giản

```csharp
Console.WriteLine("Xin chào backend!");
```

Từ .NET 6, C# cho phép viết **top-level statements**: không cần khai báo `class`
và hàm `Main`. Compiler tự sinh chúng ra. Khi đọc code cũ, bạn sẽ gặp dạng đầy đủ:

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

Hai cách chạy như nhau. Dạng đầy đủ nói rõ hơn về cấu trúc, nên phần còn lại của
khoá dùng dạng này khi cần bàn tới `class` hay `namespace`.

## Ghi nhớ

- **Solution** chứa nhiều **project**; mỗi project build ra một **assembly** (file `.dll` hoặc `.exe`).
- `dotnet build` chỉ dịch, `dotnet run` dịch rồi chạy, `dotnet watch run` chạy lại mỗi khi file đổi.
