---
title: Sự kiện
minutes: 5
---

Nút "Lưu" đã hiện nhưng bấm vào không có gì xảy ra. App console chạy từ trên
xuống rồi dừng, còn app có cửa sổ thì chờ người dùng bấm, gõ, chọn. Bài này
cho code chạy đúng vào những lúc đó.

## Khái niệm

🔔 **Event (sự kiện)**: thông báo mà một object phát ra khi có chuyện xảy ra, ví dụ `Click` của nút khi được bấm.

🎧 **Event handler**: method được gắn vào event bằng `+=`, chạy mỗi lần event đó xảy ra.

Cách chạy này giống controller ở khoá ASP.NET Core: action không tự chạy, nó
chờ request tới. Handler cũng chờ, nhưng chờ cú bấm của người dùng.

## Ví dụ

```csharp
class MainForm : Form
{
    private int _count = 0;
    private readonly Label _countLabel = new Label
    {
        Text = "Giỏ hàng: 0",
        AutoSize = true
    };

    public MainForm()
    {
        Text = "Giỏ hàng";

        var panel = new FlowLayoutPanel
        {
            Dock = DockStyle.Fill,
            FlowDirection = FlowDirection.TopDown,
            Padding = new Padding(12)
        };
        var addButton = new Button { Text = "Thêm" };
        addButton.Click += AddButton_Click;

        panel.Controls.Add(addButton);
        panel.Controls.Add(_countLabel);
        Controls.Add(panel);
    }

    private void AddButton_Click(
        object? sender, EventArgs e)
    {
        _count = _count + 1;
        _countLabel.Text = $"Giỏ hàng: {_count}";
    }
}
```

- `addButton.Click += AddButton_Click;` gắn method vào event. Không có dấu
  `()` sau tên method: ta giao method cho nút giữ để gọi sau, không gọi ngay.
- Handler của `Click` luôn nhận hai tham số: `sender` là object phát ra
  event (ở đây là nút), `e` chứa thông tin thêm.
- `_countLabel` là field, vì cả constructor lẫn handler đều cần dùng nó.
  Biến `addButton` chỉ dùng trong constructor nên để là biến cục bộ.

## Handler viết bằng lambda

Handler ngắn có thể viết thẳng bằng lambda, như bài Lambda của khoá C# Core:

```csharp
// Trong constructor, thay dòng gắn handler cũ
addButton.Click += (sender, e) =>
{
    _count = _count + 1;
    _countLabel.Text = $"Giỏ hàng: {_count}";
};
```

Hai tham số `sender`, `e` đặt trong ngoặc `( )`, thân nhiều lệnh đặt trong
`{ }`. Handler dài thì nên viết thành method riêng có tên cho dễ đọc.

## Thử ngay

Dùng lại bản có method `AddButton_Click`. Trong constructor, chép thêm một
lần dòng gắn handler, để có hai dòng giống hệt:

```csharp
addButton.Click += AddButton_Click;
addButton.Click += AddButton_Click;
```

**Đoán trước khi chạy:** bấm "Thêm" một lần thì nhãn hiện `Giỏ hàng: 1` hay
`Giỏ hàng: 2`?

<details>
<summary>Xem kết quả</summary>

```text
Giỏ hàng: 2
```

Mỗi `+=` gắn thêm một lần. Event giữ một danh sách handler và gọi lần lượt
từng cái, nên cùng một method được gọi hai lần cho một cú bấm.

</details>

## Lỗi hay gặp

**Dùng `=` thay cho `+=`.** Event chỉ cho gắn thêm hoặc gỡ ra, không cho gán
đè.

```csharp
// SAI — lỗi compile: event chỉ đứng trước += hoặc -=
var addButton = new Button { Text = "Thêm" };
addButton.Click = (sender, e) => { };
```

```csharp
// ĐÚNG
var addButton = new Button { Text = "Thêm" };
addButton.Click += (sender, e) => { };
```

**Gắn handler trong chính handler.** Mỗi lần bấm lại `+=` thêm một lần, lần
bấm sau chạy gấp đôi. Gắn handler một lần duy nhất, trong constructor.

## Tóm tắt

- Event là thông báo object phát ra, như `Click` của nút.
- Gắn handler bằng `+=`, không có `()` sau tên method.
- Handler nhận `sender` và `e`. Viết bằng method hoặc lambda đều được.
- Mỗi `+=` gắn thêm một lần, nên gắn handler một lần trong constructor.

```quiz
[
  {
    "prompt": "Dòng nào gắn method SaveButton_Click vào nút saveButton đúng cách?",
    "options": [
      "saveButton.Click += SaveButton_Click();",
      "saveButton.Click = SaveButton_Click;",
      "saveButton.Click += SaveButton_Click;",
      "SaveButton_Click += saveButton.Click;"
    ],
    "answer": 3,
    "explain": "Dùng += và tên method không kèm (). Có () là gọi method ngay, còn = thì compiler không cho với event."
  },
  {
    "prompt": "Handler cần đổi chữ trên một Label. Vì sao Label đó thường là field chứ không phải biến trong constructor?",
    "options": [
      "Label không thể là biến cục bộ",
      "Handler là method khác, không thấy biến cục bộ của constructor",
      "Field chạy nhanh hơn",
      "Để Label tự hiện lên form"
    ],
    "answer": 2,
    "explain": "Biến cục bộ chỉ sống trong constructor. Field thuộc về object nên mọi method của form đều dùng được."
  },
  {
    "prompt": "Hai nút cùng gắn một handler. Trong handler, làm sao biết nút nào vừa được bấm?",
    "options": [
      "Không biết được, phải viết hai handler",
      "Đọc e.Button",
      "Đếm số lần handler chạy",
      "Xem tham số sender"
    ],
    "answer": 4,
    "explain": "sender là object phát ra event, tức chính nút vừa được bấm."
  }
]
```
