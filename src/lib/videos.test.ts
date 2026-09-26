import { describe, expect, it } from "vitest";
import { buildTranscript, cueIndexAt, getVideoLessons, lastStartedCueIndex, type VideoCue } from "@/lib/videos";

const cue = (s: number, e: number, t: string): VideoCue => ({ s, e, t, words: [{ t, at: s }] });

describe("buildTranscript", () => {
  it("ghép câu Việt theo thời gian, kể cả khi một câu Hàn bị tách làm hai câu Việt", () => {
    const lines = buildTranscript({
      koCues: [cue(0, 2, "가"), cue(2, 5, "나")],
      viCues: [cue(0, 2, "A"), cue(2, 3.5, "B1"), cue(3.5, 5, "B2")],
    });
    expect(lines.map((l) => l.vi)).toEqual(["A", "B1 B2"]);
  });

  it("tập không có phụ đề Việt thì không có dòng Việt", () => {
    const lines = buildTranscript({ koCues: [cue(0, 2, "가")], viCues: [] });
    expect(lines[0].vi).toBeUndefined();
  });

  it("mọi câu Việt của dữ liệu thật đều được ghép vào một câu Hàn nào đó", () => {
    for (const lesson of getVideoLessons()) {
      const joined = buildTranscript(lesson).map((l) => l.vi ?? "").join(" ");
      for (const c of lesson.viCues) expect(joined).toContain(c.t);
    }
  });
});

describe("tìm câu theo thời gian", () => {
  const cues = [cue(1, 2, "a"), cue(3, 4, "b")];

  it("khoảng lặng giữa hai câu: không có câu đang phát, câu hiện tại là câu vừa qua", () => {
    expect(cueIndexAt(cues, 2.5)).toBe(-1);
    expect(lastStartedCueIndex(cues, 2.5)).toBe(0);
  });

  it("trước câu đầu tiên thì chưa có câu hiện tại", () => {
    expect(lastStartedCueIndex(cues, 0.5)).toBe(-1);
  });
});
