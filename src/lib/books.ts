/**
 * Metadata tĩnh cho các sách đang có trong thư viện.
 *
 * Không cần database: số trang cố định theo từng cuốn, URL ảnh từng trang
 * được suy ra từ `id` sách + số trang (xem getPageUrl/getThumbUrl bên dưới).
 * Khi thêm sách mới: tải ảnh bằng `download_ebook.py`, thêm 1 entry vào mảng
 * BOOKS, rồi chạy `npm run prepare-images -- --book <id>`.
 */
export interface Book {
  /** Khớp với tên thư mục nguồn `SB_<id>_images` và R2 key `books/<id>/...` */
  id: string;
  /** Cấp độ (1 = sơ cấp nhất) */
  level: number;
  titleVi: string;
  titleKo: string;
  totalPages: number;
}

export const BOOKS: readonly Book[] = [
  {
    id: "step1",
    level: 1,
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 1",
    titleKo: "사회통합프로그램 문화 1",
    totalPages: 228,
  },
  {
    id: "step2",
    level: 2,
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 2",
    titleKo: "사회통합프로그램 문화 2",
    totalPages: 228,
  },
  {
    id: "step3",
    level: 3,
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 3",
    titleKo: "사회통합프로그램 문화 3",
    totalPages: 244,
  },
  {
    id: "step4",
    level: 4,
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 4",
    titleKo: "사회통합프로그램 문화 4",
    totalPages: 248,
  },
] as const;

/**
 * Kích thước ảnh trang sau khi xử lý (xem scripts/prepare-images.ts) — đã đo
 * và xác nhận GIỐNG NHAU ở cả 4 cuốn.
 *
 * Cần hằng số này để đặt vùng bấm xem bản dịch: ảnh hiển thị bằng
 * `object-contain` nên không lấp đầy khung, phải tự tính khung ảnh thật thì
 * toạ độ vùng (lưu theo tỉ lệ 0–1 của ẢNH) mới khớp.
 */
export const PAGE_IMAGE_WIDTH = 1200;
export const PAGE_IMAGE_HEIGHT = 1562;
export const PAGE_ASPECT_RATIO = PAGE_IMAGE_WIDTH / PAGE_IMAGE_HEIGHT;

export function getBook(id: string): Book | undefined {
  return BOOKS.find((b) => b.id === id);
}

export function isValidPage(book: Book, page: number): boolean {
  return Number.isInteger(page) && page >= 1 && page <= book.totalPages;
}

/** Số trang -> "0001" */
export function padPage(page: number): string {
  return String(page).padStart(4, "0");
}

/**
 * Base URL public của bucket R2 (vd domain r2.dev hoặc custom domain đã map).
 * Bắt buộc phải là biến NEXT_PUBLIC_* vì next/image cần build URL này ở
 * client. Không chứa thông tin nhạy cảm — chỉ là 1 URL public.
 */
let warnedMissingBase = false;

function imageBaseUrl(): string {
  const base = process.env.NEXT_PUBLIC_IMAGE_BASE_URL;
  if (!base) {
    if (!warnedMissingBase) {
      console.warn(
        "⚠ Thiếu biến môi trường NEXT_PUBLIC_IMAGE_BASE_URL — ảnh sách sẽ không " +
          "tải được. Xem web/.env.local.example."
      );
      warnedMissingBase = true;
    }
    return "";
  }
  return base.replace(/\/$/, "");
}

export function getPageUrl(bookId: string, page: number): string {
  return `${imageBaseUrl()}/books/${bookId}/pages/${padPage(page)}.webp`;
}

export function getThumbUrl(bookId: string, page: number): string {
  return `${imageBaseUrl()}/books/${bookId}/thumbs/${padPage(page)}.webp`;
}
