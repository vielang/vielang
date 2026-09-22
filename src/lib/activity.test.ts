import { describe, expect, it } from "vitest";
import {
  bestStreak,
  countStudyDays,
  currentStreak,
  dayKey,
  EMPTY_DAY,
  heatLevel,
  heatmapWeeks,
  minutesThisWeek,
  type DayStats,
} from "./activity";

const MIN = 60_000;
const day = (minutes: number): DayStats => ({ ...EMPTY_DAY, activeMs: minutes * MIN });

// Thứ Tư 10/9/2025, giờ địa phương.
const WED = new Date(2025, 8, 10, 9, 0);

describe("khoá ngày", () => {
  it("theo giờ địa phương, không theo UTC", () => {
    // 0h30 sáng: cắt theo UTC ở múi giờ dương sẽ ra ngày hôm trước.
    expect(dayKey(new Date(2025, 8, 10, 0, 30))).toBe("2025-09-10");
  });
});

describe("chuỗi ngày học", () => {
  it("đếm các ngày liên tiếp tới hôm nay", () => {
    const days = { "2025-09-08": day(5), "2025-09-09": day(5), "2025-09-10": day(5) };
    expect(currentStreak(days, WED)).toBe(3);
  });

  it("hôm nay chưa học thì chuỗi vẫn tính tới hôm qua", () => {
    // Ngày còn chưa hết — mở app buổi sáng mà thấy chuỗi về 0 là phạt oan.
    const days = { "2025-09-08": day(5), "2025-09-09": day(5) };
    expect(currentStreak(days, WED)).toBe(2);
  });

  it("nghỉ một ngày là gãy", () => {
    const days = { "2025-09-07": day(5), "2025-09-09": day(5), "2025-09-10": day(5) };
    expect(currentStreak(days, WED)).toBe(2);
  });

  it("lướt dưới 1 phút không tính là ngày học", () => {
    const days = { "2025-09-09": { ...EMPTY_DAY, activeMs: 30_000 }, "2025-09-10": day(5) };
    expect(currentStreak(days, WED)).toBe(1);
    expect(countStudyDays(days)).toBe(1);
  });

  it("chuỗi dài nhất vượt qua ranh giới tháng", () => {
    const days = {
      "2025-08-30": day(5),
      "2025-08-31": day(5),
      "2025-09-01": day(5),
      "2025-09-05": day(5),
    };
    expect(bestStreak(days)).toBe(3);
  });
});

describe("phút học trong tuần", () => {
  it("tính từ thứ Hai, không lấy Chủ nhật tuần trước", () => {
    const days = { "2025-09-07": day(100), "2025-09-08": day(20), "2025-09-10": day(15) };
    expect(minutesThisWeek(days, WED)).toBe(35);
  });
});

describe("lịch 12 tuần", () => {
  it("12 cột × 7 ngày, cột cuối là tuần này, ngày sau hôm nay để trống", () => {
    const weeks = heatmapWeeks({ "2025-09-10": day(25) }, WED);
    expect(weeks).toHaveLength(12);
    expect(weeks.every((w) => w.length === 7)).toBe(true);

    const thisWeek = weeks[11];
    expect(thisWeek[0].key).toBe("2025-09-08"); // thứ Hai
    expect(thisWeek[2]).toMatchObject({ key: "2025-09-10", minutes: 25, future: false });
    expect(thisWeek[3].future).toBe(true);
  });

  it("mức đậm theo ngưỡng phút cố định", () => {
    expect([0, 5, 15, 30, 90].map(heatLevel)).toEqual([0, 1, 2, 3, 4]);
  });
});
