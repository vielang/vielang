import { describe, expect, it } from "vitest";
import { BOOKS } from "@/lib/books";
import { EMPTY_DAY, type DayStats } from "@/lib/activity";
import { computeAbility } from "@/lib/ability";
import {
  buildSuggestions,
  companionMessage,
  daysSinceLastStudy,
  greeting,
  shortBookName,
} from "./companion";

const MIN = 60_000;
const day = (minutes: number): DayStats => ({ ...EMPTY_DAY, activeMs: minutes * MIN });
const WED = new Date(2025, 8, 10, 20, 0);
const base = { now: WED, streak: 0, weekMinutes: 0, goal: 90 };

describe("tên sách ngắn", () => {
  it("đủ ngắn để không đẩy mất số trang trên điện thoại", () => {
    expect(shortBookName(BOOKS.find((b) => b.id === "wb-step1")!)).toBe("Bài tập Sơ cấp 1");
    expect(shortBookName(BOOKS.find((b) => b.id === "step3")!)).toBe("Giáo trình Trung cấp 1");
  });
});

describe("lời chào", () => {
  it("theo giờ trong ngày", () => {
    expect(greeting(new Date(2025, 8, 10, 7))).toBe("Chào buổi sáng");
    expect(greeting(new Date(2025, 8, 10, 20))).toBe("Chào buổi tối");
  });
});

describe("lời nhắn đồng hành", () => {
  it("đã học hôm nay thì ghi nhận số phút", () => {
    const msg = companionMessage({ ...base, days: { "2025-09-10": day(25) }, streak: 3 });
    expect(msg.title).toBe("Hôm nay bạn đã học 25 phút");
    expect(msg.body).toContain("3 ngày liên tiếp");
  });

  it("đạt mục tiêu tuần thì chúc mừng", () => {
    const msg = companionMessage({ ...base, days: { "2025-09-10": day(25) }, weekMinutes: 95 });
    expect(msg.title).toContain("đạt mục tiêu tuần");
  });

  it("chưa học hôm nay nhưng đang có chuỗi thì nhắc giữ chuỗi", () => {
    const msg = companionMessage({ ...base, days: { "2025-09-09": day(20) }, streak: 4 });
    expect(msg.title).toBe("Giữ chuỗi 4 ngày nhé");
  });

  it("nghỉ lâu thì chào mừng quay lại, không đếm ngày đã bỏ", () => {
    const msg = companionMessage({ ...base, days: { "2025-06-01": day(20) } });
    expect(msg.title).toBe("Mừng bạn quay lại!");
    expect(msg.body).not.toMatch(/\d+ ngày/);
  });

  it("đếm đúng số ngày kể từ lần học gần nhất", () => {
    expect(daysSinceLastStudy({ "2025-09-07": day(5) }, WED)).toBe(3);
    expect(daysSinceLastStudy({}, WED)).toBeNull();
  });
});

describe("gợi ý việc nên làm", () => {
  it("mỗi loại một việc, tối đa ba, ngữ pháp yếu nhất lên đầu", () => {
    const ability = computeAbility(
      {
        "wb-step1:12:p12-1": { grade: 0, at: "" },
        "wb-step1:10:p10-1": { grade: 0, at: "" },
        "wb-step1:10:p10-2": { grade: 0.5, at: "" },
      },
      {}
    );
    const out = buildSuggestions(
      ability,
      [{ bookId: "step1", page: 19, where: "읽기 1 · 2)" }],
      BOOKS
    );

    expect(out.map((s) => s.kind)).toEqual(["grammar", "redo", "quiz"]);
    expect(out[0].href).toBe("/read/step1/15"); // trang giải thích trong giáo trình
    // Bài của chính điểm ngữ pháp vừa gợi ý thì không nhắc lại lần nữa.
    expect(out[1].href).toBe("/read/wb-step1/10");
  });

  it("chưa có gì để gợi ý thì trả rỗng", () => {
    expect(buildSuggestions(computeAbility({}, {}), [], BOOKS)).toEqual([]);
  });
});
