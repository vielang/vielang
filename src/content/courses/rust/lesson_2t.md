# Kiểu Dữ Liệu Cơ Bản và Biến

> **Mô tả ngắn gọn**: Tìm hiểu cách khai báo biến, các kiểu dữ liệu nguyên thủy và khái niệm immutability - nền tảng quan trọng trong Rust.

## Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu và áp dụng được cơ chế khai báo biến với từ khóa `let`
- [ ] Nắm vững khái niệm bất biến (immutability) trong Rust
- [ ] Sử dụng thành thạo các kiểu dữ liệu cơ bản
- [ ] Phân biệt được constants, static variables và biến thông thường

### Kiến Thức Yêu Cầu

- Hoàn thành Bài 1: Giới thiệu Rust
- Đã cài đặt môi trường Rust
- Hiểu biết cơ bản về kiểu dữ liệu trong lập trình

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Khai báo biến và Immutability | 15 phút |
| 2 | Kiểu dữ liệu nguyên thủy | 15 phút |
| 3 | Thực hành | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## Phần 1: Kiến Thức Nền Tảng

### 1.1. Khai Báo Biến với `let`

> **Định nghĩa**: Trong Rust, biến được khai báo bằng từ khóa `let` và mặc định là bất biến (immutable).

**Tại sao điều này quan trọng?**

- Đảm bảo an toàn về bộ nhớ
- Ngăn chặn lỗi thay đổi không mong muốn
- Tối ưu hóa performance
- Hỗ trợ lập trình đồng thời

#### Khai báo biến cơ bản

```rust
let x = 5;
// x = 6; // Lỗi! Cannot assign twice to immutable variable
```

**Giải thích:**

- `let` tạo biến bất biến mặc định
- Compiler sẽ báo lỗi nếu cố gắng thay đổi giá trị

#### Biến có thể thay đổi (mutable)

```rust
let mut y = 5;
y = 6; // Hợp lệ
println!("Giá trị của y là: {}", y);
```

**Giải thích:**

- Từ khóa `mut` cho phép biến thay đổi giá trị
- Nên sử dụng có chủ đích, khi thực sự cần thiết

### 1.2. Kiểu Dữ Liệu Nguyên Thủy

#### Kiểu số nguyên (Integer)

| Kiểu | Kích thước | Signed | Unsigned |
|------|------------|--------|----------|
| 8-bit | 1 byte | i8: -128 → 127 | u8: 0 → 255 |
| 16-bit | 2 bytes | i16 | u16 |
| 32-bit | 4 bytes | i32 (mặc định) | u32 |
| 64-bit | 8 bytes | i64 | u64 |
| 128-bit | 16 bytes | i128 | u128 |
| arch | tùy máy | isize | usize |

```rust
let a: i32 = 42;
let b: u8 = 255;
let c = 100_000; // Underscore để dễ đọc
```

#### Kiểu số thực (Float)

```rust
let x = 2.0;      // f64 (mặc định)
let y: f32 = 3.0; // f32
```

**Lưu ý:**

- `f64` có độ chính xác cao hơn, là kiểu mặc định
- `f32` nhanh hơn trên một số hardware

#### Kiểu boolean và character

```rust
let is_active: bool = true;
let emoji: char = '🦀'; // 4 bytes, Unicode scalar
```

### 1.3. Type Annotation vs Type Inference

| Cách tiếp cận | Ví dụ | Khi nào dùng |
|---------------|-------|--------------|
| Type Annotation | `let x: i32 = 5;` | Muốn kiểu cụ thể |
| Type Inference | `let x = 5;` | Kiểu mặc định OK |

```rust
// Type annotation cần thiết khi parse
let guess: u32 = "42".parse().expect("Không phải là số!");
```

---

## Phần 2: Phân Tích & Tư Duy

### 2.1. Constants vs Static Variables

**Scenario**: Bạn cần lưu trữ các giá trị cố định trong chương trình...

| Đặc điểm | Constants | Static Variables |
|----------|-----------|------------------|
| Khai báo | `const MAX: u32 = 100;` | `static MSG: &str = "Hi";` |
| Địa chỉ bộ nhớ | Không cố định | Cố định |
| Inline | Thường được inline | Không inline |
| Mutable | Không bao giờ | Có thể (unsafe) |

```rust
const MAX_POINTS: u32 = 100_000;
static HELLO_WORLD: &str = "Xin chào!";
```

**Câu hỏi suy ngẫm:**

1. Khi nào dùng `const` vs `static`?
2. Tại sao `static mut` cần unsafe?

<details>
<summary>Gợi ý phân tích</summary>

- Dùng `const` cho giá trị đơn giản, compile-time
- Dùng `static` khi cần địa chỉ bộ nhớ cố định
- `static mut` cần unsafe vì có thể gây data race

</details>

### 2.2. Shadowing trong Rust

> **Lưu ý quan trọng**: Shadowing không phải là mutation!

#### Nên Làm

```rust
let spaces = "   ";       // &str
let spaces = spaces.len(); // usize - thay đổi cả kiểu
```

**Tại sao tốt:**

- Giữ nguyên tên biến có ý nghĩa
- Có thể thay đổi kiểu dữ liệu
- Rõ ràng về ý định

#### Không Nên Làm

```rust
let mut spaces = "   ";
// spaces = spaces.len(); // Lỗi! Không thể thay đổi kiểu
```

**Tại sao không tốt:**

- `mut` chỉ cho phép thay đổi giá trị, không phải kiểu
- Gây nhầm lẫn về mục đích sử dụng

### 2.3. Phạm Vi của Shadowing

```rust
let x = 5;
{
    let x = 12; // Shadow trong block
    println!("x trong block: {}", x); // 12
}
println!("x ngoài block: {}", x); // 5
```

### 2.4. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Quên `mut` | Cố thay đổi biến immutable | Thêm `mut` hoặc dùng shadowing |
| Integer overflow | Giá trị vượt phạm vi | Dùng kiểu lớn hơn hoặc checked operations |
| Type mismatch | Sai kiểu annotation | Kiểm tra lại kiểu dữ liệu |

---

## Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Hiểu sự khác biệt giữa immutable, mutable và shadowing

#### Bước 1: Tạo project

```bash
cargo new variables_demo
cd variables_demo
```

#### Bước 2: Viết code demo

```rust
fn main() {
    // Immutable (mặc định)
    let x = 5;
    println!("x = {}", x);

    // Mutable
    let mut y = 5;
    println!("y trước: {}", y);
    y = 10;
    println!("y sau: {}", y);

    // Shadowing
    let z = 5;
    let z = z + 1;
    let z = z * 2;
    println!("z = {}", z); // 12
}
```

#### Bước 3: Chạy và quan sát

```bash
cargo run
```

### 3.2. Bài Tập Tự Luyện

#### Cấp độ Cơ Bản

**Bài tập 1**: Thử nghiệm với các kiểu số nguyên

<details>
<summary>Gợi ý</summary>

Khai báo biến với các kiểu i8, u8, i32, u32 và in ra giá trị.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn main() {
    let a: i8 = 127;
    let b: u8 = 255;
    let c: i32 = -1_000_000;
    let d: u32 = 4_294_967_295;

    println!("i8 max: {}", a);
    println!("u8 max: {}", b);
    println!("i32: {}", c);
    println!("u32 max: {}", d);
}
```

</details>

**Bài tập 2**: So sánh shadowing và mutable

<details>
<summary>Gợi ý</summary>

Tạo biến string, sau đó convert sang length sử dụng shadowing.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn main() {
    // Shadowing - thay đổi cả kiểu
    let data = "Hello, Rust!";
    let data = data.len();
    println!("Length: {}", data);

    // Mutable - chỉ thay đổi giá trị
    let mut count = 0;
    count = count + 1;
    count = count + 1;
    println!("Count: {}", count);
}
```

</details>

#### Cấp độ Nâng Cao

**Bài tập 3**: Chuyển đổi nhiệt độ

<details>
<summary>Giải pháp mẫu</summary>

```rust
const CONVERSION_FACTOR: f64 = 9.0 / 5.0;
const OFFSET: f64 = 32.0;

fn main() {
    let celsius: f64 = 25.0;
    let fahrenheit = celsius * CONVERSION_FACTOR + OFFSET;

    println!("{}°C = {}°F", celsius, fahrenheit);
}
```

</details>

### 3.3. Mini Project

**Dự án**: Temperature Converter CLI

**Mô tả**: Xây dựng công cụ chuyển đổi nhiệt độ giữa Celsius và Fahrenheit.

**Yêu cầu chức năng:**

1. Nhận input từ người dùng
2. Chuyển đổi C → F và F → C
3. Sử dụng constants cho hệ số chuyển đổi
4. Xử lý input không hợp lệ

**Hướng dẫn triển khai:**

```rust
use std::io;

const FACTOR: f64 = 9.0 / 5.0;
const OFFSET: f64 = 32.0;

fn main() {
    println!("Nhập nhiệt độ Celsius:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Lỗi đọc input");

    let celsius: f64 = input.trim()
        .parse()
        .expect("Vui lòng nhập số!");

    let fahrenheit = celsius * FACTOR + OFFSET;
    println!("{}°C = {:.2}°F", celsius, fahrenheit);
}
```

---

## Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu sự khác biệt giữa immutable và mutable
- [ ] Nắm vững các kiểu dữ liệu nguyên thủy
- [ ] Phân biệt được type annotation và type inference
- [ ] Hiểu shadowing và khi nào nên dùng
- [ ] Hoàn thành ít nhất 2 bài tập

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Tại sao Rust mặc định biến là immutable?
2. **Ứng dụng**: Khi nào dùng `const` vs `static`?
3. **Phân tích**: Shadowing khác `mut` như thế nào?
4. **Thực hành**: Demo chương trình chuyển đổi nhiệt độ.

### 4.3. Tài Liệu Tham Khảo

- [The Rust Book - Chapter 3](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html)
- [Rust by Example - Variables](https://doc.rust-lang.org/rust-by-example/variable_bindings.html)

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Khi nào nên dùng mut vs shadowing?</strong></summary>

- Dùng `mut` khi chỉ thay đổi giá trị, giữ nguyên kiểu
- Dùng shadowing khi cần thay đổi kiểu hoặc muốn biến mới immutable

</details>

<details>
<summary><strong>Q2: isize và usize dùng khi nào?</strong></summary>

Chủ yếu dùng cho indexing collections và pointer arithmetic. Kích thước phụ thuộc vào kiến trúc máy (32-bit hoặc 64-bit).

</details>

<details>
<summary><strong>Q3: Có thể khai báo biến không khởi tạo không?</strong></summary>

Có, nhưng phải có type annotation và phải khởi tạo trước khi sử dụng:

```rust
let x: i32;
x = 5;
println!("{}", x);
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Rust Programming | **Lesson**: 2

</footer>
