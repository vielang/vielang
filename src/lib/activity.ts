/**
 * Lịch sử học theo ngày — phần tính toán thuần (không đụng store, không đụng
 * DOM), để My page và test cùng dựa vào một chỗ.
 *
 * Vì sao TỔNG HỢP THEO NGÀY chứ không ghi từng thao tác: lịch sử nằm trong
 * localStorage (vài MB, dùng chung với note và tiến độ). Mỗi lần lật trang,
 * mỗi lần chấm câu mà ghi một dòng thì học đều vài tháng là phình tới mức
 * đáng lo; gộp theo ngày thì cả năm chỉ ~365 dòng nhỏ. Cái mất đi là thứ tự
 * thao tác trong ngày — My page không cần đến nó.
 */

export interface DayStats {
  /** Thời gian học thật (trang đọc đang hiện VÀ có thao tác gần đây), ms. */
  activeMs: number;
  /** Trang đã học trong ngày, dạng "bookId:page", không trùng. */
  pages: string[];
  /** Số lượt bấm "Kiểm tra" trên câu chấm được. */
  quizChecked: number;
  /** Trong số đó, bao nhiêu lượt đúng. */
  quizCorrect: number;
  /** Số lần mở bong bóng đáp án sách. */
  answersOpened: number;
  /** Số bản ghi âm luyện nói đã lưu. */
  recordings: number;
}

export const EMPTY_DAY: DayStats = {
  activeMs: 0,
  pages: [],
  quizChecked: 0,
  quizCorrect: 0,
  answersOpened: 0,
  recordings: 0,
};

/**
 * Học ít nhất ngần này trong ngày mới tính là "một ngày học" (cho chuỗi ngày
 * và lịch). Mở app lướt 10 giây rồi tắt thì không nên giữ được chuỗi.
 */
export const MIN_STUDY_DAY_MS = 60_000;

/**
 * "YYYY-MM-DD" theo GIỜ ĐỊA PHƯƠNG của máy, không phải UTC.
 *
 * Dùng `toISOString()` là cắt theo UTC: người ở Hàn (UTC+9) học lúc 8 giờ
 * sáng sẽ bị tính vào ngày hôm trước, và chuỗi ngày gãy vô cớ.
 */
export function dayKey(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/** Cộng/trừ ngày theo lịch (không cộng 24h — đổi giờ mùa hè sẽ lệch). */
export function addDays(d: Date, n: number): Date {
  const out = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  out.setDate(out.getDate() + n);
  return out;
}

/** Thứ Hai đầu tuần chứa `d` — tuần bắt đầu từ thứ Hai như lịch Việt/Hàn. */
export function weekStart(d: Date): Date {
  const offset = (d.getDay() + 6) % 7; // CN=0 -> 6, T2=1 -> 0
  return addDays(d, -offset);
}

export function isStudyDay(stats: DayStats | undefined): boolean {
  return (stats?.activeMs ?? 0) >= MIN_STUDY_DAY_MS;
}

/**
 * Chuỗi ngày học liên tục tính tới hôm nay.
 *
 * Hôm nay chưa học thì chuỗi vẫn tính tới hôm qua: mở app buổi sáng mà thấy
 * chuỗi 12 ngày thành 0 chỉ vì chưa kịp học là phạt oan — ngày còn chưa hết.
 */
export function currentStreak(days: Record<string, DayStats>, today: Date): number {
  let cursor = isStudyDay(days[dayKey(today)]) ? today : addDays(today, -1);
  let streak = 0;
  while (isStudyDay(days[dayKey(cursor)])) {
    streak++;
    cursor = addDays(cursor, -1);
  }
  return streak;
}

/** Chuỗi dài nhất từng có. */
export function bestStreak(days: Record<string, DayStats>): number {
  const keys = Object.keys(days)
    .filter((k) => isStudyDay(days[k]))
    .sort();
  let best = 0;
  let run = 0;
  let prev: Date | null = null;
  for (const key of keys) {
    const [y, m, d] = key.split("-").map(Number);
    const date = new Date(y, m - 1, d);
    run = prev && dayKey(addDays(prev, 1)) === key ? run + 1 : 1;
    best = Math.max(best, run);
    prev = date;
  }
  return best;
}

export function countStudyDays(days: Record<string, DayStats>): number {
  return Object.values(days).filter(isStudyDay).length;
}

/** Số phút học trong tuần (thứ Hai → hôm nay) chứa `today`. */
export function minutesThisWeek(days: Record<string, DayStats>, today: Date): number {
  const start = weekStart(today);
  let ms = 0;
  for (let i = 0; i < 7; i++) {
    ms += days[dayKey(addDays(start, i))]?.activeMs ?? 0;
  }
  return Math.floor(ms / 60_000);
}

export interface HeatCell {
  key: string;
  minutes: number;
  /** Ngày sau hôm nay — vẽ ô trống, không phải "0 phút". */
  future: boolean;
}

/**
 * Lưới lịch học: `weeks` cột (cũ → mới), mỗi cột 7 ô thứ Hai → Chủ nhật.
 * Cột cuối là tuần hiện tại, nên các ngày sau hôm nay đánh dấu `future`.
 */
export function heatmapWeeks(
  days: Record<string, DayStats>,
  today: Date,
  weeks = 12
): HeatCell[][] {
  const todayKey = dayKey(today);
  const firstMonday = addDays(weekStart(today), -7 * (weeks - 1));
  return Array.from({ length: weeks }, (_, w) =>
    Array.from({ length: 7 }, (_, d) => {
      const date = addDays(firstMonday, w * 7 + d);
      const key = dayKey(date);
      return {
        key,
        minutes: Math.floor((days[key]?.activeMs ?? 0) / 60_000),
        future: key > todayKey,
      };
    })
  );
}

/**
 * Mức đậm nhạt của một ô lịch, 0–4. Ngưỡng theo phút học thật chứ không chia
 * tỉ lệ theo ngày nhiều nhất: chia tỉ lệ thì một ngày học dồn 3 tiếng làm mọi
 * ngày khác nhạt đi, và cùng 20 phút mà tuần này đậm tuần sau nhạt.
 */
export function heatLevel(minutes: number): 0 | 1 | 2 | 3 | 4 {
  if (minutes < 1) return 0;
  if (minutes < 10) return 1;
  if (minutes < 20) return 2;
  if (minutes < 40) return 3;
  return 4;
}
