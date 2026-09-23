// Thế giới ví dụ của khoá C# Core, khai báo sẵn để khối code trong bài
// biên dịch được mà không phải viết lại bối cảnh.
//
// File này KHÔNG nằm trong công cụ — nó được nạp vào như một file nguồn phụ
// mỗi lần biên dịch một khối code (xem csproj: Compile Remove).
//
// Vì sao cần: code trong bài là mảnh rời, gọi tới `db`, `logger`, `order`…
// mà không khai báo. Không có stub thì mọi khối đều đỏ vì thiếu tên, và lỗi
// thật chìm trong đống lỗi oan. Có stub thì `o.Items` (property không tồn
// tại) mới nổi lên thành CS1061 — đúng loại lỗi cần bắt.
//
// Nguyên tắc: stub chỉ khai báo những gì bài GIẢ ĐỊNH có sẵn. Đừng thêm thứ
// bài đang dạy sai — ví dụ đừng thêm `Order.Items` chỉ để một khối hết đỏ.
using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using System.Threading;
using System.Threading.Tasks;

// Mọi kiểu nằm trong namespace, không ở global. Nhờ vậy khi bài tự khai báo
// `class Person` thì khai báo của bài THẮNG (khai báo trong global namespace
// ưu tiên hơn tên nạp bằng using) — thay vì đụng nhau thành CS0229.
namespace CourseWorld;

    // ── Mô hình nghiệp vụ dùng suốt khoá ────────────────────────────────────
    public class Customer
    {
        public int Id { get; set; }
        public string Name { get; set; } = "";
        public string Phone { get; set; } = "";
    }

    public class Order
    {
        public int Id { get; set; }
        public int CustomerId { get; set; }
        public Customer Customer { get; set; } = new();
        public decimal Total { get; set; }
        public bool IsPaid { get; set; }
        public bool IsPending { get; set; }
        public bool IsCancelled { get; set; }
        public List<OrderLine> Lines { get; set; } = new();
        public string Code { get; set; } = "";
        public DateTime CreatedAt { get; set; }
        public OrderStatus Status { get; set; }
    }

    public enum OrderStatus { New = 0, Paid = 1, Shipped = 2, Cancelled = 3 }

    public record OrderDto(int Id, decimal Total);
    public record OrderRow(int Id, string Customer, decimal Total);

    public class Product
    {
        public string Code { get; set; } = "";
        public decimal Price { get; set; }
    }

    public class Person { public string Name { get; set; } = "Huy"; }

    public class OrderLine
    {
        public string Name { get; set; } = "";
        public decimal Price { get; set; }
        public int Quantity { get; set; }
    }

    public class Job { }

    public class Item
    {
        public string Name { get; set; } = "";
        public decimal Price { get; set; }
    }

    public class OrderException : Exception
    {
        public OrderException(string m) : base(m) { }
        public OrderException(string m, Exception inner) : base(m, inner) { }
    }

    // ── Hạ tầng giả: EF Core, logging, DI, SMTP ─────────────────────────────
    public class DbSet<T> : IQueryable<T> where T : class
    {
        public ValueTask<T?> FindAsync(object? key) => new((T?)null);
        public T? Find(object? key) => null;
        public void Add(T entity) { }

        public Type ElementType => typeof(T);
        public Expression Expression => null!;
        public IQueryProvider Provider => null!;
        public IEnumerator<T> GetEnumerator() => new List<T>().GetEnumerator();
        System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator() => GetEnumerator();
    }

    /// Các phép EF Core mà bài dùng như extension trên IQueryable.
    public static class EfStubs
    {
        public static Task<List<T>> ToListAsync<T>(this IQueryable<T> src, CancellationToken ct = default) =>
            Task.FromResult(src.ToList());
        public static Task<T?> FirstOrDefaultAsync<T>(this IQueryable<T> src, CancellationToken ct = default) =>
            Task.FromResult(src.FirstOrDefault());
        public static Task<bool> AnyAsync<T>(this IQueryable<T> src, Expression<Func<T, bool>>? p = null, CancellationToken ct = default) =>
            Task.FromResult(false);
        public static Task<int> CountAsync<T>(this IQueryable<T> src, Expression<Func<T, bool>>? p = null, CancellationToken ct = default) =>
            Task.FromResult(0);
        public static IQueryable<T> AsNoTracking<T>(this IQueryable<T> src) => src;
        public static IQueryable<T> Include<T, TProp>(this IQueryable<T> src, Expression<Func<T, TProp>> path) => src;
    }

    public class DbContextOptionsBuilder
    {
        public DbContextOptionsBuilder UseSqlServer(string cs) => this;
        public DbContextOptionsBuilder LogTo(Action<string> sink, LogLevel level) => this;
    }

    public enum LogLevel { Information, Warning, Error }

    public class AppDbContext : IDisposable
    {
        public DbSet<Order> Orders { get; } = new();
        public DbSet<Customer> Customers { get; } = new();
        public DbSet<Product> Products { get; } = new();
        public int SaveChanges() => 0;
        public Task<int> SaveChangesAsync(CancellationToken ct = default) => Task.FromResult(0);
        public void Dispose() { }
    }

    public interface ILogger { }
    public interface ILogger<T> : ILogger { }

    /// Trong .NET thật, LogInformation và bạn bè là extension method, không phải
    /// thành viên của interface. Stub phải giống hệt, nếu không một class nhận
    /// `ILogger<T>` rồi gọi `logger.LogInformation(...)` sẽ báo lỗi oan.
    public static class LoggerStubs
    {
        public static void LogInformation(this ILogger l, string message, params object?[] args) { }
        public static void LogWarning(this ILogger l, string message, params object?[] args) { }
        public static void LogWarning(this ILogger l, Exception ex, string message, params object?[] args) { }
        public static void LogError(this ILogger l, string message, params object?[] args) { }
        public static void LogError(this ILogger l, Exception ex, string message, params object?[] args) { }
    }

    public class SqlException : Exception { }

    public interface ISmtp { Task SendAsync(string to, string? message = null); }

    public class ServiceCollectionLike
    {
        public ServiceCollectionLike AddScoped<TService, TImpl>() => this;
        public ServiceCollectionLike AddHttpClient<T>() => this;
        public ServiceCollectionLike AddDbContext<T>() => this;
    }

    public class BuilderLike { public ServiceCollectionLike Services { get; } = new(); }

    /// Những tên mà bài học dùng như thể đã có sẵn trong ngữ cảnh.
    public static class Stubs
    {
        public static AppDbContext db = new();
        public static ILogger logger = null!;
        public static ISmtp smtp = null!;
        public static BuilderLike builder = new();

        public static string path = "data.txt";
        public static string cs = "Server=.;Database=App";
        public static string conn = "Server=.;Database=App";
        public static string url = "https://example.com";
        public static string input = "10";
        public static string line = "GET:/orders";
        public static int id = 1;
        public static DateTime fromDate = DateTime.Today;
        public static DateTime cutoff = DateTime.Today.AddMonths(-6);

        public static List<Order> orders = new();
        public static List<Customer> customers = new();
        public static List<Item> items = new();
        public static List<string> oldCodes = new();
        public static List<string> newCodes = new();
        public static List<string> files = new();

        public static Task Save(Order order) => Task.CompletedTask;
        public static void Archive(Order order) { }
        public static void Add(string code) { }
        public static void Process(string text) { }
        public static decimal Score(Order o) => 0;
    }
