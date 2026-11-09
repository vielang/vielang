# VieLang

Web app học **lập trình .NET** bằng tiếng Việt, không cần tài khoản:

- **Khoá học** (trang chủ `/`): lộ trình .NET developer — C#, OOP & thiết kế,
  SQL, WinForms, ASP.NET Core, cấu trúc dữ liệu & giải thuật, kiến trúc phần
  mềm. Bài viết dạng chữ + code, có phần tự kiểm tra, ghi nhớ tiến độ từng
  khoá.
- **Cẩm nang** (`/cam-nang`): lộ trình nghề, câu hỏi phỏng vấn, công cụ, việc
  làm IT.
- **Góc học tập** (`/my`): thời gian học theo tuần, chuỗi ngày, khoá đang
  học, sao lưu dữ liệu ra file.

Tiến độ và bài làm lưu ngay trên trình duyệt (localStorage), sao lưu sang
máy khác bằng file.

## Tech stack

- Next.js 16 (App Router, TypeScript) + Tailwind CSS v4 + shadcn/ui
- Nội dung: Markdown trong `content/it/` và `content/cam-nang/`, dựng sang
  HTML lúc build (`scripts/build-content.ts`, tự chạy qua `predev`/`prebuild`)
  — code tô màu bằng Shiki, sơ đồ Mermaid vẽ sẵn thành SVG
- Zustand (`persist`) cho tiến độ — không backend
- Vitest cho unit test

## Cài đặt

```bash
npm install
cp .env.local.example .env.local   # tuỳ chọn, xem file mẫu
git config core.hooksPath .githooks # bật hook commit-msg, xem CLAUDE.md
npm run dev
```

## Thêm nội dung

- **Bài học IT:** `content/it/<khoá>/<chương>/<bài>.md` — xem
  `content/it/FORMAT.md`.
- **Cẩm nang:** thêm mục vào `GUIDE_SECTIONS` (`src/lib/guide-sections.ts`)
  và bài vào `content/cam-nang/<mục>/*.md` — xem `content/cam-nang/FORMAT.md`.

## Deploy lên Vercel

1. Import repo trên [vercel.com](https://vercel.com).
2. Environment Variables (tuỳ chọn): `NEXT_PUBLIC_SITE_URL` (tên miền chính,
   cho canonical/sitemap), `NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION`.
3. Deploy.

## Scripts

| Lệnh                    | Mô tả                                                     |
| ----------------------- | --------------------------------------------------------- |
| `npm run dev`           | Chạy dev server                                           |
| `npm run build`         | Build production                                          |
| `npm run start`         | Chạy server production (sau khi build)                    |
| `npm run lint`          | ESLint                                                    |
| `npm run test`          | Unit test (Vitest)                                        |
| `npm run build-content` | Dựng nội dung IT/cẩm nang (tự chạy qua predev/prebuild)   |
| `npm run check-code`    | Biên dịch thử code C# trong bài học (.NET SDK)            |
| `npm run check-csharp`  | Kiểm tra cú pháp C# trong bài học (.NET SDK)              |
| `npm run check-sql`     | Chạy thử câu SQL trong bài học (Oracle local)             |
| `npm run check-prose`   | Soát lỗi văn bản trong bài học                            |
| `npm run make-icons`    | Sinh icon PWA và ảnh chia sẻ `og.png`                     |

## Cấu trúc chính

```
src/app/(main)/page.tsx       trang chủ — danh sách khoá học
src/app/(main)/it, learn      trang khoá học và bài học
src/app/(main)/cam-nang       cẩm nang nghề IT
src/app/(main)/my             góc học tập
src/components/it/            khoá học, bài học
src/components/quiz/          câu tự kiểm tra cuối bài
src/lib/nav.ts                điều hướng chính
src/lib/courses.ts            dữ liệu khoá học
src/lib/guide.ts              dữ liệu cẩm nang
content/it/                   nguồn bài học (.md)
content/cam-nang/             nguồn bài cẩm nang (.md)
```
