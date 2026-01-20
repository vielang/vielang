# Hàm và Scope trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu cách xây dựng hàm, truyền tham số, giá trị trả về, khái niệm scope và giới thiệu sơ lược về ownership trong Rust.

## Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Xây dựng và gọi hàm trong Rust với cú pháp đúng
- [ ] Truyền tham số và trả về giá trị từ hàm
- [ ] Phân biệt statements và expressions
- [ ] Hiểu khái niệm scope và quản lý biến
- [ ] Nắm bắt sơ lược về ownership

### Kiến Thức Yêu Cầu

- Hoàn thành Bài 4: Cấu trúc điều khiển và vòng lặp
- Hiểu về kiểu dữ liệu cơ bản
- Nắm vững expressions vs statements

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Định nghĩa và gọi hàm | 15 phút |
| 2 | Scope và Ownership | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## Phần 1: Kiến Thức Nền Tảng

### 1.1. Định Nghĩa Hàm

> **Định nghĩa**: Hàm trong Rust được khai báo với từ khóa `fn`, theo quy ước đặt tên snake_case.

**Cú pháp cơ bản:**

```rust
fn tên_hàm(tham_số: kiểu) -> kiểu_trả_về {
    // Thân hàm
}
```

**Ví dụ đơn giản:**

```rust
fn say_hello() {
    println!("Xin chào!");
}

fn main() {
    say_hello(); // Gọi hàm
}
```

**Quy ước đặt tên:**

- Sử dụng `snake_case` (chữ thường, nối bằng `_`)
- Tên mô tả hành động của hàm
- Ví dụ: `calculate_area`, `get_user_name`

### 1.2. Tham Số và Giá Trị Trả Về

#### Truyền tham số

```rust
fn greet(name: &str) {
    println!("Xin chào, {}!", name);
}

fn main() {
    greet("Nguyễn Văn A");

    let student = "Trần Thị B";
    greet(student);
}
```

**Giải thích:**

- Tham số phải chỉ định kiểu dữ liệu
- `&str` là string slice (tham chiếu đến string)

#### Giá trị trả về

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // Không có ; → trả về giá trị
}

fn main() {
    let sum = add(5, 3);
    println!("Tổng: {}", sum); // 8
}
```

**Lưu ý:**

- Kiểu trả về sau `->`
- Expression cuối không có `;` sẽ được trả về
- `()` là unit type (không trả về gì)

### 1.3. Expressions và Return Values

#### Trả về ngầm định vs `return`

```rust
// Cách 1: Trả về ngầm định (khuyến khích)
fn square(num: i32) -> i32 {
    num * num  // Không có ;
}

// Cách 2: Dùng return (khi cần return sớm)
fn absolute(num: i32) -> i32 {
    if num >= 0 {
        return num;  // Return sớm
    }
    -num  // Trả về ngầm định
}
```

#### Block expression

```rust
fn main() {
    let y = {
        let x = 3;
        x + 1  // Expression, trả về 4
    };
    println!("y = {}", y); // 4
}
```

---

## Phần 2: Phân Tích & Tư Duy

### 2.1. Scope trong Rust

**Scenario**: Hiểu vòng đời của biến trong chương trình...

```rust
fn main() {
    let outer_var = 10;

    {
        let inner_var = 20;
        println!("Trong block: outer={}, inner={}",
                 outer_var, inner_var);
    } // inner_var bị hủy ở đây

    // println!("{}", inner_var); // Lỗi!
    println!("Ngoài block: outer={}", outer_var);
}
```

**Quy tắc scope:**

- Biến tồn tại từ khi khai báo đến cuối block `{}`
- Block con có thể truy cập biến của block cha
- Block cha không thể truy cập biến của block con

### 2.2. Giới Thiệu Ownership

> **Lưu ý quan trọng**: Đây là khái niệm cốt lõi của Rust!

| Kiểu dữ liệu | Khi truyền vào hàm |
|--------------|-------------------|
| Primitive (i32, bool, f64...) | Copy - biến gốc vẫn dùng được |
| Heap (String, Vec...) | Move - biến gốc không còn hợp lệ |

```rust
fn main() {
    let s = String::from("hello");

    takes_ownership(s); // s bị move

    // println!("{}", s); // Lỗi! s không còn hợp lệ
}

fn takes_ownership(some_string: String) {
    println!("{}", some_string);
} // some_string bị drop
```

**Câu hỏi suy ngẫm:**

1. Tại sao Rust cần ownership?
2. Làm sao để dùng lại biến sau khi truyền vào hàm?
3. Copy vs Move khác nhau thế nào?

<details>
<summary>Gợi ý phân tích</summary>

- Ownership giúp quản lý bộ nhớ tự động, không cần GC
- Dùng reference `&` để mượn (borrow) thay vì move
- Primitive types implement Copy trait, được copy tự động
- Heap types bị move để tránh double free

</details>

### 2.3. Best Practices

#### Nên Làm

```rust
// Dùng implicit return cho code ngắn gọn
fn double(x: i32) -> i32 {
    x * 2
}

// Dùng reference để tránh ownership transfer
fn print_string(s: &String) {
    println!("{}", s);
}
```

**Tại sao tốt:**

- Code ngắn gọn, idiomatic Rust
- Không chuyển ownership không cần thiết

#### Không Nên Làm

```rust
// Đừng dùng return cho expression cuối
fn add_bad(a: i32, b: i32) -> i32 {
    return a + b; // Không cần return
}
```

**Tại sao không tốt:**

- Verbose, không theo style Rust
- `return` chỉ cần cho early return

### 2.4. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Missing return type | Quên `-> Type` | Thêm kiểu trả về |
| Value moved | Ownership transferred | Dùng `&` hoặc `.clone()` |
| Extra semicolon | `;` ở expression cuối | Bỏ `;` để trả về giá trị |
| Type mismatch | Return sai kiểu | Kiểm tra kiểu dữ liệu |

---

## Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Viết hàm tính giai thừa

#### Bước 1: Tạo project

```bash
cargo new functions_demo
cd functions_demo
```

#### Bước 2: Phiên bản đệ quy

```rust
fn factorial(n: u64) -> u64 {
    if n == 0 || n == 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn main() {
    println!("5! = {}", factorial(5));  // 120
    println!("10! = {}", factorial(10)); // 3628800
}
```

**Giải thích:**

- Base case: 0! = 1! = 1
- Recursive case: n! = n * (n-1)!
- Dùng `u64` để tránh overflow

#### Bước 3: Phiên bản iterative (tốt hơn)

```rust
fn factorial_iter(n: u64) -> u64 {
    let mut result = 1;
    for i in 1..=n {
        result *= i;
    }
    result
}
```

### 3.2. Bài Tập Tự Luyện

#### Cấp độ Cơ Bản

**Bài tập 1**: Tìm số lớn nhất trong 3 số

<details>
<summary>Gợi ý</summary>

Sử dụng if expression và implicit return.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn max_of_three(a: i32, b: i32, c: i32) -> i32 {
    let max_ab = if a > b { a } else { b };
    if max_ab > c { max_ab } else { c }
}

fn main() {
    println!("Max(10, 5, 15) = {}", max_of_three(10, 5, 15)); // 15
}
```

</details>

**Bài tập 2**: Tính tổng từ 1 đến n

<details>
<summary>Gợi ý</summary>

Có thể dùng vòng lặp hoặc công thức n*(n+1)/2.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn sum_to_n(n: u32) -> u32 {
    (1..=n).sum()
}

// Hoặc với công thức
fn sum_formula(n: u32) -> u32 {
    n * (n + 1) / 2
}

fn main() {
    println!("Sum 1..100 = {}", sum_to_n(100)); // 5050
}
```

</details>

#### Cấp độ Nâng Cao

**Bài tập 3**: Fibonacci

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn fibonacci(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut a = 0u64;
    let mut b = 1u64;

    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

fn main() {
    for i in 0..=10 {
        println!("F({}) = {}", i, fibonacci(i));
    }
}
```

</details>

**Bài tập 4**: Kiểm tra palindrome

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for i in 0..len / 2 {
        if chars[i] != chars[len - 1 - i] {
            return false;
        }
    }
    true
}

fn main() {
    println!("radar: {}", is_palindrome("radar"));   // true
    println!("hello: {}", is_palindrome("hello"));   // false
    println!("madam: {}", is_palindrome("madam"));   // true
}
```

</details>

### 3.3. Mini Project

**Dự án**: Simple Calculator

**Mô tả**: Xây dựng calculator với các hàm toán học cơ bản.

**Yêu cầu chức năng:**

1. Các phép tính: +, -, *, /
2. Tính giai thừa
3. Tính lũy thừa
4. Xử lý division by zero

**Hướng dẫn triển khai:**

```rust
fn add(a: f64, b: f64) -> f64 {
    a + b
}

fn subtract(a: f64, b: f64) -> f64 {
    a - b
}

fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

fn power(base: f64, exp: u32) -> f64 {
    let mut result = 1.0;
    for _ in 0..exp {
        result *= base;
    }
    result
}

fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

fn main() {
    println!("10 + 5 = {}", add(10.0, 5.0));
    println!("10 - 5 = {}", subtract(10.0, 5.0));
    println!("10 * 5 = {}", multiply(10.0, 5.0));

    match divide(10.0, 5.0) {
        Some(result) => println!("10 / 5 = {}", result),
        None => println!("Cannot divide by zero"),
    }

    match divide(10.0, 0.0) {
        Some(result) => println!("10 / 0 = {}", result),
        None => println!("Cannot divide by zero"),
    }

    println!("2^10 = {}", power(2.0, 10));
    println!("5! = {}", factorial(5));
}
```

---

## Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Định nghĩa và gọi hàm đúng cú pháp
- [ ] Hiểu implicit return vs `return`
- [ ] Phân biệt scope của biến
- [ ] Nắm bắt khái niệm ownership cơ bản
- [ ] Hoàn thành ít nhất 2 bài tập

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Khi nào dùng `return` explicit vs implicit?
2. **Ứng dụng**: Làm sao để giữ ownership sau khi truyền vào hàm?
3. **Phân tích**: Copy vs Move trong Rust khác nhau thế nào?
4. **Thực hành**: Demo hàm tính factorial.

### 4.3. Tài Liệu Tham Khảo

- [The Rust Book - Functions](https://doc.rust-lang.org/book/ch03-03-how-functions-work.html)
- [The Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Khi nào dùng &str vs String làm tham số?</strong></summary>

- `&str`: Khi chỉ cần đọc, không cần ownership
- `String`: Khi hàm cần sở hữu và có thể modify

```rust
fn print_str(s: &str) { ... }      // Đọc
fn process_string(s: String) { ... } // Sở hữu
```

</details>

<details>
<summary><strong>Q2: Hàm có thể trả về nhiều giá trị không?</strong></summary>

Có, dùng tuple:

```rust
fn min_max(arr: &[i32]) -> (i32, i32) {
    let min = *arr.iter().min().unwrap();
    let max = *arr.iter().max().unwrap();
    (min, max)
}
```

</details>

<details>
<summary><strong>Q3: Recursive có bị stack overflow không?</strong></summary>

Có thể. Rust không có tail call optimization mặc định. Với đệ quy sâu, nên dùng iterative approach hoặc explicit stack.

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Rust Programming | **Lesson**: 5

</footer>
