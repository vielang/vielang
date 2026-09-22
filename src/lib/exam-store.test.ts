import { beforeEach, describe, expect, it } from "vitest";
import { activeAttempt, finishedAttempts, migrateExamState, useExamStore } from "./exam-store";

beforeEach(() => useExamStore.setState({ practice: {}, attempts: [] }));

describe("luyện tập", () => {
  it("đổi đáp án thì bỏ dấu đã kiểm tra của câu đó", () => {
    const s = useExamStore.getState();
    s.setPracticeAnswer("e", "reading:1", 2);
    s.checkPractice("e", "reading:1");
    s.setPracticeAnswer("e", "reading:1", 3);

    expect(useExamStore.getState().practice.e).toEqual({ answers: { "reading:1": 3 }, checked: [] });
  });
});

describe("thi thử", () => {
  it("mốc hết giờ tính từ lúc bắt đầu, sang phần sau thì đặt mốc mới", () => {
    const s = useExamStore.getState();
    const a = s.startMock("e", 40, 1_000);
    expect(a.deadline).toBe(1_000 + 40 * 60_000);

    s.nextMockSection(a.id, 60, 5_000);
    const now = activeAttempt(useExamStore.getState().attempts, "e")!;
    expect(now).toMatchObject({ sectionIndex: 1, deadline: 5_000 + 60 * 60_000 });
  });

  it("nộp rồi thì không sửa đáp án được nữa", () => {
    const s = useExamStore.getState();
    const a = s.startMock("e", 40);
    s.setMockAnswer(a.id, "listening:1", 2);
    s.setMockText(a.id, "53:0", "초안");
    s.finishMock(a.id, 4, null);
    s.setMockAnswer(a.id, "listening:1", 3);
    s.setMockText(a.id, "53:0", "고친 글");

    const done = finishedAttempts(useExamStore.getState().attempts, "e")[0];
    expect(done.answers["listening:1"]).toBe(2);
    expect(done.texts).toEqual({ "53:0": "초안" });
  });

  it("một đề chỉ có một lượt đang làm dở — bắt đầu lượt mới là bỏ lượt dở cũ", () => {
    const s = useExamStore.getState();
    s.startMock("e", 40, 1);
    s.startMock("e", 40, 2);

    expect(useExamStore.getState().attempts).toHaveLength(1);
  });

  it("câu viết tự chấm SAU khi nộp, điểm tổng cập nhật theo", () => {
    const s = useExamStore.getState();
    const a = s.startMock("e", 60);
    s.gradeMock(a.id, 53, 20, 120, "Cấp 3");
    expect(useExamStore.getState().attempts[0].grades).toBeUndefined(); // chưa nộp thì chưa chấm

    s.finishMock(a.id, 100, null);
    s.gradeMock(a.id, 53, 20, 120, "Cấp 3");
    expect(useExamStore.getState().attempts[0]).toMatchObject({ grades: { 53: 20 }, score: 120, level: "Cấp 3" });
  });
});

describe("bản lưu cũ (khoá theo số câu trơn)", () => {
  it("chuyển sang khoá theo phần: 1–30 là nghe, 31–70 là đọc (lúc đó mới có TOPIK I)", () => {
    const old = {
      practice: { "102-topik1": { answers: { 3: 2, 45: 1 }, checked: [3, 45] } },
      attempts: [{ id: "x", examId: "102-topik1", answers: { 30: 4, 31: 1 } }],
    };
    expect(migrateExamState(old, 0)).toMatchObject({
      practice: {
        "102-topik1": { answers: { "listening:3": 2, "reading:45": 1 }, checked: ["listening:3", "reading:45"] },
      },
      attempts: [{ answers: { "listening:30": 4, "reading:31": 1 } }],
    });
  });

  it("bản lưu mới thì giữ nguyên", () => {
    const now = { practice: {}, attempts: [] };
    expect(migrateExamState(now, 1)).toBe(now);
  });
});
