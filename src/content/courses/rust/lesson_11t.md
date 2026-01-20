# Structs và Method Syntax trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu cách sử dụng structs để nhóm dữ liệu liên quan và định nghĩa methods để thêm hành vi cho các kiểu dữ liệu tùy chỉnh.

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu và sử dụng được structs trong Rust
- [ ] Thành thạo các phương pháp khởi tạo structs
- [ ] Nắm vững method syntax và cách triển khai
- [ ] Hiểu rõ về associated functions

### Kiến Thức Yêu Cầu

- Ownership, borrowing và references (Bài 6, 7)
- Các kiểu dữ liệu cơ bản trong Rust
- Cú pháp hàm trong Rust

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về Structs | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: Struct (cấu trúc) là một kiểu dữ liệu tổng hợp cho phép đóng gói nhiều giá trị có kiểu dữ liệu khác nhau vào một đơn vị có ý nghĩa.

**Tại sao điều này quan trọng?**

- **Tổ chức dữ liệu**: Nhóm các dữ liệu liên quan thành một đơn vị
- **Tái sử dụng mã**: Định nghĩa một lần, sử dụng nhiều lần
- **Mô hình hóa thực tế**: Biểu diễn các đối tượng thế giới thực trong code

### 1.2. Kiến Thức Cốt Lõi

#### Định nghĩa và khởi tạo Struct

```rust
// Định nghĩa struct
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

fn main() {
    // Khởi tạo struct
    let user1 = User {
        email: String::from("someone@example.com"),
        username: String::from("someuser123"),
        active: true,
        sign_in_count: 1,
    };

    // Truy cập fields
    println!("Email: {}", user1.email);
}
```

**📝 Giải thích:**
- Mỗi field có thể có kiểu dữ liệu khác nhau
- Truy cập fields qua toán tử dấu chấm (`.`)
- Struct là immutable theo mặc định

#### Struct Mutable

```rust
fn main() {
    let mut user1 = User {
        email: String::from("someone@example.com"),
        username: String::from("someuser123"),
        active: true,
        sign_in_count: 1,
    };

    // Thay đổi giá trị field
    user1.email = String::from("new_email@example.com");
}
```

#### Field Init Shorthand

```rust
fn build_user(email: String, username: String) -> User {
    User {
        email,      // Thay vì email: email
        username,   // Thay vì username: username
        active: true,
        sign_in_count: 1,
    }
}
```

#### Struct Update Syntax

```rust
fn main() {
    let user1 = User {
        email: String::from("first@example.com"),
        username: String::from("user1"),
        active: true,
        sign_in_count: 1,
    };

    // Tạo user2 từ user1
    let user2 = User {
        email: String::from("another@example.com"),
        ..user1  // Sao chép các field còn lại từ user1
    };
}
```

> **⚠️ Lưu ý**: Nếu struct có fields kiểu String, ownership sẽ được chuyển giao khi sử dụng struct update syntax.

#### Tuple Structs

```rust
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    // Truy cập bằng index
    println!("Red: {}", black.0);
}
```

#### Unit-Like Structs

```rust
struct AlwaysEqual;

fn main() {
    let subject = AlwaysEqual;
}
```

#### Method Syntax

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // Method với &self
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // Method với &mut self
    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    // Associated function (không có self)
    fn square(size: u32) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }
}

fn main() {
    let rect = Rectangle { width: 30, height: 50 };
    println!("Area: {}", rect.area());

    // Gọi associated function
    let square = Rectangle::square(20);
}
```

**📝 Các kiểu tham số self:**
- `&self`: Mượn struct immutable
- `&mut self`: Mượn struct mutable
- `self`: Lấy ownership của struct

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | Classic Struct | Tuple Struct | Unit-Like Struct |
|----------|---------------|--------------|------------------|
| Có tên field | Có | Không | Không có field |
| Truy cập | `.field_name` | `.0, .1, ...` | N/A |
| Khi nào dùng | Cần rõ ràng | Cần kiểu mới | Marker types |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng struct để quản lý thông tin hình chữ nhật

**Yêu cầu**:
- Tính diện tích và chu vi
- So sánh hai hình chữ nhật
- Xoay hình (đổi width và height)

**🤔 Câu hỏi suy ngẫm:**

1. Method nào cần `&self` và method nào cần `&mut self`?
2. Khi nào nên dùng associated function?
3. Làm sao để so sánh hai hình chữ nhật?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // Constructor - associated function
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    // Read-only methods - &self
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn perimeter(&self) -> u32 {
        2 * (self.width + self.height)
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width >= other.width && self.height >= other.height
    }

    // Mutating method - &mut self
    fn rotate(&mut self) {
        std::mem::swap(&mut self.width, &mut self.height);
    }
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Sử dụng `Self` thay vì tên struct trong impl block để dễ refactor.

#### ✅ Nên Làm

```rust
impl Rectangle {
    // Sử dụng Self
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    // &str cho tham số string
    fn with_name(name: &str, width: u32) -> Self {
        // ...
    }
}
```

**Tại sao tốt:**
- `Self` tự động thay đổi nếu đổi tên struct
- `&str` linh hoạt hơn `String`

#### ❌ Không Nên Làm

```rust
impl Rectangle {
    // Hardcode tên struct
    fn new(width: u32, height: u32) -> Rectangle {
        Rectangle { width, height }
    }

    // Public fields không cần thiết
    pub width: u32,  // Nên dùng getter method
}
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Cannot borrow as mutable | Struct không khai báo mut | Thêm `mut` khi khai báo |
| Partial move | Struct update với String fields | Clone hoặc borrow |
| Missing lifetime | Struct chứa references | Thêm lifetime parameter |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng hệ thống hình học cơ bản

**Yêu cầu kỹ thuật:**
- Struct Point và Circle
- Methods tính toán khoảng cách, diện tích
- Associated functions cho constructor

#### Bước 1: Định nghĩa structs

```rust
#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Circle {
    center: Point,
    radius: f64,
}
```

#### Bước 2: Implement methods cho Point

```rust
impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}
```

#### Bước 3: Implement methods cho Circle

```rust
impl Circle {
    fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    fn contains(&self, point: &Point) -> bool {
        self.center.distance_to(point) <= self.radius
    }
}

fn main() {
    let origin = Point::origin();
    let p1 = Point::new(3.0, 4.0);

    println!("Distance: {}", origin.distance_to(&p1)); // 5.0

    let circle = Circle::new(origin, 10.0);
    println!("Area: {:.2}", circle.area());
    println!("Contains p1? {}", circle.contains(&p1)); // true
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Tạo struct `Book` với các methods

```rust
struct Book {
    title: String,
    author: String,
    pages: u32,
    is_read: bool,
}

// Implement:
// - new(title, author, pages) -> Book
// - mark_as_read(&mut self)
// - summary(&self) -> String
```

<details>
<summary>💡 Gợi ý</summary>

Sử dụng `format!` macro để tạo summary string.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
impl Book {
    fn new(title: &str, author: &str, pages: u32) -> Self {
        Self {
            title: String::from(title),
            author: String::from(author),
            pages,
            is_read: false,
        }
    }

    fn mark_as_read(&mut self) {
        self.is_read = true;
    }

    fn summary(&self) -> String {
        let status = if self.is_read { "Read" } else { "Unread" };
        format!("{} by {} ({} pages) - {}",
            self.title, self.author, self.pages, status)
    }
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Hệ thống tài khoản ngân hàng

```rust
struct BankAccount {
    account_number: String,
    holder_name: String,
    balance: f64,
}

// Implement:
// - new(account_number, holder_name) -> BankAccount
// - deposit(&mut self, amount: f64) -> Result<(), String>
// - withdraw(&mut self, amount: f64) -> Result<(), String>
// - transfer(&mut self, other: &mut BankAccount, amount: f64) -> Result<(), String>
```

**Mở rộng**:
- Thêm lịch sử giao dịch
- Thêm giới hạn rút tiền hàng ngày

### 3.3. Mini Project

**Dự án**: Hệ thống quản lý sản phẩm

**Mô tả**: Xây dựng struct Product và ProductCatalog

**Yêu cầu chức năng:**

1. Thêm/xóa sản phẩm
2. Tìm kiếm theo tên
3. Lọc theo giá
4. Tính tổng giá trị kho

**Technical Stack:**
- Structs với Vec
- Methods với references
- Associated functions

**Hướng dẫn triển khai:**

```rust
#[derive(Debug, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    quantity: u32,
}

struct ProductCatalog {
    products: Vec<Product>,
    next_id: u32,
}

impl Product {
    fn new(id: u32, name: &str, price: f64, quantity: u32) -> Self {
        Self {
            id,
            name: String::from(name),
            price,
            quantity,
        }
    }

    fn total_value(&self) -> f64 {
        self.price * self.quantity as f64
    }
}

impl ProductCatalog {
    fn new() -> Self {
        Self {
            products: Vec::new(),
            next_id: 1,
        }
    }

    fn add_product(&mut self, name: &str, price: f64, quantity: u32) -> u32 {
        let id = self.next_id;
        self.products.push(Product::new(id, name, price, quantity));
        self.next_id += 1;
        id
    }

    fn find_by_name(&self, name: &str) -> Vec<&Product> {
        self.products.iter()
            .filter(|p| p.name.to_lowercase().contains(&name.to_lowercase()))
            .collect()
    }

    fn filter_by_price(&self, max_price: f64) -> Vec<&Product> {
        self.products.iter()
            .filter(|p| p.price <= max_price)
            .collect()
    }

    fn total_inventory_value(&self) -> f64 {
        self.products.iter().map(|p| p.total_value()).sum()
    }
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu cách định nghĩa và khởi tạo structs
- [ ] Phân biệt được 3 loại structs
- [ ] Biết cách viết methods với impl
- [ ] Hoàn thành bài tập hướng dẫn
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện
- [ ] (Tùy chọn) Hoàn thành mini project

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Sự khác biệt giữa method và associated function?
2. **Ứng dụng**: Khi nào dùng `&self` vs `&mut self` vs `self`?
3. **Phân tích**: Giải thích struct update syntax và ownership?
4. **Thực hành**: Demo ProductCatalog?

### 4.3. Bài Tập Trình Bày (Optional)

**Chuẩn bị presentation 5-10 phút về:**

- So sánh structs trong Rust với classes trong OOP
- Demo hệ thống hình học
- Các lỗi thường gặp và cách giải quyết

**Format:**
- Slides (3-5 slides) hoặc
- Live coding demo hoặc
- Technical blog post

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: Method nào cho phép thay đổi struct?

- A. `fn area(&self) -> u32`
- B. `fn resize(&mut self, w: u32)`
- C. `fn new(w: u32) -> Self`
- D. `fn destroy(self)`

**Câu 2**: Associated function được gọi như thế nào?

- A. `instance.function()`
- B. `StructName::function()`
- C. `&instance.function()`
- D. `function(instance)`

**Câu 3**: Struct update syntax `..user1` làm gì?

- A. Clone toàn bộ user1
- B. Sao chép các field còn lại từ user1
- C. Tạo reference đến user1
- D. Xóa user1

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: Struct có thể chứa references không?</strong></summary>

Có, nhưng cần lifetime annotation:

```rust
struct Excerpt<'a> {
    content: &'a str,
}

fn main() {
    let text = String::from("Hello world");
    let excerpt = Excerpt { content: &text };
}
```

</details>

<details>
<summary><strong>Q2: Có thể có nhiều impl blocks không?</strong></summary>

Có, một struct có thể có nhiều impl blocks. Điều này hữu ích khi implement traits hoặc tổ chức code:

```rust
impl Rectangle {
    fn area(&self) -> u32 { ... }
}

impl Rectangle {
    fn perimeter(&self) -> u32 { ... }
}
```

</details>

<details>
<summary><strong>Q3: Sự khác biệt giữa struct và class?</strong></summary>

Rust không có classes theo nghĩa OOP truyền thống:
- Không có inheritance (kế thừa)
- Sử dụng traits thay vì interfaces
- Composition over inheritance
- Không có constructor đặc biệt (dùng associated functions)

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
