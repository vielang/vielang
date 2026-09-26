---
title: Docker cơ bản cho .NET developer
summary: Đóng gói Web API ASP.NET Core thành image, chạy cùng Oracle bằng Docker Compose, và các lệnh Docker dùng hằng ngày.
updated: 2026-09-27
---

Khoá SQL đã dùng Docker để chạy Oracle. Bài này đi tiếp: đóng gói chính API
của bạn thành container, để ai cũng chạy được dự án bằng một lệnh, không cần
cài .NET SDK.

## Ba khái niệm

- **Image**: gói chỉ đọc chứa ứng dụng và mọi thứ nó cần để chạy.
- **Container**: một lần chạy của image. Một image chạy được nhiều container.
- **Dockerfile**: file mô tả các bước dựng image.

## Dockerfile cho Web API

Đặt file `Dockerfile` cạnh file `.csproj` (ví dụ project `ShopApi`, .NET 9):

```dockerfile
# Giai đoạn build: có SDK
FROM mcr.microsoft.com/dotnet/sdk:9.0 AS build
WORKDIR /src
COPY ShopApi.csproj .
RUN dotnet restore
COPY . .
RUN dotnet publish -c Release -o /app

# Giai đoạn chạy: chỉ có runtime ASP.NET Core, image nhỏ hơn
FROM mcr.microsoft.com/dotnet/aspnet:9.0
WORKDIR /app
COPY --from=build /app .
ENTRYPOINT ["dotnet", "ShopApi.dll"]
```

- Hai `FROM` là **multi-stage build**: SDK chỉ dùng để build, image cuối
  không chứa nó.
- Copy `.csproj` và `restore` trước khi copy code: code đổi mà package không
  đổi thì Docker dùng lại bước restore đã cache.
- Tag `9.0` phải khớp `TargetFramework` của project (`net9.0`).

Thêm `.dockerignore` để không copy file build trên máy vào image:

```
bin/
obj/
```

Build và chạy:

```bash
docker build -t shopapi .
docker run --rm -p 5000:8080 shopapi
```

Từ .NET 8, image ASP.NET Core lắng nghe cổng `8080` bên trong container.
`-p 5000:8080` nối cổng 5000 của máy bạn vào đó; mở
`http://localhost:5000/api/products`.

## Chạy API cùng Oracle bằng Docker Compose

API cần database. `compose.yaml` khai báo cả hai rồi bật bằng một lệnh:

```yaml
services:
  oracle:
    image: gvenzl/oracle-free:slim-faststart
    environment:
      ORACLE_PASSWORD: oracle_pw
      APP_USER: shopapi
      APP_USER_PASSWORD: shopapi_pw
    ports:
      - "1521:1521"

  api:
    build: .
    ports:
      - "5000:8080"
    environment:
      ConnectionStrings__Shop: "User Id=shopapi;Password=shopapi_pw;Data Source=oracle:1521/FREEPDB1"
    depends_on:
      - oracle
```

- Trong Compose, các service gọi nhau bằng tên: API kết nối tới `oracle`,
  không phải `localhost`.
- `ConnectionStrings__Shop` ghi đè `ConnectionStrings:Shop` trong
  `appsettings.json`. Dấu `__` thay cho `:` trong biến môi trường.
- `depends_on` chỉ đợi container Oracle khởi động, không đợi database sẵn
  sàng. Lần đầu Oracle mất một lúc; API báo lỗi kết nối thì chạy lại
  `docker compose up -d api`.
- User `shopapi` mới tạo chưa có bảng. Chạy `dotnet ef database update` từ
  máy bạn (Oracle đã mở cổng 1521 ra `localhost`) như ở bài
  [Migration](/learn/aspnet-core/du-lieu-voi-ef-core/migration).

```bash
docker compose up -d --build
docker compose logs -f api
docker compose down
```

## Lệnh dùng hằng ngày

| Việc | Lệnh |
|---|---|
| Xem container đang chạy | `docker ps` (thêm `-a` để xem cả container đã dừng) |
| Xem log | `docker logs -f <tên>` |
| Mở shell trong container | `docker exec -it <tên> bash` |
| Dừng, bật lại | `docker stop <tên>`, `docker start <tên>` |
| Xem image trên máy | `docker images` |
| Dọn image, container không dùng | `docker system prune` |

`docker compose down -v` xoá luôn volume, tức là mất dữ liệu database. Chỉ
dùng khi muốn làm lại từ đầu.

## Nguồn

- [Microsoft Learn — Containerize a .NET app](https://learn.microsoft.com/en-us/dotnet/core/docker/build-container)
- [Docker Docs — Multi-stage builds](https://docs.docker.com/build/building/multi-stage/)
- [Docker Docs — Docker Compose](https://docs.docker.com/compose/)
- [Docker Docs — Get started](https://docs.docker.com/get-started/)
