# Slices trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu về slices - cách tham chiếu đến một phần của collection mà không cần sở hữu toàn bộ dữ liệu.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu khái niệm slice và cách hoạt động
- [ ] Sử dụng thành thạo string slices (&str)
- [ ] Làm việc với array slices (&[T])
- [ ] Áp dụng pattern matching với slices

### Kiến Thức Yêu Cầu

- Hiểu về ownership và borrowing (Bài 6, 7)
- Cơ bản về String và arrays
- Khái niệm reference trong Rust

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Slices | 15 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Slice là con trỏ đến một phần của collection, không sở hữu dữ liệu mà chỉ "view" vào dữ liệu gốc.

**Tại sao điều này quan trọng?**

- Truy cập một phần dữ liệu mà không cần copy
- Đảm bảo an toàn bộ nhớ thông qua borrowing system
- Tối ưu hiệu suất khi làm việc với dữ liệu lớn

### 1.2. Kiến Thức Cốt Lõi

#### String Slices (&str)

```rust
let s = String::from("Học lập trình Rust");

// Các cách tạo slice
let slice1 = &s[0..3];   // "Học"
let slice2 = &s[4..15];  // "lập trình"
let slice3 = &s[..3];    // "Học" (từ đầu)
let slice4 = &s[16..];   // "Rust" (đến cuối)
let slice5 = &s[..];     // toàn bộ chuỗi

println!("slice1: {}", slice1);
println!("slice4: {}", slice4);
```

**📝 Giải thích:**
- `[start..end]`: Lấy từ index `start` đến `end-1`
- `[..end]`: Lấy từ đầu đến `end-1`
- `[start..]`: Lấy từ `start` đến cuối
- `[..]`: Lấy toàn bộ

#### String Literals là &str

```rust
// String literal có kiểu &'static str
let s: &str = "Đây là string literal";

// So sánh String và &str
let owned: String = String::from("owned"); // Sở hữu dữ liệu
let borrowed: &str = "borrowed";           // Không sở hữu
```

#### Lỗi phổ biến với UTF-8

```rust
let s = String::from("Xin chào");

// LỖI: slice không khớp ranh giới UTF-8
// let bad_slice = &s[0..2]; // Panic!

// Cách xử lý đúng
for (i, c) in s.char_indices() {
    println!("Vị trí {}: '{}'", i, c);
}
```

> **⚠️ Lưu ý**: Tiếng Việt sử dụng ký tự UTF-8 nhiều bytes. Slice theo byte index có thể gây lỗi.

#### Array Slices (&[T])

```rust
let numbers = [1, 2, 3, 4, 5];
let slice = &numbers[1..4];  // [2, 3, 4]

println!("Slice: {:?}", slice);
```

#### Hàm làm việc với Slices

```rust
// Nhận slice thay vì array/vec cụ thể
fn sum_of_slice(slice: &[i32]) -> i32 {
    slice.iter().sum()
}

fn main() {
    let numbers = [1, 2, 3, 4, 5];
    let vec_nums = vec![10, 20, 30];

    println!("Sum array: {}", sum_of_slice(&numbers));
    println!("Sum vec: {}", sum_of_slice(&vec_nums));
    println!("Sum partial: {}", sum_of_slice(&numbers[0..3]));
}
```

**📝 Giải thích:**
- `&[i32]` chấp nhận slice từ array hoặc Vec
- Linh hoạt hơn so với nhận kiểu cụ thể

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | String | &str | &String |
|----------|--------|------|---------|
| Sở hữu dữ liệu | Có | Không | Không |
| Có thể thay đổi | Có (với mut) | Không | Không |
| Trên Stack/Heap | Heap | Tùy thuộc | Tham chiếu |
| Khi nào dùng | Cần sở hữu | Tham số hàm | Hiếm khi |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Viết hàm tìm từ đầu tiên trong câu

**Yêu cầu**:
- Nhận một chuỗi bất kỳ
- Trả về từ đầu tiên (slice)
- Không copy dữ liệu

**🤔 Câu hỏi suy ngẫm:**

1. Tại sao trả về &str thay vì String?
2. Slice có ảnh hưởng gì đến chuỗi gốc?
3. Điều gì xảy ra nếu chuỗi gốc bị thay đổi sau khi tạo slice?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}

fn main() {
    let sentence = String::from("Học Rust rất vui");
    let first = first_word(&sentence);
    println!("Từ đầu tiên: {}", first); // "Học"

    // Slice ngăn chặn việc thay đổi chuỗi gốc
    // sentence.clear(); // LỖI: không thể mượn mutable
                         // khi immutable borrow đang tồn tại

    println!("First word: {}", first);
}
```

**Tại sao slice an toàn:**
- Slice giữ immutable borrow đến dữ liệu gốc
- Không thể thay đổi dữ liệu khi slice còn tồn tại
- Compiler đảm bảo an toàn bộ nhớ

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Luôn sử dụng &str thay vì &String trong tham số hàm.

#### ✅ Nên Làm

```rust
// Tốt: Nhận &str - linh hoạt
fn process(s: &str) {
    println!("Processing: {}", s);
}

// Tốt: Nhận slice generic
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

fn main() {
    let string = String::from("Hello");
    let literal = "World";

    process(&string); // OK
    process(literal); // OK

    let arr = [1, 2, 3];
    let vec = vec![4, 5, 6];

    sum(&arr);  // OK
    sum(&vec);  // OK
}
```

**Tại sao tốt:**
- Linh hoạt, chấp nhận nhiều kiểu input
- Không yêu cầu caller tạo kiểu cụ thể

#### ❌ Không Nên Làm

```rust
// Không tối ưu: Yêu cầu &String
fn less_flexible(s: &String) {
    println!("{}", s);
}

// Không tối ưu: Trả về String khi có thể trả về &str
fn get_first_word_bad(s: &String) -> String {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return s[0..i].to_string(); // Tạo String mới
        }
    }
    s.clone() // Clone không cần thiết
}
```

**Tại sao không tốt:**
- Không chấp nhận string literal
- Tạo bản sao không cần thiết, tốn bộ nhớ

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "byte index not a char boundary" | Slice ở giữa ký tự UTF-8 | Sử dụng char_indices() |
| "borrowed value does not live long enough" | Slice outlives data | Đảm bảo data sống đủ lâu |
| Index out of bounds | Index vượt quá độ dài | Kiểm tra bounds trước |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng các hàm xử lý chuỗi với slices

**Yêu cầu kỹ thuật:**
- Tìm từ đầu tiên trong câu
- Tìm từ cuối cùng trong câu
- Đếm từ trong câu

#### Bước 1: Hàm tìm từ đầu tiên

```rust
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}
```

#### Bước 2: Hàm tìm từ cuối cùng

```rust
fn last_word(s: &str) -> &str {
    let bytes = s.as_bytes();

    for i in (0..bytes.len()).rev() {
        if bytes[i] == b' ' {
            return &s[i + 1..];
        }
    }

    &s[..]
}
```

#### Bước 3: Sử dụng các phương thức slice

```rust
fn main() {
    let text = "Rust programming language";

    // Các phương thức hữu ích
    println!("Độ dài: {}", text.len());
    println!("Chứa 'program': {}", text.contains("program"));
    println!("Bắt đầu với 'Rust': {}", text.starts_with("Rust"));

    // Tách chuỗi
    let parts: Vec<&str> = text.split(' ').collect();
    println!("Các phần: {:?}", parts);

    // Trim whitespace
    let padded = "   Hello   ";
    println!("Trimmed: '{}'", padded.trim());
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Viết hàm tìm từ thứ n trong câu

```rust
fn nth_word(s: &str, n: usize) -> Option<&str> {
    // Implement here
}

fn main() {
    let sentence = "Rust là ngôn ngữ an toàn";
    println!("Từ thứ 2: {:?}", nth_word(sentence, 2)); // Some("là")
    println!("Từ thứ 10: {:?}", nth_word(sentence, 10)); // None
}
```

<details>
<summary>💡 Gợi ý</summary>

Sử dụng `split_whitespace()` và `nth()` method.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
fn nth_word(s: &str, n: usize) -> Option<&str> {
    s.split_whitespace().nth(n)
}
```

**Giải thích:**
- `split_whitespace()` tạo iterator các từ
- `nth(n)` trả về phần tử thứ n (0-indexed)
- Trả về Option để xử lý trường hợp không đủ từ

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Pattern matching với slices

```rust
fn analyze_slice(slice: &[i32]) {
    match slice {
        [] => println!("Slice rỗng"),
        [single] => println!("Một phần tử: {}", single),
        [first, second] => println!("Hai phần tử: {}, {}", first, second),
        [first, .., last] => println!("Đầu: {}, Cuối: {}", first, last),
    }
}

fn main() {
    analyze_slice(&[]);
    analyze_slice(&[100]);
    analyze_slice(&[10, 20]);
    analyze_slice(&[1, 2, 3, 4, 5]);
}
```

**Mở rộng**:
- Thêm case xử lý slice có 3 phần tử
- Tính tổng các phần tử ở giữa

### 3.3. Mini Project

**Dự án**: Text Analyzer

**Mô tả**: Xây dựng công cụ phân tích văn bản sử dụng slices

**Yêu cầu chức năng:**

1. Đếm số từ, câu, đoạn
2. Tìm từ dài nhất
3. Trích xuất câu đầu tiên
4. Tìm kiếm từ khóa

**Technical Stack:**
- String slices
- Iterator methods
- Pattern matching

**Hướng dẫn triển khai:**

```rust
struct TextAnalyzer<'a> {
    content: &'a str,
}

impl<'a> TextAnalyzer<'a> {
    fn new(content: &'a str) -> Self {
        TextAnalyzer { content }
    }

    fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    fn sentence_count(&self) -> usize {
        self.content
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .count()
    }

    fn longest_word(&self) -> Option<&str> {
        self.content
            .split_whitespace()
            .max_by_key(|word| word.len())
    }

    fn first_sentence(&self) -> &str {
        match self.content.find(|c| c == '.' || c == '!' || c == '?') {
            Some(idx) => &self.content[..=idx],
            None => self.content,
        }
    }

    fn contains_word(&self, target: &str) -> bool {
        self.content
            .split_whitespace()
            .any(|word| word == target)
    }
}

fn main() {
    let text = "Rust is amazing. It provides memory safety without garbage collection!";
    let analyzer = TextAnalyzer::new(text);

    println!("Words: {}", analyzer.word_count());
    println!("Sentences: {}", analyzer.sentence_count());
    println!("Longest word: {:?}", analyzer.longest_word());
    println!("First sentence: {}", analyzer.first_sentence());
    println!("Contains 'Rust': {}", analyzer.contains_word("Rust"));
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu khái niệm slice là "view" vào dữ liệu
- [ ] Phân biệt được String, &str, và &String
- [ ] Sử dụng được string slices và array slices
- [ ] Hoàn thành bài tập hướng dẫn
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện
- [ ] (Tùy chọn) Hoàn thành mini project TextAnalyzer

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Slice khác gì so với clone dữ liệu?
2. **Ứng dụng**: Tại sao nên dùng &str thay vì &String?
3. **Phân tích**: Giải thích lỗi "byte index not a char boundary"?
4. **Thực hành**: Demo hàm tìm từ dài nhất?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- So sánh slices vs cloning
- Demo TextAnalyzer
- Xử lý UTF-8 với tiếng Việt
- Performance benchmarks

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: `&str` là gì trong Rust?

- A. Một kiểu String có thể thay đổi
- B. Một string slice, tham chiếu đến một phần chuỗi
- C. Một String được sở hữu
- D. Một con trỏ null

**Câu 2**: Khi nào slice sẽ gây panic?

- A. Khi chuỗi rỗng
- B. Khi index không nằm trên ranh giới ký tự UTF-8
- C. Khi slice quá dài
- D. Khi sử dụng với Vec

**Câu 3**: Hàm nào linh hoạt hơn?

- A. `fn process(s: String)`
- B. `fn process(s: &String)`
- C. `fn process(s: &str)`
- D. Cả ba như nhau

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Sự khác biệt giữa String::from() và to_string()?</strong></summary>

Về kết quả thì giống nhau, đều tạo String mới. Sự khác biệt:
- `String::from("hello")`: Rõ ràng, dễ đọc
- `"hello".to_string()`: Method chaining thuận tiện

```rust
let s1 = String::from("hello");
let s2 = "hello".to_string();
// s1 và s2 giống nhau
```

</details>

<details>
<summary><strong>Q2: Làm sao xử lý UTF-8 đúng cách?</strong></summary>

Sử dụng `char_indices()` hoặc các method xử lý character:

```rust
let s = "Việt Nam";

// Sai: slice theo byte
// let bad = &s[0..2]; // Panic!

// Đúng: sử dụng char_indices
for (i, c) in s.char_indices() {
    println!("{}: {}", i, c);
}

// Hoặc dùng chars()
let chars: Vec<char> = s.chars().collect();
println!("Ký tự đầu: {}", chars[0]); // 'V'
```

</details>

<details>
<summary><strong>Q3: Slice có tốn thêm bộ nhớ không?</strong></summary>

Slice chỉ tốn một lượng nhỏ cố định (thường 2 words: pointer + length), không copy dữ liệu gốc.

```rust
let large_string = "a".repeat(1_000_000); // 1MB
let slice = &large_string[0..10]; // Chỉ tốn ~16 bytes

// slice không copy 1MB data, chỉ trỏ đến vị trí trong large_string
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
