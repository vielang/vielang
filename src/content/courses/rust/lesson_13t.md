# Collections - Vectors trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu về Vec<T> - collection động lưu trữ các phần tử cùng kiểu, một trong những cấu trúc dữ liệu quan trọng nhất trong Rust.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu và sử dụng Vec<T> trong Rust
- [ ] Thành thạo các phương thức cơ bản của Vector
- [ ] Nắm vững ownership và borrowing với Vectors
- [ ] Sử dụng iterators để duyệt và biến đổi Vector

### Kiến Thức Yêu Cầu

- Ownership và borrowing (Bài 6, 7)
- Generics cơ bản
- Enums và pattern matching (Bài 12)

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Vectors | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Vector (Vec<T>) là một collection có thể thay đổi kích thước, lưu trữ các phần tử cùng kiểu dữ liệu và được cấp phát trên heap.

**Tại sao điều này quan trọng?**

- Có thể tăng giảm kích thước linh hoạt
- Lưu trữ dữ liệu liên tiếp trong bộ nhớ
- Truy cập phần tử O(1) theo index

### 1.2. Kiến Thức Cốt Lõi

#### Khởi tạo Vector

```rust
// Tạo vector rỗng
let mut v1: Vec<i32> = Vec::new();

// Sử dụng macro vec!
let v2 = vec![1, 2, 3, 4, 5];

// Tạo với capacity
let mut v3: Vec<String> = Vec::with_capacity(10);

// Khởi tạo với giá trị mặc định
let v4 = vec![0; 10]; // 10 phần tử, mỗi phần tử là 0
```

#### Thêm và xóa phần tử

```rust
fn main() {
    let mut v = Vec::new();

    // Thêm phần tử
    v.push(1);
    v.push(2);
    v.push(3);
    println!("Sau push: {:?}", v); // [1, 2, 3]

    // Xóa phần tử cuối
    let last = v.pop(); // Option<T>
    println!("Pop: {:?}", last); // Some(3)

    // Chèn tại vị trí
    v.insert(1, 10);
    println!("Sau insert: {:?}", v); // [1, 10, 2]

    // Xóa tại vị trí
    v.remove(1);
    println!("Sau remove: {:?}", v); // [1, 2]
}
```

#### Truy cập phần tử

```rust
fn main() {
    let v = vec![10, 20, 30, 40, 50];

    // Sử dụng index - panic nếu out of bounds
    let third = &v[2];
    println!("Phần tử thứ 3: {}", third);

    // Sử dụng get() - an toàn hơn
    match v.get(2) {
        Some(value) => println!("Phần tử thứ 3: {}", value),
        None => println!("Không tồn tại"),
    }

    // Truy cập ngoài phạm vi
    // let x = &v[100]; // Panic!
    let x = v.get(100); // None
}
```

**📝 Giải thích:**
- `&v[i]` trả về `&T`, panic nếu index không hợp lệ
- `v.get(i)` trả về `Option<&T>`, an toàn hơn

#### Các phương thức hữu ích

```rust
fn main() {
    let mut v = vec![1, 2, 3];

    println!("Độ dài: {}", v.len());      // 3
    println!("Rỗng? {}", v.is_empty());   // false
    println!("Capacity: {}", v.capacity());

    // Phần tử đầu và cuối
    println!("First: {:?}", v.first()); // Some(&1)
    println!("Last: {:?}", v.last());   // Some(&3)

    // Xóa tất cả
    v.clear();

    // Thay đổi kích thước
    v.resize(5, 0); // [0, 0, 0, 0, 0]
}
```

#### Duyệt qua Vector

```rust
fn main() {
    let v = vec![10, 20, 30];

    // Duyệt immutable
    for element in &v {
        println!("{}", element);
    }

    // Duyệt mutable
    let mut v = vec![10, 20, 30];
    for element in &mut v {
        *element += 5;
    }
    println!("Sau thay đổi: {:?}", v); // [15, 25, 35]

    // Sử dụng iterators
    let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
    let even: Vec<&i32> = v.iter().filter(|&&x| x % 2 == 0).collect();
    let sum: i32 = v.iter().sum();
}
```

#### Ba loại iterators

```rust
fn main() {
    let v = vec![1, 2, 3];

    // iter() - mượn immutable
    for x in v.iter() {
        println!("{}", x);
    }
    println!("v vẫn dùng được: {:?}", v);

    // iter_mut() - mượn mutable
    let mut v = vec![1, 2, 3];
    for x in v.iter_mut() {
        *x *= 2;
    }

    // into_iter() - lấy ownership
    for x in v.into_iter() {
        println!("{}", x);
    }
    // v không còn dùng được sau into_iter()
}
```

#### Vector với Enums (nhiều kiểu)

```rust
#[derive(Debug)]
enum Cell {
    Int(i32),
    Float(f64),
    Text(String),
}

fn main() {
    let row = vec![
        Cell::Int(42),
        Cell::Float(3.14),
        Cell::Text(String::from("Rust")),
    ];

    for cell in &row {
        match cell {
            Cell::Int(v) => println!("Int: {}", v),
            Cell::Float(v) => println!("Float: {}", v),
            Cell::Text(v) => println!("Text: {}", v),
        }
    }
}
```

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | Vec<T> | [T; N] | &[T] |
|----------|--------|--------|------|
| Kích thước | Động | Cố định | Tham chiếu |
| Bộ nhớ | Heap | Stack | Không sở hữu |
| Thay đổi size | Có | Không | Không |
| Use case | Danh sách động | Kích thước biết trước | Tham số hàm |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng ứng dụng quản lý danh sách công việc

**Yêu cầu**:
- Thêm/xóa công việc
- Đánh dấu hoàn thành
- Lọc theo trạng thái

**🤔 Câu hỏi suy ngẫm:**

1. Khi nào dùng `&v[i]` vs `v.get(i)`?
2. Làm sao tránh borrow checker errors?
3. Iterator nào phù hợp cho từng use case?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
struct TodoItem {
    id: usize,
    title: String,
    completed: bool,
}

struct TodoList {
    tasks: Vec<TodoItem>,
    next_id: usize,
}

impl TodoList {
    fn new() -> Self {
        TodoList { tasks: Vec::new(), next_id: 1 }
    }

    fn add(&mut self, title: &str) -> usize {
        let id = self.next_id;
        self.tasks.push(TodoItem {
            id,
            title: String::from(title),
            completed: false,
        });
        self.next_id += 1;
        id
    }

    fn complete(&mut self, id: usize) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = true;
            true
        } else {
            false
        }
    }

    fn pending(&self) -> Vec<&TodoItem> {
        self.tasks.iter().filter(|t| !t.completed).collect()
    }
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Không thể có immutable borrow và mutable borrow cùng lúc.

#### ✅ Nên Làm

```rust
// Sử dụng get() cho truy cập an toàn
if let Some(value) = v.get(index) {
    println!("{}", value);
}

// Pre-allocate nếu biết size
let mut v = Vec::with_capacity(1000);

// Sử dụng slice cho tham số hàm
fn process(data: &[i32]) {
    // Chấp nhận cả Vec và array
}
```

**Tại sao tốt:**
- An toàn, không panic
- Tránh reallocations
- Linh hoạt với nhiều kiểu input

#### ❌ Không Nên Làm

```rust
// Truy cập trực tiếp không kiểm tra
let x = v[100]; // Có thể panic

// Giữ reference khi modify
let first = &v[0];
v.push(4); // Error! first có thể invalidate
println!("{}", first);

// Clone trong loop
for item in items {
    process(item.clone()); // Tốn bộ nhớ
}
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Borrow while mutating | Giữ reference khi push | Kết thúc borrow trước |
| Index out of bounds | Không kiểm tra | Dùng get() |
| Iterator invalidation | Modify trong loop | Collect trước hoặc dùng indices |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng TodoList với Vec

**Yêu cầu kỹ thuật:**
- CRUD operations
- Filtering
- Statistics

#### Bước 1: Định nghĩa structs

```rust
#[derive(Debug, Clone)]
struct TodoItem {
    id: usize,
    title: String,
    completed: bool,
}

struct TodoList {
    tasks: Vec<TodoItem>,
    next_id: usize,
}
```

#### Bước 2: Implement methods

```rust
impl TodoList {
    fn new() -> Self {
        TodoList { tasks: Vec::new(), next_id: 1 }
    }

    fn add(&mut self, title: &str) -> usize {
        let id = self.next_id;
        self.tasks.push(TodoItem {
            id,
            title: String::from(title),
            completed: false,
        });
        self.next_id += 1;
        id
    }

    fn remove(&mut self, id: usize) -> Option<TodoItem> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }

    fn complete(&mut self, id: usize) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = true;
            true
        } else {
            false
        }
    }

    fn list(&self) {
        println!("{:<5} {:<30} {:<10}", "ID", "Title", "Status");
        println!("{}", "-".repeat(50));
        for task in &self.tasks {
            let status = if task.completed { "Done" } else { "Pending" };
            println!("{:<5} {:<30} {:<10}", task.id, task.title, status);
        }
    }

    fn pending_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.completed).count()
    }
}
```

#### Bước 3: Sử dụng

```rust
fn main() {
    let mut todo = TodoList::new();

    todo.add("Learn Rust vectors");
    todo.add("Build todo app");
    todo.add("Practice iterators");

    todo.list();

    todo.complete(1);
    println!("\nAfter completing task 1:");
    todo.list();

    println!("\nPending tasks: {}", todo.pending_count());
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Thống kê vector số

```rust
fn statistics(numbers: &[i32]) -> (i32, i32, f64) {
    // Return: (min, max, average)
    // Implement here
}

fn main() {
    let nums = vec![5, 2, 8, 1, 9, 3];
    let (min, max, avg) = statistics(&nums);
    println!("Min: {}, Max: {}, Avg: {:.2}", min, max, avg);
}
```

<details>
<summary>💡 Gợi ý</summary>

Sử dụng `iter().min()`, `iter().max()`, và `iter().sum()`.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
fn statistics(numbers: &[i32]) -> (i32, i32, f64) {
    if numbers.is_empty() {
        return (0, 0, 0.0);
    }

    let min = *numbers.iter().min().unwrap();
    let max = *numbers.iter().max().unwrap();
    let sum: i32 = numbers.iter().sum();
    let avg = sum as f64 / numbers.len() as f64;

    (min, max, avg)
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Loại bỏ trùng lặp

```rust
fn remove_duplicates(v: Vec<i32>) -> Vec<i32> {
    // Giữ thứ tự, loại bỏ duplicates
    // Implement here
}
```

**Mở rộng**:
- Merge hai vectors đã sắp xếp
- Tìm phần tử xuất hiện nhiều nhất

### 3.3. Mini Project

**Dự án**: Student Grade Manager

**Mô tả**: Quản lý điểm sinh viên với Vec

**Yêu cầu chức năng:**

1. Thêm sinh viên và điểm
2. Tính điểm trung bình
3. Xếp hạng lớp
4. Lọc theo mức điểm

**Hướng dẫn triển khai:**

```rust
#[derive(Debug, Clone)]
struct Student {
    id: u32,
    name: String,
    scores: Vec<f64>,
}

impl Student {
    fn new(id: u32, name: &str) -> Self {
        Student {
            id,
            name: String::from(name),
            scores: Vec::new(),
        }
    }

    fn add_score(&mut self, score: f64) {
        self.scores.push(score);
    }

    fn average(&self) -> f64 {
        if self.scores.is_empty() {
            0.0
        } else {
            self.scores.iter().sum::<f64>() / self.scores.len() as f64
        }
    }

    fn grade(&self) -> char {
        match self.average() {
            avg if avg >= 90.0 => 'A',
            avg if avg >= 80.0 => 'B',
            avg if avg >= 70.0 => 'C',
            avg if avg >= 60.0 => 'D',
            _ => 'F',
        }
    }
}

struct Classroom {
    students: Vec<Student>,
}

impl Classroom {
    fn new() -> Self {
        Classroom { students: Vec::new() }
    }

    fn add_student(&mut self, student: Student) {
        self.students.push(student);
    }

    fn class_average(&self) -> f64 {
        if self.students.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.students.iter().map(|s| s.average()).sum();
        sum / self.students.len() as f64
    }

    fn ranking(&self) -> Vec<(String, f64, char)> {
        let mut ranked: Vec<_> = self.students.iter()
            .map(|s| (s.name.clone(), s.average(), s.grade()))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked
    }

    fn filter_by_grade(&self, grade: char) -> Vec<&Student> {
        self.students.iter().filter(|s| s.grade() == grade).collect()
    }
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu cách tạo và sử dụng Vec
- [ ] Biết các phương thức CRUD
- [ ] Sử dụng được iterators
- [ ] Hoàn thành TodoList
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Khác biệt giữa iter(), iter_mut(), into_iter()?
2. **Ứng dụng**: Khi nào dùng get() thay vì index?
3. **Phân tích**: Tại sao cần with_capacity()?
4. **Thực hành**: Demo Student Grade Manager?

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: `v.get(i)` trả về kiểu gì?

- A. T
- B. &T
- C. Option<T>
- D. Option<&T>

**Câu 2**: Iterator nào lấy ownership của vector?

- A. iter()
- B. iter_mut()
- C. into_iter()
- D. Tất cả

**Câu 3**: Code nào gây compile error?

- A. `let first = &v[0]; println!("{}", first);`
- B. `let first = &v[0]; v.push(1); println!("{}", first);`
- C. `v.push(1); let first = &v[0];`
- D. `let first = v.get(0);`

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Khi nào Vec reallocate?</strong></summary>

Khi số phần tử vượt quá capacity. Để tránh reallocations:

```rust
// Biết trước số phần tử
let mut v = Vec::with_capacity(1000);

// Kiểm tra capacity
println!("Capacity: {}", v.capacity());

// Đảm bảo đủ capacity
v.reserve(500); // Thêm ít nhất 500 slots
```

</details>

<details>
<summary><strong>Q2: Vec vs VecDeque?</strong></summary>

- `Vec`: O(1) cho push/pop cuối, O(n) cho push/pop đầu
- `VecDeque`: O(1) cho push/pop cả hai đầu

Dùng VecDeque khi cần queue hoặc deque.

</details>

<details>
<summary><strong>Q3: Làm sao lưu nhiều kiểu trong Vec?</strong></summary>

Sử dụng enum hoặc trait objects:

```rust
// Enum
enum Value { Int(i32), Float(f64), Text(String) }
let v: Vec<Value> = vec![...];

// Trait objects
let v: Vec<Box<dyn Display>> = vec![...];
```

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
