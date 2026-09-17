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
 * en-elementary đã khảo sát đủ 12 File (toàn bộ 8 trang bài học chính mỗi
 * File, 96 trang từ trang in "6" đến "101"), khớp bộ audio đầy đủ đã tải ở
 * SB-audio/Elementary.
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
    32: ["4.6"], // trang in "31"
    33: ["4.7", "4.9", "4.10"], // trang in "32"
    34: ["4.11", "4.12", "4.15"], // trang in "33"
    35: ["4.16", "4.21", "4.22"], // trang in "34"
    38: ["4.23"], // trang in "37"
    39: ["5.2", "5.3", "5.4"], // trang in "38"
    40: ["5.6", "5.7", "5.8"], // trang in "39"
    41: ["5.9", "5.10"], // trang in "40"
    42: ["5.11", "5.13", "5.14", "5.15", "5.16"], // trang in "41"
    43: ["5.19"], // trang in "42"
    44: ["5.21"], // trang in "43"
    45: ["5.22", "5.23", "5.24", "5.25"], // trang in "44"
    46: ["5.26", "5.27", "5.28"], // trang in "45"
    47: ["6.1"], // trang in "46"
    48: ["6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9"], // trang in "47"
    49: ["6.11", "6.12", "6.13", "6.14"], // trang in "48"
    51: ["6.16", "6.17", "6.18"], // trang in "50"
    52: ["6.22", "6.23", "6.24"], // trang in "51"
    54: ["6.25"], // trang in "53"
    55: ["7.1", "7.3", "7.4"], // trang in "54"
    56: ["7.5"], // trang in "55"
    58: ["7.8", "7.10", "7.11", "7.12", "7.13"], // trang in "57"
    60: ["7.14", "7.17", "7.18"], // trang in "59"
    61: ["7.19", "7.20", "7.21"], // trang in "60"
    62: ["7.22", "7.23", "7.24"], // trang in "61"
    63: ["8.1"], // trang in "62"
    64: ["8.4", "8.5", "8.6", "8.7", "8.8"], // trang in "63"
    65: ["8.11"], // trang in "64"
    66: ["8.13", "8.14", "8.16", "8.17", "8.18"], // trang in "65"
    67: ["8.19"], // trang in "66"
    68: ["8.20", "8.24"], // trang in "67"
    70: ["8.25"], // trang in "69"
    71: ["9.3", "9.4", "9.5"], // trang in "70"
    72: ["9.6", "9.7", "9.9", "9.10", "9.11"], // trang in "71"
    73: ["9.12"], // trang in "72"
    75: ["9.13", "9.15"], // trang in "74"
    76: ["9.19", "9.20"], // trang in "75"
    77: ["9.21", "9.22", "9.23"], // trang in "76"
    78: ["9.25", "9.26", "9.27", "9.28"], // trang in "77"
    79: ["10.3"], // trang in "78"
    80: ["10.6", "10.7"], // trang in "79"
    81: ["10.8"], // trang in "80"
    82: ["10.11", "10.12"], // trang in "81"
    84: ["10.13"], // trang in "83"
    85: ["10.14", "10.15", "10.16", "10.17", "10.18", "10.19"], // trang in "84"
    86: ["10.21"], // trang in "85"
    87: ["11.3", "11.4", "11.5"], // trang in "86"
    89: ["11.7", "11.8"], // trang in "88"
    91: ["11.9", "11.10"], // trang in "90"
    92: ["11.13"], // trang in "91"
    93: ["11.14", "11.15", "11.16"], // trang in "92"
    94: ["11.17", "11.18", "11.19"], // trang in "93"
    95: ["12.1"], // trang in "94"
    96: ["12.3", "12.4", "12.5", "12.6"], // trang in "95"
    98: ["12.10", "12.11", "12.12"], // trang in "97"
    99: ["12.13"], // trang in "98"
    100: ["12.14"], // trang in "99"
  },
};

/**
 * en-pre-intermediate — LƯU Ý offset khác en-elementary: trang in "6" (bài
 * 1A) trùng đúng số `page` app dùng "6" (không lệch +1), vì sách này có ít
 * hơn 1 trang mở đầu không đánh số. Đã verify: page-0006.png = bài "1A".
 * Cũng đã khảo sát đủ 12 File (96 trang, trang in "6" đến "101"), khớp bộ
 * audio đầy đủ đã tải ở SB-audio/Pre-Intermediate.
 *
 * Ghi chú: nguồn tải có 1 file bị trùng số "1.26" (2 file khác nội dung,
 * cùng nhãn) — không ảnh hưởng mapping này vì cả hai đều không xuất hiện
 * trên 8 trang bài học chính (thuộc phần Vocabulary Bank cuối sách).
 */
const EN_PRE_INTERMEDIATE_LAYOUT: EnglishAudioLayout = {
  pageTracks: {
    6: ["1.2", "1.3"],
    7: ["1.6", "1.7", "1.8", "1.9", "1.10"],
    8: ["1.11", "1.12"],
    9: ["1.17", "1.18", "1.19", "1.20", "1.21", "1.22"],
    10: ["1.24", "1.25"],
    11: ["1.28"],
    12: ["1.30"],
    13: ["1.31", "1.32", "1.33", "1.34"],
    14: ["2.1"],
    15: ["2.3", "2.4", "2.5", "2.9"],
    17: ["2.13", "2.14", "2.15"],
    18: ["2.16"],
    19: ["2.16", "2.20"],
    21: ["2.23"],
    22: ["3.1", "3.2", "3.3"],
    23: ["3.4", "3.5"],
    24: ["3.8", "3.9", "3.10"],
    25: ["3.11", "3.13"],
    26: ["3.14"],
    28: ["3.20", "3.21"],
    29: ["3.23", "3.24", "3.25"],
    31: ["4.3", "4.6", "4.7", "4.8", "4.9", "4.10"],
    32: ["4.15", "4.16", "4.17", "4.18"],
    34: ["4.21"],
    35: ["4.23", "4.24"],
    37: ["4.25"],
    39: ["5.1", "5.2", "5.3", "5.6"],
    41: ["5.13", "5.14"],
    42: ["5.15", "5.16"],
    44: ["5.21", "5.22", "5.23", "5.24"],
    45: ["5.25", "5.26", "5.27"],
    46: ["6.1", "6.3", "6.5", "6.6", "6.7", "6.8"],
    48: ["6.9"],
    49: ["6.13"],
    50: ["6.14", "6.15", "6.16"],
    53: ["6.21"],
    55: ["7.1", "7.4", "7.5"],
    57: ["7.8", "7.9", "7.10", "7.11"],
    59: ["7.12", "7.15", "7.16", "7.17"],
    60: ["7.18", "7.19"],
    61: ["7.20", "7.21", "7.22", "7.23", "7.24", "7.25"],
    62: ["8.1", "8.3"],
    63: ["8.5", "8.6", "8.7", "8.9", "8.10"],
    65: ["8.13", "8.14", "8.16", "8.17"],
    66: ["8.18", "8.19"],
    67: ["8.23", "8.24", "8.25"],
    69: ["8.27"],
    70: ["9.1", "9.4"],
    73: ["9.6", "9.8", "9.9", "9.10", "9.11"],
    74: ["9.12", "9.13"],
    75: ["9.16"],
    76: ["9.17", "9.18", "9.19"],
    77: ["9.20", "9.21", "9.22"],
    78: ["10.1", "10.2"],
    81: ["10.7", "10.8", "10.9"],
    82: ["10.10", "10.11", "10.12"],
    83: ["10.13"],
    85: ["10.15"],
    86: ["11.1", "11.2", "11.3"],
    87: ["11.5", "11.6", "11.7"],
    88: ["11.8", "11.9"],
    89: ["11.14", "11.15"],
    90: ["11.16"],
    91: ["11.18", "11.20", "11.21", "11.22"],
    92: ["11.23", "11.24"],
    93: ["11.25", "11.26", "11.27"],
    96: ["12.3"],
    97: ["12.6", "12.7"],
    101: ["12.10"],
  },
};

export const ENGLISH_AUDIO_LAYOUTS: Record<string, EnglishAudioLayout> = {
  "en-elementary": EN_ELEMENTARY_LAYOUT,
  "en-pre-intermediate": EN_PRE_INTERMEDIATE_LAYOUT,
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
