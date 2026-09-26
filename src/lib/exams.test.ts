import { describe, expect, it } from "vitest";
import { existsSync } from "node:fs";
import path from "node:path";
import {
  countChars,
  groupAudio,
  isImage,
  isPointsOnly,
  levelFor,
  listExams,
  maxScore,
  nextLevel,
  nextPracticeTarget,
  practiceProgress,
  qKey,
  questionAudio,
  questionPrompt,
  splitInstruction,
  writingBlocks,
  scoreExam,
  sectionMax,
  type Answers,
  type Exam,
  type ExamQuestion,
} from "./exams";
import { tagsIn } from "./exam-html";

/**
 * Đề mẫu tự soạn, rút gọn — bộ chấm điểm, chia phần, nghe lại theo khối
 * không phụ thuộc đề thật nào (sẽ dùng lại cho TOEIC), nên không bám vào dữ
 * liệu đề đã nhập. Mỗi phần vẫn đủ 100 điểm để quy ra cấp như đề thật.
 */
const OPTIONS: ExamQuestion["options"] = [{ html: "①" }, { html: "②" }, { html: "③" }, { html: "④" }];

const q = (
  no: number,
  points: number,
  answer: ExamQuestion["answer"],
  timing: Pick<ExamQuestion, "audio" | "replay"> = {}
): ExamQuestion => ({ no, points, answer, layout: 4, prompt: { html: "" }, options: OPTIONS, ...timing });

/**
 * TOPIK I rút gọn. Phần nghe:
 * - khối [1~2]: lời chỉ dẫn 0–5 giây, mỗi câu một đoạn riêng;
 * - khối [3~4]: nghe một hội thoại (60–70) rồi trả lời hai câu — đoạn của
 *   câu 4 chỉ còn tiếng đọc số câu và khoảng dừng.
 */
const exam: Exam = {
  id: "mau-1",
  round: 1,
  year: 2025,
  level: "TOPIK I",
  assetDir: "mau",
  source: "Đề mẫu",
  sections: [
    {
      id: "listening",
      title: "Nghe",
      minutes: 40,
      audio: "listening.mp3",
      groups: [
        { from: 1, to: 2, instruction: "※ [1~2] 다음을 듣고", audio: [0, 5] },
        { from: 3, to: 4, instruction: "※ [3~4] 다음을 듣고", audio: [55, 60], dialogue: [60, 70] },
      ],
      questions: [
        q(1, 30, 2, { audio: [5, 30], replay: [5, 10] }),
        q(2, 25, 1, { audio: [30, 55], replay: [30, 35] }),
        q(3, 25, 4, { audio: [60, 90], replay: [60, 75] }),
        q(4, 20, 3, { audio: [90, 115], replay: [90, 95] }),
      ],
    },
    {
      id: "reading",
      title: "Đọc",
      minutes: 60,
      groups: [{ from: 5, to: 8, instruction: "※ [5~8] 다음을 읽고" }],
      questions: [q(5, 25, 1), q(6, 25, 2), q(7, 25, 3), q(8, 25, 4)],
    },
  ],
};

/**
 * TOPIK II rút gọn: nghe → viết → đọc, phần đọc đánh số LẠI từ 1. Khối nghe
 * [1~2] không có lời chỉ dẫn riêng, hội thoại chung nằm trong đoạn câu 1.
 */
const exam2: Exam = {
  id: "mau-2",
  round: 1,
  year: 2025,
  level: "TOPIK II",
  assetDir: "mau",
  source: "Đề mẫu",
  sections: [
    {
      id: "listening",
      title: "Nghe",
      minutes: 60,
      audio: "listening.mp3",
      groups: [{ from: 1, to: 2, instruction: "※ [1~2] 다음을 듣고", dialogue: [5, 20] }],
      questions: [
        q(1, 50, 1, { audio: [5, 40], replay: [5, 25] }),
        q(2, 50, 2, { audio: [40, 60], replay: [40, 45] }),
      ],
    },
    {
      id: "writing",
      title: "Viết",
      minutes: 50,
      groups: [],
      questions: [],
      writing: {
        tasks: [
          { no: 51, points: 10, kind: "blanks", image: "writing/51.webp", answer: "answer-key/51.webp" },
          { no: 52, points: 10, kind: "blanks", image: "writing/52.webp", answer: "answer-key/52.webp" },
          { no: 53, points: 30, kind: "essay", chars: [200, 300], image: "writing/53.webp", answer: "answer-key/53.webp" },
          { no: 54, points: 50, kind: "essay", chars: [600, 700], image: "writing/54.webp", answer: "answer-key/54.webp" },
        ],
      },
    },
    {
      id: "reading",
      title: "Đọc",
      minutes: 70,
      groups: [{ from: 1, to: 2, instruction: "※ [1~2] 다음을 읽고" }],
      questions: [q(1, 50, 3), q(2, 50, 4)],
    },
  ],
};

/** Chọn đúng hết mọi câu trắc nghiệm. */
const allRight = (e: Exam): Answers =>
  Object.fromEntries(e.sections.flatMap((s) => s.questions.map((q) => [qKey(s.id, q.no), q.answer])));

describe("chấm điểm và quy ra cấp", () => {
  it("ngưỡng cấp theo quy định TOPIK", () => {
    expect(levelFor("TOPIK I", 79)).toBeNull();
    expect(levelFor("TOPIK I", 80)).toBe("Cấp 1");
    expect(levelFor("TOPIK I", 140)).toBe("Cấp 2");
    expect(levelFor("TOPIK II", 119)).toBeNull();
    expect(levelFor("TOPIK II", 150)).toBe("Cấp 4");
    expect(levelFor("TOPIK II", 230)).toBe("Cấp 6");
  });

  it("làm đúng hết thì được điểm tối đa", () => {
    const r = scoreExam(exam, allRight(exam));
    expect(r.score).toBe(200);
    expect(r.level).toBe("Cấp 2");
  });

  it("câu bỏ trống hay sai thì không có điểm, điểm tính theo đúng hệ số từng câu", () => {
    // Câu 1 (30 điểm) đúng, câu 3 (25 điểm) sai, còn lại bỏ trống.
    const r = scoreExam(exam, { "listening:1": 2, "listening:3": 1 });
    expect(r.score).toBe(30);
    expect(r.sections[0]).toMatchObject({ score: 30, correct: 1, total: 4 });
  });
});

describe("thang cấp và tiến độ luyện", () => {
  it("cấp kế tiếp và số điểm còn thiếu", () => {
    expect(nextLevel("TOPIK I", 50)).toEqual({ level: "Cấp 1", need: 30 });
    expect(nextLevel("TOPIK I", 100)).toEqual({ level: "Cấp 2", need: 40 });
    expect(nextLevel("TOPIK I", 150)).toBeNull();
    expect(nextLevel("TOPIK II", 200)).toEqual({ level: "Cấp 6", need: 30 });
  });

  it("tiến độ tính cả câu trắc nghiệm đã kiểm tra và câu viết đã tự chấm", () => {
    expect(practiceProgress(exam2, { checked: ["listening:1", "reading:1"], grades: { 53: 20 } })).toEqual({
      done: 3,
      total: 8,
    });
  });

  it("luyện tiếp từ câu đầu tiên chưa làm, theo thứ tự các phần", () => {
    expect(nextPracticeTarget(exam, undefined)).toEqual({ section: "listening", no: 1 });
    const listening = exam.sections[0].questions.map((q) => qKey("listening", q.no));
    expect(nextPracticeTarget(exam, { checked: [...listening, "reading:5"] })).toEqual({ section: "reading", no: 6 });
    const l2 = exam2.sections[0].questions.map((q) => qKey("listening", q.no));
    expect(nextPracticeTarget(exam2, { checked: l2, grades: { 51: 10 } })).toEqual({ section: "writing", no: 52 });
  });
});

describe("TOPIK II", () => {
  it("ba phần theo thứ tự buổi thi: nghe → viết → đọc, tổng 300 điểm", () => {
    expect(exam2.sections.map((s) => s.id)).toEqual(["listening", "writing", "reading"]);
    expect(maxScore(exam2)).toBe(300);
  });

  it("câu 1 nghe và câu 1 đọc trùng số nhưng chấm riêng", () => {
    const r1 = exam2.sections[2].questions[0];
    const r = scoreExam(exam2, { [qKey("reading", 1)]: r1.answer });
    expect(r.sections.map((s) => s.score)).toEqual([0, 0, r1.points]);
  });

  it("phần viết tính theo điểm tự chấm; chưa chấm câu nào thì báo còn chưa chấm", () => {
    const mc = scoreExam(exam2, allRight(exam2));
    expect(mc).toMatchObject({ score: 200, ungraded: 4, level: "Cấp 5" });

    const full = scoreExam(exam2, allRight(exam2), { 51: 10, 52: 10, 53: 30, 54: 50 });
    expect(full).toMatchObject({ score: 300, ungraded: 0, level: "Cấp 6" });
    expect(full.sections[1]).toMatchObject({ id: "writing", score: 100, correct: 4, total: 4 });
  });

  it("phần viết chia 3 khối [51–52] [53] [54], mỗi câu có ảnh đề và đáp án mẫu riêng", () => {
    const tasks = exam2.sections[1].writing!.tasks;
    expect(writingBlocks(tasks).map((b) => b.map((t) => t.no))).toEqual([[51, 52], [53], [54]]);
    for (const e of [exam2, ...listExams().filter((x) => x.level === "TOPIK II")]) {
      const ts = e.sections.find((s) => s.writing)!.writing!.tasks;
      expect(new Set(ts.map((t) => t.image)).size).toBe(4);
      expect(new Set(ts.map((t) => t.answer)).size).toBe(4);
    }
  });

  it("điểm tự chấm vượt thang của câu thì bị kẹp lại", () => {
    expect(scoreExam(exam2, {}, { 51: 99, 53: -5 }).score).toBe(10);
  });

  it("đếm chữ như ô 원고지: tính dấu cách, bỏ xuống dòng", () => {
    expect(countChars("가 나\n다")).toBe(4);
  });
});

describe("đề câu hỏi chỉ ghi điểm", () => {
  it("không hiện lại — số điểm đã có ở tiêu đề câu", () => {
    expect(isPointsOnly({ html: "(4점)" })).toBe(true);
    expect(isPointsOnly({ html: "" })).toBe(true);
    expect(isPointsOnly({ html: "여자가 왜 이 이야기를 하고 있는지 고르십시오. (3점)" })).toBe(false);
  });

  it('bỏ "(N점)" ở đầu đề hoặc đứng riêng dòng cuối, giữ khi nằm trong câu hỏi', () => {
    const box = '<div class="exam-box">저는 일이 많습니다.</div>';
    expect(questionPrompt({ html: `(2점)<br><br>${box}` })).toEqual({ html: box });
    expect(questionPrompt({ html: `${box}<br>(3점)` })).toEqual({ html: box });
    const inline = { html: "알맞은 것을 고르십시오. (3점)" };
    expect(questionPrompt(inline)).toBe(inline);
  });

  it("không đề nào còn hiện (N점) ở đầu", () => {
    for (const e of listExams())
      for (const s of e.sections)
        for (const q of s.questions) {
          const p = questionPrompt(q.prompt);
          if (p && !isImage(p)) expect(p.html).not.toMatch(/^\s*\(\d+점\)/);
        }
  });
});

describe("lời chỉ dẫn", () => {
  it("tách khoảng câu thành nhãn", () => {
    expect(splitInstruction("※ [1～4] 다음을 듣고 고르십시오.")).toEqual({ range: "1–4", text: "다음을 듣고 고르십시오." });
    expect(splitInstruction("※ [44~45] 다음을 읽고")).toEqual({ range: "44–45", text: "다음을 읽고" });
    expect(splitInstruction("※ 다음을 읽고")).toEqual({ range: null, text: "다음을 읽고" });
  });
});

describe("nghe lại một câu", () => {
  it("câu dùng chung hội thoại thì phát hội thoại trước", () => {
    const listening = exam.sections[0];
    const q4 = listening.questions.find((q) => q.no === 4)!;
    const segs = questionAudio(listening, q4);
    expect(segs).toHaveLength(2);
    expect(segs[1]).toEqual(q4.replay);
  });

  it("nghe lại chỉ phát phần lời đọc — không kèm khoảng dừng trả lời ~20 giây", () => {
    const q1 = exam.sections[0].questions[0];
    const [start, end] = questionAudio(exam.sections[0], q1)[0];
    expect(start).toBe(q1.replay![0]);
    expect(q1.audio![1] - end).toBeGreaterThan(15);
  });

  it("đề chưa đo mốc thời gian thì không có đoạn riêng (giao diện cho nghe cả bài)", () => {
    const listening = exam.sections[0];
    const bare = {
      ...listening,
      groups: listening.groups.map((g) => ({ ...g, audio: undefined, dialogue: undefined })),
      questions: listening.questions.map((q) => ({ ...q, audio: undefined, replay: undefined })),
    };
    expect(questionAudio(bare, bare.questions[0])).toEqual([]);
    expect(groupAudio(bare, bare.groups[0])).toEqual([]);
  });

  it("audio của khối như đề thật: một đoạn liền từ lời chỉ dẫn tới hết khoảng dừng câu cuối", () => {
    const listening = exam.sections[0];
    const g = listening.groups[0]; // [1~2]
    const items = listening.questions.filter((q) => q.no >= g.from && q.no <= g.to);
    const [[start, end]] = groupAudio(listening, g);
    expect(start).toBe(g.audio![0]);
    // Dừng trong khoảng dừng của câu 2, trước khi tiếng chuông của khối sau vang lên.
    expect(end).toBeLessThan(items.at(-1)!.audio![1] - 0.5);
    expect(end).toBeGreaterThan(items.at(-1)!.replay![1]);
  });

  it("mọi khối của mọi đề là MỘT đoạn liền (không lấn sang khối sau)", () => {
    for (const e of [exam, exam2, ...listExams()])
      for (const s of e.sections.filter((x) => x.questions.some((q) => q.audio)))
        for (const g of s.groups) {
          const segs = groupAudio(s, g);
          expect(segs, `${e.id} khối ${g.from}`).toHaveLength(1);
          const next = s.groups.find((x) => x.from > g.to);
          const nextStart = next && (next.audio ?? s.questions.find((q) => q.no === next.from)!.audio!)[0];
          // Kết thúc TRƯỚC khi khối sau bắt đầu (không lọt tiếng của khối sau).
          if (nextStart !== undefined) expect(segs[0][1]).toBeLessThan(nextStart - 0.4);
        }
  });

  it("TOPIK II khối hai câu: hội thoại phát một lần, rồi tới câu sau", () => {
    const listening = exam2.sections[0];
    const g = listening.groups.find((x) => x.from === 1)!;
    const [q21, q22] = listening.questions.filter((q) => q.no === 1 || q.no === 2);
    const segs = groupAudio(listening, g);
    expect(segs).toHaveLength(1);
    expect(segs[0][0]).toBe(q21.audio![0]);
    expect(segs[0][1]).toBeGreaterThan(q22.replay![1]);
    expect(segs[0][1]).toBeLessThan(q22.audio![1]);
    expect(questionAudio(listening, q22)).toEqual([g.dialogue, q22.replay]);
  });

  it("câu đơn thì chỉ một đoạn", () => {
    const listening = exam.sections[0];
    expect(questionAudio(listening, listening.questions[0])).toHaveLength(1);
  });
});

/** Soát dữ liệu MỌI đề — đề nhập sau này cũng phải qua được (đề mẫu chạy kèm để phần soát luôn có việc). */
describe.each([exam, exam2, ...listExams()].map((e) => [e.id, e] as [string, Exam]))("dữ liệu đề %s", (_id, e) => {
  it("số câu liền mạch trong từng phần, đáp án 1–4, mỗi phần đủ 100 điểm", () => {
    for (const s of e.sections) {
      const nos = s.writing ? s.writing.tasks.map((t) => t.no) : s.questions.map((q) => q.no);
      expect(nos).toEqual(nos.map((_, i) => nos[0] + i));
      expect(s.questions.every((q) => [1, 2, 3, 4].includes(q.answer))).toBe(true);
      expect(sectionMax(s)).toBe(100);
    }
  });

  it("câu nào cũng thuộc đúng một khối chỉ dẫn", () => {
    for (const s of e.sections) {
      for (const q of s.questions) {
        expect(s.groups.filter((g) => q.no >= g.from && q.no <= g.to)).toHaveLength(1);
      }
    }
  });

  it("đủ bốn lựa chọn có nội dung, HTML chỉ chứa thẻ được phép", () => {
    const ALLOWED = new Set(["br", "b", "u", "div", "img"]);
    for (const s of e.sections) {
      const htmls = [
        ...s.groups.flatMap((g) => [g.passage ?? "", g.example?.html ?? ""]),
        ...s.questions.flatMap((q) => [q.prompt, ...q.options].map((c) => (isImage(c) ? "" : c.html))),
      ];
      for (const h of htmls) expect(tagsIn(h).every((t) => ALLOWED.has(t))).toBe(true);
      for (const q of s.questions) {
        expect(q.options).toHaveLength(4);
        for (const o of q.options) expect(isImage(o) ? o.image : o.html.trim()).toBeTruthy();
      }
    }
  });

  it("ảnh nào đề trỏ tới cũng có file (khi đã nhập tài nguyên về máy)", () => {
    const dir = path.resolve("public/img/exams", e.assetDir);
    if (!existsSync(dir)) return; // máy chưa chạy import-topik — không có gì để soát
    const refs = JSON.stringify(e).match(/(?:images|writing|answer-key)\/[^"\\]+?\.webp/g) ?? [];
    for (const r of new Set(refs)) expect(existsSync(path.join(dir, r)), r).toBe(true);
  });

  it("phần nghe đã đo mốc thời gian thì đo ĐỦ mọi câu, theo thứ tự, không chồng nhau", () => {
    // Đề chưa đo thì không có đoạn nào — giao diện cho nghe cả bài. Đo dở
    // chừng (có câu có, có câu không) mới là lỗi.
    for (const s of e.sections.filter((s) => s.audio && s.questions.some((q) => q.audio))) {
      let last = 0;
      for (const q of s.questions) {
        expect(q.audio).toBeDefined();
        const [start, end] = q.audio!;
        expect(end).toBeGreaterThan(start);
        expect(start).toBeGreaterThanOrEqual(last);
        last = end;
      }
    }
  });

  it("đoạn nghe lại từng câu nằm gọn trong đoạn của câu, theo thứ tự, không lấn sang câu sau", () => {
    for (const s of e.sections.filter((s) => s.questions.some((q) => q.audio))) {
      let last = 0;
      for (const q of s.questions) {
        expect(q.replay, `câu ${q.no}`).toBeDefined();
        const [start, end] = q.replay!;
        expect(end).toBeGreaterThan(start);
        expect(start).toBeGreaterThanOrEqual(last);
        // Kết thúc trước hết đoạn của câu (tức trước tiếng đọc số câu sau).
        expect(end).toBeLessThanOrEqual(q.audio![1]);
        last = end;
      }
    }
  });
});
