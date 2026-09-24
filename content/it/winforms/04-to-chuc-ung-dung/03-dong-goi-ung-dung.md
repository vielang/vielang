---
title: Đóng gói ứng dụng
minutes: 5
---

`dotnet run` chỉ chạy được trên máy có mã nguồn và .NET SDK. Máy của nhân
viên cửa hàng không có những thứ đó. Bài này đóng gói app thành một file
`.exe`, chép sang máy khác là chạy.

## Khái niệm

📦 **Publish**: biên dịch app ở chế độ Release và gom mọi thứ nó cần vào một thư mục để đem đi cài.

🧳 **Self-contained**: bản publish mang theo cả .NET, nên máy chạy nó không cần cài .NET.

## Ví dụ

```bash
dotnet publish -c Release -r win-x64 --self-contained -p:PublishSingleFile=true -o publish
```

- `-c Release` biên dịch bản đã tối ưu. `dotnet run` mặc định chạy bản
  Debug, dành cho lúc lập trình.
- `-r win-x64` chọn máy đích là Windows 64-bit.
- `--self-contained` mang theo .NET.
- `PublishSingleFile=true` gom tất cả vào một file `ShopDesk.exe`.
- `-o publish` đặt kết quả vào thư mục `publish`.

Thư mục `publish` có `ShopDesk.exe` nặng khoảng 120 MB. Phần lớn dung lượng
là .NET, Windows Forms, EF Core và thư viện Oracle đi kèm.

## Thử ngay

Chạy lại lệnh trên, đổi `--self-contained` thành `--self-contained false`
và `-o publish` thành `-o publish-nho`, rồi so kích thước hai file `.exe`.

**Đoán trước khi chạy:** file `.exe` mới nhỏ hơn bao nhiêu? Chép nó sang một
máy chưa cài .NET thì chạy được không?

<details>
<summary>Xem kết quả</summary>

```text
--self-contained        ShopDesk.exe ~ 120 MB
--self-contained false  ShopDesk.exe ~ 12 MB
```

Nhỏ hơn khoảng 10 lần, vì không mang theo .NET. Máy chưa cài .NET Desktop
Runtime thì không chạy được: mở file lên chỉ thấy hộp thoại báo thiếu .NET,
kèm đường dẫn tải về.

</details>

## Lỗi hay gặp

**Chuỗi kết nối vẫn là `localhost`.** Trên máy nhân viên, `localhost` là
chính máy đó, không có Oracle. App mở lên là báo lỗi kết nối như bài Xử lý
lỗi.

```csharp
// SAI — máy nhân viên không có Oracle
var cs = "User Id=shopapi;Password=shopapi_pw;"
    + "Data Source=localhost:1521/FREEPDB1";
```

```csharp
// ĐÚNG — trỏ tới máy chủ đang chạy Oracle
var cs = "User Id=shopapi;Password=shopapi_pw;"
    + "Data Source=192.168.1.10:1521/FREEPDB1";
```

Mật khẩu viết trong code sẽ nằm luôn trong file `.exe`, ai có file cũng đọc
được, và đổi máy chủ là phải publish lại. App thật không để mật khẩu dùng
chung trong file đi kèm `.exe`: mỗi nhân viên một tài khoản Oracle riêng,
hoặc app gọi API thay vì nối thẳng vào database.

## Tóm tắt

- `dotnet publish -c Release -r win-x64` đóng gói app để đem đi.
- `--self-contained` mang theo .NET: file to hơn, không cần cài gì thêm.
- `PublishSingleFile=true` gom tất cả vào một file `.exe`.
- Trước khi đem đi, sửa chuỗi kết nối trỏ tới máy chủ Oracle thật.

```quiz
[
  {
    "prompt": "Cửa hàng có 20 máy, chưa máy nào cài .NET, cũng không có người cài giúp. Nên publish thế nào?",
    "options": [
      "--self-contained false để file nhỏ",
      "--self-contained để mang theo .NET",
      "Dùng dotnet run trên từng máy",
      "Chép thư mục bin/Debug sang"
    ],
    "answer": 2,
    "explain": "Máy không có .NET thì bản self-contained mới chạy được mà không phải cài gì."
  },
  {
    "prompt": "App publish xong chạy tốt trên máy lập trình, sang máy nhân viên thì báo không kết nối được database. Nghi ngờ đầu tiên?",
    "options": [
      "Thiếu PublishSingleFile",
      "Sai -r win-x64",
      "Chuỗi kết nối vẫn trỏ localhost",
      "Máy nhân viên chưa cài Oracle client"
    ],
    "answer": 3,
    "explain": "localhost trên máy nhân viên là chính máy đó, không có Oracle. Phải trỏ tới địa chỉ máy chủ. Oracle.EntityFrameworkCore không cần cài Oracle client."
  },
  {
    "prompt": "PublishSingleFile=true làm gì?",
    "options": [
      "Nén mã nguồn thành một file zip",
      "Chỉ publish file Program.cs",
      "Bỏ .NET ra khỏi bản publish",
      "Gom app vào một file .exe"
    ],
    "answer": 4,
    "explain": "App và thư viện được gom lại: thay vì thư mục đầy file .dll, bản publish chỉ còn một file .exe để chép đi."
  }
]
```
