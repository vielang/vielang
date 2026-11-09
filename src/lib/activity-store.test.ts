import { beforeEach, describe, expect, it } from "vitest";
import { useActivityStore } from "./activity-store";

const MON = new Date(2025, 8, 8, 20, 0);
const TUE = new Date(2025, 8, 9, 7, 0);

beforeEach(() => useActivityStore.setState({ days: {} }));

describe("lịch sử học", () => {
  it("cộng thời gian học vào đúng ngày", () => {
    const { addActiveTime } = useActivityStore.getState();
    addActiveTime(15_000, MON);
    addActiveTime(15_000, MON);
    addActiveTime(15_000, TUE);
    const { days } = useActivityStore.getState();

    expect(days["2025-09-08"].activeMs).toBe(30_000);
    expect(days["2025-09-09"].activeMs).toBe(15_000);
  });

  it("đếm lượt chấm và lượt đúng", () => {
    const { recordQuizCheck } = useActivityStore.getState();
    recordQuizCheck(true, MON);
    recordQuizCheck(false, MON);

    expect(useActivityStore.getState().days["2025-09-08"]).toMatchObject({
      quizChecked: 2,
      quizCorrect: 1,
    });
  });
});

describe("nhiều tab cùng mở", () => {
  it("tab khác ghi thì tab này nạp lại, không ghi đè bằng bản cũ", async () => {
    const fromOtherTab = {
      state: { days: { "2025-09-08": { ...useActivityStore.getState().days["2025-09-08"], activeMs: 999_000, quizChecked: 0, quizCorrect: 0 } }, weeklyGoalMinutes: 90 },
      version: 0,
    };
    const newValue = JSON.stringify(fromOtherTab);
    localStorage.setItem("kiip-activity-v1", newValue);
    window.dispatchEvent(new StorageEvent("storage", { key: "kiip-activity-v1", newValue }));
    await Promise.resolve();

    expect(useActivityStore.getState().days["2025-09-08"].activeMs).toBe(999_000);
  });
});
