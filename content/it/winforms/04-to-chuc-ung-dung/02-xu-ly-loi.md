---
title: Xử lý lỗi
minutes: 5
---

Oracle tắt, mạng rớt, dữ liệu vi phạm ràng buộc: sớm muộn app cũng gặp lỗi.
Không xử lý thì nhân viên thấy một hộp lỗi dài, khó hiểu của .NET. Bài
này báo lỗi dễ hiểu và giữ app chạy tiếp.

## Khái niệm

🧯 **Application.ThreadException**: event nhận mọi exception chưa được bắt trong các handler của giao diện, kể cả handler `async void`.

Hai tầng giống khoá ASP.NET Core. Lỗi lường trước được thì bắt tại chỗ bằng
`try/catch` như bài Exception của khoá C# Core. Lỗi còn lại đi về một chỗ
chung, như bài Xử lý lỗi tập trung.

## Ví dụ

Bắt lỗi kết nối ngay trong `LoadAsync` của bài trước:

```csharp
using Oracle.ManagedDataAccess.Client;

// Trong MainForm, thay LoadAsync cũ
private async Task LoadAsync()
{
    try
    {
        _source.DataSource =
            await _store.InStockAsync();
    }
    catch (OracleException)
    {
        MessageBox.Show(
            "Không kết nối được database. "
            + "Kiểm tra Oracle rồi thử lại.",
            "Lỗi kết nối",
            MessageBoxButtons.OK,
            MessageBoxIcon.Error);
    }
}
```

Chỗ chung cho mọi lỗi khác, đặt trong `Main` trước `Application.Run`:

```csharp
Application.ThreadException += (sender, e) =>
{
    MessageBox.Show(
        "Có lỗi xảy ra: " + e.Exception.Message,
        "Lỗi",
        MessageBoxButtons.OK,
        MessageBoxIcon.Error);
};
```

- `OracleException` nằm trong `Oracle.ManagedDataAccess.Client`, cùng thư
  viện với `OracleCommand` ở bài SQL injection. EF Core ném thẳng nó ra khi
  không kết nối được.
- Chỉ bắt `OracleException` ở đây. Lỗi khác vẫn bay lên, rơi vào
  `ThreadException`.
- Handler của `ThreadException` hiện thông báo, rồi app chạy tiếp như chưa
  có gì.
- `MessageBoxIcon.Error` thêm biểu tượng lỗi màu đỏ vào hộp thông báo.

## Thử ngay

Tắt Oracle bằng `docker stop oracle`, rồi chạy app.

**Đoán trước khi chạy:** app sập, treo, hay hiện gì?

<details>
<summary>Xem kết quả</summary>

```text
Khoảng vài giây sau, hộp "Lỗi kết nối" hiện ra:
"Không kết nối được database. Kiểm tra Oracle rồi thử lại."
Bấm OK: cửa sổ kho vẫn mở, lưới trống.
```

`InStockAsync` chờ Oracle trả lời rồi ném `OracleException` với mã
`ORA-50201`. `catch` bắt được nên app không sập. Chạy `docker start oracle`
rồi mở lại app là có dữ liệu.

</details>

## Lỗi hay gặp

**`catch (Exception)` rồi bỏ trống.** Mọi lỗi bị nuốt mất, kể cả lỗi code
sai. Nhân viên thấy lưới trống mà không biết vì sao.

```csharp
// SAI — lỗi gì cũng im lặng
try
{
    _source.DataSource =
        await _store.InStockAsync();
}
catch (Exception)
{
}
```

```csharp
// ĐÚNG — chỉ bắt lỗi biết cách xử lý, và báo ra
try
{
    _source.DataSource =
        await _store.InStockAsync();
}
catch (OracleException)
{
    MessageBox.Show("Không kết nối được database.");
}
```

## Tóm tắt

- Lỗi lường trước được thì `try/catch` đúng loại, như `OracleException`.
- `Application.ThreadException` là chỗ chung cho mọi lỗi chưa được bắt.
- Báo lỗi bằng `MessageBox` với lời dễ hiểu, app vẫn chạy tiếp.
- Không `catch` rồi bỏ trống.

```quiz
[
  {
    "prompt": "Handler async void của nút Lưu ném exception sau một lệnh await, không có try/catch. Main đã gắn Application.ThreadException. Chuyện gì xảy ra?",
    "options": [
      "App sập ngay",
      "Exception bị bỏ qua, không ai biết",
      "Handler của ThreadException nhận exception đó",
      "Lỗi compile vì thiếu try/catch"
    ],
    "answer": 3,
    "explain": "Exception trong handler async void được đưa về UI thread và đi vào Application.ThreadException."
  },
  {
    "prompt": "Nên bắt OracleException ở đâu?",
    "options": [
      "Ở chỗ gọi database, nơi biết cách báo lỗi cho người dùng",
      "Trong constructor của Product",
      "Không bao giờ bắt",
      "Trong class Program, bằng catch (Exception)"
    ],
    "answer": 1,
    "explain": "Bắt lỗi ở nơi biết cách xử lý nó. Mọi lỗi khác để ThreadException lo."
  },
  {
    "prompt": "Vì sao catch (Exception) { } bỏ trống là lỗi?",
    "options": [
      "Vì compiler không cho",
      "Vì làm app chạy chậm",
      "Vì Exception không bắt được lỗi Oracle",
      "Vì mọi lỗi bị nuốt, kể cả lỗi code sai, không ai biết để sửa"
    ],
    "answer": 4,
    "explain": "Khối catch rỗng giấu mọi lỗi. App chạy sai mà không có dấu hiệu gì."
  }
]
```
