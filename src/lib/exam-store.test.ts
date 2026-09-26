import { beforeEach, describe, expect, it } from "vitest";
import { activeAttempt, finishedAttempts, migrateExamState, useExamStore } from "./exam-store";

beforeEach(() => useExamStore.setState({ practice: {}, attempts: [] }));

describe("luyện tập", () => {
  it("đổi đáp án thì bỏ dấu đã kiểm tra của câu đó", () => {
    const s = useExamStore.getState();
    s.setPracticeAnswer("e", "reading:101", 2);
    s.checkPractice("e", "reading:101");
    s.setPracticeAnswer("e", "reading:101", 3);

    expect(useExamStore.getState().practice.e).toEqual({ answers: { "reading:101": 3 }, checked: [] });
  });

  it("làm lại một khối thì xoá đáp án và dấu đã kiểm tra của đúng các câu đó", () => {
    const s = useExamStore.getState();
    s.setPracticeAnswer("e", "reading:131", 1);
    s.setPracticeAnswer("e", "reading:132", 2);
    s.checkPractice("e", "reading:131");
    s.checkPractice("e", "reading:132");
    s.resetPractice("e", ["reading:131"]);

    expect(useExamStore.getState().practice.e).toEqual({ answers: { "reading:132": 2 }, checked: ["reading:132"] });
  });
});

describe("thi thử", () => {
  it("mốc hết giờ tính từ lúc bắt đầu, sang phần sau thì đặt mốc mới", () => {
    const s = useExamStore.getState();
    const a = s.startMock("e", 45, 1_000);
    expect(a.deadline).toBe(1_000 + 45 * 60_000);

    s.nextMockSection(a.id, 75, 5_000);
    const now = activeAttempt(useExamStore.getState().attempts, "e")!;
    expect(now).toMatchObject({ sectionIndex: 1, deadline: 5_000 + 75 * 60_000 });
  });

  it("nộp rồi thì không sửa đáp án được nữa, điểm lúc nộp được lưu sẵn", () => {
    const s = useExamStore.getState();
    const a = s.startMock("e", 75);
    s.setMockAnswer(a.id, "reading:101", 2);
    s.finishMock(a.id, 320);
    s.setMockAnswer(a.id, "reading:101", 3);

    const done = finishedAttempts(useExamStore.getState().attempts, "e")[0];
    expect(done.answers["reading:101"]).toBe(2);
    expect(done.score).toBe(320);
  });

  it("một đề chỉ có một lượt đang làm dở — bắt đầu lượt mới là bỏ lượt dở cũ", () => {
    const s = useExamStore.getState();
    s.startMock("e", 75, 1);
    s.startMock("e", 75, 2);

    expect(useExamStore.getState().attempts).toHaveLength(1);
  });
});

describe("bản lưu cũ", () => {
  it("khoá theo số câu trơn (version 0) chuyển sang khoá theo phần: 1–30 là nghe, 31–70 là đọc", () => {
    const old = {
      practice: { "de-cu": { answers: { 3: 2, 45: 1 }, checked: [3, 45] } },
      attempts: [{ id: "x", examId: "de-cu", answers: { 30: 4, 31: 1 } }],
    };
    expect(migrateExamState(old, 0)).toMatchObject({
      practice: {
        "de-cu": { answers: { "listening:3": 2, "reading:45": 1 }, checked: ["listening:3", "reading:45"] },
      },
      attempts: [{ answers: { "listening:30": 4, "reading:31": 1 } }],
    });
  });

  it("bản lưu mới thì giữ nguyên, kể cả trường thừa của bản cũ (bài viết, cấp) — không đọc tới nhưng không vỡ", () => {
    const now = {
      practice: { "de-cu": { answers: {}, checked: [], texts: { "53:0": "x" }, grades: { 53: 20 } } },
      attempts: [{ id: "x", examId: "de-cu", answers: {}, finishedAt: "2026-01-01", score: 150, level: "Cấp 3" }],
    };
    expect(migrateExamState(now, 1)).toBe(now);
    useExamStore.setState(now as never);
    expect(finishedAttempts(useExamStore.getState().attempts)[0].score).toBe(150);
  });
});
