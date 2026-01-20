# Collections - HashMaps trong Rust

> **Mô tả ngắn gọn**: Tìm hiểu về HashMap - cấu trúc dữ liệu key-value cho phép lưu trữ và truy xuất dữ liệu nhanh chóng với độ phức tạp O(1).

## 📚 Tổng Quan

### Mục Tiêu Học Tập

Sau khi hoàn thành bài học này, bạn sẽ có khả năng:

- [ ] Hiểu và sử dụng HashMap trong Rust
- [ ] Thành thạo các phương thức CRUD với HashMap
- [ ] Nắm vững ownership và borrowing với HashMap
- [ ] Sử dụng Entry API để cập nhật giá trị

### Kiến Thức Yêu Cầu

- Ownership và borrowing (Bài 6, 7)
- Generics và traits cơ bản
- Vectors và Strings (Bài 13, 14)

### Thời Gian & Cấu Trúc

| Phần | Nội dung | Thời gian |
|------|----------|-----------|
| 1 | Kiến thức nền tảng về HashMap | 20 phút |
| 2 | Phân tích & Tư duy | 15 phút |
| 3 | Thực hành | 25 phút |
| 4 | Tổng kết & Đánh giá | 10 phút |

---

## 📖 Phần 1: Kiến Thức Nền Tảng

### 1.1. Giới Thiệu Khái Niệm

> **💡 Định nghĩa**: HashMap<K, V> là một collection lưu trữ dữ liệu theo cặp key-value, cho phép truy xuất giá trị nhanh chóng thông qua key.

**Tại sao điều này quan trọng?**

- Truy xuất nhanh O(1) trong trường hợp tốt nhất
- Phổ biến trong cache, counting, indexing
- Type-safe với generics

### 1.2. Kiến Thức Cốt Lõi

#### Khởi tạo HashMap

```rust
use std::collections::HashMap;

fn main() {
    // Khởi tạo rỗng
    let mut scores: HashMap<String, i32> = HashMap::new();

    // Khởi tạo với kiểu tự động suy luận
    let mut map = HashMap::new();
    map.insert("Blue", 10);

    // Từ vectors với collect
    let teams = vec![String::from("Blue"), String::from("Red")];
    let initial_scores = vec![10, 50];
    let scores: HashMap<_, _> = teams.into_iter()
        .zip(initial_scores.into_iter())
        .collect();
}
```

#### Thêm và truy xuất dữ liệu

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();

    // Insert
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Red"), 50);

    // Ghi đè nếu key tồn tại
    scores.insert(String::from("Blue"), 25); // Blue = 25

    // Get - trả về Option<&V>
    let team_name = String::from("Blue");
    match scores.get(&team_name) {
        Some(score) => println!("Score: {}", score),
        None => println!("Team not found"),
    }

    // Kiểm tra key tồn tại
    if scores.contains_key("Blue") {
        println!("Blue team exists");
    }
}
```

**📝 Giải thích:**
- `insert` ghi đè nếu key đã tồn tại
- `get` trả về `Option<&V>`, an toàn
- `contains_key` kiểm tra sự tồn tại

#### Entry API

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);

    // or_insert: thêm nếu chưa tồn tại
    scores.entry(String::from("Yellow")).or_insert(0);
    scores.entry(String::from("Blue")).or_insert(0); // Không thay đổi

    // or_insert trả về &mut V
    let count = scores.entry(String::from("Yellow")).or_insert(0);
    *count += 1;

    // or_insert_with: lazy evaluation
    scores.entry(String::from("Green")).or_insert_with(|| {
        println!("Computing default...");
        42
    });

    // and_modify: modify nếu tồn tại
    scores.entry(String::from("Blue"))
        .and_modify(|v| *v += 10)
        .or_insert(0);
}
```

#### Duyệt HashMap

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    scores.insert("Blue", 10);
    scores.insert("Red", 50);

    // Duyệt tất cả
    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }

    // Duyệt chỉ keys
    for key in scores.keys() {
        println!("Team: {}", key);
    }

    // Duyệt chỉ values
    for value in scores.values() {
        println!("Score: {}", value);
    }

    // Duyệt mutable
    for value in scores.values_mut() {
        *value += 5;
    }
}
```

#### Ownership với HashMap

```rust
use std::collections::HashMap;

fn main() {
    let team_name = String::from("Blue");
    let team_score = 10;

    let mut scores = HashMap::new();
    scores.insert(team_name, team_score);

    // team_score vẫn dùng được (i32 implement Copy)
    println!("Score: {}", team_score);

    // team_name không dùng được (String không Copy)
    // println!("Team: {}", team_name); // Error!

    // Giải pháp: clone
    let name = String::from("Red");
    scores.insert(name.clone(), 20);
    println!("Team: {}", name); // OK

    // Hoặc sử dụng references (cần lifetime)
}
```

#### Đếm tần suất

```rust
use std::collections::HashMap;

fn main() {
    let text = "hello world wonderful world";
    let mut word_count = HashMap::new();

    for word in text.split_whitespace() {
        let count = word_count.entry(word).or_insert(0);
        *count += 1;
    }

    println!("{:?}", word_count);
    // {"world": 2, "hello": 1, "wonderful": 1}
}
```

### 1.3. So Sánh & Đối Chiếu

| Tiêu chí | HashMap | BTreeMap |
|----------|---------|----------|
| Thứ tự | Không đảm bảo | Sorted by key |
| Lookup | O(1) average | O(log n) |
| Key requirement | Hash + Eq | Ord |
| Use case | Fast lookup | Sorted iteration |

---

## 🧠 Phần 2: Phân Tích & Tư Duy

### 2.1. Tình Huống Thực Tế

**Scenario**: Xây dựng ứng dụng quản lý cấu hình

**Yêu cầu**:
- Đọc/ghi cấu hình từ file
- CRUD operations
- Persist to file

**🤔 Câu hỏi suy ngẫm:**

1. Key nên là String hay &str?
2. Làm sao xử lý key không tồn tại?
3. Entry API giúp gì?

<details>
<summary>💭 Gợi ý phân tích</summary>

```rust
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

struct ConfigManager {
    config: HashMap<String, String>,
    filename: String,
}

impl ConfigManager {
    fn new(filename: &str) -> Self {
        let config = Self::load_from_file(filename)
            .unwrap_or_else(|_| HashMap::new());
        ConfigManager {
            config,
            filename: filename.to_string(),
        }
    }

    fn load_from_file(filename: &str) -> io::Result<HashMap<String, String>> {
        let contents = fs::read_to_string(filename)?;
        let mut config = HashMap::new();

        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                config.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Ok(config)
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    fn set(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    fn save(&self) -> io::Result<()> {
        let mut file = fs::File::create(&self.filename)?;
        for (key, value) in &self.config {
            writeln!(file, "{} = {}", key, value)?;
        }
        Ok(())
    }
}
```

</details>

### 2.2. Best Practices

> **⚠️ Lưu ý quan trọng**: Entry API hiệu quả hơn check-then-insert pattern.

#### ✅ Nên Làm

```rust
use std::collections::HashMap;

// Sử dụng Entry API
let count = map.entry(key).or_insert(0);
*count += 1;

// Clone key chỉ khi cần
if !map.contains_key(&key) {
    map.insert(key.clone(), value);
}

// Sử dụng get() cho safe access
if let Some(value) = map.get(&key) {
    process(value);
}
```

**Tại sao tốt:**
- Entry API tránh double lookup
- Clone on demand tiết kiệm memory
- get() an toàn, không panic

#### ❌ Không Nên Làm

```rust
// Check-then-insert (2 lookups)
if !map.contains_key(&key) {
    map.insert(key, value);
}

// Dùng [] cho access (có thể panic)
let value = map[&key]; // Panic nếu không tồn tại

// Clone tất cả
map.insert(key.clone(), value.clone()); // Có thể không cần
```

### 2.3. Common Pitfalls

| Lỗi Thường Gặp | Nguyên Nhân | Cách Khắc Phục |
|----------------|-------------|----------------|
| Key moved | Insert takes ownership | Clone hoặc dùng &str |
| Panic on [] | Key không tồn tại | Dùng get() |
| Borrow conflict | Get then insert | Dùng Entry API |

---

## 💻 Phần 3: Thực Hành

### 3.1. Bài Tập Hướng Dẫn

**Mục tiêu**: Xây dựng Word Counter

**Yêu cầu kỹ thuật:**
- Đếm từ trong văn bản
- Normalize (lowercase, remove punctuation)
- Sort by frequency

#### Bước 1: Hàm đếm từ

```rust
use std::collections::HashMap;

fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    // Normalize và đếm
    let text = text.to_lowercase();
    let text = text.replace(&['.', ',', '!', '?', ':', ';', '"'][..], "");

    for word in text.split_whitespace() {
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }

    counts
}
```

#### Bước 2: Sắp xếp theo tần suất

```rust
fn top_words(counts: &HashMap<String, usize>, limit: usize) -> Vec<(&String, &usize)> {
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    sorted.into_iter().take(limit).collect()
}
```

#### Bước 3: Sử dụng

```rust
fn main() {
    let text = "Rust is great. Rust is fast. Rust is safe. I love Rust!";

    let counts = count_words(text);

    println!("Top 5 words:");
    for (word, count) in top_words(&counts, 5) {
        println!("  '{}': {} times", word, count);
    }
}
```

### 3.2. Bài Tập Tự Luyện

#### 🎯 Cấp độ Cơ Bản

**Bài tập 1**: Two Sum Problem

```rust
fn two_sum(nums: &[i32], target: i32) -> Option<(usize, usize)> {
    // Tìm 2 indices i, j sao cho nums[i] + nums[j] == target
    // Implement here
}

fn main() {
    let nums = vec![2, 7, 11, 15];
    let target = 9;
    println!("{:?}", two_sum(&nums, target)); // Some((0, 1))
}
```

<details>
<summary>💡 Gợi ý</summary>

Dùng HashMap để lưu value -> index. Với mỗi số, kiểm tra xem complement (target - num) đã tồn tại chưa.

</details>

<details>
<summary>✅ Giải pháp mẫu</summary>

```rust
use std::collections::HashMap;

fn two_sum(nums: &[i32], target: i32) -> Option<(usize, usize)> {
    let mut seen: HashMap<i32, usize> = HashMap::new();

    for (i, &num) in nums.iter().enumerate() {
        let complement = target - num;
        if let Some(&j) = seen.get(&complement) {
            return Some((j, i));
        }
        seen.insert(num, i);
    }

    None
}
```

</details>

#### 🎯 Cấp độ Nâng Cao

**Bài tập 2**: Group Anagrams

```rust
fn group_anagrams(words: Vec<String>) -> Vec<Vec<String>> {
    // Group words that are anagrams of each other
    // ["eat", "tea", "tan", "ate", "nat", "bat"]
    // -> [["eat", "tea", "ate"], ["tan", "nat"], ["bat"]]
    // Implement here
}
```

**Mở rộng**:
- Implement custom hash function
- Use BTreeMap for sorted output

### 3.3. Mini Project

**Dự án**: Simple Key-Value Store

**Mô tả**: In-memory key-value store với persistence

**Yêu cầu chức năng:**

1. GET, SET, DELETE operations
2. Save/Load from file
3. TTL (Time To Live) for keys

**Hướng dẫn triển khai:**

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct Entry {
    value: String,
    expires_at: Option<Instant>,
}

struct KeyValueStore {
    data: HashMap<String, Entry>,
}

impl KeyValueStore {
    fn new() -> Self {
        KeyValueStore {
            data: HashMap::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), Entry {
            value: value.to_string(),
            expires_at: None,
        });
    }

    fn set_with_ttl(&mut self, key: &str, value: &str, ttl_secs: u64) {
        self.data.insert(key.to_string(), Entry {
            value: value.to_string(),
            expires_at: Some(Instant::now() + Duration::from_secs(ttl_secs)),
        });
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|entry| {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    return None; // Expired
                }
            }
            Some(entry.value.as_str())
        })
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| now < exp)
        });
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }
}

fn main() {
    let mut store = KeyValueStore::new();

    store.set("name", "Rust");
    store.set_with_ttl("temp", "value", 5); // Expires in 5 seconds

    println!("name: {:?}", store.get("name"));
    println!("temp: {:?}", store.get("temp"));

    store.delete("name");
    println!("After delete - name: {:?}", store.get("name"));

    println!("Keys: {:?}", store.keys());
}
```

## 🎤 Phần 4: Trình Bày & Chia Sẻ

### 4.1. Checklist Hoàn Thành

- [ ] Hiểu cách tạo và sử dụng HashMap
- [ ] Biết sử dụng Entry API
- [ ] Xử lý được ownership với HashMap
- [ ] Hoàn thành Word Counter
- [ ] Hoàn thành ít nhất 1 bài tập tự luyện

### 4.2. Câu Hỏi Tự Đánh Giá

1. **Lý thuyết**: Entry API giải quyết vấn đề gì?
2. **Ứng dụng**: HashMap vs BTreeMap khi nào?
3. **Phân tích**: Ownership khi insert String key?
4. **Thực hành**: Demo KeyValueStore?

## ✅ Phần 5: Kiểm Tra & Đánh Giá

**Câu 1**: `map.get(&key)` trả về kiểu gì?

- A. V
- B. &V
- C. Option<V>
- D. Option<&V>

**Câu 2**: Entry API method nào thêm giá trị nếu key chưa tồn tại?

- A. or_insert()
- B. and_modify()
- C. insert()
- D. get_or_insert()

**Câu 3**: Khi insert String key vào HashMap, điều gì xảy ra với key?

- A. Key được copy
- B. Key được move (ownership transferred)
- C. Key được borrow
- D. Key được clone tự động

### Câu Hỏi Thường Gặp

<details>
<summary><strong>Q1: HashMap có đảm bảo thứ tự không?</strong></summary>

Không. HashMap không đảm bảo thứ tự iteration. Nếu cần sorted order, sử dụng BTreeMap.

```rust
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert("c", 3);
map.insert("a", 1);
map.insert("b", 2);

for (k, v) in &map {
    println!("{}: {}", k, v); // a, b, c (sorted)
}
```

</details>

<details>
<summary><strong>Q2: Làm sao dùng custom struct làm key?</strong></summary>

Struct cần implement `Hash` và `Eq`:

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

// Hoặc derive
#[derive(Hash, PartialEq, Eq)]
struct Point2 {
    x: i32,
    y: i32,
}
```

</details>

<details>
<summary><strong>Q3: HashMap có thread-safe không?</strong></summary>

HashMap tiêu chuẩn không thread-safe. Sử dụng:
- `Arc<Mutex<HashMap>>` cho shared mutable access
- `DashMap` crate cho concurrent HashMap
- `RwLock` cho multiple readers/single writer

</details>

<footer>

**Version**: 1.0.0 | **Last Updated**: 2024-01-19
**License**: MIT | **Author**: VieVlog

</footer>
