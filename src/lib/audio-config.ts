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
