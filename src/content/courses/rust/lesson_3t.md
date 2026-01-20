# Kiểu Dữ Liệu và Biến Nâng Cao

> **Mô tả ngắn gọn**: Đi sâu vào cơ chế quản lý biến, shadowing, constants, static variables và các kỹ thuật xử lý kiểu dữ liệu nâng cao trong Rust.

## Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Áp dụng thành thạo cơ chế khai báo biến với `let` và `mut`
- [ ] Hiểu sâu về khái niệm bất biến (immutability) và lợi ích của nó
- [ ] Phân biệt và sử dụng đúng constants, static variables
- [ ] Áp dụng shadowing hiệu quả trong các tình huống thực tế
- [ ] Nhận diện và khắc phục các lỗi phổ biến liên quan đến quản lý biến

### Kiến Thức Yêu Cầu

- Hoàn thành Bài 2: Kiểu dữ liệu cơ bản
- Nắm vững cú pháp khai báo biến cơ bản
- Hiểu các kiểu dữ liệu nguyên thủy

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Immutability và Mutability | 15 phút |
| 2 | Constants, Static và Shadowing | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## Phần 1: Kiến Thức Nền Tảng

### 1.1. Tính Bất Biến (Immutability)

> **Định nghĩa**: Trong Rust, tất cả biến mặc định là bất biến - một thiết kế có chủ đích để đảm bảo an toàn.

**Tại sao Rust chọn mặc định là bất biến?**

- **An toàn bộ nhớ**: Ngăn chặn thay đổi không mong muốn
- **Ngăn chặn lỗi**: Compiler bắt lỗi tại compile-time
- **Tối ưu hóa**: Compiler có thể optimize tốt hơn
- **Lập trình đồng thời**: Tránh data races

```rust
let x = 5;
// x = 6; // Lỗi: cannot assign twice to immutable variable
println!("x = {}", x);
```

**Giải thích:**

- Biến `x` được khai báo không có `mut`
- Cố gắng thay đổi sẽ gây lỗi compile-time
- Đây là tính năng an toàn quan trọng của Rust

### 1.2. Biến Mutable

```rust
let mut y = 5;
println!("y trước: {}", y);
y = 10;
println!("y sau: {}", y); // 10
```

**Nguyên tắc sử dụng `mut`:**

- Chỉ dùng khi thực sự cần thay đổi giá trị
- Không thể thay đổi kiểu dữ liệu
- Giúp code an toàn và dễ đọc hơn

### 1.3. Biểu Diễn Số Nguyên

Rust hỗ trợ nhiều cách biểu diễn số:

```rust
let decimal = 98_222;      // Thập phân (underscore để dễ đọc)
let hex = 0xff;            // Thập lục phân
let octal = 0o77;          // Bát phân
let binary = 0b1111_0000;  // Nhị phân
let byte = b'A';           // Byte (u8)
```

**Giải thích:**

- Dấu `_` giúp số dễ đọc hơn, không ảnh hưởng giá trị
- `0x` prefix cho hex, `0o` cho octal, `0b` cho binary
- `b'char'` cho giá trị ASCII

---

## Phần 2: Phân Tích & Tư Duy

### 2.1. Constants vs Static Variables

**Scenario**: Bạn cần lưu trữ giá trị cấu hình cho ứng dụng...

| Đặc điểm | Constants | Static Variables |
|----------|-----------|------------------|
| Khai báo | `const MAX: u32 = 100;` | `static MSG: &str = "Hi";` |
| Bất biến | Luôn luôn | Có thể mutable (unsafe) |
| Địa chỉ bộ nhớ | Không cố định (inline) | Cố định |
| Lifetime | Compile-time | Runtime |
| Use case | Giá trị cấu hình | Shared state |

```rust
// Constants - thường được inline
const MAX_USERS: u32 = 100_000;
const PI: f64 = 3.14159265359;

// Static - có địa chỉ bộ nhớ cố định
static PROGRAM_NAME: &str = "Rust Demo";
```

**Câu hỏi suy ngẫm:**

1. Khi nào dùng `const` vs `static`?
2. Tại sao `static mut` là unsafe?
3. Có thể tính toán expression trong `const` không?

<details>
<summary>Gợi ý phân tích</summary>

- `const`: Dùng cho compile-time constants, thường được inline
- `static`: Dùng khi cần địa chỉ bộ nhớ cố định hoặc shared state
- `static mut` unsafe vì có thể gây data race trong multi-threading
- `const` chỉ hỗ trợ const expressions (compile-time evaluable)

</details>

### 2.2. Shadowing Deep Dive

> **Lưu ý quan trọng**: Shadowing tạo biến MỚI, không phải thay đổi biến cũ!

#### Nên Làm

```rust
// Thay đổi kiểu dữ liệu với shadowing
let spaces = "   ";        // &str
let spaces = spaces.len(); // usize

// Transformation pipeline
let input = "  42  ";
let input = input.trim();        // &str
let input: i32 = input.parse().unwrap(); // i32
```

**Tại sao tốt:**

- Giữ nguyên tên biến có ý nghĩa
- Cho phép thay đổi kiểu dữ liệu
- Biến mới vẫn immutable

#### Không Nên Làm

```rust
let mut word = "hello";
// word = word.len(); // Lỗi! Không thể thay đổi kiểu
```

**Tại sao không tốt:**

- `mut` không cho phép thay đổi kiểu
- Sử dụng sai mục đích của `mut`

### 2.3. Phạm Vi Shadowing

```rust
let x = 5;
{
    let x = 12;
    println!("x trong block: {}", x); // 12
}
println!("x ngoài block: {}", x); // 5
```

**Key insight:** Shadowing chỉ có hiệu lực trong scope khai báo.

### 2.4. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Integer overflow | Vượt phạm vi kiểu | Dùng kiểu lớn hơn hoặc checked ops |
| Type mismatch với mut | Cố thay đổi kiểu | Dùng shadowing thay vì mut |
| Unsafe static mut | Không hiểu data race | Dùng Mutex hoặc atomic types |
| Lạm dụng global state | Static mut everywhere | Refactor sang struct/module |

---

## Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Nắm vững sự khác biệt giữa immutable, mutable và shadowing

#### Bước 1: Tạo project

```bash
cargo new variables_advanced
cd variables_advanced
```

#### Bước 2: Demo immutable vs mutable

```rust
fn main() {
    // Immutable
    let x = 5;
    println!("x = {}", x);
    // x = 6; // Uncomment để thấy lỗi

    // Mutable
    let mut y = 10;
    println!("y trước: {}", y);
    y = 15;
    println!("y sau: {}", y);
}
```

#### Bước 3: Demo shadowing

```rust
fn main() {
    // Shadowing thay đổi kiểu
    let spaces = "   ";
    println!("spaces: '{}' (kiểu &str)", spaces);

    let spaces = spaces.len();
    println!("spaces: {} (kiểu usize)", spaces);

    // So sánh với mut
    let mut word = "hello";
    word = "world"; // OK - cùng kiểu
    // word = word.len(); // Lỗi - khác kiểu

    let word = word.len(); // OK với shadowing
    println!("word length: {}", word);
}
```

### 3.2. Bài Tập Tự Luyện

#### Cấp độ Cơ Bản

**Bài tập 1**: Các kiểu số nguyên và phạm vi

<details>
<summary>Gợi ý</summary>

Sử dụng type annotation và thử các giá trị max của mỗi kiểu.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn main() {
    let a: i8 = 127;
    let b: u8 = 255;
    let c: i32 = 2_147_483_647;
    let d: u32 = 4_294_967_295;

    println!("i8 max: {}", a);
    println!("u8 max: {}", b);
    println!("i32 max: {}", c);
    println!("u32 max: {}", d);

    // Các cách biểu diễn
    println!("Hex 0xff = {}", 0xff);
    println!("Octal 0o77 = {}", 0o77);
    println!("Binary 0b1111 = {}", 0b1111);
}
```

</details>

**Bài tập 2**: Constants và Static

<details>
<summary>Gợi ý</summary>

Khai báo const ở global scope và sử dụng trong function.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
const MAX_USERS: u32 = 100_000;
const PI: f64 = 3.14159265359;
static PROGRAM_NAME: &str = "Rust Demo";

fn main() {
    println!("Program: {}", PROGRAM_NAME);
    println!("Max users: {}", MAX_USERS);

    let radius = 5.0;
    let area = PI * radius * radius;
    println!("Circle area (r={}): {:.2}", radius, area);
}
```

</details>

#### Cấp độ Nâng Cao

**Bài tập 3**: Static mutable (unsafe)

<details>
<summary>Giải pháp mẫu</summary>

```rust
static mut COUNTER: u32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;
    }
}

fn get_count() -> u32 {
    unsafe { COUNTER }
}

fn main() {
    println!("Initial: {}", get_count());
    increment();
    increment();
    println!("After 2 increments: {}", get_count());
}
```

**Lưu ý:** Tránh `static mut` trong production. Dùng `Mutex` hoặc `AtomicU32` thay thế.

</details>

### 3.3. Mini Project

**Dự án**: Temperature & Geometry Calculator

**Mô tả**: Xây dựng công cụ tính toán nhiệt độ và diện tích hình học.

**Yêu cầu chức năng:**

1. Chuyển đổi Celsius ↔ Fahrenheit
2. Tính diện tích hình tròn, tam giác, chữ nhật
3. Sử dụng constants cho hằng số toán học
4. Áp dụng shadowing cho data transformation

**Hướng dẫn triển khai:**

```rust
use std::io;

// Constants
const CELSIUS_TO_FAHRENHEIT_FACTOR: f64 = 9.0 / 5.0;
const FAHRENHEIT_OFFSET: f64 = 32.0;
const PI: f64 = 3.14159265359;

fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * CELSIUS_TO_FAHRENHEIT_FACTOR + FAHRENHEIT_OFFSET
}

fn circle_area(radius: f64) -> f64 {
    PI * radius * radius
}

fn rectangle_area(width: f64, height: f64) -> f64 {
    width * height
}

fn triangle_area(base: f64, height: f64) -> f64 {
    0.5 * base * height
}

fn main() {
    // Temperature conversion với shadowing
    let input = "25";
    let input: f64 = input.parse().unwrap();
    let fahrenheit = celsius_to_fahrenheit(input);
    println!("{}°C = {:.2}°F", input, fahrenheit);

    // Geometry calculations
    let radius = 5.0;
    println!("Circle area (r={}): {:.2}", radius, circle_area(radius));

    let width = 4.0;
    let height = 6.0;
    println!("Rectangle area ({}x{}): {:.2}", width, height,
             rectangle_area(width, height));

    let base = 8.0;
    let tri_height = 5.0;
    println!("Triangle area (b={}, h={}): {:.2}", base, tri_height,
             triangle_area(base, tri_height));
}
```

---

## Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu sâu về immutability và lợi ích của nó
- [ ] Phân biệt được `mut` vs shadowing
- [ ] Sử dụng thành thạo constants và static
- [ ] Biết khi nào cần unsafe với static mut
- [ ] Hoàn thành ít nhất 2 bài tập

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Tại sao Rust mặc định immutable? Lợi ích gì?
2. **Ứng dụng**: Khi nào dùng shadowing vs `mut`?
3. **Phân tích**: `const` vs `static` khác nhau thế nào?
4. **Thực hành**: Demo chương trình với shadowing thay đổi kiểu.

### 4.3. Tài Liệu Tham Khảo

- [The Rust Book - Variables](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
- [Rust Reference - Static Items](https://doc.rust-lang.org/reference/items/static-items.html)

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Tại sao không dùng static mut?</strong></summary>

`static mut` có thể gây data race trong multi-threaded code. Thay vào đó:
- Dùng `Mutex<T>` cho mutable shared state
- Dùng `AtomicU32`, `AtomicBool` cho primitive types
- Dùng `lazy_static!` hoặc `once_cell` cho lazy initialization

</details>

<details>
<summary><strong>Q2: Shadowing có tốn thêm bộ nhớ không?</strong></summary>

Không nhất thiết. Rust compiler thường tối ưu và reuse memory khi có thể. Biến cũ bị drop khi bị shadow (nếu không còn reference).

</details>

<details>
<summary><strong>Q3: Có thể const function trong Rust không?</strong></summary>

Có! Từ Rust 1.31+, bạn có thể đánh dấu function là `const fn`:

```rust
const fn add(a: i32, b: i32) -> i32 {
    a + b
}
const RESULT: i32 = add(5, 3);
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Rust Programming | **Lesson**: 3

</footer>
