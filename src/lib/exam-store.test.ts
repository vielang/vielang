import { beforeEach, describe, expect, it } from "vitest";
import { activeAttempt, finishedAttempts, useExamStore } from "./exam-store";

beforeEach(() => useExamStore.setState({ practice: {}, attempts: [] }));

describe("luyện tập", () => {
  it("đổi đáp án thì bỏ dấu đã kiểm tra của câu đó", () => {
    const s = useExamStore.getState();
    s.setPracticeAnswer("e", 1, 2);
    s.checkPractice("e", 1);
    s.setPracticeAnswer("e", 1, 3);

    expect(useExamStore.getState().practice.e).toEqual({ answers: { 1: 3 }, checked: [] });
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
    s.setMockAnswer(a.id, 1, 2);
    s.finishMock(a.id, 4, null);
    s.setMockAnswer(a.id, 1, 3);

    expect(finishedAttempts(useExamStore.getState().attempts, "e")[0].answers[1]).toBe(2);
  });

  it("một đề chỉ có một lượt đang làm dở — bắt đầu lượt mới là bỏ lượt dở cũ", () => {
    const s = useExamStore.getState();
    s.startMock("e", 40, 1);
    s.startMock("e", 40, 2);

    expect(useExamStore.getState().attempts).toHaveLength(1);
  });
});
