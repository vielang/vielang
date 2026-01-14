# Bài 1: Giới thiệu về Gin Framework và thiết lập dự án

## 🎯 Mục tiêu bài học

1. Hiểu được Gin Framework là gì và các ưu điểm của nó trong việc phát triển API bằng Go
2. Nắm được sự khác biệt giữa Gin và các framework Go khác
3. Biết cách cấu trúc một dự án API hiện đại theo Domain-Driven Design (DDD)


## 📝 Nội dung chi tiết

### 1. Giới thiệu về Gin Framework

#### 1.1 Gin là gì?

Gin là một web framework viết bằng Go (Golang), được thiết kế với mục tiêu tạo ra các API có hiệu suất cao và code dễ bảo trì.

```go
package main

import "github.com/gin-gonic/gin"

func main() {
    r := gin.Default()
    
    r.GET("/ping", func(c *gin.Context) {
        c.JSON(200, gin.H{
            "message": "pong",
        })
    })
    
    r.Run(":8080")
}
```

#### 1.2 Các ưu điểm của Gin Framework

1. **Hiệu suất cao**: 
2. **Middleware linh hoạt**: 
3. **Routing mạnh mẽ**: 
4. **Binding và validation**: 
5. **Xử lý lỗi tích hợp**:
6. **JSON render**:
7. **Cộng đồng lớn và sự hỗ trợ**: 

### 2. So sánh Gin với các framework Go khác

#### 2.1 Gin vs thư viện chuẩn (net/http)

**Thư viện chuẩn (net/http)**:
- Ưu điểm: Không cần thêm dependencies, đơn giản, hiệu năng tốt
- Nhược điểm: Thiếu nhiều tính năng cao cấp, code dài dòng cho các tác vụ phức tạp

**Gin**:
- Ưu điểm: API đơn giản, hiệu năng cao, nhiều tính năng built-in
- Nhược điểm: Thêm dependency vào dự án


#### 2.2 Gin vs Echo

**Echo**:
- Có hiệu năng tương đương với Gin
- API hơi khác, nhưng cũng rất trực quan

#### 2.3 Gin vs Fiber

**Fiber**:
- Lấy cảm hứng từ Express.js của Node.js
- Được xây dựng trên thư viện fasthttp thay vì net/http
- Có hiệu năng tốt, API thân thiện, nhưng tương thích kém hơn với các thư viện Go chuẩn

### 3. Cấu trúc dự án API hiện đại

#### 3.1 Domain-Driven Design (DDD) trong Go

Domain-Driven Design là một phương pháp thiết kế phần mềm tập trung vào việc hiểu rõ và mô hình hóa lĩnh vực kinh doanh (domain) của ứng dụng.

#### 3.2 Cấu trúc thư mục theo DDD

```
vielang-gin/
├── api/                  
│   ├── controllers/      
│   ├── middleware/       
│   ├── routes/           
│   └── validators/       
│
├── config/               
│   ├── config.go         
│   ├── database.go       
│   └── server.go         
│
├── internal/            
│   ├── domain/           
│   └── utils/            
│
├── pkg/                  
│   └── jwt/              
│
├── storage/              
│   ├── cache/            
│   ├── database/         
│   └── repositories/     
│
├── tests/                
│   ├── integration/      
│   └── unit/             
│
├── .env.example          
├── .gitignore            
├── go.mod                
├── go.sum                
├── README.md             
└── main.go               
```

#### 3.3 Ưu điểm của cấu trúc này

- **Phân tách rõ ràng các thành phần**:
- **Giảm sự phụ thuộc giữa các module**: 
- **Dễ dàng mở rộng và bảo trì**: 
- **Tuân thủ nguyên tắc SOLID**: 

### 4. Thiết lập môi trường phát triển

#### 4.1 Cài đặt Go


#### 4.2 Cài đặt Git


#### 4.3 Cài đặt một IDE/Editor


### 5. Khởi tạo dự án với Go Modules

#### 5.1 Tạo thư mục dự án

```bash
mkdir vielang-gin
cd vielang-gin
```

#### 5.2 Khởi tạo Go module

```bash
go mod init github.com/khieu-dv/vielang-gin
```

Lệnh này sẽ tạo file `go.mod` trong thư mục dự án.

#### 5.3 Cài đặt Gin framework

```bash
go get -u github.com/gin-gonic/gin
```

### 6. Tạo cấu trúc thư mục dự án


### 7. Viết endpoint API đầu tiên "Hello World"

#### 7.1 Tạo file main.go

#### 7.2 Tạo cấu trúc routes cơ bản

### 8. Chạy và kiểm tra API đầu tiên

#### 8.1 Chạy ứng dụng

#### 8.2 Kiểm tra API với cURL


## 🔑 Những điểm quan trọng cần lưu ý

1. **Cấu trúc dự án là nền tảng quan trọng**: Việc tuân thủ một cấu trúc dự án tốt từ đầu sẽ giúp dự án dễ bảo trì và mở rộng sau này. 

2. **Go Modules**: Luôn sử dụng Go Modules cho quản lý dependencies.
3. **Gin middleware**: `gin.Default()` đã tích hợp sẵn hai middleware quan trọng: Logger và Recovery. 
4. **API versioning**: Luôn thiết kế API có versioning (như `/api/v1/...`) để sau này có thể nâng cấp API mà không làm ảnh hưởng đến các client hiện tại.

5. **Cấu hình môi trường**: Trong các dự án thực tế, hãy sử dụng environment variables và file `.env` để cấu hình ứng dụng thay vì hardcode các giá trị như cổng server.

