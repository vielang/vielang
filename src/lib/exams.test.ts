import { describe, expect, it } from "vitest";
import { existsSync } from "node:fs";
import path from "node:path";
import {
  examTitle,
  groupAudio,
  isImage,
  listExams,
  milestoneLabel,
  milestones,
  nextMilestone,
  nextPracticeTarget,
  optionLabel,
  practiceBlocks,
  practiceProgress,
  qKey,
  questionAudio,
  questionPrompt,
  reachedMilestone,
  scaledScore,
  scoreExam,
  splitInstruction,
  type Answers,
  type Exam,
  type ExamQuestion,
} from "./exams";
import { examLevelOfPath } from "./exam-levels";

/**
 * Đề mẫu tự soạn, rút gọn — bộ chấm điểm, chia khối, nghe lại theo khối
 * không phụ thuộc đề thật nào, nên không bám vào dữ liệu trong content/exams.
 */
const OPTIONS: ExamQuestion["options"] = [{ html: "one" }, { html: "two" }, { html: "three" }, { html: "four" }];

const q = (
  no: number,
  answer: ExamQuestion["answer"],
  extra: Partial<Pick<ExamQuestion, "audio" | "replay" | "prompt" | "explanation">> = {}
): ExamQuestion => ({ no, points: 1, answer, layout: 2, prompt: { html: "" }, options: OPTIONS, ...extra });

const PART5 =
  "Part 5 — Incomplete Sentences. A word or phrase is missing in each of the sentences below. Four answer choices are given below each sentence. Select the best answer to complete the sentence.";

/**
 * Đề đủ hai phần, rút gọn. Phần nghe:
 * - khối 1–2: lời chỉ dẫn 0–5 giây, mỗi câu một đoạn riêng;
 * - khối 3–4: nghe một hội thoại (60–70) rồi trả lời hai câu — đoạn của
 *   câu 4 chỉ còn tiếng đọc câu hỏi và khoảng dừng.
 * Phần đọc: Part 5 (hai câu độc lập) và Part 6 (văn bản có hai chỗ trống).
 */
const full: Exam = {
  id: "toeic-mau-1",
  round: 1,
  year: 2026,
  level: "TOEIC",
  title: "Đề mẫu TOEIC",
  assetDir: "mau",
  source: "Đề mẫu",
  sections: [
    {
      id: "listening",
      title: "Listening",
      minutes: 45,
      audio: "listening.mp3",
      groups: [
        { from: 1, to: 2, instruction: "Part 1 — Photographs.", audio: [0, 5] },
        {
          from: 3,
          to: 4,
          instruction: "Part 3 — Questions 3-4 refer to the following conversation.",
          audio: [55, 60],
          dialogue: [60, 70],
        },
      ],
      questions: [
        q(1, 2, { audio: [5, 30], replay: [5, 10] }),
        q(2, 1, { audio: [30, 55], replay: [30, 35] }),
        q(3, 4, { audio: [60, 90], replay: [60, 75] }),
        q(4, 3, { audio: [90, 115], replay: [90, 95] }),
      ],
    },
    {
      id: "reading",
      title: "Reading",
      minutes: 75,
      groups: [
        { from: 101, to: 102, instruction: PART5 },
        {
          from: 131,
          to: 132,
          instruction: "Part 6 — Questions 131-132 refer to the following e-mail.",
          passage: '<div class="exam-box"><b>To:</b> All staff<br>Our office <b>(131)</b> ------- next month. <b>(132)</b> -------</div>',
        },
      ],
      questions: [
        q(101, 1, { prompt: { html: "The manager will ------- the report." }, explanation: "Sau will là động từ nguyên mẫu." }),
        q(102, 2, { prompt: { html: "Please send ------- by Friday." } }),
        q(131, 3),
        q(132, 4),
      ],
    },
  ],
};

/** Đề chỉ có phần đọc (như đề TOEIC Reading đầu tiên của VieLang). */
const readingOnly: Exam = { ...full, id: "toeic-mau-2", title: undefined, round: 2, sections: [full.sections[1]] };

/** Chọn đúng hết mọi câu. */
const allRight = (e: Exam): Answers =>
  Object.fromEntries(e.sections.flatMap((s) => s.questions.map((x) => [qKey(s.id, x.no), x.answer])));

describe("quy đổi điểm TOEIC (ước tính)", () => {
  it("đúng hết là 495, không đúng câu nào là 5 — mỗi phần", () => {
    for (const id of ["listening", "reading"] as const) {
      expect(scaledScore(id, 100, 100)).toBe(495);
      expect(scaledScore(id, 0, 100)).toBe(5);
    }
  });

  it("điểm luôn là bội của 5 và không giảm khi đúng thêm câu", () => {
    for (const id of ["listening", "reading"] as const) {
      let last = 0;
      for (let c = 0; c <= 100; c++) {
        const s = scaledScore(id, c, 100);
        expect(s % 5).toBe(0);
        expect(s).toBeGreaterThanOrEqual(last);
        last = s;
      }
    }
  });

  it("nội suy giữa các mốc của bảng tham khảo; phần nghe nhỉnh hơn phần đọc ở cùng số câu đúng", () => {
    expect(scaledScore("reading", 50, 100)).toBe(215);
    expect(scaledScore("listening", 50, 100)).toBe(235);
    for (const c of [30, 60, 90]) {
      expect(scaledScore("listening", c, 100)).toBeGreaterThanOrEqual(scaledScore("reading", c, 100));
    }
  });

  it("phần ít hơn 100 câu thì quy về thang 100 trước khi tra bảng", () => {
    expect(scaledScore("reading", 2, 4)).toBe(scaledScore("reading", 50, 100));
    expect(scaledScore("reading", 4, 4)).toBe(495);
  });

  it("tổng điểm là tổng điểm quy đổi các phần, tối đa 495 × số phần", () => {
    const r = scoreExam(full, allRight(full));
    expect(r).toMatchObject({ score: 990, max: 990 });
    expect(r.sections).toEqual([
      { id: "listening", score: 495, max: 495, correct: 4, total: 4 },
      { id: "reading", score: 495, max: 495, correct: 4, total: 4 },
    ]);
    expect(scoreExam(readingOnly, {})).toMatchObject({ score: 5, max: 495, milestone: null });
  });

  it("câu bỏ trống hay sai thì không tính là đúng", () => {
    const r = scoreExam(readingOnly, { "reading:101": 1, "reading:102": 1 });
    expect(r.sections[0]).toMatchObject({ correct: 1, total: 4, score: scaledScore("reading", 1, 4) });
  });
});

describe("mốc mục tiêu", () => {
  it("đề đủ hai phần: 450 / 600 / 750 / 900", () => {
    expect(milestones(full).map((m) => m.score)).toEqual([450, 600, 750, 900]);
    expect(milestoneLabel(full, milestones(full)[1])).toBe("600");
  });

  it("đề chỉ có phần đọc: một nửa, tương đương tổng ~450 / 600 / 750 / 900", () => {
    expect(milestones(readingOnly)).toEqual([
      { total: 450, score: 225 },
      { total: 600, score: 300 },
      { total: 750, score: 375 },
      { total: 900, score: 450 },
    ]);
    expect(milestoneLabel(readingOnly, milestones(readingOnly)[1])).toBe("~600");
  });

  it("mốc đã chạm và mốc kế tiếp", () => {
    expect(reachedMilestone(readingOnly, 224)).toBeNull();
    expect(reachedMilestone(readingOnly, 310)).toEqual({ total: 600, score: 300 });
    expect(nextMilestone(readingOnly, 310)).toEqual({ milestone: { total: 750, score: 375 }, need: 65 });
    expect(nextMilestone(readingOnly, 495)).toBeNull();
    expect(scoreExam(full, allRight(full)).milestone).toEqual({ total: 900, score: 900 });
  });
});

describe("hiển thị đề", () => {
  it("nhãn lựa chọn (A)–(D) như đề in", () => {
    expect([1, 2, 3, 4].map(optionLabel)).toEqual(["(A)", "(B)", "(C)", "(D)"]);
  });

  it("tên đề lấy từ `title`, không có thì dựng từ kỳ thi + số đề", () => {
    expect(examTitle(full)).toBe("Đề mẫu TOEIC");
    expect(examTitle(readingOnly)).toBe("TOEIC · Đề 2");
  });

  it("đề câu rỗng (Part 6 — chỗ trống nằm trong văn bản) thì không hiện", () => {
    expect(questionPrompt({ html: "" })).toBeNull();
    expect(questionPrompt({ html: "  " })).toBeNull();
    const p = { html: "Why was the e-mail sent?" };
    expect(questionPrompt(p)).toBe(p);
  });

  it("tách lời chỉ dẫn thành nhãn phần, khoảng câu và phần chữ", () => {
    expect(splitInstruction("Part 6 — Questions 131-134 refer to the following e-mail.")).toEqual({
      part: "Part 6",
      range: "131–134",
      text: "Questions 131-134 refer to the following e-mail.",
    });
    expect(splitInstruction(PART5)).toMatchObject({ part: "Part 5", range: null });
    expect(splitInstruction(PART5).text).toMatch(/^Incomplete Sentences\./);
    expect(splitInstruction("Directions: read the text.")).toEqual({
      part: null,
      range: null,
      text: "Directions: read the text.",
    });
  });

  it("đường dẫn đề suy ra kỳ thi mà không cần nạp dữ liệu đề", () => {
    expect(examLevelOfPath("/exam/toeic")).toBe("TOEIC");
    expect(examLevelOfPath("/exam/toeic-reading-01/practice")).toBe("TOEIC");
    expect(examLevelOfPath("/exam/khac")).toBeUndefined();
    expect(examLevelOfPath("/my")).toBeUndefined();
  });
});

describe("luyện tập theo khối", () => {
  it("Part 5 (không có văn bản) tách mỗi câu một khối; Part 6 giữ cả khối cùng văn bản", () => {
    const blocks = practiceBlocks(readingOnly.sections[0]);
    expect(blocks.map((b) => [b.from, b.to])).toEqual([
      [101, 101],
      [102, 102],
      [131, 132],
    ]);
    expect(blocks[0].instruction).toBe(PART5);
    expect(blocks[2].passage).toContain("(131)");
  });

  it("phần nghe giữ khối như đề — audio của khối phát liền", () => {
    expect(practiceBlocks(full.sections[0])).toBe(full.sections[0].groups);
  });

  it("tiến độ tính số câu đã kiểm tra", () => {
    expect(practiceProgress(full, { checked: ["listening:1", "reading:101"] })).toEqual({ done: 2, total: 8 });
    expect(practiceProgress(full, undefined)).toEqual({ done: 0, total: 8 });
  });

  it("luyện tiếp từ câu đầu tiên chưa làm, theo thứ tự các phần; làm hết thì về câu đầu", () => {
    expect(nextPracticeTarget(full, undefined)).toEqual({ section: "listening", no: 1 });
    const listening = full.sections[0].questions.map((x) => qKey("listening", x.no));
    expect(nextPracticeTarget(full, { checked: [...listening, "reading:101"] })).toEqual({
      section: "reading",
      no: 102,
    });
    const all = Object.keys(allRight(full));
    expect(nextPracticeTarget(full, { checked: all })).toEqual({ section: "listening", no: 1 });
  });
});

describe("nghe lại một câu", () => {
  const listening = full.sections[0];

  it("câu dùng chung hội thoại thì phát hội thoại trước", () => {
    const q4 = listening.questions.find((x) => x.no === 4)!;
    expect(questionAudio(listening, q4)).toEqual([listening.groups[1].dialogue, q4.replay]);
  });

  it("nghe lại chỉ phát phần lời đọc — không kèm khoảng dừng trả lời", () => {
    const q1 = listening.questions[0];
    const [start, end] = questionAudio(listening, q1)[0];
    expect(start).toBe(q1.replay![0]);
    expect(q1.audio![1] - end).toBeGreaterThan(15);
  });

  it("đề chưa đo mốc thời gian thì không có đoạn riêng (giao diện cho nghe cả bài)", () => {
    const bare = {
      ...listening,
      groups: listening.groups.map((g) => ({ ...g, audio: undefined, dialogue: undefined })),
      questions: listening.questions.map((x) => ({ ...x, audio: undefined, replay: undefined })),
    };
    expect(questionAudio(bare, bare.questions[0])).toEqual([]);
    expect(groupAudio(bare, bare.groups[0])).toEqual([]);
  });

  it("audio của khối như đề thật: một đoạn liền từ lời chỉ dẫn tới hết khoảng dừng câu cuối", () => {
    const g = listening.groups[0];
    const [[start, end]] = groupAudio(listening, g);
    expect(start).toBe(g.audio![0]);
    // Dừng trong khoảng dừng của câu 2, trước khi khối sau bắt đầu.
    expect(end).toBeLessThan(listening.questions[1].audio![1] - 0.5);
    expect(end).toBeGreaterThan(listening.questions[1].replay![1]);
  });

  it("phần đọc không có audio", () => {
    const reading = full.sections[1];
    expect(questionAudio(reading, reading.questions[0])).toEqual([]);
    expect(groupAudio(reading, reading.groups[0])).toEqual([]);
  });
});

/**
 * Soát dữ liệu MỌI đề — đề thêm sau này cũng phải qua được (đề mẫu chạy kèm
 * để phần soát luôn có việc). Chỉ soát HÌNH DẠNG dữ liệu, không bám nội dung.
 */
const ALLOWED_TAGS = new Set(["br", "b", "u", "div", "img"]);
const tagsIn = (html: string) => [...html.matchAll(/<\/?([a-zA-Z][a-zA-Z0-9]*)\b/g)].map((m) => m[1].toLowerCase());

describe.each([full, readingOnly, ...listExams()].map((e) => [e.id, e] as [string, Exam]))("dữ liệu đề %s", (_id, e) => {
  it("kỳ thi TOEIC, mỗi phần có thời gian và câu hỏi", () => {
    expect(e.level).toBe("TOEIC");
    expect(e.source.trim()).toBeTruthy();
    for (const s of e.sections) {
      expect(["listening", "reading"]).toContain(s.id);
      expect(s.minutes).toBeGreaterThan(0);
      expect(s.questions.length).toBeGreaterThan(0);
    }
  });

  it("số câu tăng dần không trùng, mỗi câu 1 điểm, đáp án 1–4", () => {
    for (const s of e.sections) {
      const nos = s.questions.map((x) => x.no);
      nos.slice(1).forEach((no, i) => expect(no).toBeGreaterThan(nos[i]));
      for (const x of s.questions) {
        expect(x.points).toBe(1);
        expect([1, 2, 3, 4]).toContain(x.answer);
      }
    }
  });

  it("câu nào cũng thuộc đúng một khối chỉ dẫn", () => {
    for (const s of e.sections) {
      for (const x of s.questions) {
        expect(s.groups.filter((g) => x.no >= g.from && x.no <= g.to), `câu ${x.no}`).toHaveLength(1);
      }
    }
  });

  it("đủ bốn lựa chọn có nội dung, không tự ghi nhãn; HTML chỉ chứa thẻ được phép", () => {
    for (const s of e.sections) {
      const htmls = [
        ...s.groups.map((g) => g.passage ?? ""),
        ...s.questions.flatMap((x) => [x.prompt, ...x.options].map((c) => (isImage(c) ? "" : c.html))),
      ];
      for (const h of htmls) expect(tagsIn(h).filter((t) => !ALLOWED_TAGS.has(t)), h.slice(0, 80)).toEqual([]);
      for (const x of s.questions) {
        expect(x.options).toHaveLength(4);
        for (const o of x.options) {
          expect(isImage(o) ? o.image : o.html.trim()).toBeTruthy();
          // Giao diện tự thêm (A)–(D); dữ liệu ghi nhãn nữa là hiện hai lần.
          if (!isImage(o)) expect(o.html).not.toMatch(/^\s*\([A-D]\)/);
        }
        if (x.explanation !== undefined) expect(x.explanation.trim()).toBeTruthy();
      }
    }
  });

  it("ảnh nào đề trỏ tới cũng có file (khi đã đặt tài nguyên về máy)", () => {
    const dir = path.resolve("public/img/exams", e.assetDir);
    if (!existsSync(dir)) return; // máy chưa có tài nguyên — không có gì để soát
    const refs = JSON.stringify(e).match(/images\/[^"\\]+?\.webp/g) ?? [];
    for (const r of new Set(refs)) expect(existsSync(path.join(dir, r)), r).toBe(true);
  });

  it("phần nghe đã đo mốc thời gian thì đo ĐỦ mọi câu, theo thứ tự, không chồng nhau", () => {
    // Đề chưa đo thì không có đoạn nào — giao diện cho nghe cả bài. Đo dở
    // chừng (có câu có, có câu không) mới là lỗi.
    for (const s of e.sections.filter((x) => x.audio && x.questions.some((y) => y.audio))) {
      let last = 0;
      for (const x of s.questions) {
        expect(x.audio, `câu ${x.no}`).toBeDefined();
        const [start, end] = x.audio!;
        expect(end).toBeGreaterThan(start);
        expect(start).toBeGreaterThanOrEqual(last);
        last = end;
      }
    }
  });

  it("đoạn nghe lại từng câu nằm gọn trong đoạn của câu, theo thứ tự", () => {
    for (const s of e.sections.filter((x) => x.questions.some((y) => y.audio))) {
      let last = 0;
      for (const x of s.questions) {
        expect(x.replay, `câu ${x.no}`).toBeDefined();
        const [start, end] = x.replay!;
        expect(end).toBeGreaterThan(start);
        expect(start).toBeGreaterThanOrEqual(last);
        expect(end).toBeLessThanOrEqual(x.audio![1]);
        last = end;
      }
    }
  });

  it("mọi khối nghe là MỘT đoạn liền, không lấn sang khối sau", () => {
    for (const s of e.sections.filter((x) => x.questions.some((y) => y.audio))) {
      for (const g of s.groups) {
        const segs = groupAudio(s, g);
        expect(segs, `khối ${g.from}`).toHaveLength(1);
        const next = s.groups.find((x) => x.from > g.to);
        const nextStart = next && (next.audio ?? s.questions.find((y) => y.no === next.from)!.audio!)[0];
        if (nextStart !== undefined) expect(segs[0][1]).toBeLessThan(nextStart - 0.4);
      }
    }
  });
});
