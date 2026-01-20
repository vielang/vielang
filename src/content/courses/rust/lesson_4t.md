# Cấu Trúc Điều Khiển và Vòng Lặp

> **Mô tả ngắn gọn**: Tìm hiểu cấu trúc điều khiển (if-else, if let), các loại vòng lặp (loop, while, for) và khái niệm Expressions vs Statements trong Rust.

## Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Áp dụng thành thạo câu lệnh điều kiện `if-else` và `if let`
- [ ] Sử dụng các loại vòng lặp: `loop`, `while`, và `for`
- [ ] Điều khiển luồng lặp với `break` và `continue`
- [ ] Phân biệt Expressions và Statements trong Rust
- [ ] Tận dụng `if` và `loop` như expressions có giá trị trả về

### Kiến Thức Yêu Cầu

- Hoàn thành Bài 3: Kiểu dữ liệu nâng cao
- Nắm vững cú pháp khai báo biến
- Hiểu về kiểu Option trong Rust (cơ bản)

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Cấu trúc điều kiện | 15 phút |
| 2 | Vòng lặp và điều khiển luồng | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## Phần 1: Kiến Thức Nền Tảng

### 1.1. Câu Lệnh Điều Kiện if-else

> **Định nghĩa**: Trong Rust, `if-else` không cần dấu ngoặc đơn cho điều kiện, nhưng luôn yêu cầu dấu ngoặc nhọn.

**Đặc điểm quan trọng:**

- Điều kiện **phải** là kiểu `bool` - không tự động convert
- Dấu ngoặc nhọn `{}` bắt buộc
- `if` có thể là expression (trả về giá trị)

```rust
let number = 7;

if number < 5 {
    println!("Nhỏ hơn 5");
} else if number < 10 {
    println!("Từ 5 đến 9");
} else {
    println!("Lớn hơn hoặc bằng 10");
}
```

**Giải thích:**

- Không cần `()` quanh điều kiện
- Mỗi nhánh phải có `{}`
- Điều kiện phải là `bool`, không phải số

#### if như Expression

```rust
let condition = true;
let number = if condition { 5 } else { 6 };
println!("number = {}", number); // 5
```

**Lưu ý:** Các nhánh phải trả về cùng kiểu dữ liệu.

### 1.2. if let - Pattern Matching Đơn Giản

```rust
let some_value = Some(3);

// Thay vì match đầy đủ
match some_value {
    Some(value) => println!("Có: {}", value),
    None => (),
}

// Dùng if let ngắn gọn hơn
if let Some(value) = some_value {
    println!("Có: {}", value);
}
```

**Khi nào dùng `if let`:**

- Chỉ quan tâm đến một pattern cụ thể
- Muốn code ngắn gọn hơn `match`
- Kết hợp với `else` cho trường hợp còn lại

### 1.3. Vòng Lặp trong Rust

#### loop - Vòng lặp vô hạn

```rust
let mut counter = 0;

let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2; // Trả về giá trị
    }
};

println!("result = {}", result); // 20
```

#### while - Vòng lặp có điều kiện

```rust
let mut number = 3;

while number != 0 {
    println!("{}!", number);
    number -= 1;
}
println!("Kết thúc!");
```

#### for - Duyệt qua collection

```rust
// Range: 1..4 = [1, 2, 3]
for i in 1..4 {
    println!("{}!", i);
}

// Duyệt array
let arr = [10, 20, 30];
for element in arr.iter() {
    println!("Giá trị: {}", element);
}
```

---

## Phần 2: Phân Tích & Tư Duy

### 2.1. Break và Continue

**Scenario**: Xử lý vòng lặp lồng nhau phức tạp...

#### Break với nhãn (label)

```rust
'outer: for i in 1..5 {
    for j in 1..5 {
        if i * j > 10 {
            println!("Thoát ở i={}, j={}", i, j);
            break 'outer; // Thoát vòng lặp ngoài
        }
    }
}
```

#### Continue

```rust
for i in 1..6 {
    if i % 2 == 0 {
        continue; // Bỏ qua số chẵn
    }
    println!("Số lẻ: {}", i);
}
```

**Câu hỏi suy ngẫm:**

1. Khi nào dùng `loop` vs `while`?
2. `break value` hoạt động như thế nào?
3. Làm sao thoát khỏi vòng lặp lồng nhau?

<details>
<summary>Gợi ý phân tích</summary>

- `loop`: Khi không biết trước số lần lặp, cần `break` có điều kiện
- `while`: Khi có điều kiện rõ ràng từ đầu
- `break value`: Chỉ hoạt động với `loop`, trả về giá trị từ vòng lặp
- Label `'name:` cho phép `break`/`continue` thoát vòng lặp cụ thể

</details>

### 2.2. Expressions vs Statements

> **Lưu ý quan trọng**: Rust là ngôn ngữ expression-oriented!

| Khái niệm | Đặc điểm | Ví dụ |
|-----------|----------|-------|
| Statement | Không trả về giá trị, có `;` | `let x = 5;` |
| Expression | Trả về giá trị, không có `;` | `x + 1` |

```rust
// Block là expression
let y = {
    let x = 3;
    x + 1  // Không có ; → expression, trả về 4
}; // y = 4

// if là expression
let max = if a > b { a } else { b };

// loop có thể là expression
let result = loop {
    if done { break value; }
};
```

### 2.3. Best Practices

#### Nên Làm

```rust
// Dùng if expression thay vì gán trong if
let status = if score >= 50 { "Pass" } else { "Fail" };

// Dùng for với range
for i in 0..n {
    // ...
}
```

**Tại sao tốt:**

- Code ngắn gọn, dễ đọc
- An toàn hơn, ít lỗi bounds checking

#### Không Nên Làm

```rust
// Đừng dùng while cho simple iteration
let mut i = 0;
while i < arr.len() {
    println!("{}", arr[i]);
    i += 1;
}
```

**Tại sao không tốt:**

- Dễ quên tăng index
- Không tận dụng iterator của Rust

### 2.4. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Điều kiện không phải bool | Dùng `if number` | Dùng `if number != 0` |
| Thiếu `{}` | Quen từ ngôn ngữ khác | Luôn dùng `{}` |
| Kiểu không khớp trong if expr | Các nhánh khác kiểu | Đảm bảo cùng kiểu |
| Infinite loop | Quên `break` | Thêm điều kiện thoát |

---

## Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Viết chương trình tính dãy Fibonacci

#### Bước 1: Tạo project

```bash
cargo new control_flow_demo
cd control_flow_demo
```

#### Bước 2: Implement Fibonacci

```rust
fn main() {
    let n = 10;
    println!("Dãy {} số Fibonacci đầu tiên:", n);

    let mut a = 0;
    let mut b = 1;

    print!("{}, {}", a, b);

    for _ in 2..n {
        let next = a + b;
        print!(", {}", next);
        a = b;
        b = next;
    }
    println!();
}
```

**Giải thích:**

- Khởi tạo a=0, b=1 (hai số đầu)
- Dùng `for _ in 2..n` vì đã in 2 số đầu
- `_` cho biến không sử dụng

### 3.2. Bài Tập Tự Luyện

#### Cấp độ Cơ Bản

**Bài tập 1**: Tìm số lớn nhất trong 3 số

<details>
<summary>Gợi ý</summary>

Sử dụng `if` expression lồng nhau.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn main() {
    let a = 15;
    let b = 27;
    let c = 10;

    let max = if a > b {
        if a > c { a } else { c }
    } else {
        if b > c { b } else { c }
    };

    println!("Max({}, {}, {}) = {}", a, b, c, max);
}
```

</details>

**Bài tập 2**: FizzBuzz

<details>
<summary>Gợi ý</summary>

Dùng `for` loop và kiểm tra chia hết bằng `%`.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn main() {
    for i in 1..=100 {
        if i % 15 == 0 {
            println!("FizzBuzz");
        } else if i % 3 == 0 {
            println!("Fizz");
        } else if i % 5 == 0 {
            println!("Buzz");
        } else {
            println!("{}", i);
        }
    }
}
```

</details>

#### Cấp độ Nâng Cao

**Bài tập 3**: Kiểm tra số nguyên tố

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

fn main() {
    for num in 1..=20 {
        if is_prime(num) {
            println!("{} là số nguyên tố", num);
        }
    }
}
```

</details>

**Bài tập 4**: if let với Option

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn find_divisible_by_3(numbers: &[i32]) -> Option<i32> {
    for &num in numbers {
        if num % 3 == 0 {
            return Some(num);
        }
    }
    None
}

fn main() {
    let numbers = [1, 2, 4, 5, 6, 8];

    if let Some(first) = find_divisible_by_3(&numbers) {
        println!("Số đầu tiên chia hết cho 3: {}", first);
    } else {
        println!("Không tìm thấy");
    }
}
```

</details>

### 3.3. Mini Project

**Dự án**: Number Guessing Game

**Mô tả**: Xây dựng trò chơi đoán số với các cấu trúc điều khiển.

**Yêu cầu chức năng:**

1. Random số từ 1-100
2. Cho người chơi đoán với gợi ý "cao hơn/thấp hơn"
3. Đếm số lần đoán
4. Thông báo khi đoán đúng

**Hướng dẫn triển khai:**

```rust
use std::io;
use std::cmp::Ordering;

fn main() {
    let secret = 42; // Thay bằng random trong thực tế
    let mut attempts = 0;

    println!("Đoán số từ 1 đến 100!");

    loop {
        println!("Nhập số của bạn:");

        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Lỗi đọc input");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Vui lòng nhập số hợp lệ!");
                continue;
            }
        };

        attempts += 1;

        match guess.cmp(&secret) {
            Ordering::Less => println!("Cao hơn!"),
            Ordering::Greater => println!("Thấp hơn!"),
            Ordering::Equal => {
                println!("Chính xác! Bạn đoán {} lần.", attempts);
                break;
            }
        }
    }
}
```

---

## Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Sử dụng thành thạo if-else và if let
- [ ] Phân biệt được loop, while, for
- [ ] Hiểu cách dùng break và continue với labels
- [ ] Nắm vững Expressions vs Statements
- [ ] Hoàn thành ít nhất 2 bài tập

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Tại sao Rust yêu cầu điều kiện phải là `bool`?
2. **Ứng dụng**: Khi nào dùng `loop` thay vì `while`?
3. **Phân tích**: `if` expression khác `if` statement như thế nào?
4. **Thực hành**: Demo chương trình FizzBuzz.

### 4.3. Tài Liệu Tham Khảo

- [The Rust Book - Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
- [Rust by Example - Flow Control](https://doc.rust-lang.org/rust-by-example/flow_control.html)

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Có thể dùng số như điều kiện if không?</strong></summary>

Không. Rust không tự động convert số sang bool. Phải viết rõ:

```rust
// Sai
if number { ... }

// Đúng
if number != 0 { ... }
```

</details>

<details>
<summary><strong>Q2: break có thể trả về giá trị từ while không?</strong></summary>

Không, chỉ `loop` hỗ trợ `break value`. `while` và `for` không hỗ trợ vì chúng có thể không chạy lần nào.

</details>

<details>
<summary><strong>Q3: Làm sao iterate với index trong for?</strong></summary>

Sử dụng `.enumerate()`:

```rust
for (index, value) in arr.iter().enumerate() {
    println!("arr[{}] = {}", index, value);
}
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Rust Programming | **Lesson**: 4

</footer>
