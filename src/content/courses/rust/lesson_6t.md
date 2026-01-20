# Ownership - Khái niệm cốt lõi trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu về ownership - hệ thống quản lý bộ nhớ độc đáo của Rust, giúp đảm bảo an toàn bộ nhớ mà không cần garbage collector.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu rõ khái niệm ownership và vai trò của nó trong Rust
- [ ] Phân biệt được Stack và Heap trong quản lý bộ nhớ
- [ ] Nắm vững 3 nguyên tắc ownership cơ bản
- [ ] Hiểu sự khác biệt giữa di chuyển (move) và sao chép (copy)

### Kiến Thức Yêu Cầu

- Hiểu biết cơ bản về lập trình Rust (biến, kiểu dữ liệu)
- Khái niệm về bộ nhớ trong lập trình
- Cài đặt môi trường Rust và Cargo

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Ownership | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Ownership là một tập hợp các quy tắc xác định cách Rust quản lý bộ nhớ, đảm bảo an toàn mà không cần garbage collector.

**Tại sao điều này quan trọng?**

- **Quản lý bộ nhớ thủ công** (C/C++): Dễ gây lỗi memory leak, dangling pointer
- **Garbage collector** (Java, Python): Tiêu tốn tài nguyên, không dự đoán được thời điểm giải phóng
- **Rust với ownership**: Giải phóng bộ nhớ tự động, có thể dự đoán, không ảnh hưởng hiệu suất

### 1.2. Kiến Thức Cốt Lõi

#### Stack vs Heap

**Stack:**
- Dữ liệu có kích thước cố định và biết trước lúc biên dịch
- Hoạt động LIFO (Last In First Out)
- Nhanh và hiệu quả

```rust
fn example() {
    let x = 5;    // x được lưu trên stack
    let y = true; // y cũng được lưu trên stack
    let z = 'c';  // z cũng vậy
} // Khi ra khỏi hàm, x, y, z tự động bị hủy
```

**Heap:**
- Dữ liệu có kích thước không xác định tại thời điểm biên dịch
- Yêu cầu bộ nhớ → OS tìm không gian trống → trả về con trỏ

```rust
fn example() {
    let s = String::from("hello"); // s trỏ đến dữ liệu trên heap
} // Khi ra khỏi hàm, s bị hủy và bộ nhớ heap được giải phóng
```

**📝 Minh họa bộ nhớ:**

```
Stack                  |  Heap
---------------------- | -----------------------
[x: 5]                 | [address: 0x001]
[y: true]              | ["hello"]
[s: ptr to 0x001]      |
```

#### Ba nguyên tắc ownership

**Nguyên tắc 1:** Mỗi giá trị có một owner

```rust
let s = String::from("hello"); // s là owner của string "hello"
```

**Nguyên tắc 2:** Tại một thời điểm chỉ có một owner

```rust
let s1 = String::from("hello");
let s2 = s1; // ownership chuyển sang s2
// println!("{}", s1); // Lỗi: s1 đã bị move
```

**Nguyên tắc 3:** Khi owner ra khỏi scope, giá trị bị hủy

```rust
{
    let s = String::from("hello"); // s hợp lệ từ đây
    // làm việc với s
} // scope kết thúc, s bị hủy và bộ nhớ được giải phóng
```

#### Move và Copy

**Move** - áp dụng cho dữ liệu trên heap:

```rust
let s1 = String::from("hello");
let s2 = s1; // s1 bị vô hiệu hóa, ownership đã di chuyển sang s2
```

**Copy** - áp dụng cho dữ liệu trên stack:

```rust
let x = 5;
let y = x; // x vẫn hợp lệ, y là một bản sao riêng biệt
println!("x = {}, y = {}", x, y); // Không lỗi
```

**Các kiểu implement trait Copy:**
- Kiểu số nguyên (i32, u32, ...)
- Kiểu boolean (bool)
- Kiểu ký tự (char)
- Kiểu số thực (f32, f64)
- Tuple/Array chứa các kiểu Copy

#### Clone - Sao chép sâu

```rust
let s1 = String::from("hello");
let s2 = s1.clone(); // Sao chép dữ liệu trên heap

println!("s1 = {}, s2 = {}", s1, s2); // Cả hai đều hợp lệ
```

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | Move | Copy | Clone |
|----------|------|------|-------|
| Vị trí dữ liệu | Heap | Stack | Heap |
| Biến gốc | Không hợp lệ | Vẫn hợp lệ | Vẫn hợp lệ |
| Chi phí | Thấp | Thấp | Cao |
| Khi nào dùng | Mặc định với heap | Tự động với stack | Cần bản sao độc lập |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng hàm xử lý chuỗi và cần sử dụng lại dữ liệu sau khi gọi hàm.

**Yêu cầu**:
- In chuỗi ra màn hình
- Sử dụng lại chuỗi sau khi gọi hàm

**🤔 Câu hỏi suy ngẫm:**

1. Tại sao code sau gây lỗi?
2. Có những giải pháp nào?
3. Trade-offs của từng giải pháp là gì?

```rust
fn main() {
    let s = String::from("hello");
    print_string(s);
    print_string(s); // Lỗi: s đã bị move
}

fn print_string(s: String) {
    println!("{}", s);
}
```

<details>
<summary>💭 Gợi ý phân tích</summary>

**Phương pháp 1: Sử dụng references**
```rust
fn print_string(s: &String) {
    println!("{}", s);
}

fn main() {
    let s = String::from("hello");
    print_string(&s);
    print_string(&s); // OK
}
```

**Phương pháp 2: Sử dụng clone**
```rust
fn main() {
    let s = String::from("hello");
    print_string(s.clone());
    print_string(s); // OK
}
```

**Phương pháp 3: Trả về ownership**
```rust
fn print_and_return(s: String) -> String {
    println!("{}", s);
    s
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Luôn tự hỏi "đây là kiểu Copy hay Move?" khi gán giá trị.

#### ✅ Nên Làm

```rust
// Sử dụng references khi chỉ cần đọc
fn analyze(data: &String) {
    println!("Analyzing: {}", data);
}

// Sử dụng &str thay vì &String khi có thể
fn process(s: &str) {
    println!("Processing: {}", s);
}
```

**Tại sao tốt:**
- Không chuyển ownership, có thể tái sử dụng
- &str linh hoạt hơn, chấp nhận cả String và &str

#### ❌ Không Nên Làm

```rust
// Clone không cần thiết
fn process(data: String) {
    let copy = data.clone(); // Không cần thiết
    println!("{}", copy);
}

// Lấy ownership khi chỉ cần đọc
fn print_value(s: String) { // Nên dùng &String hoặc &str
    println!("{}", s);
}
```

**Tại sao không tốt:**
- Tốn bộ nhớ và CPU để clone
- Caller không thể sử dụng dữ liệu sau khi gọi hàm

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| "value borrowed after move" | Sử dụng biến sau khi move | Dùng reference hoặc clone trước |
| "cannot borrow as mutable" | Vi phạm quy tắc borrowing | Tách scope hoặc dùng immutable |
| Nhầm Copy và Move | Không hiểu kiểu dữ liệu | Kiểm tra kiểu có implement Copy không |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xác định kiểu di chuyển hay sao chép trong code

**Yêu cầu kỹ thuật:**
- Phân tích từng dòng code
- Xác định Move hay Copy
- Giải thích lý do

#### Bước 1: Phân tích code

```rust
fn main() {
    let a = 10;
    let b = a;          // Câu 1

    let s1 = String::from("rust");
    let s2 = s1;        // Câu 2

    let t1 = (1, 2);
    let t2 = t1;        // Câu 3

    let v1 = vec![1, 2, 3];
    let v2 = v1;        // Câu 4
}
```

#### Bước 2: Xác định kết quả

```rust
fn main() {
    // Câu 1: COPY - i32 là kiểu Copy
    let a = 10;
    let b = a;
    println!("a = {}, b = {}", a, b); // OK

    // Câu 2: MOVE - String không phải kiểu Copy
    let s1 = String::from("rust");
    let s2 = s1;
    // println!("s1 = {}", s1); // Lỗi
    println!("s2 = {}", s2);

    // Câu 3: COPY - Tuple chứa các kiểu i32 là Copy
    let t1 = (1, 2);
    let t2 = t1;
    println!("t1 = {:?}, t2 = {:?}", t1, t2); // OK

    // Câu 4: MOVE - Vec không phải kiểu Copy
    let v1 = vec![1, 2, 3];
    let v2 = v1;
    // println!("v1 = {:?}", v1); // Lỗi
    println!("v2 = {:?}", v2);
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Sửa lỗi ownership trong code sau

```rust
fn main() {
    let name = String::from("Rust");
    let greeting = create_greeting(name);
    println!("Original name: {}", name);
    println!("Greeting: {}", greeting);
}

fn create_greeting(name: String) -> String {
    format!("Hello, {}!", name)
}
```

<details>
<summary>💡 Gợi ý</summary>

Có 3 cách giải quyết:
1. Sử dụng reference (&String hoặc &str)
2. Clone trước khi truyền vào hàm
3. Thay đổi hàm để trả về cả name và greeting

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
// Cách 1: Sử dụng reference
fn main() {
    let name = String::from("Rust");
    let greeting = create_greeting(&name);
    println!("Original name: {}", name);
    println!("Greeting: {}", greeting);
}

fn create_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Cách 2: Sử dụng clone
fn main() {
    let name = String::from("Rust");
    let greeting = create_greeting(name.clone());
    println!("Original name: {}", name);
    println!("Greeting: {}", greeting);
}
```

**Giải thích chi tiết:**
- Cách 1 tối ưu hơn vì không tạo bản sao
- Cách 2 đơn giản nhưng tốn bộ nhớ hơn

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Sửa lỗi và tối ưu code sau

```rust
fn main() {
    let mut values = vec![1, 2, 3, 4, 5];
    let first_three = get_first_three(values);
    values.push(6);
    println!("Values: {:?}", values);
    println!("First three: {:?}", first_three);
}

fn get_first_three(values: Vec<i32>) -> Vec<i32> {
    values.iter().take(3).cloned().collect()
}
```

**Mở rộng**:
- Sử dụng slices thay vì Vec khi có thể
- Tránh clone không cần thiết
- Áp dụng best practices về ownership

### 3.3. Mini Project

**Dự án**: Quản lý nhà hàng đơn giản

**Mô tả**: Tạo chương trình mô phỏng quản lý nhà hàng với các món ăn và đơn hàng

**Yêu cầu chức năng:**

1. Tạo món ăn mới (struct Food)
2. Thêm món vào đơn hàng (struct Order)
3. Hoàn thành và hiển thị đơn hàng
4. Xử lý ownership đúng cách

**Technical Stack:**
- Structs và impl
- Vec và ownership
- References và borrowing

**Hướng dẫn triển khai:**

```rust
struct Food {
    name: String,
    price: f64,
}

struct Order {
    items: Vec<Food>,
}

impl Order {
    fn new() -> Self {
        Order { items: Vec::new() }
    }

    fn add_item(&mut self, food: Food) {
        self.items.push(food);
    }

    fn total(&self) -> f64 {
        self.items.iter().map(|f| f.price).sum()
    }

    fn display(&self) {
        for item in &self.items {
            println!("{}: ${:.2}", item.name, item.price);
        }
        println!("Total: ${:.2}", self.total());
    }
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu rõ 3 nguyên tắc ownership
- [ ] Phân biệt được Move, Copy và Clone
- [ ] Hiểu sự khác biệt Stack và Heap
- [ ] Hoàn thành bài tập hướng dẫn
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện
- [ ] (Tùy chọn) Hoàn thành mini project

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Giải thích 3 nguyên tắc ownership bằng lời của bạn?
2. **Ứng dụng**: Khi nào bạn sử dụng clone() thay vì reference?
3. **Phân tích**: So sánh Move và Copy trong trường hợp cụ thể?
4. **Thực hành**: Demo code của bạn cho bài tập Order?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- Tóm tắt 3 nguyên tắc ownership
- Demo một trong các bài tập của bạn
- Chia sẻ challenges và cách bạn giải quyết
- Best practices rút ra được

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Đoạn code nào sau đây sẽ gây lỗi compile?

- A. `let x = 5; let y = x; println!("{}", x);`
- B. `let s = String::from("hi"); let t = s; println!("{}", s);`
- C. `let a = (1, 2); let b = a; println!("{:?}", a);`
- D. `let arr = [1, 2, 3]; let brr = arr; println!("{:?}", arr);`

**Câu 2**: Kiểu dữ liệu nào KHÔNG implement trait Copy?

- A. i32
- B. bool
- C. String
- D. char

**Câu 3**: Khi nào bộ nhớ của một String được giải phóng?

- A. Khi gọi hàm drop() thủ công
- B. Khi owner ra khỏi scope
- C. Khi garbage collector chạy
- D. Khi chương trình kết thúc

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Tại sao Rust không có garbage collector?</strong></summary>

Rust sử dụng ownership system để quản lý bộ nhớ tại thời điểm biên dịch. Điều này giúp:
- Hiệu suất cao hơn (không có runtime overhead)
- Thời điểm giải phóng bộ nhớ có thể dự đoán được
- Không có "stop the world" pauses như GC

</details>

<details>
<summary><strong>Q2: Khi nào nên dùng clone()?</strong></summary>

Sử dụng clone() khi:
- Cần giữ dữ liệu gốc và làm việc trên bản sao
- Không thể sử dụng borrowing vì lý do lifetime
- Dữ liệu nhỏ và chi phí clone không đáng kể

Tránh clone() khi:
- Dữ liệu lớn, tốn nhiều bộ nhớ
- Có thể sử dụng reference đơn giản
- Trong vòng lặp hoặc thao tác diễn ra nhiều lần

</details>

<details>
<summary><strong>Q3: Sự khác biệt giữa &String và &str?</strong></summary>

- `&String`: Reference đến String object
- `&str`: String slice, có thể tham chiếu đến String hoặc string literal

Nên dùng `&str` trong function parameters vì linh hoạt hơn:
```rust
fn process(s: &str) { } // Chấp nhận cả &String và &str
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
