# Lifetime trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu về lifetime - cách Rust đảm bảo references luôn hợp lệ và không trỏ đến dữ liệu đã bị giải phóng.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu rõ khái niệm lifetime và vai trò của nó
- [ ] Biết cách sử dụng lifetime annotations
- [ ] Nắm được lifetime elision rules
- [ ] Hiểu về static lifetime và các trường hợp sử dụng

### Kiến Thức Yêu Cầu

- Ownership và borrowing (Bài 6, 7)
- References và slices (Bài 7, 8)
- Cơ bản về structs trong Rust

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Lifetime | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 20 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Lifetime là "thời gian sống" của một reference, được Rust sử dụng để đảm bảo references luôn hợp lệ.

**Tại sao điều này quan trọng?**

- Ngăn ngừa dangling references (trỏ đến dữ liệu đã giải phóng)
- Đảm bảo memory safety tại compile time
- Không cần runtime checks, hiệu suất cao

### 1.2. Kiến Thức Cốt Lõi

#### Vấn đề cần giải quyết

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x; // `x` không tồn tại đủ lâu!
    }
    println!("r: {}", r); // LỖI: borrowed value does not live long enough
}
```

**📝 Giải thích:**
- `x` bị hủy khi ra khỏi block `{}`
- `r` vẫn trỏ đến vị trí bộ nhớ của `x` (dangling reference)
- Rust ngăn chặn điều này tại compile time

#### Cách Rust giải quyết

Rust sử dụng **borrow checker** để theo dõi lifetime của references và đảm bảo:
- Reference không sống lâu hơn dữ liệu nó tham chiếu
- Dữ liệu không bị giải phóng khi còn reference đến nó

#### Lifetime Annotations

Cú pháp: `'a`, `'b`, `'c`, ... (tick + chữ cái)

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

**📝 Giải thích:**
- `<'a>`: Khai báo generic lifetime parameter
- `&'a str`: Reference với lifetime `'a`
- Giá trị trả về có cùng lifetime với inputs

#### Lifetime trong Structs

```rust
struct BookExcerpt<'a> {
    content: &'a str,
}

impl<'a> BookExcerpt<'a> {
    fn get_first_line(&self) -> &str {
        self.content.lines().next().unwrap_or("")
    }
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let excerpt = BookExcerpt {
        content: &novel[..],
    };

    println!("First line: {}", excerpt.get_first_line());
}
```

**📝 Giải thích:**
- Struct chứa reference cần lifetime annotation
- `BookExcerpt<'a>` chứa reference sống ít nhất `'a`
- excerpt không thể outlive novel

#### Lifetime Elision Rules

Rust có 3 quy tắc tự động suy luận lifetime:

**Rule 1**: Mỗi reference parameter được gán lifetime riêng

```rust
fn foo(x: &i32)              // thực tế: fn foo<'a>(x: &'a i32)
fn foo(x: &i32, y: &i32)     // thực tế: fn foo<'a, 'b>(x: &'a i32, y: &'b i32)
```

**Rule 2**: Nếu có một input lifetime, nó được gán cho tất cả output

```rust
fn first_word(s: &str) -> &str  // thực tế: fn first_word<'a>(s: &'a str) -> &'a str
```

**Rule 3**: Nếu có `&self` hoặc `&mut self`, lifetime của self được gán cho output

```rust
impl<'a> BookExcerpt<'a> {
    fn get_content(&self) -> &str {  // thực tế: -> &'a str
        self.content
    }
}
```

#### Static Lifetime

`'static` là lifetime tồn tại trong suốt chương trình:

```rust
// String literals có 'static lifetime
let s: &'static str = "I have a static lifetime.";

// Có thể sử dụng trong cả chương trình
fn get_static() -> &'static str {
    "Hello, world!"
}
```

> **⚠️ Lưu ý**: Không nên lạm dụng `'static` để "fix" lỗi lifetime.

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | Không annotation | Có annotation |
|----------|-----------------|---------------|
| Một input ref | Tự động | Không cần |
| Nhiều input refs | Cần chỉ định | Bắt buộc |
| Struct với ref | Bắt buộc | Bắt buộc |
| Return ref | Tùy thuộc rules | Khi cần rõ ràng |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Viết hàm trả về chuỗi dài hơn giữa hai inputs

**Yêu cầu**:
- Nhận hai string slices
- Trả về slice dài hơn
- Không copy dữ liệu

**🤔 Câu hỏi suy ngẫm:**

1. Tại sao cần lifetime annotation ở đây?
2. Lifetime của output phụ thuộc vào gì?
3. Điều gì xảy ra nếu hai inputs có lifetime khác nhau?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
// Không có annotation - LỖI
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}
// Error: missing lifetime specifier

// Có annotation - OK
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**Tại sao cần annotation:**
- Compiler không biết output là x hay y
- Không thể suy luận lifetime của output
- Phải chỉ định rõ: output sống ít nhất bằng min(lifetime x, lifetime y)

**Với lifetimes khác nhau:**
```rust
fn main() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(&string1, &string2);
        println!("Longest: {}", result); // OK trong block này
    }
    // println!("Longest: {}", result); // LỖI nếu uncomment
    // string2 đã hết scope, result có thể trỏ đến nó
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Lifetime annotations không thay đổi thời gian sống của references, chúng chỉ mô tả mối quan hệ.

#### ✅ Nên Làm

```rust
// Sử dụng elision rules khi có thể
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// Chỉ annotate khi cần thiết
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct với reference - rõ ràng về lifetime
struct Parser<'a> {
    input: &'a str,
}
```

**Tại sao tốt:**
- Code sạch, dễ đọc
- Chỉ annotate khi compiler yêu cầu
- Mối quan hệ lifetime rõ ràng

#### ❌ Không Nên Làm

```rust
// Thêm 'static để "fix" lỗi
fn bad_idea() -> &'static str {
    let s = String::from("hello");
    &s // Vẫn LỖI - 'static không giúp gì
}

// Quá nhiều lifetime parameters
fn overly_complex<'a, 'b, 'c>(
    x: &'a str,
    y: &'b str,
    z: &'c str
) -> &'a str {
    x // Chỉ cần 'a nếu chỉ return x
}
```

**Tại sao không tốt:**
- `'static` không sửa được lifetime thực tế của dữ liệu
- Lifetime parameters thừa gây confusion

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "missing lifetime specifier" | Cần annotation | Thêm lifetime parameter |
| "borrowed value does not live long enough" | Reference outlives data | Mở rộng scope của data |
| "lifetime mismatch" | Output lifetime không match | Kiểm tra lại annotations |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng struct và functions với lifetime annotations

**Yêu cầu kỹ thuật:**
- Struct chứa references
- Methods trả về references
- Hàm với multiple lifetime parameters

#### Bước 1: Struct với lifetime

```rust
struct BookExcerpt<'a> {
    content: &'a str,
}

impl<'a> BookExcerpt<'a> {
    fn new(content: &'a str) -> Self {
        BookExcerpt { content }
    }

    fn get_first_sentence(&self) -> &str {
        match self.content.find('.') {
            Some(idx) => &self.content[..=idx],
            None => self.content,
        }
    }
}
```

**Giải thích:**
- `BookExcerpt<'a>` chứa reference với lifetime `'a`
- `new` nhận reference và trả về struct
- `get_first_sentence` trả về slice từ content

#### Bước 2: Hàm với lifetime

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let string1 = String::from("abcd");
    let string2 = "xyz";

    let result = longest(string1.as_str(), string2);
    println!("The longest string is: {}", result);
}
```

#### Bước 3: Multiple lifetimes

```rust
fn first_char_of_first<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str {
    &x[0..1]
}

fn main() {
    let s1 = String::from("hello");
    let s2 = String::from("world");

    let result = first_char_of_first(&s1, &s2);
    println!("First char: {}", result);
}
```

**Giải thích:**
- Hai lifetime parameters khác nhau
- Output chỉ phụ thuộc vào `'a` (x)
- `'b` độc lập với output

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Sửa lỗi lifetime trong code sau

```rust
fn main() {
    let result;
    {
        let s = String::from("hello");
        result = &s;
    }
    println!("{}", result);
}
```

<details>
<summary>💡 Gợi ý</summary>

`s` bị drop trước khi `result` được sử dụng. Cần mở rộng scope của `s`.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
fn main() {
    // Cách 1: Mở rộng scope của s
    let s = String::from("hello");
    let result = &s;
    println!("{}", result);

    // Cách 2: Sử dụng owned value thay vì reference
    let result2: String;
    {
        let s2 = String::from("world");
        result2 = s2; // Move, không phải borrow
    }
    println!("{}", result2);
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Viết struct Document chứa references

```rust
struct Document<'a> {
    title: &'a str,
    content: &'a str,
    author: &'a str,
}

impl<'a> Document<'a> {
    fn new(title: &'a str, content: &'a str, author: &'a str) -> Self {
        // Implement
    }

    fn summary(&self) -> String {
        // Return: "Title by Author"
    }

    fn word_count(&self) -> usize {
        // Count words in content
    }
}
```

**Mở rộng**:
- Thêm method `contains` kiểm tra keyword
- Thêm method trả về first paragraph

### 3.3. Mini Project

**Dự án**: Text Parser với Lifetime

**Mô tả**: Xây dựng parser đơn giản cho text có cấu trúc

**Yêu cầu chức năng:**

1. Parse key-value pairs từ text
2. Giữ references đến original text
3. Cung cấp methods để query

**Technical Stack:**
- Structs với lifetime
- Iterator methods
- Pattern matching

**Hướng dẫn triển khai:**

```rust
struct KeyValueParser<'a> {
    input: &'a str,
    pairs: Vec<(&'a str, &'a str)>,
}

impl<'a> KeyValueParser<'a> {
    fn new(input: &'a str) -> Self {
        let mut pairs = Vec::new();

        for line in input.lines() {
            if let Some(idx) = line.find(':') {
                let key = line[..idx].trim();
                let value = line[idx + 1..].trim();
                pairs.push((key, value));
            }
        }

        KeyValueParser { input, pairs }
    }

    fn get(&self, key: &str) -> Option<&'a str> {
        self.pairs
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }

    fn keys(&self) -> Vec<&'a str> {
        self.pairs.iter().map(|(k, _)| *k).collect()
    }

    fn values(&self) -> Vec<&'a str> {
        self.pairs.iter().map(|(_, v)| *v).collect()
    }
}

fn main() {
    let config = "
name: Rust App
version: 1.0.0
author: Developer
";

    let parser = KeyValueParser::new(config);

    println!("Name: {:?}", parser.get("name"));
    println!("Version: {:?}", parser.get("version"));
    println!("Keys: {:?}", parser.keys());
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu vấn đề lifetime giải quyết (dangling references)
- [ ] Biết cách viết lifetime annotations
- [ ] Nắm được 3 elision rules
- [ ] Hoàn thành bài tập hướng dẫn
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện
- [ ] (Tùy chọn) Hoàn thành mini project Parser

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Lifetime annotations có thay đổi thời gian sống của references không?
2. **Ứng dụng**: Khi nào cần viết lifetime annotations?
3. **Phân tích**: Giải thích 3 elision rules?
4. **Thực hành**: Demo KeyValueParser?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Vấn đề dangling reference và cách Rust giải quyết
- Demo struct với lifetime
- So sánh với C++ (raw pointers)
- Các lỗi thường gặp và cách debug

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Lifetime annotations dùng để làm gì?

- A. Thay đổi thời gian sống của variables
- B. Mô tả mối quan hệ giữa lifetimes của references
- C. Tăng hiệu suất chương trình
- D. Giảm bộ nhớ sử dụng

**Câu 2**: `'static` lifetime có nghĩa là gì?

- A. Variable không thể thay đổi
- B. Reference tồn tại trong suốt chương trình
- C. Variable được lưu trên stack
- D. Reference được lưu trên heap

**Câu 3**: Khi nào compiler tự động suy luận lifetime?

- A. Không bao giờ
- B. Khi có một input reference
- C. Khi có &self trong method
- D. Cả B và C

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Khi nào cần viết lifetime annotations?</strong></summary>

Cần viết khi:
1. Hàm có nhiều reference inputs và trả về reference
2. Struct chứa references
3. Compiler báo lỗi "missing lifetime specifier"

Không cần viết khi:
1. Hàm có một reference input (Rule 2 áp dụng)
2. Method có &self (Rule 3 áp dụng)

</details>

<details>
<summary><strong>Q2: Tại sao không dùng 'static cho mọi thứ?</strong></summary>

`'static` yêu cầu dữ liệu tồn tại suốt chương trình:
- Chỉ string literals và constants thực sự có `'static`
- Không thể biến một String thành `&'static str`
- Lạm dụng `'static` sẽ gây lỗi compile hoặc memory leaks

```rust
// Sai - String không phải 'static
fn bad() -> &'static str {
    let s = String::from("hello");
    &s // Vẫn LỖI
}

// Đúng - string literal là 'static
fn good() -> &'static str {
    "hello"
}
```

</details>

<details>
<summary><strong>Q3: Làm sao debug lỗi lifetime?</strong></summary>

1. Đọc kỹ error message - Rust compiler rất chi tiết
2. Sử dụng `rustc --explain EXXXX` để xem giải thích
3. Vẽ diagram lifetime của các references
4. Kiểm tra xem data có sống đủ lâu không
5. Thử mở rộng scope của data

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
