# Kế hoạch khóa học Golang Gin API

# Script Bài 1: Giới thiệu và cấu trúc dự án Gin API

## PHẦN MỞ ĐẦU (2-3 phút)

[Bắt đầu video với intro và logo khóa học]

**Lời chào**:
Xin chào các bạn! Chào mừng đến với khóa học Xây dựng API với Golang và Gin Framework. Hôm nay chúng ta bắt đầu hành trình xây dựng một API đầy đủ chức năng bằng Golang.

**Giới thiệu về khóa học**:
Khóa học này được thiết kế cho các bạn đã có kiến thức cơ bản về Golang và muốn học cách xây dựng một API hoàn chỉnh, có thể mở rộng và triển khai vào môi trường thực tế. Qua 16 bài học, chúng ta sẽ xây dựng một RESTful API từ con số 0, kết nối với cơ sở dữ liệu PostgreSQL, xử lý authentication, và triển khai lên môi trường production.

**So sánh với các framework khác**:

- **Gin vs Standard library net/http**: Gin dựa trên net/http nhưng cung cấp nhiều tính năng hơn
- **Gin vs Echo**: Gin có cộng đồng lớn hơn, tài liệu phong phú hơn
- **Gin vs Fiber**: Gin dựa trên net/http tiêu chuẩn, trong khi Fiber sử dụng fasthttp

## Giới thiệu cấu trúc thư mục trong dự án mẫu

```
vielang-gin/
├── api/                  # Tất cả các thành phần liên quan đến API
│   ├── controllers/      # Xử lý logic điều khiển HTTP request
│   │   ├── post.go       # Controller cho post endpoints
│   │   ├── auth.go       # Controller cho authentication endpoints
│   │   └── user.go       # Controller cho user endpoints
│   │
│   ├── middleware/       # Middleware xử lý request/response
│   │   ├── auth.go       # Authentication middleware
│   │   ├── cors.go       # CORS middleware
│   │   └── logging.go    # Logging middleware
│   │
│   ├── routes/           # Router cấu hình
│   │   ├── post.go       # Post routes
│   │   ├── auth.go       # Authentication routes
│   │   ├── router.go     # Main router setup
│   │   └── user.go       # User routes
│   │
│   └── validators/       # Validation cho request
│       ├── post.go       # Validation cho post requests
│       ├── auth.go       # Validation cho authentication requests
│       ├── base.go       # Base validation logic
│       └── user.go       # Validation cho user requests
│
├── config/               # Cấu hình ứng dụng
│   ├── config.go         # Main config loader
│   ├── database.go       # Database configuration
│   └── server.go         # Server configuration
│
├── internal/             # Internal packages không được import từ bên ngoài
│   ├── domain/           # Business domain models
│   │   ├── post.go       # Post domain model
│   │   ├── errors.go     # Domain errors
│   │   └── user.go       # User domain model
│   │
│   └── utils/            # Utility functions
│       ├── crypto.go     # Cryptography utilities
│       ├── helpers.go    # Helper functions
│       └── validator.go  # Common validation helpers
│
├── pkg/                  # Các package có thể tái sử dụng
│   └── jwt/              # JWT utilities
│       └── jwt.go        # JWT operations
│
├── storage/              # Data storage và repositories
│   ├── cache/            # Cache implementation (Redis)
│   │   └── redis.go      # Redis operations
│   │
│   ├── database/         # Database operations
│   │   └── database.go   # PostgreSQL connection
│   │
│   └── repositories/     # Data access layer
│       ├── post.go    # Post repository
│       ├── repository.go # Base repository interface
│       └── user.go       # User repository
│
├── tests/                # Tests
│   ├── integration/      # Integration tests
│   │   └── api_test.go   # API integration tests
│   └── unit/             # Unit tests
│       ├── controllers/  # Controller tests
│       └── repositories/ # Repository tests
│
│
├── .env.example          # Environment variables template
├── .gitignore            # Git ignore file
├── go.mod                # Go module dependencies
├── go.sum                # Go dependencies checksums
├── README.md             # Project documentation
└── main.go               # Entry point of the application
```

**Mục đích của cấu trúc này**:

- Phân tách rõ ràng các thành phần
- Giảm sự phụ thuộc giữa các module
- Dễ dàng mở rộng và bảo trì
- Tuân thủ nguyên tắc SOLID

# Khóa học Golang Gin Framework

## **PHẦN I: CƠ BẢN (Bài 1-8)**

### **Bài 1: Giới thiệu về Gin Framework**

**Nội dung cơ bản:**

- Gin Framework là gì và tại sao nên sử dụng?
- So sánh Gin với các framework khác (Echo, Fiber, net/http)
- Ưu điểm và nhược điểm của Gin
- Kiến trúc tổng quan của Gin

**Hoạt động thực hành:**

- Cài đặt Go và thiết lập môi trường phát triển
- Khởi tạo module Go mới
- Cài đặt Gin package: `go get github.com/gin-gonic/gin`
- Tạo ứng dụng "Hello World" đầu tiên
- Chạy server và test qua browser

### **Bài 2: Cấu trúc dự án và Routing cơ bản**

**Nội dung cơ bản:**

- Cấu trúc thư mục tiêu chuẩn cho dự án Gin
- Khái niệm Routing trong Gin
- Các HTTP methods cơ bản (GET, POST, PUT, DELETE)
- Route parameters và query parameters

**Hoạt động thực hành:**

- Tổ chức cấu trúc thư mục dự án
- Tạo các route cơ bản với GET, POST, PUT, DELETE
- Xử lý route parameters: `/users/:id`
- Xử lý query parameters: `/users?name=john&age=30`
- Test tất cả routes bằng Postman hoặc curl

### **Bài 3: Handlers và Context**

**Nội dung cơ bản:**

- Handler function trong Gin
- Gin Context và các method quan trọng
- Trả về JSON, XML, HTML responses
- Status codes và headers

**Hoạt động thực hành:**

- Viết handler functions cho các endpoints
- Sử dụng `c.JSON()`, `c.XML()`, `c.HTML()`
- Đọc và ghi headers
- Trả về các status codes khác nhau
- Tạo một API đơn giản quản lý danh sách sản phẩm

### **Bài 4: Request Binding và Validation**

**Nội dung cơ bản:**

- Binding JSON, Form data, Query parameters
- Struct tags cho validation
- Custom validation rules
- Error handling trong binding

**Hoạt động thực hành:**

- Tạo struct với validation tags
- Bind JSON request body: `c.ShouldBindJSON()`
- Bind form data: `c.ShouldBind()`
- Bind query parameters: `c.ShouldBindQuery()`
- Xử lý validation errors
- Tạo API đăng ký người dùng với validation

### **Bài 5: Template Rendering**

**Nội dung cơ bản:**

- Template engine trong Gin
- HTML template syntax
- Passing data to templates
- Static files serving

**Hoạt động thực hành:**

- Thiết lập template directory
- Tạo layout template
- Render HTML templates với data
- Serve static files (CSS, JS, images)
- Tạo một trang web đơn giản với form

### **Bài 6: Middleware cơ bản**

**Nội dung cơ bản:**

- Middleware là gì và cách hoạt động
- Built-in middlewares: Logger, Recovery, CORS
- Tạo custom middleware
- Middleware cho specific routes và route groups

**Hoạt động thực hành:**

- Sử dụng `gin.Logger()` và `gin.Recovery()`
- Tạo custom logging middleware
- Tạo authentication middleware đơn giản
- Apply middleware cho route groups
- Test middleware hoạt động

### **Bài 7: Error Handling**

**Nội dung cơ bản:**

- Error handling patterns trong Gin
- Custom error responses
- Error middleware
- Logging errors

**Hoạt động thực hành:**

- Tạo custom error types
- Implement global error handler middleware
- Xử lý validation errors
- Logging errors vào file
- Tạo consistent error response format

### **Bài 8: File Upload và Download**

**Nội dung cơ bản:**

- Single file upload
- Multiple files upload
- File validation (size, type)
- File download và streaming

**Hoạt động thực hành:**

- Tạo endpoint upload single file
- Validate file type và size
- Upload multiple files
- Tạo endpoint download file
- Stream large files
- Tạo API quản lý file đơn giản

## **PHẦN II: TRUNG CẤP (Bài 9-16)**

### **Bài 9: Database Integration với GORM**

**Nội dung cơ bản:**

- Giới thiệu GORM
- Database connection và configuration
- Model definition và migration
- Basic CRUD operations

**Hoạt động thực hành:**

- Cài đặt GORM và database driver
- Thiết lập database connection
- Tạo models và chạy migration
- Implement CRUD operations
- Tạo API quản lý người dùng với database

### **Bài 10: Advanced GORM Operations**

**Nội dung cơ bản:**

- Associations (One-to-One, One-to-Many, Many-to-Many)
- Query optimization
- Transactions
- Hooks và callbacks

**Hoạt động thực hành:**

- Tạo models với relationships
- Implement complex queries với joins
- Sử dụng transactions
- Tạo hooks cho validation
- Build API blog với user-post relationship

### **Bài 11: Authentication JWT**

**Nội dung cơ bản:**

- JWT tokens là gì
- Tạo và verify JWT tokens
- Login/logout functionality
- Protected routes

**Hoạt động thực hành:**

- Implement đăng ký và đăng nhập
- Tạo JWT tokens khi login
- Tạo middleware verify JWT
- Protect routes cần authentication
- Implement logout và token blacklist

### **Bài 12: Authorization và Role-Based Access**

**Nội dung cơ bản:**

- Phân biệt Authentication vs Authorization
- Role-based access control (RBAC)
- Permission-based authorization
- Middleware for authorization

**Hoạt động thực hành:**

- Tạo role và permission models
- Implement RBAC middleware
- Tạo admin và user roles
- Protect endpoints theo roles
- Test authorization với different users

### **Bài 13: API Versioning**

**Nội dung cơ bản:**

- Tại sao cần API versioning
- URL versioning vs Header versioning
- Backward compatibility
- Version deprecation

**Hoạt động thực hành:**

- Implement URL-based versioning (/v1/, /v2/)
- Tạo version-specific handlers
- Maintain multiple API versions
- Implement version deprecation warnings
- Test với different API versions

### **Bài 14: Rate Limiting và Security**

**Nội dung cơ bản:**

- Rate limiting strategies
- Security headers
- Input sanitization
- SQL injection prevention

**Hoạt động thực hành:**

- Implement rate limiting middleware
- Add security headers (CORS, CSP, etc.)
- Input validation và sanitization
- Prevent common security vulnerabilities
- Load testing với rate limits

### **Bài 15: Caching Strategies**

**Nội dung cơ bản:**

- In-memory caching
- Redis integration
- Cache invalidation strategies
- HTTP caching headers

**Hoạt động thực hành:**

- Implement in-memory cache
- Integrate Redis cho distributed caching
- Cache database queries
- Implement cache-aside pattern
- Set proper HTTP cache headers

### **Bài 16: Background Jobs và Message Queues**

**Nội dung cơ bản:**

- Background job processing
- Message queue patterns
- Integration với Redis/RabbitMQ
- Job scheduling

**Hoạt động thực hành:**

- Implement background email sending
- Set up Redis queue
- Create job workers
- Schedule periodic tasks
- Handle job failures và retries

## **PHẦN III: NÂNG CAO (Bài 17-22)**

### **Bài 17: Testing Gin Applications**

**Nội dung cơ bản:**

- Unit testing handlers
- Integration testing
- Mocking dependencies
- Test coverage

**Hoạt động thực hành:**

- Viết unit tests cho handlers
- Mock database và external services
- Integration tests với test database
- Measure và improve test coverage
- Set up CI/CD pipeline với tests

### **Bài 18: API Documentation với Swagger**

**Nội dung cơ bản:**

- Swagger/OpenAPI specification
- Auto-generating documentation
- Interactive API docs
- API documentation best practices

**Hoạt động thực hành:**

- Install và setup gin-swagger
- Add Swagger annotations
- Generate interactive documentation
- Customize documentation
- Deploy documentation với API

### **Bài 19: Monitoring và Logging**

**Nội dung cơ bản:**

- Structured logging
- Metrics collection
- Health checks
- APM integration

**Hoạt động thực hành:**

- Implement structured logging với logrus
- Add metrics endpoints
- Create health check endpoints
- Integrate với Prometheus
- Set up monitoring dashboard

### **Bài 20: WebSocket và Real-time Features**

**Nội dung cơ bản:**

- WebSocket integration
- Real-time notifications
- Chat applications
- Connection management

**Hoạt động thực hành:**

- Implement WebSocket endpoints
- Create real-time chat room
- Handle connection lifecycle
- Broadcast messages
- Integration với frontend

### **Bài 21: Microservices với Gin**

**Nội dung cơ bản:**

- Microservices architecture
- Service discovery
- Inter-service communication
- API Gateway pattern

**Hoạt động thực hành:**

- Split monolith thành microservices
- Implement service discovery
- Service-to-service authentication
- Create API gateway
- Handle distributed transactions

### **Bài 22: Deployment và Production Best Practices**

**Nội dung cơ bản:**

- Production configuration
- Docker containerization
- Load balancing
- Performance optimization
- Security hardening

**Hoạt động thực hành:**

- Create production Dockerfile
- Set up load balancer
- Implement graceful shutdown
- Performance profiling và optimization
- Deploy lên cloud platform (AWS/GCP/DigitalOcean)
- Set up monitoring và alerting

## **Tài liệu và Tools cần thiết:**

- Go 1.19+
- Gin Framework
- GORM
- PostgreSQL/MySQL
- Redis
- Docker
- Postman
- Git
- IDE: VS Code hoặc GoLand

## **Dự án cuối khóa:**

Xây dựng một REST API hoàn chỉnh cho hệ thống E-commerce với tất cả tính năng đã học, bao gồm authentication, authorization, payment integration, real-time notifications, và deployment lên production.
