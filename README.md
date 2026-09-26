# VieLang

Web app học **lập trình** và **tiếng Anh** bằng tiếng Việt, không cần tài khoản:

- **IT** (trang chủ `/`): lộ trình .NET developer — C#, OOP & thiết kế, SQL,
  WinForms, ASP.NET Core, cấu trúc dữ liệu & giải thuật, kiến trúc phần mềm.
  Bài viết dạng chữ + code, có phần tự kiểm tra, ghi nhớ tiến độ từng khoá.
- **Tiếng Anh** (`/en`): giáo trình English File theo cấp độ — đọc theo trang
  ảnh scan, zoom/vuốt trang, audio theo trang, ghi chú, đánh dấu, tải về đọc
  offline.
- **Luyện thi** (TOEIC) và **Cẩm nang** (chủ đề nghề IT) đang xây — tab tự
  ẩn khi chưa có nội dung.

Tiến độ, ghi chú, đánh dấu lưu ngay trên trình duyệt (localStorage /
IndexedDB), sao lưu sang máy khác bằng file.

## Tech stack

- Next.js 16 (App Router, TypeScript) + Tailwind CSS v4 + shadcn/ui
- Ảnh + audio trang sách lưu trên Cloudflare R2, đi qua cùng origin
  (`/img/books/*`, xem `next.config.ts`) để service worker cache được
- Nội dung IT: Markdown trong `content/it/`, dựng sang HTML lúc build
  (`scripts/build-content.ts`, tự chạy qua `predev`/`prebuild`)
- Zustand (`persist`) cho tiến độ/bookmark — không backend
- Vitest cho unit test

## 1. Cài đặt

```bash
npm install
cp .env.local.example .env.local   # rồi điền giá trị (xem mục 2)
git config core.hooksPath .githooks # bật hook commit-msg, xem CLAUDE.md
npm run dev
```

## 2. Tạo Cloudflare R2 bucket (chỉ cần làm 1 lần)

Ảnh trang sách **không** commit vào git — lưu trên R2. Các bước trên
[dash.cloudflare.com](https://dash.cloudflare.com):

1. **R2 Object Storage** → **Create bucket**.
2. Vào bucket → **Settings** → **Public access** → bật **Allow Access** cho
   `r2.dev` subdomain (hoặc gắn custom domain). Copy URL public (dạng
   `https://pub-xxxxxxxx.r2.dev`).
3. **R2** → **Manage API tokens** → **Create API token** → quyền
   **Object Read & Write**, giới hạn vào đúng bucket. Lưu lại **Account ID**,
   **Access Key ID**, **Secret Access Key**.
4. Điền vào `.env.local` (copy từ `.env.local.example`):

   ```
   R2_ACCOUNT_ID=...
   R2_ACCESS_KEY_ID=...
   R2_SECRET_ACCESS_KEY=...
   R2_BUCKET_NAME=...
   NEXT_PUBLIC_IMAGE_BASE_URL=https://pub-xxxxxxxx.r2.dev
   ```

## 3. Đẩy ảnh và audio sách lên R2

```bash
npm run prepare-images                       # tất cả sách, bỏ qua ảnh đã có
npm run prepare-images -- --book en-beginner # chỉ 1 sách
npm run prepare-images -- --force            # upload lại kể cả đã có
npm run prepare-audio                        # audio theo trang
```

Kiểm tra nhanh: mở
`<NEXT_PUBLIC_IMAGE_BASE_URL>/books/en-beginner/pages/0001.webp` — phải thấy
ảnh trang 1. Mapping audio → trang: `src/lib/audio-config.ts`.

## 4. Thêm nội dung

- **Bài học IT:** `content/it/<khoá>/<chương>/<bài>.md` — xem
  `content/it/FORMAT.md`.
- **Sách mới:** thêm 1 entry vào `BOOKS` (`src/lib/books.ts`), chạy
  `prepare-images`, (tuỳ chọn) thêm layout audio vào `src/lib/audio-config.ts`.
- **Cẩm nang:** thêm mục vào `GUIDE_SECTIONS` (`src/lib/guide-sections.ts`)
  và bài vào `content/cam-nang/<mục>/*.md`.

## 5. Deploy lên Vercel

1. Import repo trên [vercel.com](https://vercel.com).
2. Environment Variables: `NEXT_PUBLIC_IMAGE_BASE_URL` (bắt buộc),
   `NEXT_PUBLIC_SITE_URL` (tên miền chính, cho canonical/sitemap),
   `NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION` (tuỳ chọn). Các biến `R2_*` chỉ
   dùng cho script chạy local.
3. Deploy.

## Scripts

| Lệnh                     | Mô tả                                              |
| ------------------------ | -------------------------------------------------- |
| `npm run dev`            | Chạy dev server                                    |
| `npm run build`          | Build production                                   |
| `npm run start`          | Chạy server production (sau khi build)             |
| `npm run lint`           | ESLint                                             |
| `npm run test`           | Unit test (Vitest)                                 |
| `npm run build-content`  | Dựng nội dung IT/cẩm nang (tự chạy qua predev/prebuild) |
| `npm run prepare-images` | Convert + upload ảnh trang sách lên R2             |
| `npm run prepare-audio`  | Upload audio trang sách lên R2                     |
| `npm run check-code`     | Biên dịch thử code C# trong bài học (.NET SDK)     |
| `npm run check-sql`      | Kiểm tra câu SQL trong bài học                     |
| `npm run make-icons`     | Sinh icon PWA và ảnh chia sẻ `og.png`              |

## Cấu trúc chính

```
src/app/(main)/page.tsx       trang chủ — danh sách khoá IT
src/app/(main)/it, learn      trang khoá học và bài học IT
src/app/(main)/[lang], books  thư viện sách tiếng Anh
src/app/read/[bookId]/[page]  trang đọc full-screen — không header
src/components/it/            khoá học, bài học
src/components/reader/        zoom/pan, toolbar, ghi chú, audio
src/lib/tracks.ts             các mảng của Thư viện (IT, Tiếng Anh)
src/lib/nav.ts                điều hướng chính
src/lib/courses.ts            dữ liệu khoá học IT
src/lib/books.ts              metadata sách + URL ảnh
content/it/                   nguồn bài học IT (.md)
```
