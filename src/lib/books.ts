import type { LanguageCode } from "@/lib/languages";

/**
 * Metadata tĩnh cho các sách đang có trong thư viện.
 *
 * Không cần database: số trang cố định theo từng cuốn, URL ảnh từng trang
 * được suy ra từ `id` sách + số trang (xem getPageUrl/getThumbUrl bên dưới).
 * Khi thêm sách mới: tải ảnh bằng `download_ebook.py`, thêm 1 entry vào mảng
 * BOOKS, rồi chạy `npm run prepare-images -- --book <id>`.
 */
export interface Book {
  /** Khớp với R2 key `books/<id>/...` — dùng trong URL (`/books/<id>`) */
  id: string;
  /**
   * Tên thư mục nguồn ảnh (và audio nếu có), dạng `<sourceDir>_images/pages/`
   * và `<sourceDir>_audio/` — khớp với slug ebook gốc trên kcenter.korean.go.kr
   * (vd `SB_step1`, `WB_step1`).
   */
  sourceDir: string;
  /** Ngôn ngữ giáo trình — khớp `code` trong lib/languages.ts, dùng để lọc sách theo trang thư viện từng ngôn ngữ */
  lang: LanguageCode;
  /** Cấp độ (1 = sơ cấp nhất) */
  level: number;
  /** Tên cấp độ in trên bìa bằng ký tự bản ngữ (vd 초급1/초급2 tiếng Hàn), dùng làm tiêu đề nhóm ở trang thư viện — bỏ trống nếu ngôn ngữ không có nhãn bìa kiểu này */
  levelLabelKo?: string;
  /** textbook = giáo trình chính, workbook = sách bài tập (익힘책, không có audio riêng) */
  kind: "textbook" | "workbook";
  titleVi: string;
  /** Tiêu đề bản ngữ hiển thị dưới titleVi (vd tiếng Hàn) — bỏ trống nếu không áp dụng (vd sách tiếng Anh) */
  titleKo?: string;
  totalPages: number;
  /**
   * Tỉ lệ rộng/cao thật của ảnh trang (đo từ khổ giấy PDF gốc) — bỏ trống thì
   * dùng `PAGE_ASPECT_RATIO` mặc định (khổ sách tiếng Hàn). Cần chính xác để
   * tính khung ảnh trong `page-viewer.tsx`, đặc biệt khi ghép 2 trang cạnh
   * nhau ở chế độ xem 2 trang — khổ giấy lệch (vd Intermediate dùng A4) sẽ
   * để hở viền nếu dùng tỉ lệ mặc định.
   */
  aspectRatio?: number;
}

export const BOOKS: readonly Book[] = [
  {
    id: "step1",
    sourceDir: "SB_step1",
    lang: "ko",
    level: 1,
    levelLabelKo: "초급1",
    kind: "textbook",
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 1",
    titleKo: "사회통합프로그램 문화 1",
    totalPages: 228,
  },
  {
    id: "step2",
    sourceDir: "SB_step2",
    lang: "ko",
    level: 2,
    levelLabelKo: "초급2",
    kind: "textbook",
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 2",
    titleKo: "사회통합프로그램 문화 2",
    totalPages: 228,
  },
  {
    id: "step3",
    sourceDir: "SB_step3",
    lang: "ko",
    level: 3,
    levelLabelKo: "중급1",
    kind: "textbook",
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 3",
    titleKo: "사회통합프로그램 문화 3",
    totalPages: 244,
  },
  {
    id: "step4",
    sourceDir: "SB_step4",
    lang: "ko",
    level: 4,
    levelLabelKo: "중급2",
    kind: "textbook",
    titleVi: "Văn hóa – Xã hội Hàn Quốc, Tập 4",
    titleKo: "사회통합프로그램 문화 4",
    totalPages: 248,
  },
  {
    id: "wb-step1",
    sourceDir: "WB_step1",
    lang: "ko",
    level: 1,
    levelLabelKo: "초급1",
    kind: "workbook",
    titleVi: "Sách bài tập – Văn hóa Xã hội Hàn Quốc, Tập 1",
    titleKo: "사회통합프로그램 문화 1 익힘책",
    totalPages: 136,
  },
  {
    id: "wb-step2",
    sourceDir: "WB_step2",
    lang: "ko",
    level: 2,
    levelLabelKo: "초급2",
    kind: "workbook",
    titleVi: "Sách bài tập – Văn hóa Xã hội Hàn Quốc, Tập 2",
    titleKo: "사회통합프로그램 문화 2 익힘책",
    totalPages: 136,
  },
  {
    id: "wb-step3",
    sourceDir: "WB_step3",
    lang: "ko",
    level: 3,
    levelLabelKo: "중급1",
    kind: "workbook",
    titleVi: "Sách bài tập – Văn hóa Xã hội Hàn Quốc, Tập 3",
    titleKo: "사회통합프로그램 문화 3 익힘책",
    totalPages: 160,
  },
  {
    id: "wb-step4",
    sourceDir: "WB_step4",
    lang: "ko",
    level: 4,
    levelLabelKo: "중급2",
    kind: "workbook",
    titleVi: "Sách bài tập – Văn hóa Xã hội Hàn Quốc, Tập 4",
    titleKo: "사회통합프로그램 문화 4 익힘책",
    totalPages: 160,
  },
  {
    id: "en-beginner",
    sourceDir: "SB_EN_beginner",
    lang: "en",
    level: 1,
    kind: "textbook",
    titleVi: "English File – Beginner",
    totalPages: 137,
    aspectRatio: 1152 / 1451.25,
  },
  {
    id: "en-elementary",
    sourceDir: "SB_EN_elementary",
    lang: "en",
    level: 2,
    kind: "textbook",
    titleVi: "English File – Elementary",
    totalPages: 169,
    aspectRatio: 1152 / 1452,
  },
  {
    id: "en-pre-intermediate",
    sourceDir: "SB_EN_pre_intermediate",
    lang: "en",
    level: 3,
    kind: "textbook",
    titleVi: "English File – Pre-Intermediate",
    totalPages: 167,
    aspectRatio: 1152 / 1451.25,
  },
  {
    id: "en-intermediate",
    sourceDir: "SB_EN_intermediate",
    lang: "en",
    level: 4,
    kind: "textbook",
    titleVi: "English File – Intermediate",
    totalPages: 169,
    aspectRatio: 595.22 / 842,
  },
  {
    id: "en-intermediate-plus",
    sourceDir: "SB_EN_intermediate_plus",
    lang: "en",
    level: 5,
    kind: "textbook",
    titleVi: "English File – Intermediate Plus",
    totalPages: 169,
    aspectRatio: 864 / 1088.44,
  },
  {
    id: "en-upper-intermediate",
    sourceDir: "SB_EN_upper_intermediate",
    lang: "en",
    level: 6,
    kind: "textbook",
    titleVi: "English File – Upper-Intermediate",
    totalPages: 170,
    aspectRatio: 1152 / 1451.25,
  },
  {
    id: "en-advanced",
    sourceDir: "SB_EN_advanced",
    lang: "en",
    level: 7,
    kind: "textbook",
    titleVi: "English File – Advanced",
    totalPages: 178,
    aspectRatio: 1152 / 1451.25,
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
const PAGE_IMAGE_WIDTH = 1200;
const PAGE_IMAGE_HEIGHT = 1562;
const PAGE_ASPECT_RATIO = PAGE_IMAGE_WIDTH / PAGE_IMAGE_HEIGHT;

/** Tỉ lệ rộng/cao ảnh trang của 1 sách cụ thể — dùng cái này thay vì PAGE_ASPECT_RATIO thẳng để khung ảnh chính xác cho từng khổ giấy. */
export function getPageAspectRatio(book: Book): number {
  return book.aspectRatio ?? PAGE_ASPECT_RATIO;
}

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

/**
 * Đường ảnh mà TRÌNH DUYỆT xin — cùng origin với app, không phải URL R2 trần.
 *
 * `/img/books/...` được `rewrites()` trong `next.config.ts` chuyển tiếp sang
 * R2 ở phía máy chủ. Cùng origin là điều kiện bắt buộc, không phải cho đẹp:
 * `sw.js` bỏ qua mọi request khác origin (xem chốt `url.origin !==
 * self.location.origin`), nên trỏ thẳng sang r2.dev là mất cả cache ảnh lẫn
 * tính năng đọc offline.
 *
 * Đuôi `.webp` cũng phải giữ: `isPageImage` trong `sw.js` nhận ảnh theo đuôi
 * tệp.
 */
export function getPageUrl(bookId: string, page: number): string {
  return `/img/books/${bookId}/pages/${padPage(page)}.webp`;
}

export function getThumbUrl(bookId: string, page: number): string {
  return `/img/books/${bookId}/thumbs/${padPage(page)}.webp`;
}

/**
 * Gốc R2 tuyệt đối.
 *
 * Ảnh trang đi qua `/img/...` cùng origin, còn bài nghe thì trỏ thẳng vào
 * đây — xem `lib/audio.ts` để biết vì sao hai thứ đi hai đường.
 */
export function mediaOriginBase(): string {
  return imageBaseUrl();
}

