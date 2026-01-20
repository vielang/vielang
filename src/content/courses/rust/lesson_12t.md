# Enums và Pattern Matching trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu về enums - kiểu dữ liệu liệt kê mạnh mẽ của Rust và pattern matching - công cụ xử lý các trường hợp khác nhau một cách an toàn.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu và định nghĩa enums với dữ liệu đính kèm
- [ ] Sử dụng thành thạo Option và Result enums
- [ ] Thành thạo pattern matching với match
- [ ] Sử dụng if let và while let cho xử lý pattern đơn giản

### Kiến Thức Yêu Cầu

- Structs và method syntax (Bài 11)
- Ownership và borrowing (Bài 6, 7)
- Kiểu dữ liệu cơ bản trong Rust

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Enums | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Enum (Enumeration) cho phép định nghĩa một kiểu dữ liệu bằng cách liệt kê các biến thể có thể có của nó.

**Tại sao điều này quan trọng?**

- Biểu diễn dữ liệu có nhiều trạng thái khác nhau
- Type-safe: compiler đảm bảo xử lý tất cả các trường hợp
- Kết hợp với pattern matching để viết code an toàn và rõ ràng

### 1.2. Kiến Thức Cốt Lõi

#### Enum cơ bản

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

fn main() {
    let direction = Direction::North;

    match direction {
        Direction::North => println!("Hướng Bắc"),
        Direction::South => println!("Hướng Nam"),
        Direction::East => println!("Hướng Đông"),
        Direction::West => println!("Hướng Tây"),
    }
}
```

#### Enum với dữ liệu

```rust
enum Message {
    Quit,                       // Không có dữ liệu
    Move { x: i32, y: i32 },    // Struct nội tuyến
    Write(String),              // Tuple với một phần tử
    ChangeColor(i32, i32, i32), // Tuple với ba phần tử
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => println!("Thoát"),
        Message::Move { x, y } => println!("Di chuyển đến ({}, {})", x, y),
        Message::Write(text) => println!("Tin nhắn: {}", text),
        Message::ChangeColor(r, g, b) => println!("Màu RGB: ({}, {}, {})", r, g, b),
    }
}
```

**📝 Giải thích:**
- Mỗi variant có thể chứa kiểu dữ liệu khác nhau
- Tương đương với nhiều structs khác nhau nhưng gom vào một kiểu

#### Option Enum

```rust
// Định nghĩa trong standard library
enum Option<T> {
    None,    // Không có giá trị
    Some(T), // Có giá trị kiểu T
}

fn main() {
    let some_number = Some(5);
    let absent_number: Option<i32> = None;

    // Xử lý Option với match
    match some_number {
        Some(value) => println!("Có giá trị: {}", value),
        None => println!("Không có giá trị"),
    }

    // Các phương thức hữu ích
    let doubled = some_number.map(|x| x * 2);
    let value = absent_number.unwrap_or(0);
}
```

> **⚠️ Lưu ý**: Tránh sử dụng `unwrap()` trong production code vì có thể gây panic.

#### Result Enum

```rust
enum Result<T, E> {
    Ok(T),  // Thành công với giá trị T
    Err(E), // Lỗi với giá trị E
}

use std::fs::File;
use std::io::{self, Read};

fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    match read_file("hello.txt") {
        Ok(contents) => println!("Nội dung: {}", contents),
        Err(error) => println!("Lỗi: {}", error),
    }
}
```

#### Pattern Matching với match

```rust
fn main() {
    let number = 13;

    match number {
        // Khớp giá trị cụ thể
        0 => println!("Số không"),

        // Khớp nhiều giá trị
        1 | 2 => println!("Một hoặc hai"),

        // Khớp phạm vi
        3..=9 => println!("Từ ba đến chín"),

        // Guard condition
        n if n % 2 == 0 => println!("{} là số chẵn", n),

        // Wildcard
        _ => println!("Số khác"),
    }
}
```

#### if let và while let

```rust
fn main() {
    let some_value = Some(42);

    // Thay vì match dài dòng
    if let Some(value) = some_value {
        println!("Giá trị: {}", value);
    } else {
        println!("Không có giá trị");
    }

    // while let cho vòng lặp
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("Phần tử: {}", top);
    }
}
```

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | match | if let |
|----------|-------|--------|
| Xử lý tất cả cases | Bắt buộc | Không bắt buộc |
| Khi nào dùng | Nhiều patterns | Một pattern cụ thể |
| Exhaustiveness | Compiler kiểm tra | Không kiểm tra |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng state machine cho máy ATM

**Yêu cầu**:
- Các trạng thái: Idle, CardInserted, PinEntered, AmountSelected
- Xử lý các thao tác: InsertCard, EnterPin, SelectAmount, Cancel

**🤔 Câu hỏi suy ngẫm:**

1. Làm sao biểu diễn các trạng thái với dữ liệu khác nhau?
2. Pattern matching giúp gì trong việc xử lý transitions?
3. Làm sao đảm bảo xử lý tất cả các trường hợp?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
enum ATMState {
    Idle,
    CardInserted(String),              // Số thẻ
    PinEntered(String, String),        // Số thẻ, PIN
    AmountSelected(String, String, u64), // Số thẻ, PIN, Số tiền
}

enum ATMOperation {
    InsertCard(String),
    EnterPin(String),
    SelectAmount(u64),
    Cancel,
}

fn process(state: ATMState, op: ATMOperation) -> ATMState {
    match (state, op) {
        (ATMState::Idle, ATMOperation::InsertCard(card)) => {
            ATMState::CardInserted(card)
        }
        (ATMState::CardInserted(card), ATMOperation::EnterPin(pin)) => {
            ATMState::PinEntered(card, pin)
        }
        (ATMState::PinEntered(card, pin), ATMOperation::SelectAmount(amount)) => {
            ATMState::AmountSelected(card, pin, amount)
        }
        (_, ATMOperation::Cancel) => {
            ATMState::Idle
        }
        (state, _) => state, // Invalid operation, keep current state
    }
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Pattern matching trong Rust phải exhaustive - xử lý tất cả trường hợp.

#### ✅ Nên Làm

```rust
// Xử lý tất cả cases rõ ràng
fn describe(opt: Option<i32>) -> String {
    match opt {
        Some(0) => String::from("Số không"),
        Some(n) if n > 0 => format!("Số dương: {}", n),
        Some(n) => format!("Số âm: {}", n),
        None => String::from("Không có giá trị"),
    }
}

// Sử dụng if let khi chỉ quan tâm một case
if let Some(value) = option {
    process(value);
}
```

**Tại sao tốt:**
- Rõ ràng về xử lý từng trường hợp
- Compiler đảm bảo không bỏ sót case

#### ❌ Không Nên Làm

```rust
// Dùng unwrap() không an toàn
let value = option.unwrap(); // Panic nếu None

// Wildcard quá sớm bỏ qua các cases
match result {
    Ok(v) => process(v),
    _ => (), // Bỏ qua tất cả lỗi - nguy hiểm!
}
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Non-exhaustive patterns | Thiếu case | Thêm wildcard `_` hoặc xử lý đầy đủ |
| unwrap() panic | None hoặc Err | Dùng match, if let, hoặc unwrap_or |
| Moved value in match | Pattern lấy ownership | Dùng ref pattern hoặc borrow |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng hệ thống thanh toán đơn giản

**Yêu cầu kỹ thuật:**
- Enum Payment với các phương thức thanh toán
- Hàm xử lý thanh toán với pattern matching
- Error handling với Result

#### Bước 1: Định nghĩa enums

```rust
#[derive(Debug)]
enum Payment {
    Cash(f64),
    CreditCard { number: String, amount: f64 },
    MobilePayment { phone: String, amount: f64 },
}

#[derive(Debug)]
enum PaymentError {
    InsufficientFunds,
    InvalidCard,
    NetworkError,
}
```

#### Bước 2: Implement xử lý thanh toán

```rust
fn process_payment(payment: Payment) -> Result<String, PaymentError> {
    match payment {
        Payment::Cash(amount) => {
            if amount > 0.0 {
                Ok(format!("Thanh toán tiền mặt: {:.2} VND", amount))
            } else {
                Err(PaymentError::InsufficientFunds)
            }
        }
        Payment::CreditCard { number, amount } => {
            if number.len() == 16 {
                Ok(format!("Thanh toán thẻ *{}: {:.2} VND",
                    &number[12..], amount))
            } else {
                Err(PaymentError::InvalidCard)
            }
        }
        Payment::MobilePayment { phone, amount } => {
            Ok(format!("Thanh toán qua {}: {:.2} VND", phone, amount))
        }
    }
}
```

#### Bước 3: Sử dụng và xử lý kết quả

```rust
fn main() {
    let payments = vec![
        Payment::Cash(100_000.0),
        Payment::CreditCard {
            number: String::from("1234567890123456"),
            amount: 500_000.0,
        },
        Payment::MobilePayment {
            phone: String::from("0901234567"),
            amount: 200_000.0,
        },
    ];

    for payment in payments {
        match process_payment(payment) {
            Ok(message) => println!("Thành công: {}", message),
            Err(e) => println!("Lỗi: {:?}", e),
        }
    }
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo enum TrafficLight

```rust
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

// Implement:
// - duration(&self) -> u32 (giây)
// - next(&self) -> TrafficLight
```

<details>
<summary>💡 Gợi ý</summary>

Sử dụng match trong impl block để xử lý từng variant.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
impl TrafficLight {
    fn duration(&self) -> u32 {
        match self {
            TrafficLight::Red => 60,
            TrafficLight::Yellow => 5,
            TrafficLight::Green => 45,
        }
    }

    fn next(&self) -> TrafficLight {
        match self {
            TrafficLight::Red => TrafficLight::Green,
            TrafficLight::Yellow => TrafficLight::Red,
            TrafficLight::Green => TrafficLight::Yellow,
        }
    }
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Order State Machine

```rust
enum OrderState {
    Created,
    Processing,
    Shipped { tracking: String },
    Delivered,
    Cancelled { reason: String },
}

enum OrderEvent {
    Process,
    Ship(String),    // tracking number
    Deliver,
    Cancel(String),  // reason
}

// Implement: fn transition(state: OrderState, event: OrderEvent) -> OrderState
```

**Mở rộng**:
- Thêm validation cho transitions
- Return Result<OrderState, TransitionError>

### 3.3. Mini Project

**Dự án**: Calculator với Expression Enum

**Mô tả**: Xây dựng calculator sử dụng enum để biểu diễn expressions

**Yêu cầu chức năng:**

1. Hỗ trợ +, -, *, /
2. Hỗ trợ nested expressions
3. Error handling cho division by zero

**Technical Stack:**
- Recursive enums với Box
- Pattern matching
- Result type

**Hướng dẫn triển khai:**

```rust
enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
enum CalcError {
    DivisionByZero,
}

impl Expr {
    fn eval(&self) -> Result<f64, CalcError> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::Add(a, b) => Ok(a.eval()? + b.eval()?),
            Expr::Sub(a, b) => Ok(a.eval()? - b.eval()?),
            Expr::Mul(a, b) => Ok(a.eval()? * b.eval()?),
            Expr::Div(a, b) => {
                let divisor = b.eval()?;
                if divisor == 0.0 {
                    Err(CalcError::DivisionByZero)
                } else {
                    Ok(a.eval()? / divisor)
                }
            }
        }
    }
}

fn main() {
    // (3 + 4) * 2
    let expr = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Number(3.0)),
            Box::new(Expr::Number(4.0)),
        )),
        Box::new(Expr::Number(2.0)),
    );

    match expr.eval() {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu cách định nghĩa enums với dữ liệu
- [ ] Sử dụng được Option và Result
- [ ] Thành thạo pattern matching với match
- [ ] Biết khi nào dùng if let
- [ ] Hoàn thành bài tập Payment
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Enum trong Rust khác gì C/C++?
2. **Ứng dụng**: Khi nào dùng Option vs Result?
3. **Phân tích**: Tại sao match phải exhaustive?
4. **Thực hành**: Demo Expression Calculator?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- So sánh enum Rust với union trong C
- State machine pattern với enums
- Error handling best practices

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Option<T> có bao nhiêu variants?

- A. 1
- B. 2
- C. 3
- D. Tùy thuộc vào T

**Câu 2**: Toán tử `?` làm gì?

- A. Kiểm tra null
- B. Unwrap và return early nếu Err
- C. Tạo Option mới
- D. So sánh hai giá trị

**Câu 3**: Khi nào nên dùng if let thay vì match?

- A. Luôn luôn
- B. Khi chỉ quan tâm một pattern
- C. Khi có nhiều patterns
- D. Không bao giờ

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Enum có thể có methods không?</strong></summary>

Có, sử dụng impl block giống như struct:

```rust
impl Message {
    fn call(&self) {
        match self {
            Message::Write(text) => println!("{}", text),
            _ => println!("Other message"),
        }
    }
}
```

</details>

<details>
<summary><strong>Q2: Sao cần Box cho recursive enums?</strong></summary>

Compiler cần biết kích thước của enum tại compile time. Recursive enum có kích thước vô hạn. Box là pointer với kích thước cố định:

```rust
// Không compile - kích thước vô hạn
enum List {
    Node(i32, List),
    Nil,
}

// OK - Box có kích thước cố định
enum List {
    Node(i32, Box<List>),
    Nil,
}
```

</details>

<details>
<summary><strong>Q3: Khác nhau giữa unwrap() và expect()?</strong></summary>

Cả hai đều panic nếu None/Err, nhưng expect() cho phép custom message:

```rust
let x: Option<i32> = None;
x.unwrap(); // panic: "called `Option::unwrap()` on a `None` value"
x.expect("Custom error message"); // panic: "Custom error message"
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
