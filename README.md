# KIIP Reader

Web app đọc sách văn hóa – xã hội Hàn Quốc (chương trình 사회통합프로그램 / KIIP),
dành cho người Việt học tiếng Hàn. Đọc theo trang ảnh scan, có zoom/vuốt
trang, bài giảng tiếng Việt + audio nghe/phát âm theo từng trang, ghi nhớ
tiến độ đọc và bookmark ngay trên trình duyệt (không cần tài khoản).

## Tech stack

- Next.js 16 (App Router, TypeScript) + Tailwind CSS v4 + shadcn/ui
- Ảnh + audio trang sách lưu trên Cloudflare R2, phục vụ qua `next/image`
  (remote loader + Image Optimization của Vercel) và thẻ `<audio>`
- Note "bài giảng" tiếng Việt: file Markdown tĩnh (`content/notes/`), render
  bằng `react-markdown`
- Zustand (`persist` → localStorage) cho tiến độ đọc/bookmark — không backend
- Vitest cho unit test

## 1. Cài đặt

```bash
npm install
cp .env.local.example .env.local   # rồi điền giá trị (xem mục 2)
npm run dev
```

## 2. Tạo Cloudflare R2 bucket (chỉ cần làm 1 lần)

Ảnh trang sách (~470MB) **không** commit vào git — lưu trên R2 + phục vụ qua
CDN. Các bước trên [dash.cloudflare.com](https://dash.cloudflare.com):

1. **R2 Object Storage** → **Create bucket** → đặt tên vd `kiip-reader`.
2. Vào bucket vừa tạo → **Settings** → **Public access** → bật
   **Allow Access** cho `r2.dev` subdomain (hoặc gắn custom domain riêng nếu
   có). Copy URL public (dạng `https://pub-xxxxxxxx.r2.dev`).
3. **R2** → **Manage API tokens** → **Create API token** → quyền
   **Object Read & Write**, giới hạn vào đúng bucket vừa tạo. Lưu lại 3 giá
   trị: **Account ID**, **Access Key ID**, **Secret Access Key**.
4. Điền cả 5 giá trị vào `web/.env.local` (copy từ `.env.local.example`):

   ```
   R2_ACCOUNT_ID=...
   R2_ACCESS_KEY_ID=...
   R2_SECRET_ACCESS_KEY=...
   R2_BUCKET_NAME=kiip-reader
   NEXT_PUBLIC_IMAGE_BASE_URL=https://pub-xxxxxxxx.r2.dev
   ```

## 3. Đẩy ảnh lên R2

Ảnh gốc (JPG) đã tải sẵn ở `../SB_step{1..4}_images/pages/` (một cấp trên
thư mục `web/`, xem `../*/download_ebook.py`). Script sau convert sang WebP
rồi upload:

```bash
npm run prepare-images                 # tất cả sách, bỏ qua ảnh đã có (idempotent)
npm run prepare-images -- --book step1 # chỉ 1 sách
npm run prepare-images -- --force      # upload lại kể cả đã có
```

Kiểm tra nhanh sau khi upload: mở
`https://<NEXT_PUBLIC_IMAGE_BASE_URL>/books/step1/pages/0001.webp` trên
trình duyệt — phải thấy ảnh trang 1.

## 4. Đẩy audio lên R2

Audio nghe/phát âm/nói mẫu — nguồn công khai từ NXB Hawoo (không cần đăng
nhập): `https://hawoopub01.cafe24.com/satongmp3/{step1..4}.zip`. Tải + giải
nén vào `../SB_step<N>_audio/*.mp3` (cùng cấp với `SB_step<N>_images/`), rồi:

```bash
npm run prepare-audio                 # tất cả sách, bỏ qua file đã có
npm run prepare-audio -- --book step1 # chỉ 1 sách
```

Mapping "bài học → trang" đã xác minh thủ công, xem
`src/lib/audio-config.ts` (có ghi chú giả định cho step4 cần spot-check
nếu sau này phát hiện sai).

## 5. Thêm bài giảng (note)

Mỗi trang có thể có 1 file `content/notes/<bookId>/<page>.md` (4 chữ số,
vd `0018.md`) — nội dung tiếng Việt giải thích trang đó, theo template:
Tóm tắt / Từ vựng / Ngữ pháp / Văn hoá. Không cần chạy gì thêm — script
`build-notes.ts` tự gộp thành JSON qua `predev`/`prebuild`.

## 6. Thêm sách mới sau này

1. Tải ảnh bằng `download_ebook.py` (ở thư mục gốc `kiip/`, xem hướng dẫn
   trong đó) → ra `SB_step<N>_images/pages/`.
2. Thêm 1 entry vào mảng `BOOKS` trong `src/lib/books.ts`.
3. `npm run prepare-images -- --book step<N>`.
4. (Tuỳ chọn) tải audio + `npm run prepare-audio -- --book step<N>`, thêm
   layout vào `src/lib/audio-config.ts`.
5. (Tuỳ chọn) viết note vào `content/notes/step<N>/`.
6. Deploy lại.

## 7. Deploy lên Vercel

1. Push repo này lên GitHub.
2. Trên [vercel.com](https://vercel.com) → **Add New Project** → import repo
   → **Root Directory** chọn `web/` (vì repo gốc `kiip/` còn chứa các thư
   mục ảnh không phải part của app).
3. Thêm **Environment Variable**: `NEXT_PUBLIC_IMAGE_BASE_URL` = URL public
   R2 (giống `.env.local`). Không cần các biến `R2_*` khác trên Vercel —
   chúng chỉ dùng cho script `prepare-images` chạy local.
4. Deploy.

## Scripts

| Lệnh                     | Mô tả                                    |
| ------------------------ | ----------------------------------------- |
| `npm run dev`             | Chạy dev server                           |
| `npm run build`           | Build production                          |
| `npm run start`           | Chạy server production (sau khi build)    |
| `npm run lint`            | ESLint                                    |
| `npm run test`            | Unit test (Vitest)                        |
| `npm run prepare-images`  | Convert + upload ảnh trang sách lên R2    |
| `npm run prepare-audio`   | Upload audio trang sách lên R2            |
| `npm run build-notes`     | Gộp note .md → JSON (tự chạy qua predev/prebuild) |

## Cấu trúc chính

```
src/app/(main)/              trang chủ (thư viện) + chi tiết sách — có header
src/app/read/[bookId]/[page] trang đọc full-screen — không header
src/components/reader/       zoom/pan (page-viewer), toolbar, note-sheet, audio-bar
src/components/library/      book card, lưới thumbnail trang
src/lib/books.ts             metadata sách + URL ảnh
src/lib/notes.ts             nội dung bài giảng theo trang
src/lib/audio.ts, audio-config.ts   mapping + URL audio theo trang
src/lib/progress-store.ts    tiến độ đọc/bookmark (Zustand + localStorage)
scripts/prepare-images.ts    pipeline convert JPG→WebP + upload R2
scripts/prepare-audio.ts     pipeline upload audio lên R2
scripts/build-notes.ts       gộp content/notes/*.md → JSON
content/notes/               nguồn nội dung bài giảng (.md, 1 file/trang)
```
