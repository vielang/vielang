---
title: Fake thay phụ thuộc
minutes: 6
---

`OrderService` kiểm tồn kho rồi trừ kho qua `IProductStore`. Test nó bằng
`DbProductStore` thật thì phải bật Oracle, chuẩn bị dữ liệu, và test chậm,
lúc qua lúc đỏ tuỳ database. Thay `IProductStore` bằng một bản giả trong bộ
nhớ thì test chạy trong vài mili giây, lần nào cũng cho cùng kết quả.

## Khái niệm

♻️ **Fake**: class tự viết implement cùng interface với phụ thuộc thật, giữ dữ liệu trong bộ nhớ để test dùng thay cho database hay API.

`FakeProductStore` của khoá WinForms là một fake. Nhờ DIP ở khoá OOP,
`OrderService` chỉ biết `IProductStore`, nên nhận fake hay bản thật đều được.

## Ví dụ

Class cần test, nằm trong project `ShopApi`. `IProductStore` và `Product`
giữ đúng như bài Truy vấn với EF Core:

```csharp
public class OrderService
{
    private readonly IProductStore _store;

    public OrderService(IProductStore store)
    {
        _store = store;
    }

    public async Task<bool> PlaceAsync(
        int productId, int quantity)
    {
        List<Product> inStock =
            await _store.InStockAsync();
        Product? product =
            inStock.Find(p => p.Id == productId);
        if (product == null || product.Stock < quantity)
        {
            return false;
        }
        await _store.SetStockAsync(
            productId, product.Stock - quantity);
        return true;
    }
}

// Có sẵn trong ShopApi từ khoá ASP.NET Core
public interface IProductStore
{
    Task<List<Product>> InStockAsync();
    Task<bool> SetStockAsync(int id, int stock);
}

public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = "";
    public decimal Price { get; set; }
    public int Stock { get; set; }
}
```

Fake và test, nằm trong `ShopApi.Tests`:

```csharp
using Xunit;

public class FakeProductStore : IProductStore
{
    public List<Product> Products { get; } =
        new List<Product>();
    public int SetStockCalls { get; private set; }

    public Task<List<Product>> InStockAsync()
    {
        var result = Products.FindAll(p => p.Stock > 0);
        return Task.FromResult(result);
    }

    public Task<bool> SetStockAsync(int id, int stock)
    {
        SetStockCalls++;
        Product? p = Products.Find(x => x.Id == id);
        if (p == null)
        {
            return Task.FromResult(false);
        }
        p.Stock = stock;
        return Task.FromResult(true);
    }
}

public class OrderServiceTests
{
    [Fact]
    public async Task Place_EnoughStock_ReducesStock()
    {
        var store = new FakeProductStore();
        store.Products.Add(new Product
        {
            Id = 1, Name = "Bút bi", Stock = 120
        });
        var service = new OrderService(store);

        bool ok = await service.PlaceAsync(1, 20);

        Assert.True(ok);
        Assert.Equal(100, store.Products[0].Stock);
    }
}
```

- Test có `await` thì khai báo `async Task`, như method async ở khoá C# Core.
- Arrange tạo fake và nạp đúng dữ liệu test cần, không phụ thuộc database.
- `SetStockCalls` đếm số lần service gọi lưu, để test kiểm được cả việc "có
  ghi xuống kho hay không".
- `Task.FromResult` như bài Tách giao diện và dữ liệu của khoá WinForms.

## Thử ngay

Thêm test cho trường hợp hết hàng vào `OrderServiceTests`:

```csharp
using Xunit;

public class OrderServiceTests
{
    [Fact]
    public async Task Place_OutOfStock_DoesNotSave()
    {
        var store = new FakeProductStore();
        store.Products.Add(new Product
        {
            Id = 2, Name = "Vở", Stock = 0
        });
        var service = new OrderService(store);

        bool ok = await service.PlaceAsync(2, 1);

        Assert.False(ok);
        Assert.Equal(0, store.SetStockCalls);
    }
}
```

**Đoán trước khi chạy:** test qua hay đỏ? `SetStockCalls` bằng mấy?

<details>
<summary>Xem kết quả</summary>

```text
Passed!  - Failed: 0, ...
```

Qua. Vở có tồn kho 0 nên `InStockAsync` không trả về nó, service trả `false`
ngay mà không gọi `SetStockAsync`. `SetStockCalls` vẫn là 0, tức không có
lần ghi nào xuống kho.

</details>

## Lỗi hay gặp

**Quên `await` khi gọi method async trong test.** `PlaceAsync` trả về
`Task<bool>` chứ không phải `bool`.

```csharp
// SAI — lỗi compile CS1503: Task<bool> không phải bool
using Xunit;

public class OrderServiceTests
{
    [Fact]
    public void Place_NoAwait()
    {
        var service = new OrderService(
            new FakeProductStore());
        Assert.False(service.PlaceAsync(1, 5));
    }
}
```

```csharp
// ĐÚNG — test async Task, await kết quả
using Xunit;

public class OrderServiceTests
{
    [Fact]
    public async Task Place_Unknown_ReturnsFalse()
    {
        var service = new OrderService(
            new FakeProductStore());
        Assert.False(await service.PlaceAsync(1, 5));
    }
}
```

Đừng viết test `async void`: xUnit đã cảnh báo (`xUnit1048`) và sẽ bỏ hỗ trợ
kiểu này.

## Tóm tắt

- Fake implement cùng interface, giữ dữ liệu trong bộ nhớ.
- Class nhận interface qua constructor thì test truyền fake vào được.
- Fake có thể đếm lời gọi để kiểm "có ghi hay không".
- Test có `await` thì khai báo `async Task`, không dùng `async void`.

```quiz
[
  {
    "prompt": "Vì sao test OrderService bằng fake tốt hơn bằng DbProductStore thật?",
    "options": [
      "Vì fake chứa logic đúng hơn",
      "Nhanh, không cần Oracle, lần chạy nào cũng cùng kết quả",
      "Vì DbProductStore không chạy được trong test",
      "Vì fake tự sửa lỗi"
    ],
    "answer": 2,
    "explain": "Fake giữ dữ liệu trong bộ nhớ nên test không phụ thuộc mạng hay trạng thái database."
  },
  {
    "prompt": "Điều gì khiến OrderService dùng được fake trong test?",
    "options": [
      "Nó kế thừa FakeProductStore",
      "Nó dùng static",
      "Nó tự new DbProductStore bên trong",
      "Nó nhận IProductStore qua constructor"
    ],
    "answer": 4,
    "explain": "Phụ thuộc vào interface và nhận qua constructor là DIP. Test truyền fake, app thật truyền DbProductStore."
  },
  {
    "prompt": "Test có dùng await nên khai báo thế nào?",
    "options": [
      "public void",
      "public async void",
      "public async Task",
      "public static void"
    ],
    "answer": 3,
    "explain": "async Task để xUnit chờ test chạy xong và bắt được lỗi. async void bị cảnh báo xUnit1048."
  }
]
```
