import { beforeEach, describe, expect, it } from "vitest";
import { answersHidden, useReaderPrefsStore } from "./reader-prefs-store";

beforeEach(() => useReaderPrefsStore.setState({ hiddenAnswerBooks: [] }));

describe("bật/tắt chấm đáp án theo từng sách", () => {
  it("mặc định là hiện, kể cả với sách chưa từng đụng tới", () => {
    // Lưu danh sách TẮT: sách thêm vào sau vẫn hiện mà không phải bật tay.
    expect(answersHidden(useReaderPrefsStore.getState(), "step1")).toBe(false);
  });

  it("tắt một cuốn không ảnh hưởng cuốn khác", () => {
    useReaderPrefsStore.getState().toggleAnswersHidden("wb-step1");
    const state = useReaderPrefsStore.getState();

    expect(answersHidden(state, "wb-step1")).toBe(true);
    expect(answersHidden(state, "step1")).toBe(false);
  });

  it("bấm lần nữa thì bật lại", () => {
    const { toggleAnswersHidden } = useReaderPrefsStore.getState();
    toggleAnswersHidden("step1");
    toggleAnswersHidden("step1");

    expect(useReaderPrefsStore.getState().hiddenAnswerBooks).toEqual([]);
  });
});
