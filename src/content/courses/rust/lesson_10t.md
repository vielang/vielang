# Ownership trong Thực Tiễn

> **Mô tả ngắn gọn**: Áp dụng kiến thức về ownership, borrowing và lifetime vào các design patterns và dự án thực tế.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Nắm vững các design patterns liên quan đến ownership
- [ ] Biết khi nào sử dụng Clone và Copy
- [ ] Tối ưu hóa code thông qua quản lý ownership
- [ ] Áp dụng kiến thức vào dự án thực tế

### Kiến Thức Yêu Cầu

- Ownership, borrowing, lifetime (Bài 6, 7, 9)
- Slices (Bài 8)
- Structs và impl blocks

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng - Design Patterns | 20 phút |
| 2 | Phân tích & Tư duy - Optimization | 15 phút |
| 3 | Thực hành - Dự án thực tế | 30 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Design patterns với ownership là các mẫu thiết kế giúp quản lý dữ liệu hiệu quả trong Rust.

**Tại sao điều này quan trọng?**

- Viết code Rust idiomatic và hiệu quả
- Tránh clone không cần thiết
- Thiết kế API rõ ràng về ownership

### 1.2. Kiến Thức Cốt Lõi

#### Pattern 1: Passing Ownership

Khi hàm cần sở hữu dữ liệu hoàn toàn:

```rust
fn process_and_consume(data: String) {
    println!("Processing: {}", data);
    // data tự động được giải phóng khi hàm kết thúc
}

fn main() {
    let s = String::from("hello");
    process_and_consume(s);
    // s không còn hợp lệ ở đây
}
```

**Khi nào dùng:**
- Hàm là "điểm cuối" của dữ liệu
- Không cần dữ liệu sau khi gọi hàm
- Muốn rõ ràng về ownership transfer

#### Pattern 2: Borrowing với References

Khi chỉ cần truy cập tạm thời:

```rust
fn analyze(data: &String) {
    println!("Analyzing: {}", data);
}

fn main() {
    let s = String::from("hello");
    analyze(&s);
    println!("Original: {}", s); // s vẫn hợp lệ
}
```

**Khi nào dùng:**
- Chỉ cần đọc dữ liệu
- Caller cần giữ ownership
- Nhiều hàm cùng cần truy cập dữ liệu

#### Pattern 3: Taking and Returning Ownership

Khi cần thay đổi và trả lại:

```rust
fn transform(mut data: String) -> String {
    data.push_str(" world");
    data  // Trả lại ownership
}

fn main() {
    let s1 = String::from("hello");
    let s2 = transform(s1);
    // s1 không còn hợp lệ, s2 là owner mới
    println!("Transformed: {}", s2);
}
```

#### Pattern 4: RAII (Resource Acquisition Is Initialization)

Sử dụng structs để quản lý tài nguyên:

```rust
struct ResourceManager {
    resource: String,
}

impl ResourceManager {
    fn new(resource: String) -> Self {
        println!("Resource acquired: {}", resource);
        ResourceManager { resource }
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        println!("Resource released: {}", self.resource);
    }
}

fn main() {
    {
        let manager = ResourceManager::new(String::from("database"));
        // Sử dụng resource...
    } // manager bị drop, resource được giải phóng
}
```

#### Clone vs Copy

**Copy** - cho kiểu đơn giản trên stack:

```rust
fn main() {
    let x = 5;
    let y = x;  // x được copy, cả hai hợp lệ

    println!("x: {}, y: {}", x, y);
}
```

Các kiểu implement Copy:
- Số nguyên (i32, u32, ...)
- Boolean, char
- Tuples chứa kiểu Copy
- Arrays với kích thước cố định chứa kiểu Copy

**Clone** - cho kiểu phức tạp trên heap:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // Deep copy

    println!("s1: {}, s2: {}", s1, s2);
}
```

### 1.3. So Sánh & Đối Chiếu

| Pattern | Ownership | Use Case |
|---------|-----------|----------|
| Pass by value | Transfer | Consume data |
| Pass by &ref | Borrow | Read-only access |
| Pass by &mut ref | Borrow | Modify in-place |
| Take and return | Transfer → Return | Transform data |
| Clone | Copy | Need independent copy |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Tối ưu hóa code xử lý chuỗi

**Code ban đầu (chưa tối ưu):**

```rust
fn get_first_word_bad(s: &String) -> String {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return s[0..i].to_string(); // Tạo String mới
        }
    }
    s.clone() // Clone toàn bộ
}
```

**🤔 Câu hỏi suy ngẫm:**

1. Có bao nhiêu allocation xảy ra?
2. Có thể tránh clone không?
3. Trade-off giữa các giải pháp?

<details>
<summary>💭 Gợi ý phân tích</summary>

**Code tối ưu:**

```rust
// Trả về slice thay vì String mới
fn get_first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i]; // Chỉ trả về slice
        }
    }
    s // Trả về cả chuỗi nếu không có space
}

fn main() {
    let sentence = String::from("Hello world");
    let first = get_first_word(&sentence);
    println!("First word: {}", first); // Không allocation
}
```

**Cải tiến:**
- Không tạo String mới
- Không clone dữ liệu
- Chỉ trả về "view" vào dữ liệu gốc

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Rust không ẩn copies đắt đỏ. Clone phải được gọi explicitly.

#### ✅ Nên Làm

```rust
// Sử dụng &str thay vì &String
fn process(s: &str) {
    println!("Processing: {}", s);
}

// Sử dụng Cow cho clone-on-write
use std::borrow::Cow;

fn maybe_transform(s: &str, transform: bool) -> Cow<str> {
    if transform {
        Cow::Owned(s.to_uppercase())
    } else {
        Cow::Borrowed(s)
    }
}

// Pre-allocate với known capacity
fn build_greeting(name: &str) -> String {
    let mut result = String::with_capacity(10 + name.len());
    result.push_str("Hello, ");
    result.push_str(name);
    result.push('!');
    result
}
```

**Tại sao tốt:**
- &str linh hoạt hơn &String
- Cow tránh clone khi không cần thiết
- Pre-allocation tránh reallocations

#### ❌ Không Nên Làm

```rust
// Clone trong loop
fn bad_process(items: &Vec<String>) {
    for item in items {
        let copy = item.clone(); // Clone mỗi iteration!
        println!("{}", copy);
    }
}

// Nhận ownership khi chỉ cần đọc
fn bad_analyze(data: String) -> usize {
    data.len() // Chỉ cần đọc, không cần ownership
}

// Không cần thiết phải clone trước khi modify
fn bad_modify(s: &str) -> String {
    let mut clone = s.to_string();
    clone.push_str(" modified");
    clone
}
```

**Tại sao không tốt:**
- Clone trong loop gây performance hit
- Lấy ownership không cần thiết
- Có thể tối ưu hơn với proper APIs

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Clone quá nhiều | Không hiểu ownership | Sử dụng references |
| &String thay vì &str | Thói quen từ ngôn ngữ khác | Dùng &str cho parameters |
| Ownership không rõ ràng | API design kém | Document ownership transfer |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng ứng dụng quản lý văn bản với ownership patterns

**Yêu cầu kỹ thuật:**
- Struct TextDocument với methods
- DocumentManager quản lý nhiều documents
- Tối ưu ownership ở mỗi layer

#### Bước 1: TextDocument struct

```rust
use std::time::SystemTime;

struct TextDocument {
    title: String,
    content: String,
    created_at: SystemTime,
    modified_at: SystemTime,
}

impl TextDocument {
    fn new(title: &str, content: &str) -> Self {
        let now = SystemTime::now();
        TextDocument {
            title: String::from(title),
            content: String::from(content),
            created_at: now,
            modified_at: now,
        }
    }

    fn update_content(&mut self, new_content: &str) {
        self.content = String::from(new_content);
        self.modified_at = SystemTime::now();
    }

    fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    fn summary(&self, max_length: usize) -> &str {
        if self.content.len() <= max_length {
            &self.content
        } else {
            let boundary = self.content[..max_length]
                .rfind(|c| c == ' ' || c == '\n')
                .unwrap_or(max_length);
            &self.content[..boundary]
        }
    }
}
```

**Giải thích:**
- `new` nhận &str, tạo owned Strings
- `update_content` nhận &str, không cần ownership của input
- `summary` trả về slice, không clone

#### Bước 2: DocumentManager

```rust
struct DocumentManager {
    documents: Vec<TextDocument>,
}

impl DocumentManager {
    fn new() -> Self {
        DocumentManager {
            documents: Vec::new(),
        }
    }

    fn add_document(&mut self, title: &str, content: &str) -> usize {
        let doc = TextDocument::new(title, content);
        self.documents.push(doc);
        self.documents.len() - 1
    }

    fn get_document(&self, index: usize) -> Option<&TextDocument> {
        self.documents.get(index)
    }

    fn get_document_mut(&mut self, index: usize) -> Option<&mut TextDocument> {
        self.documents.get_mut(index)
    }

    fn search(&self, keyword: &str) -> Vec<usize> {
        self.documents
            .iter()
            .enumerate()
            .filter(|(_, doc)| doc.content.contains(keyword))
            .map(|(idx, _)| idx)
            .collect()
    }
}
```

#### Bước 3: Client code

```rust
fn main() {
    let mut manager = DocumentManager::new();

    // Thêm documents
    let idx1 = manager.add_document(
        "Rust Ownership",
        "Ownership is a unique feature of Rust."
    );

    let idx2 = manager.add_document(
        "Programming",
        "Rust focuses on safety and performance."
    );

    // Đọc document (immutable borrow)
    if let Some(doc) = manager.get_document(idx1) {
        println!("Title: {}", doc.title);
        println!("Words: {}", doc.word_count());
        println!("Summary: {}", doc.summary(20));
    }

    // Cập nhật document (mutable borrow)
    if let Some(doc) = manager.get_document_mut(idx1) {
        doc.update_content("Ownership ensures memory safety without GC.");
    }

    // Tìm kiếm
    let results = manager.search("Rust");
    println!("Found 'Rust' in documents: {:?}", results);
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Refactor code sau để tránh clone

```rust
fn process_items(items: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for item in items.clone() {
        if item.len() > 3 {
            result.push(item.clone());
        }
    }
    result
}
```

<details>
<summary>💡 Gợi ý</summary>

- Không cần clone input
- Sử dụng into_iter() để lấy ownership
- Filter trực tiếp

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
fn process_items(items: Vec<String>) -> Vec<String> {
    items.into_iter()
        .filter(|item| item.len() > 3)
        .collect()
}

// Hoặc nếu cần giữ original:
fn process_items_borrow(items: &[String]) -> Vec<&str> {
    items.iter()
        .filter(|item| item.len() > 3)
        .map(|s| s.as_str())
        .collect()
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Implement version history cho TextDocument

```rust
struct TextDocument {
    title: String,
    content: String,
    history: Vec<String>, // Previous versions
}

impl TextDocument {
    // Implement:
    // - update_content: lưu version cũ vào history
    // - undo: rollback về version trước
    // - get_version: lấy version cụ thể
}
```

**Mở rộng**:
- Giới hạn số versions lưu trữ
- Implement redo functionality

### 3.3. Mini Project

**Dự án**: Text Editor với Ownership-aware Design

**Mô tả**: Xây dựng text editor đơn giản với undo/redo

**Yêu cầu chức năng:**

1. Create, read, update, delete documents
2. Undo/redo changes
3. Search across documents
4. Optimize memory usage

**Technical Stack:**
- Structs với proper ownership
- Vec cho collections
- References cho read-only operations

**Hướng dẫn triển khai:**

```rust
struct Edit {
    position: usize,
    deleted: String,
    inserted: String,
}

struct Document {
    content: String,
    undo_stack: Vec<Edit>,
    redo_stack: Vec<Edit>,
}

impl Document {
    fn new(content: &str) -> Self {
        Document {
            content: String::from(content),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn insert(&mut self, position: usize, text: &str) {
        let edit = Edit {
            position,
            deleted: String::new(),
            inserted: String::from(text),
        };
        self.content.insert_str(position, text);
        self.undo_stack.push(edit);
        self.redo_stack.clear();
    }

    fn delete(&mut self, start: usize, end: usize) {
        let deleted: String = self.content[start..end].to_string();
        let edit = Edit {
            position: start,
            deleted,
            inserted: String::new(),
        };
        self.content.replace_range(start..end, "");
        self.undo_stack.push(edit);
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> bool {
        if let Some(edit) = self.undo_stack.pop() {
            // Reverse the edit
            let pos = edit.position;
            let ins_len = edit.inserted.len();

            // Remove inserted text
            self.content.replace_range(pos..pos + ins_len, "");
            // Restore deleted text
            self.content.insert_str(pos, &edit.deleted);

            self.redo_stack.push(edit);
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        if let Some(edit) = self.redo_stack.pop() {
            let pos = edit.position;
            let del_len = edit.deleted.len();

            // Remove the restored deleted text
            self.content.replace_range(pos..pos + del_len, "");
            // Re-insert the inserted text
            self.content.insert_str(pos, &edit.inserted);

            self.undo_stack.push(edit);
            true
        } else {
            false
        }
    }

    fn get_content(&self) -> &str {
        &self.content
    }
}

fn main() {
    let mut doc = Document::new("Hello");

    doc.insert(5, " World");
    println!("After insert: {}", doc.get_content());

    doc.delete(5, 11);
    println!("After delete: {}", doc.get_content());

    doc.undo();
    println!("After undo: {}", doc.get_content());

    doc.redo();
    println!("After redo: {}", doc.get_content());
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu các design patterns về ownership
- [ ] Phân biệt được khi nào dùng Clone vs reference
- [ ] Biết cách tối ưu code với ownership
- [ ] Hoàn thành bài tập TextDocument
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện
- [ ] (Tùy chọn) Hoàn thành mini project Text Editor

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Khi nào nên lấy ownership vs borrowing?
2. **Ứng dụng**: Làm sao tránh clone không cần thiết?
3. **Phân tích**: Trade-offs của các ownership patterns?
4. **Thực hành**: Demo undo/redo với proper ownership?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Các ownership patterns và use cases
- Refactoring exercise: trước/sau optimization
- Demo dự án Document Manager
- Lessons learned

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Khi nào nên sử dụng Clone?

- A. Luôn luôn để an toàn
- B. Khi cần bản sao độc lập của dữ liệu
- C. Khi truyền vào hàm
- D. Không bao giờ

**Câu 2**: &str vs &String - cái nào linh hoạt hơn cho function parameters?

- A. &String vì cụ thể hơn
- B. &str vì chấp nhận cả String và string literal
- C. Như nhau
- D. Tùy thuộc context

**Câu 3**: RAII pattern giúp gì trong Rust?

- A. Tăng tốc độ chương trình
- B. Tự động giải phóng resources khi ra khỏi scope
- C. Giảm bộ nhớ sử dụng
- D. Cải thiện readability

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Khi nào function nên lấy ownership vs reference?</strong></summary>

**Lấy ownership khi:**
- Hàm là "điểm cuối" của dữ liệu
- Cần store dữ liệu trong struct
- Builder pattern

**Sử dụng reference khi:**
- Chỉ cần đọc dữ liệu
- Caller cần tiếp tục sử dụng
- Performance critical (tránh move)

```rust
// Lấy ownership - consumer function
fn process_and_drop(data: String) {
    println!("{}", data);
    // data bị drop
}

// Borrow - reader function
fn analyze(data: &str) -> usize {
    data.len()
}
```

</details>

<details>
<summary><strong>Q2: Làm sao biết khi nào clone là cần thiết?</strong></summary>

Clone cần thiết khi:
1. Cần hai bản sao độc lập
2. Không thể sử dụng reference vì lifetime issues
3. Cần modify mà không ảnh hưởng original

```rust
// Cần clone - hai bản sao độc lập
let s1 = String::from("hello");
let s2 = s1.clone();
modify(&mut s2); // s1 không bị ảnh hưởng

// Không cần clone - chỉ đọc
fn read_only(s: &str) {
    println!("{}", s);
}
```

</details>

<details>
<summary><strong>Q3: Performance impact của Clone?</strong></summary>

Clone có chi phí:
- Allocate memory mới
- Copy data
- Có thể trigger reallocations

Tránh clone trong:
- Hot paths / loops
- Large data structures
- Real-time applications

```rust
// Bad - clone trong loop
for item in items {
    process(item.clone()); // N allocations
}

// Good - borrow trong loop
for item in &items {
    process_ref(item); // 0 allocations
}
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
