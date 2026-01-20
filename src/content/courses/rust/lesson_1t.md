# Giới Thiệu Rust

> **Mô tả ngắn gọn**: Tìm hiểu nguồn gốc, triết lý và những ưu điểm nổi bật của ngôn ngữ lập trình Rust, cùng hướng dẫn cài đặt môi trường phát triển.

## Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu được nguồn gốc và triết lý của ngôn ngữ lập trình Rust
- [ ] Nắm được các ưu điểm chính của Rust so với các ngôn ngữ khác
- [ ] Xác định được các lĩnh vực phù hợp để áp dụng Rust
- [ ] Cài đặt và cấu hình môi trường phát triển Rust

### Kiến Thức Yêu Cầu

- Hiểu biết cơ bản về lập trình
- Quen thuộc với command line/terminal
- Kiến thức cơ bản về cách hoạt động của compiler

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Lịch sử và triết lý Rust | 15 phút |
| 2 | So sánh và phân tích | 10 phút |
| 3 | Thực hành cài đặt | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## Phần 1: Kiến Thức Nền Tảng

### 1.1. Lịch Sử và Nguồn Gốc

> **Định nghĩa**: Rust là ngôn ngữ lập trình hệ thống tập trung vào safety, concurrency và performance.

**Tại sao Rust ra đời?**

- Nhu cầu về ngôn ngữ an toàn bộ nhớ mà không cần garbage collector
- Giải quyết các vấn đề phổ biến trong C/C++ như buffer overflow, dangling pointers
- Mong muốn có công cụ phát triển hiện đại cho system programming

**Các mốc quan trọng:**

| Năm | Sự kiện |
|-----|---------|
| 2006 | Graydon Hoare bắt đầu phát triển Rust tại Mozilla Research |
| 2015 | Phiên bản 1.0 chính thức ra mắt |
| 2021 | Rust Foundation được thành lập |

### 1.2. Triết Lý Cốt Lõi

Rust xây dựng trên ba giá trị nền tảng:

#### Performance

Rust cho phép kiểm soát chi tiết việc sử dụng tài nguyên, không có runtime overhead.

```rust
// Zero-cost abstractions
fn main() {
    let numbers: Vec<i32> = (1..=10).collect();
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
}
```

#### Reliability

Hệ thống ownership và type system đảm bảo memory safety và thread safety tại compile time.

#### Productivity

Công cụ hiện đại như Cargo, tài liệu tốt và compiler messages hữu ích.

### 1.3. Ưu Điểm Của Rust

#### An toàn về bộ nhớ

- Hệ thống ownership và borrowing ngăn chặn lỗi bộ nhớ tại thời điểm biên dịch
- Không sử dụng garbage collector nhưng vẫn đảm bảo an toàn bộ nhớ

#### Hiệu suất cao

- Tốc độ thực thi tương đương C/C++
- Không có runtime overhead

#### Xử lý đồng thời an toàn

- Mô hình ownership giúp tránh data races tại thời điểm biên dịch

#### Hệ sinh thái hiện đại

- Cargo: công cụ quản lý gói và build system mạnh mẽ

---

## Phần 2: Phân Tích & Tư Duy

### 2.1. So Sánh Với Các Ngôn Ngữ Khác

| Tiêu chí | Rust | C/C++ | Go | Python |
|----------|------|-------|-----|--------|
| Memory Safety | Compile-time | Manual | GC | GC |
| Performance | Rất cao | Rất cao | Cao | Trung bình |
| Concurrency | Fearless | Manual | Goroutines | GIL |
| Learning Curve | Dốc | Dốc | Thoải | Dễ |

### 2.2. Các Lĩnh Vực Ứng Dụng

**Scenario**: Bạn cần chọn ngôn ngữ cho một dự án mới...

**Khi nào chọn Rust?**

- **Phát triển hệ thống**: OS, drivers, embedded systems
- **Web Development**: Backend với Actix, Rocket, Axum
- **Network Programming**: High-performance networking
- **Cloud & Distributed Systems**: Microservices, serverless
- **Game Development**: Game engines, graphics

**Câu hỏi suy ngẫm:**

1. Dự án của bạn có yêu cầu cao về performance không?
2. Memory safety có quan trọng không?
3. Đội ngũ sẵn sàng đầu tư thời gian học Rust chưa?

<details>
<summary>Gợi ý phân tích</summary>

Hãy cân nhắc các yếu tố:

1. Nếu cần performance cao và memory safety → Rust là lựa chọn tốt
2. Nếu cần prototyping nhanh → Python có thể phù hợp hơn
3. Nếu đội đã quen C/C++ → Rust là bước tiến tự nhiên

</details>

### 2.3. Best Practices Cho Người Mới

> **Lưu ý quan trọng**: Ownership là khái niệm cốt lõi cần nắm vững trước.

#### Nên Làm

```rust
// Sử dụng cargo để quản lý project
// cargo new my_project
// cargo build
// cargo run
```

**Tại sao tốt:**

- Cấu trúc project chuẩn
- Quản lý dependencies dễ dàng
- Build system nhất quán

#### Không Nên Làm

```rust
// Biên dịch thủ công từng file
// rustc main.rs
```

**Tại sao không tốt:**

- Khó quản lý dependencies
- Thiếu tính nhất quán
- Không scalable

### 2.4. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Bỏ cuộc sớm | Đường cong học tập dốc | Kiên nhẫn, làm bài tập nhỏ |
| Không đọc compiler errors | Vội vàng | Đọc kỹ error messages |
| Bỏ qua ownership | Tư duy từ ngôn ngữ khác | Học kỹ chapter 4 của Rust Book |

---

## Phần 3: Thực Hành

### 3.1. Cài Đặt Môi Trường

**Mục tiêu**: Cài đặt Rust và tạo chương trình đầu tiên

**Yêu cầu kỹ thuật:**

- Máy tính với Windows, macOS hoặc Linux
- Kết nối internet
- Terminal/Command Prompt

#### Bước 1: Cài đặt Rustup

**Windows:**
Tải xuống và chạy `rustup-init.exe` từ [rustup.rs](https://rustup.rs)

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Giải thích:**
Rustup là trình quản lý phiên bản Rust, giúp cài đặt và cập nhật Rust dễ dàng.

#### Bước 2: Xác nhận cài đặt

```bash
rustc --version
cargo --version
```

**Giải thích:**
- `rustc`: Trình biên dịch Rust
- `cargo`: Trình quản lý gói và dự án

#### Bước 3: Tạo dự án đầu tiên

```bash
cargo new hello_rust
cd hello_rust
cargo run
```

**Cấu trúc dự án:**

```
hello_rust/
├── Cargo.toml    # File cấu hình dự án
└── src/
    └── main.rs   # File mã nguồn chính
```

### 3.2. Bài Tập Tự Luyện

#### Cấp độ Cơ Bản

**Bài tập 1**: Cài đặt Rust và xác nhận phiên bản

<details>
<summary>Gợi ý</summary>

Sử dụng lệnh `rustc --version` và `cargo --version` sau khi cài đặt.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```bash
$ rustc --version
rustc 1.75.0 (82e1608df 2023-12-21)

$ cargo --version
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

</details>

**Bài tập 2**: Tạo chương trình hiển thị lời chào

<details>
<summary>Gợi ý</summary>

Sửa file `src/main.rs` trong project mới tạo.

</details>

<details>
<summary>Giải pháp mẫu</summary>

```rust
fn main() {
    println!("Xin chào từ [Tên của bạn]!");
}
```

</details>

#### Cấp độ Nâng Cao

**Bài tập 3**: Tạo chương trình nhận tên từ input và hiển thị lời chào

**Mở rộng:**

- Thêm thời gian hiện tại vào lời chào
- Xử lý input rỗng

<details>
<summary>Giải pháp mẫu</summary>

```rust
use std::io;

fn main() {
    println!("Nhập tên của bạn:");

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read line");

    let name = name.trim();

    if name.is_empty() {
        println!("Xin chào, người lạ!");
    } else {
        println!("Xin chào, {}!", name);
    }
}
```

</details>

### 3.3. Mini Project

**Dự án**: Greeting CLI Tool

**Mô tả**: Xây dựng công cụ CLI đơn giản nhận arguments và hiển thị lời chào tùy chỉnh.

**Yêu cầu chức năng:**

1. Nhận tên từ command line argument
2. Hiển thị lời chào với thời gian hiện tại
3. Hỗ trợ flag `--formal` cho lời chào trang trọng

**Hướng dẫn triển khai:**

1. Tìm hiểu `std::env::args()`
2. Sử dụng `chrono` crate cho thời gian
3. Parse command line arguments

---

## Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu được lịch sử và triết lý của Rust
- [ ] Phân tích được ưu/nhược điểm so với ngôn ngữ khác
- [ ] Cài đặt thành công môi trường Rust
- [ ] Tạo và chạy chương trình đầu tiên
- [ ] (Tùy chọn) Hoàn thành mini project

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Ba giá trị cốt lõi của Rust là gì?
2. **Ứng dụng**: Khi nào bạn nên chọn Rust thay vì Go?
3. **Phân tích**: Tại sao Rust không cần garbage collector mà vẫn an toàn bộ nhớ?
4. **Thực hành**: Demo chương trình Rust đầu tiên của bạn.

### 4.3. Tài Liệu Tham Khảo

- [The Rust Book](https://doc.rust-lang.org/book) - Tài liệu chính thức
- [Rust by Example](https://doc.rust-lang.org/rust-by-example) - Học qua ví dụ
- [Rustlings](https://github.com/rust-lang/rustlings) - Bài tập tương tác

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Rust có khó học không?</strong></summary>

Rust có đường cong học tập dốc hơn nhiều ngôn ngữ khác, đặc biệt với khái niệm ownership. Tuy nhiên, compiler messages rất hữu ích và tài liệu chất lượng cao.

</details>

<details>
<summary><strong>Q2: Rust có phù hợp cho beginners không?</strong></summary>

Rust có thể là ngôn ngữ đầu tiên, nhưng sẽ challenging. Nếu đã có kinh nghiệm lập trình, việc học Rust sẽ dễ dàng hơn.

</details>

<details>
<summary><strong>Q3: IDE nào tốt nhất cho Rust?</strong></summary>

Visual Studio Code với extension rust-analyzer là lựa chọn phổ biến và miễn phí. IntelliJ với Rust plugin cũng là option tốt.

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**Course**: Rust Programming | **Lesson**: 1

</footer>
