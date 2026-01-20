# Collections - Strings trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu cách Rust xử lý chuỗi với String và &str, đặc biệt là UTF-8 và các phương thức xử lý chuỗi hiệu quả.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu sự khác biệt giữa String và &str
- [ ] Nắm vững cách Rust xử lý UTF-8 và Unicode
- [ ] Thành thạo các phương thức xử lý chuỗi
- [ ] Tối ưu hiệu suất khi làm việc với chuỗi

### Kiến Thức Yêu Cầu

- Ownership và borrowing (Bài 6, 7)
- Slices (Bài 8)
- Vectors (Bài 13)

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Strings | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Rust có hai kiểu chuỗi chính - String (owned, heap-allocated) và &str (borrowed, string slice). Cả hai đều là UTF-8 encoded.

**Tại sao điều này quan trọng?**

- Rust đảm bảo tất cả chuỗi là UTF-8 hợp lệ
- Hiểu rõ ownership giúp viết code hiệu quả
- Unicode support mạnh mẽ cho ứng dụng quốc tế

### 1.2. Kiến Thức Cốt Lõi

#### String vs &str

```rust
// String - owned, mutable, heap-allocated
let mut s1 = String::from("Xin chào");
s1.push_str(" Việt Nam");

// &str - borrowed, immutable, có thể ở bất kỳ đâu
let s2: &str = "Hello"; // String literal - static memory
let s3: &str = &s1;     // Slice của String
```

**📝 Sự khác biệt chính:**

| Tiêu chí | String | &str |
|----------|--------|------|
| Ownership | Owned | Borrowed |
| Bộ nhớ | Heap | Tùy thuộc |
| Mutable | Có (với mut) | Không |
| Khi nào dùng | Cần sở hữu/thay đổi | Chỉ đọc |

#### Khởi tạo String

```rust
// Các cách tạo String
let s1 = String::new();                    // Rỗng
let s2 = String::from("hello");            // Từ literal
let s3 = "hello".to_string();              // Method to_string()
let s4 = String::with_capacity(20);        // Pre-allocate

// Từ bytes (UTF-8)
let bytes = vec![72, 101, 108, 108, 111];
let s5 = String::from_utf8(bytes).unwrap(); // "Hello"
```

#### Thao tác với String

```rust
fn main() {
    let mut s = String::from("Xin ");

    // Thêm vào cuối
    s.push('c');           // Thêm char
    s.push_str("hào");     // Thêm &str

    println!("{}", s);     // "Xin chào"

    // Các phương thức khác
    s.clear();             // Xóa nội dung
    s.truncate(3);         // Giữ 3 bytes đầu
}
```

#### UTF-8 và Unicode

```rust
fn main() {
    let vi_text = "Tiếng Việt";

    // len() trả về số bytes, KHÔNG phải số ký tự
    println!("Bytes: {}", vi_text.len());       // 14
    println!("Chars: {}", vi_text.chars().count()); // 10

    // KHÔNG thể truy cập trực tiếp bằng index
    // let c = vi_text[0]; // Error!

    // Duyệt qua các ký tự
    for c in vi_text.chars() {
        println!("{}", c);
    }

    // Duyệt qua bytes
    for b in vi_text.bytes() {
        println!("{}", b);
    }
}
```

> **⚠️ Lưu ý**: Ký tự Unicode có thể chiếm 1-4 bytes. Slice theo byte index phải đúng ranh giới ký tự.

#### Slicing Strings

```rust
fn main() {
    let s = String::from("Hello World");

    // Slice an toàn (ASCII)
    let hello = &s[0..5];  // "Hello"
    let world = &s[6..];   // "World"

    // Cẩn thận với UTF-8!
    let vi = "Việt Nam";
    // let bad = &vi[0..2]; // Panic! Cắt giữa ký tự

    // An toàn hơn: sử dụng char_indices
    for (i, c) in vi.char_indices() {
        println!("Index {}: '{}'", i, c);
    }
}
```

#### Nối chuỗi

```rust
fn main() {
    let s1 = String::from("Hello");
    let s2 = String::from("World");

    // Toán tử + (s1 bị move)
    let s3 = s1 + " " + &s2;
    // println!("{}", s1); // Error! s1 đã bị move

    // format! macro (không move)
    let s4 = String::from("Hello");
    let s5 = format!("{} {}", s4, s2);
    println!("{}", s4); // OK, s4 vẫn dùng được

    // push_str (hiệu quả nhất)
    let mut s6 = String::from("Hello");
    s6.push_str(" ");
    s6.push_str(&s2);
}
```

#### Các phương thức tìm kiếm và biến đổi

```rust
fn main() {
    let text = "Rust programming language";

    // Tìm kiếm
    println!("Contains 'Rust': {}", text.contains("Rust"));
    println!("Starts with 'Rust': {}", text.starts_with("Rust"));
    println!("Ends with 'age': {}", text.ends_with("age"));

    // Vị trí
    if let Some(pos) = text.find("program") {
        println!("'program' at position {}", pos);
    }

    // Biến đổi
    let trimmed = "  hello  ".trim();
    let replaced = text.replace("Rust", "Go");
    let upper = text.to_uppercase();
    let lower = text.to_lowercase();

    // Tách
    let parts: Vec<&str> = text.split(' ').collect();
    println!("Parts: {:?}", parts);
}
```

### 1.3. So Sánh & Đối Chiếu

| Phương thức | Mô tả | Ví dụ |
|-------------|-------|-------|
| `push_str` | Thêm &str vào cuối | `s.push_str("hi")` |
| `push` | Thêm char vào cuối | `s.push('!')` |
| `+` | Nối (move left operand) | `s1 + &s2` |
| `format!` | Nối (không move) | `format!("{}{}", s1, s2)` |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng Text Analyzer cho văn bản tiếng Việt

**Yêu cầu**:
- Đếm từ, câu, ký tự
- Xử lý đúng Unicode
- Tối ưu hiệu suất

**🤔 Câu hỏi suy ngẫm:**

1. Tại sao `len()` khác `chars().count()`?
2. Làm sao tách từ đúng với tiếng Việt?
3. String hay &str cho tham số hàm?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
struct TextAnalyzer<'a> {
    text: &'a str,
}

impl<'a> TextAnalyzer<'a> {
    fn new(text: &'a str) -> Self {
        TextAnalyzer { text }
    }

    fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    fn char_count(&self, include_whitespace: bool) -> usize {
        if include_whitespace {
            self.text.chars().count()
        } else {
            self.text.chars().filter(|c| !c.is_whitespace()).count()
        }
    }

    fn sentence_count(&self) -> usize {
        self.text
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .count()
    }
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Luôn sử dụng `&str` cho tham số hàm khi chỉ cần đọc.

#### ✅ Nên Làm

```rust
// Nhận &str - linh hoạt
fn process(s: &str) {
    println!("{}", s);
}

// Pre-allocate khi biết trước size
let mut s = String::with_capacity(1000);

// Dùng format! cho complex concatenation
let result = format!("{} {} {}", a, b, c);
```

**Tại sao tốt:**
- &str chấp nhận cả String và string literal
- Pre-allocation tránh reallocations
- format! rõ ràng và không move values

#### ❌ Không Nên Làm

```rust
// Nhận String khi chỉ cần đọc
fn bad_process(s: String) { // Caller phải clone
    println!("{}", s);
}

// Nối chuỗi trong loop
let mut result = String::new();
for i in 0..100 {
    result = result + &i.to_string(); // Nhiều allocations
}

// Clone không cần thiết
let s2 = s1.clone().to_lowercase(); // clone rồi mới lowercase
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Slice panic | Cắt giữa ký tự UTF-8 | Dùng char_indices |
| Nhầm len() | len() trả về bytes | Dùng chars().count() |
| Clone thừa | Không hiểu ownership | Dùng reference |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng Text Analyzer

**Yêu cầu kỹ thuật:**
- Đếm từ, câu, ký tự
- Tìm từ xuất hiện nhiều nhất
- Xử lý UTF-8 đúng cách

#### Bước 1: Struct định nghĩa

```rust
use std::collections::HashMap;

struct TextAnalyzer<'a> {
    text: &'a str,
}
```

#### Bước 2: Implement methods

```rust
impl<'a> TextAnalyzer<'a> {
    fn new(text: &'a str) -> Self {
        TextAnalyzer { text }
    }

    fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    fn sentence_count(&self) -> usize {
        self.text
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .count()
    }

    fn char_count(&self, include_spaces: bool) -> usize {
        if include_spaces {
            self.text.chars().count()
        } else {
            self.text.chars().filter(|c| !c.is_whitespace()).count()
        }
    }

    fn average_word_length(&self) -> f64 {
        let words: Vec<&str> = self.text.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }

        let total: usize = words.iter().map(|w| w.chars().count()).sum();
        total as f64 / words.len() as f64
    }

    fn most_common_words(&self, limit: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();

        for word in self.text.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if !clean.is_empty() {
                *counts.entry(clean).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(limit).collect()
    }
}
```

#### Bước 3: Sử dụng

```rust
fn main() {
    let text = "Rust là ngôn ngữ lập trình. Rust rất mạnh mẽ! Bạn thích Rust không?";

    let analyzer = TextAnalyzer::new(text);

    println!("Words: {}", analyzer.word_count());
    println!("Sentences: {}", analyzer.sentence_count());
    println!("Chars (with spaces): {}", analyzer.char_count(true));
    println!("Chars (no spaces): {}", analyzer.char_count(false));
    println!("Avg word length: {:.2}", analyzer.average_word_length());

    println!("\nTop 3 words:");
    for (word, count) in analyzer.most_common_words(3) {
        println!("  '{}': {} times", word, count);
    }
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Viết hàm đảo ngược từng từ trong câu

```rust
fn reverse_words(s: &str) -> String {
    // "Hello World" -> "olleH dlroW"
    // Implement here
}
```

<details>
<summary>💡 Gợi ý</summary>

Sử dụng `split_whitespace()`, `chars().rev()`, và `collect()`.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
fn reverse_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| word.chars().rev().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Email validator đơn giản

```rust
fn is_valid_email(email: &str) -> bool {
    // Kiểm tra:
    // - Có đúng 1 ký tự @
    // - Có phần local trước @
    // - Có domain sau @ với ít nhất 1 dấu .
    // Implement here
}
```

**Mở rộng**:
- Thêm validation cho local part (không có ký tự đặc biệt)
- Kiểm tra top-level domain hợp lệ

### 3.3. Mini Project

**Dự án**: Markdown Parser đơn giản

**Mô tả**: Parse các element cơ bản của Markdown

**Yêu cầu chức năng:**

1. Parse headers (#, ##, ###)
2. Parse bold (**text**)
3. Parse italic (*text*)
4. Parse links [text](url)

**Hướng dẫn triển khai:**

```rust
#[derive(Debug)]
enum MarkdownElement {
    Header(u8, String),      // level, content
    Bold(String),
    Italic(String),
    Link { text: String, url: String },
    Text(String),
}

fn parse_line(line: &str) -> MarkdownElement {
    let trimmed = line.trim();

    // Check for headers
    if trimmed.starts_with('#') {
        let level = trimmed.chars().take_while(|&c| c == '#').count() as u8;
        let content = trimmed[level as usize..].trim().to_string();
        return MarkdownElement::Header(level, content);
    }

    // Check for bold
    if trimmed.starts_with("**") && trimmed.ends_with("**") {
        let content = trimmed[2..trimmed.len()-2].to_string();
        return MarkdownElement::Bold(content);
    }

    // Check for italic
    if trimmed.starts_with('*') && trimmed.ends_with('*') {
        let content = trimmed[1..trimmed.len()-1].to_string();
        return MarkdownElement::Italic(content);
    }

    // Check for links [text](url)
    if trimmed.starts_with('[') {
        if let Some(bracket_end) = trimmed.find(']') {
            if trimmed[bracket_end..].starts_with("](") {
                if let Some(paren_end) = trimmed.rfind(')') {
                    let text = trimmed[1..bracket_end].to_string();
                    let url = trimmed[bracket_end+2..paren_end].to_string();
                    return MarkdownElement::Link { text, url };
                }
            }
        }
    }

    MarkdownElement::Text(trimmed.to_string())
}

fn main() {
    let markdown = r#"
# Heading 1
## Heading 2
**Bold text**
*Italic text*
[Link text](https://example.com)
Normal text
"#;

    for line in markdown.lines() {
        if !line.trim().is_empty() {
            println!("{:?}", parse_line(line));
        }
    }
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Phân biệt được String và &str
- [ ] Hiểu UTF-8 encoding trong Rust
- [ ] Sử dụng được các phương thức chuỗi
- [ ] Hoàn thành TextAnalyzer
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Tại sao không thể index vào String?
2. **Ứng dụng**: Khi nào dùng String vs &str?
3. **Phân tích**: len() vs chars().count()?
4. **Thực hành**: Demo TextAnalyzer?

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: `"Việt".len()` trả về bao nhiêu?

- A. 4 (số ký tự)
- B. 6 (số bytes)
- C. 5
- D. Compile error

**Câu 2**: Hàm nào linh hoạt hơn?

- A. `fn f(s: String)`
- B. `fn f(s: &String)`
- C. `fn f(s: &str)`
- D. Như nhau

**Câu 3**: Code nào nối chuỗi mà không move s1?

- A. `let s3 = s1 + &s2;`
- B. `let s3 = format!("{}{}", s1, s2);`
- C. Cả hai đều move s1
- D. Cả hai đều không move s1

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: String::from() vs to_string()?</strong></summary>

Về kết quả thì giống nhau. `String::from()` rõ ràng hơn, `to_string()` thuận tiện cho chaining:

```rust
let s1 = String::from("hello");
let s2 = "hello".to_string();
let s3 = 42.to_string(); // to_string() linh hoạt hơn
```

</details>

<details>
<summary><strong>Q2: Làm sao lấy ký tự thứ n?</strong></summary>

Sử dụng `chars().nth()`:

```rust
let s = "Việt Nam";
let third = s.chars().nth(2); // Some('ệ')

// Hoặc collect thành Vec
let chars: Vec<char> = s.chars().collect();
let third = chars[2]; // 'ệ'
```

</details>

<details>
<summary><strong>Q3: Sao cần format! thay vì +?</strong></summary>

`+` operator move left operand. `format!` không move:

```rust
let s1 = String::from("a");
let s2 = String::from("b");

let s3 = s1 + &s2;
// s1 không dùng được nữa

let s4 = format!("{}{}", s1, s2);
// s1, s2 vẫn dùng được (nhưng s1 đã bị move ở trên)
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
