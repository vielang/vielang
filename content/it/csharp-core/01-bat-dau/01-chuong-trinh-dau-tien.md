---
title: Chương trình đầu tiên
minutes: 4
---

Trước khi học cú pháp, bạn cần biết cách tạo và chạy một chương trình C#. Bài
này đi từ lúc cài đặt tới lúc màn hình in ra dòng chữ đầu tiên.

## Khái niệm

🧰 **.NET SDK**: bộ công cụ để tạo, build và chạy chương trình C#, dùng qua lệnh `dotnet`.

🖥️ **Console app**: chương trình chạy trong cửa sổ dòng lệnh, nhập và xuất dữ liệu đều bằng chữ.

📝 **Câu lệnh (statement)**: một chỉ thị cho máy tính, trong C# kết thúc bằng dấu `;`.

Cả khoá học dùng console app vì nó đơn giản nhất: không có giao diện, chỉ có
code và kết quả.

## Ví dụ

Cài .NET SDK từ trang dotnet.microsoft.com, rồi mở terminal và gõ:

```bash
dotnet new console -o HelloShop
cd HelloShop
dotnet run
```

- `dotnet new console -o HelloShop` tạo project mới trong thư mục
  `HelloShop`.
- `dotnet run` build rồi chạy chương trình.

Code nằm trong file `Program.cs`. Mở ra sẽ thấy:

```csharp
Console.WriteLine("Hello, World!");
```

- `Console.WriteLine(...)` in nội dung trong ngoặc ra màn hình rồi xuống
  dòng.
- Chữ đặt trong dấu nháy kép `"..."` là một chuỗi.
- Dấu `;` kết thúc câu lệnh.

## Thử ngay

Xoá hết nội dung `Program.cs`, dán đoạn sau vào rồi chạy `dotnet run`:

```csharp
Console.Write("Xin chào ");
Console.Write("cửa hàng");
Console.WriteLine("!");
Console.WriteLine("Mở cửa lúc 8 giờ");
```

**Đoán trước khi chạy:** bốn câu lệnh in ra mấy dòng?

<details>
<summary>Xem kết quả</summary>

```text
Xin chào cửa hàng!
Mở cửa lúc 8 giờ
```

Hai dòng. `Console.Write` in xong vẫn ở nguyên dòng đó, còn `Console.WriteLine`
in xong mới xuống dòng.

</details>

## Lỗi hay gặp

**Quên dấu `;`.** Compiler báo lỗi `; expected` và chỉ đúng dòng bị thiếu.

```csharp
// SAI — lỗi compile: thiếu dấu ;
Console.WriteLine("Xin chào")
```

```csharp
// ĐÚNG
Console.WriteLine("Xin chào");
```

**Viết sai hoa thường.** C# phân biệt chữ hoa và chữ thường, nên `console`
khác `Console`.

```csharp
// SAI — C# không có gì tên là console
console.writeline("Xin chào");
```

```csharp
// ĐÚNG
Console.WriteLine("Xin chào");
```

## Tóm tắt

- `dotnet new console -o Ten` tạo project, `dotnet run` build và chạy.
- Code nằm trong `Program.cs`.
- `Console.WriteLine` in rồi xuống dòng, `Console.Write` in mà không xuống
  dòng.
- Mỗi câu lệnh kết thúc bằng `;`, và C# phân biệt hoa thường.

```quiz
[
  {
    "prompt": "Bạn vừa sửa Program.cs. Lệnh nào vừa build vừa chạy chương trình?",
    "options": [
      "dotnet new console",
      "dotnet run",
      "dotnet add",
      "dotnet Program.cs"
    ],
    "answer": 2,
    "explain": "dotnet run build lại code rồi chạy luôn. dotnet new chỉ tạo project mới."
  },
  {
    "prompt": "Đoạn Console.Write(\"A\"); Console.Write(\"B\"); Console.WriteLine(\"C\"); in ra gì?",
    "options": [
      "A, B, C trên ba dòng",
      "AB trên một dòng, C trên dòng sau",
      "ABC trên một dòng",
      "Lỗi compile"
    ],
    "answer": 3,
    "explain": "Write không xuống dòng, nên A, B, C nằm chung một dòng. WriteLine chỉ xuống dòng sau khi in C."
  },
  {
    "prompt": "Dòng Console.Writeline(\"Hi\"); báo lỗi compile. Vì sao?",
    "options": [
      "Thiếu dấu ; ở cuối",
      "Chuỗi phải dùng nháy đơn",
      "Console phải viết thường",
      "Tên đúng là WriteLine, chữ L viết hoa"
    ],
    "answer": 4,
    "explain": "C# phân biệt hoa thường. Writeline và WriteLine là hai tên khác nhau, và chỉ WriteLine tồn tại."
  }
]
```
