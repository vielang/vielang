# Borrowing và References trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu cách mượn dữ liệu thông qua references mà không chuyển quyền sở hữu, một kỹ thuật quan trọng để viết code Rust hiệu quả.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu được khái niệm references và borrowing trong Rust
- [ ] Phân biệt được immutable references và mutable references
- [ ] Nắm vững các quy tắc borrowing của Rust
- [ ] Nhận biết và tránh dangling references

### Kiến Thức Yêu Cầu

- Hiểu về ownership trong Rust (Bài 6)
- Khái niệm về Stack và Heap
- Cơ bản về kiểu dữ liệu String

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về References | 15 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: References là cách truy cập dữ liệu mà không cần sở hữu nó. Borrowing là hành động tạo reference đến dữ liệu.

**Tại sao điều này quan trọng?**

- Cho phép nhiều phần của code truy cập dữ liệu mà không chuyển ownership
- Tránh clone dữ liệu không cần thiết, tăng hiệu suất
- Đảm bảo an toàn bộ nhớ thông qua quy tắc borrowing

### 1.2. Kiến Thức Cốt Lõi

#### References cơ bản

Sử dụng ký hiệu `&` để tạo reference đến một giá trị:

```rust
fn main() {
    let s1 = String::from("xin chào");

    // Chúng ta "mượn" s1 qua reference
    let len = calculate_length(&s1);

    println!("Độ dài của '{}' là {}.", s1, len);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

**📝 Giải thích:**
- `&s1` tạo một reference đến s1 mà không lấy ownership
- Hàm `calculate_length` nhận reference (`&String`)
- Sau khi gọi hàm, s1 vẫn hợp lệ

#### Immutable References (&T)

```rust
fn main() {
    let s = String::from("xin chào");

    let r1 = &s; // immutable reference 1
    let r2 = &s; // immutable reference 2

    println!("{} và {}", r1, r2);
    // Hợp lệ vì có thể có nhiều immutable references
}
```

**Đặc điểm:**
- Cho phép đọc dữ liệu nhưng không thay đổi
- Có thể có nhiều immutable references cùng một lúc

#### Mutable References (&mut T)

```rust
fn main() {
    let mut s = String::from("xin chào");

    let r = &mut s; // mutable reference

    r.push_str(", thế giới");

    println!("{}", r); // xin chào, thế giới
}
```

**Đặc điểm:**
- Cho phép đọc và thay đổi dữ liệu
- Chỉ có thể có duy nhất một mutable reference tại một thời điểm

#### Hai quy tắc Borrowing

**Quy tắc 1**: Tại một thời điểm, bạn có thể có:
- Nhiều immutable references (`&T`)
- HOẶC chính xác một mutable reference (`&mut T`)

**Quy tắc 2**: References phải luôn hợp lệ - không được tồn tại reference đến dữ liệu đã bị hủy

```rust
// Ví dụ vi phạm quy tắc 1
fn main() {
    let mut s = String::from("xin chào");

    let r1 = &s;      // immutable reference
    let r2 = &s;      // immutable reference
    let r3 = &mut s;  // LỖI: không thể có mutable reference
                      // khi đã tồn tại immutable references

    println!("{}, {}, và {}", r1, r2, r3);
}
```

#### Phạm vi của References (Non-Lexical Lifetimes)

```rust
fn main() {
    let mut s = String::from("xin chào");

    let r1 = &s;
    let r2 = &s;
    println!("{} và {}", r1, r2);
    // r1 và r2 không còn được sử dụng sau đây

    let r3 = &mut s; // OK - r1, r2 đã "hết phạm vi sử dụng"
    println!("{}", r3);
}
```

#### Dangling References

> **⚠️ Dangling reference**: Reference trỏ đến dữ liệu đã bị giải phóng

```rust
// Code này sẽ không compile
fn dangle() -> &String {
    let s = String::from("xin chào");
    &s // LỖI: s sẽ bị giải phóng khi hàm kết thúc!
}

// Cách sửa: trả về ownership thay vì reference
fn no_dangle() -> String {
    let s = String::from("xin chào");
    s // Trả về giá trị, ownership được chuyển
}
```

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | Immutable Ref (&T) | Mutable Ref (&mut T) |
|----------|-------------------|----------------------|
| Cú pháp | `&value` | `&mut value` |
| Đọc dữ liệu | Có | Có |
| Thay đổi dữ liệu | Không | Có |
| Số lượng cùng lúc | Nhiều | Một |
| Yêu cầu biến gốc | Không cần mut | Cần mut |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng hàm thêm hậu tố vào chuỗi và một hàm khác để đọc nội dung

**Yêu cầu**:
- Hàm `add_suffix` thay đổi chuỗi gốc
- Hàm `display` chỉ hiển thị chuỗi
- Sử dụng references phù hợp

**🤔 Câu hỏi suy ngẫm:**

1. Hàm nào cần mutable reference?
2. Hàm nào chỉ cần immutable reference?
3. Có thể gọi cả hai hàm cùng lúc không?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
// Cần mutable reference vì thay đổi dữ liệu
fn add_suffix(s: &mut String) {
    s.push_str(", Rust!");
}

// Chỉ cần immutable reference vì chỉ đọc
fn display(s: &String) {
    println!("Content: {}", s);
}

fn main() {
    let mut greeting = String::from("Xin chào");

    // Gọi tuần tự - OK
    display(&greeting);     // immutable borrow
    add_suffix(&mut greeting); // mutable borrow
    display(&greeting);     // immutable borrow

    // Không thể gọi đồng thời với mutable borrow!
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: References giúp tái sử dụng dữ liệu mà không chuyển ownership.

#### ✅ Nên Làm

```rust
// Sử dụng &str thay vì &String cho tham số
fn process(s: &str) {
    println!("Processing: {}", s);
}

// Sử dụng immutable reference khi chỉ đọc
fn calculate_stats(data: &Vec<i32>) -> (i32, i32) {
    let sum: i32 = data.iter().sum();
    let count = data.len() as i32;
    (sum, count)
}
```

**Tại sao tốt:**
- &str linh hoạt hơn, chấp nhận cả String và string literal
- Immutable reference cho phép nhiều reader cùng lúc

#### ❌ Không Nên Làm

```rust
// Lấy ownership khi chỉ cần đọc
fn bad_display(s: String) {
    println!("{}", s);
    // s bị drop, caller không thể dùng nữa
}

// Dùng mutable reference khi không cần thiết
fn bad_read(data: &mut Vec<i32>) -> i32 {
    data[0] // Chỉ đọc, không cần mut
}
```

**Tại sao không tốt:**
- Caller mất ownership không cần thiết
- Mutable reference ngăn concurrent access

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "cannot borrow as mutable" | Có immutable ref đang active | Kết thúc sử dụng immutable trước |
| "borrowed value does not live long enough" | Reference outlives data | Đảm bảo data sống đủ lâu |
| "cannot move out of borrowed content" | Cố move từ reference | Sử dụng clone() hoặc to_owned() |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng các hàm xử lý chuỗi sử dụng references đúng cách

**Yêu cầu kỹ thuật:**
- Viết hàm đọc chuỗi (immutable)
- Viết hàm thay đổi chuỗi (mutable)
- Đảm bảo có thể sử dụng lại sau khi gọi hàm

#### Bước 1: Hàm đọc với immutable reference

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

fn main() {
    let s = String::from("Xin chào");
    let len = calculate_length(&s);
    println!("Độ dài của '{}' là {}", s, len);
}
```

**Giải thích:**
- `&s` tạo reference, không chuyển ownership
- s vẫn hợp lệ sau khi gọi hàm

#### Bước 2: Hàm thay đổi với mutable reference

```rust
fn add_suffix(s: &mut String, suffix: &str) {
    s.push_str(suffix);
}

fn main() {
    let mut greeting = String::from("Xin chào");

    add_suffix(&mut greeting, ", Rust!");

    println!("{}", greeting); // Xin chào, Rust!
}
```

**Giải thích:**
- `&mut s` cho phép thay đổi nội dung
- Biến gốc phải được khai báo với `mut`

#### Bước 3: Kết hợp cả hai

```rust
fn display(s: &str) {
    println!("Content: {}", s);
}

fn append_world(s: &mut String) {
    s.push_str(" World");
}

fn main() {
    let mut message = String::from("Hello");

    display(&message);        // Hello
    append_world(&mut message);
    display(&message);        // Hello World
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Sửa lỗi borrowing trong code sau

```rust
fn main() {
    let mut s = String::from("xin chào");

    let r1 = &s;
    let r2 = &mut s;

    println!("{} và {}", r1, r2);
}
```

<details>
<summary>💡 Gợi ý</summary>

Không thể có cả immutable và mutable reference cùng lúc. Cần tách scope hoặc kết thúc sử dụng immutable trước.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
fn main() {
    let mut s = String::from("xin chào");

    // Cách 1: Sử dụng tuần tự
    {
        let r1 = &s;
        println!("r1: {}", r1);
    } // r1 hết scope

    let r2 = &mut s;
    r2.push_str(", thế giới");
    println!("r2: {}", r2);

    // Cách 2: Kết thúc sử dụng trước
    let mut s2 = String::from("hello");
    let r1 = &s2;
    println!("r1: {}", r1);
    // r1 không còn được sử dụng sau đây

    let r2 = &mut s2;
    r2.push_str(" world");
    println!("r2: {}", r2);
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Viết hàm tìm từ dài nhất trong chuỗi

```rust
fn find_longest_word(s: &str) -> &str {
    // Implement here
}

fn main() {
    let text = String::from("Rust là ngôn ngữ lập trình tuyệt vời");
    let longest = find_longest_word(&text);
    println!("Từ dài nhất: {}", longest);
}
```

**Mở rộng**:
- Xử lý trường hợp chuỗi rỗng
- Trả về từ đầu tiên nếu có nhiều từ cùng độ dài

### 3.3. Mini Project

**Dự án**: Quản lý sách đơn giản

**Mô tả**: Tạo struct Book và các hàm xử lý sử dụng references

**Yêu cầu chức năng:**

1. Hiển thị thông tin sách (immutable reference)
2. Cập nhật tiêu đề sách (mutable reference)
3. So sánh hai cuốn sách (immutable references)

**Technical Stack:**
- Struct với các field String
- Methods với &self và &mut self
- Functions nhận references

**Hướng dẫn triển khai:**

```rust
struct Book {
    title: String,
    author: String,
    year: u32,
}

impl Book {
    fn new(title: &str, author: &str, year: u32) -> Self {
        Book {
            title: String::from(title),
            author: String::from(author),
            year,
        }
    }

    fn display(&self) {
        println!("{} by {} ({})", self.title, self.author, self.year);
    }

    fn update_title(&mut self, new_title: &str) {
        self.title = String::from(new_title);
    }
}

fn compare_years(book1: &Book, book2: &Book) -> &Book {
    if book1.year > book2.year {
        book1
    } else {
        book2
    }
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu được sự khác biệt giữa ownership và borrowing
- [ ] Phân biệt được immutable và mutable references
- [ ] Nắm vững 2 quy tắc borrowing
- [ ] Hoàn thành bài tập hướng dẫn
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện
- [ ] (Tùy chọn) Hoàn thành mini project Book

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Giải thích sự khác biệt giữa &T và &mut T?
2. **Ứng dụng**: Khi nào bạn chọn reference thay vì ownership?
3. **Phân tích**: Tại sao không thể có mutable và immutable ref cùng lúc?
4. **Thực hành**: Demo struct Book với các methods?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Quy tắc borrowing và ví dụ minh họa
- Demo một bài tập đã hoàn thành
- Những lỗi bạn gặp và cách giải quyết
- So sánh với ngôn ngữ khác (nếu biết)

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Đoạn code nào sau đây hợp lệ?

- A. `let r1 = &s; let r2 = &mut s; println!("{}{}", r1, r2);`
- B. `let r1 = &s; println!("{}", r1); let r2 = &mut s;`
- C. `let r1 = &mut s; let r2 = &mut s; println!("{}", r1);`
- D. `let r1 = &mut s; let r2 = &s; println!("{}{}", r1, r2);`

**Câu 2**: Tại sao cần quy tắc "một mutable XOR nhiều immutable"?

- A. Để code chạy nhanh hơn
- B. Để ngăn ngừa data races
- C. Để tiết kiệm bộ nhớ
- D. Để code dễ đọc hơn

**Câu 3**: Reference nào cho phép thay đổi dữ liệu gốc?

- A. `&T` (immutable reference)
- B. `&mut T` (mutable reference)
- C. Cả hai
- D. Không có reference nào

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Tại sao Rust không cho phép nhiều mutable references?</strong></summary>

Để ngăn ngừa data races. Data race xảy ra khi:
- Hai hoặc nhiều pointers truy cập cùng dữ liệu
- Ít nhất một pointer đang ghi
- Không có cơ chế đồng bộ

Rust ngăn chặn điều này tại compile time bằng quy tắc borrowing.

</details>

<details>
<summary><strong>Q2: Khi nào reference hết "phạm vi"?</strong></summary>

Trong Rust 2018+, phạm vi của reference được tính từ khi tạo đến lần sử dụng cuối cùng (Non-Lexical Lifetimes), không phải đến cuối block.

```rust
let r1 = &s;
println!("{}", r1);
// r1 hết phạm vi ở đây (lần sử dụng cuối)

let r2 = &mut s; // OK
```

</details>

<details>
<summary><strong>Q3: Có thể trả về reference từ function không?</strong></summary>

Có, nhưng phải đảm bảo dữ liệu sống đủ lâu. Không thể trả về reference đến biến local:

```rust
// Lỗi
fn bad() -> &String {
    let s = String::from("hi");
    &s // s bị drop khi hàm kết thúc
}

// OK - trả về reference đến input
fn good<'a>(s: &'a str) -> &'a str {
    s
}
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
