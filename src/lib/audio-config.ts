/**
 * Mapping "bài học -> trang" cho audio, xác minh thủ công bằng cách đọc
 * trực tiếp ảnh trang thật của từng sách (mục lục + vài trang mẫu mỗi
 * sách) — KHÔNG suy diễn bằng công thức, vì độ dài phần 복습 (ôn tập)
 * không đều nhau giữa các sách nên công thức tuyến tính đơn giản sẽ sai.
 *
 * step4 đã verify bằng ảnh trang thật (trang 12, 17, 18, 120, 204): layout
 * giống hệt step3 (cùng seri, cùng số file audio/bài).
 */

export type AudioTrackType = "S" | "L" | "P" | "intro";

interface AudioLayout {
  /** Trang bắt đầu của từng bài, theo đúng thứ tự bài 1..N (đọc từ mục lục). */
  lessonStartPages: number[];
  /**
   * offset (0-based, tính từ trang đầu bài) -> danh sách loại track gắn ở
   * trang đó.
   */
  offsets: Partial<Record<number, AudioTrackType[]>>;
}

const STEP_1_2_LAYOUT: AudioLayout = {
  lessonStartPages: [12, 22, 32, 42, 52, 62, 72, 82, 92, 112, 122, 132, 142, 152, 162, 172, 182, 192],
  offsets: {
    6: ["S", "L"], // trang "말하기와 듣기"
    9: ["P"], // trang cuối bài "발음"
  },
};

const STEP_3_LAYOUT: AudioLayout = {
  lessonStartPages: [12, 24, 36, 48, 60, 72, 84, 96, 118, 130, 142, 154, 166, 178, 190, 202],
  offsets: {
    5: ["S"], // trang "말하기"
    6: ["L", "P"], // trang "듣기" + "발음" gộp chung
  },
};

const STEP_4_LAYOUT: AudioLayout = {
  lessonStartPages: [12, 24, 36, 48, 60, 72, 84, 96, 120, 132, 144, 156, 168, 180, 192, 204],
  offsets: STEP_3_LAYOUT.offsets, // đã verify giống step3 bằng ảnh trang thật
};

export const AUDIO_LAYOUTS: Record<string, AudioLayout> = {
  step1: STEP_1_2_LAYOUT,
  step2: STEP_1_2_LAYOUT,
  step3: STEP_3_LAYOUT,
  step4: STEP_4_LAYOUT,
};

const TRACK_LABELS: Record<AudioTrackType, string> = {
  S: "Nói theo mẫu (말하기)",
  L: "Nghe (듣기)",
  P: "Phát âm (발음)",
  intro: "Giới thiệu",
};

export function getTrackLabel(type: AudioTrackType): string {
  return TRACK_LABELS[type];
}

/**
 * Với 1 trang, trả về danh sách { lesson, type } audio gắn ở trang đó.
 * `lesson` là số bài (1-N), dùng để build tên file `${lesson}-${type}.mp3`.
 * Trang 1 luôn có track `intro` (file `0.mp3` — giới thiệu chung cả sách).
 */
export function resolvePageAudio(
  bookId: string,
  page: number
): { lesson: number | null; type: AudioTrackType }[] {
  const layout = AUDIO_LAYOUTS[bookId];
  if (!layout) return [];

  const results: { lesson: number | null; type: AudioTrackType }[] = [];
  if (page === 1) results.push({ lesson: null, type: "intro" });

  for (let i = 0; i < layout.lessonStartPages.length; i++) {
    const start = layout.lessonStartPages[i];
    const offset = page - start;
    // lessonStartPages tăng dần: hết lượt offset>=0 nghĩa là đã qua bài
    // chứa trang này (khoảng cách giữa các bài luôn >= 10 nên không có
    // chuyện offset nhỏ khớp nhầm bài trước/sau).
    if (offset < 0) break;
    const types = layout.offsets[offset];
    if (types) {
      const lesson = i + 1;
      for (const type of types) results.push({ lesson, type });
      break;
    }
  }
  return results;
}

/** Danh sách số trang có audio của 1 sách, đã sắp xếp tăng dần. */
export function getAudioPages(bookId: string): number[] {
  const layout = AUDIO_LAYOUTS[bookId];
  if (!layout) return [];

  const pages = new Set<number>([1]); // trang 1 luôn có intro
  layout.lessonStartPages.forEach((start) => {
    Object.keys(layout.offsets).forEach((offsetKey) => {
      pages.add(start + Number(offsetKey));
    });
  });
  return Array.from(pages).sort((a, b) => a - b);
}

/**
 * Audio cho sách bài tập (익힘책) — khác cấu trúc textbook hoàn toàn: không
 * có mapping "bài học -> nhiều loại track" mà chỉ có 1 trang "듣기" duy nhất
 * mỗi bài, gắn đúng 2 track SỐ THỨ TỰ liên tục (track01, track02, ...),
 * KHÔNG có track intro (0.mp3) như textbook.
 *
 * Nguồn: https://hawoopub01.cafe24.com/satongebook/WB/STEP<N>/track<NN>.mp3
 * (cùng NXB Hawoo với audio textbook, khác domain path). Phát hiện + xác
 * minh bằng cách giải mã QR in trên từng trang (không suy đoán tự động —
 * xem `download_wb_audio.py` ở thư mục gốc `kiip/`), sau đó nhận ra khoảng
 * cách giữa các trang có audio ĐỀU NHAU (không như textbook), nên rút gọn
 * thành công thức để dễ bảo trì. Tổng số track đã verify qua HTTP HEAD
 * (track N+1 kế tiếp → 404): step1/2 = 36, step3/4 = 32.
 */
interface WorkbookAudioLayout {
  /** Số bài học trong sách. */
  lessonCount: number;
  /** Trang có audio của bài 1. */
  firstPage: number;
  /** Khoảng cách trang giữa 2 bài liên tiếp (đều nhau). */
  pageStep: number;
}

const WORKBOOK_1_2_LAYOUT: WorkbookAudioLayout = {
  lessonCount: 18,
  firstPage: 14,
  pageStep: 6,
};

const WORKBOOK_3_4_LAYOUT: WorkbookAudioLayout = {
  lessonCount: 16,
  firstPage: 15,
  pageStep: 8,
};

export const WORKBOOK_AUDIO_LAYOUTS: Record<string, WorkbookAudioLayout> = {
  "wb-step1": WORKBOOK_1_2_LAYOUT,
  "wb-step2": WORKBOOK_1_2_LAYOUT,
  "wb-step3": WORKBOOK_3_4_LAYOUT,
  "wb-step4": WORKBOOK_3_4_LAYOUT,
};

/** Với 1 trang sách bài tập, trả về số thứ tự track (1-based) gắn ở trang đó (0, 1 hoặc 2 track). */
export function resolveWorkbookPageAudio(bookId: string, page: number): number[] {
  const layout = WORKBOOK_AUDIO_LAYOUTS[bookId];
  if (!layout) return [];

  for (let lesson = 1; lesson <= layout.lessonCount; lesson++) {
    const audioPage = layout.firstPage + layout.pageStep * (lesson - 1);
    if (audioPage === page) return [2 * lesson - 1, 2 * lesson];
    if (audioPage > page) break; // trang tăng dần theo bài, qua rồi thì dừng
  }
  return [];
}

/** Danh sách số trang có audio của 1 sách bài tập, đã sắp xếp tăng dần. */
export function getWorkbookAudioPages(bookId: string): number[] {
  const layout = WORKBOOK_AUDIO_LAYOUTS[bookId];
  if (!layout) return [];
  return Array.from(
    { length: layout.lessonCount },
    (_, i) => layout.firstPage + layout.pageStep * i
  );
}

/**
 * Audio sách giáo trình tiếng Anh (English File) — cấu trúc khác hẳn tiếng
 * Hàn: số track in ngay trên trang dưới dạng "<File>.<số thứ tự>" (vd "1.5"),
 * tăng dần liên tục trong toàn bộ File thay vì lặp theo loại S/L/P. Vì vậy
 * KHÔNG dùng offset công thức mà liệt kê trực tiếp track xuất hiện ở từng
 * trang, xác minh bằng cách đọc ảnh trang thật (không suy diễn).
 *
 * QUAN TRỌNG — 2 hệ đánh số trang khác nhau, dễ nhầm:
 *   - Số trang IN TRÊN SÁCH (ảnh chụp, mục lục) — bắt đầu từ "6" (bài 1A).
 *   - Số `page` app dùng (URL, tên file ảnh page-XXXX.png/webp) — chính là
 *     THỨ TỰ TRANG SCAN, lệch +1 so với số in trên sách vì có trang bìa/lời
 *     mở đầu không đánh số ở đầu sách (đã verify: page-0007.png mới là bài
 *     "1A" tức trang in "6"). `pageTracks` dưới đây dùng số `page` của app
 *     (ĐÃ +1), không phải số in trên sách — comment cuối mỗi dòng ghi số in
 *     trên sách để đối chiếu khi khảo sát thêm.
 *
 * Một phần track của mỗi File (theo số thứ tự) không nằm ở 8 trang bài học
 * chính mà nằm ở các trang tham chiếu cuối sách (Grammar Bank, Vocabulary
 * Bank...), và các trang đó gộp chung nội dung nhiều File khác nhau nên
 * không suy ra được vị trí bằng công thức — các track này CHƯA được gắn vào
 * trang nào (bỏ qua) cho tới khi khảo sát thêm.
 *
 * en-elementary hiện chỉ có dữ liệu cho File 1–4 (khớp số audio đã tải ở
 * SB-audio/Elementary) — mở rộng thêm khi khảo sát audio File 5–12.
 */
export interface EnglishAudioLayout {
  /** page (số trang app dùng, = thứ tự scan, KHÔNG phải số in trên sách) -> danh sách track (dạng "<File>.<số>") xuất hiện trên trang đó. */
  pageTracks: Partial<Record<number, string[]>>;
}

const EN_ELEMENTARY_LAYOUT: EnglishAudioLayout = {
  pageTracks: {
    7: ["1.2", "1.3"], // trang in "6"
    8: ["1.5", "1.6", "1.7", "1.8", "1.9", "1.10", "1.14", "1.15", "1.16"], // trang in "7"
    9: ["1.17", "1.20", "1.21", "1.22"], // trang in "8"
    10: ["1.23", "1.26", "1.28", "1.29", "1.30"], // trang in "9"
    11: ["1.31", "1.32", "1.35", "1.36", "1.37", "1.38", "1.39", "1.40"], // trang in "10"
    12: ["1.41", "1.42", "1.44", "1.45"], // trang in "11"
    13: ["1.46", "1.47", "1.48"], // trang in "12"
    14: ["1.49", "1.50", "1.51", "1.52", "1.53"], // trang in "13"
    16: ["2.3", "2.4", "2.5"], // trang in "15"
    17: ["2.6"], // trang in "16"
    18: ["2.9", "2.10", "2.11", "2.12"], // trang in "17"
    19: ["2.13", "2.14", "2.15"], // trang in "18"
    20: ["2.17"], // trang in "19"
    22: ["2.18"], // trang in "21"
    23: ["3.1", "3.3", "3.4", "3.6", "3.7"], // trang in "22"
    25: ["3.8"], // trang in "24"
    26: ["3.12", "3.13", "3.14", "3.15", "3.16"], // trang in "25"
    27: ["3.17", "3.18", "3.19", "3.20"], // trang in "26"
    28: ["3.21", "3.24", "3.25"], // trang in "27"
    29: ["3.27"], // trang in "28"
    30: ["3.28", "3.29", "3.30", "3.31"], // trang in "29"
    31: ["4.1"], // trang in "30"
  },
};

export const ENGLISH_AUDIO_LAYOUTS: Record<string, EnglishAudioLayout> = {
  "en-elementary": EN_ELEMENTARY_LAYOUT,
};

/** Với 1 trang, trả về danh sách track (vd "1.5") xuất hiện trên trang đó. */
export function resolveEnglishPageAudio(bookId: string, page: number): string[] {
  const layout = ENGLISH_AUDIO_LAYOUTS[bookId];
  if (!layout) return [];
  return layout.pageTracks[page] ?? [];
}

/** Danh sách số trang có audio của 1 sách tiếng Anh, đã sắp xếp tăng dần. */
export function getEnglishAudioPages(bookId: string): number[] {
  const layout = ENGLISH_AUDIO_LAYOUTS[bookId];
  if (!layout) return [];
  return Object.keys(layout.pageTracks)
    .map(Number)
    .sort((a, b) => a - b);
}
