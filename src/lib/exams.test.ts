import { describe, expect, it } from "vitest";
import { existsSync } from "node:fs";
import path from "node:path";
import {
  countChars,
  getExam,
  isImage,
  isPointsOnly,
  levelFor,
  listExams,
  maxScore,
  qKey,
  questionAudio,
  questionPrompt,
  splitInstruction,
  scoreExam,
  sectionMax,
  type Answers,
  type Exam,
} from "./exams";
import { tagsIn } from "./exam-html";

const exam = getExam("102-topik1")!;
const exam2 = getExam("102-topik2")!;

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
    // Câu 1 (4 điểm) đúng, câu 3 (3 điểm) sai, còn lại bỏ trống.
    const r = scoreExam(exam, { "listening:1": 1, "listening:3": 1 });
    expect(r.score).toBe(4);
    expect(r.sections[0]).toMatchObject({ score: 4, correct: 1, total: 30 });
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
    const q26 = listening.questions.find((q) => q.no === 26)!;
    const segs = questionAudio(listening, q26);
    expect(segs).toHaveLength(2);
    expect(segs[1]).toEqual(q26.audio);
  });

  it("đề chưa đo mốc thời gian thì không có đoạn riêng (giao diện cho nghe cả bài)", () => {
    const old = getExam("102-topik2")!.sections[0];
    expect(questionAudio(old, old.questions[0])).toEqual([]);
  });

  it("câu đơn thì chỉ một đoạn", () => {
    const listening = exam.sections[0];
    expect(questionAudio(listening, listening.questions[0])).toHaveLength(1);
  });
});

/** Soát dữ liệu MỌI đề — đề nhập sau này cũng phải qua được. */
it("đủ 12 kỳ, mỗi kỳ cả TOPIK I và TOPIK II", () => {
  const rounds = [102, 96, 91, 83, 64, 60, 52, 47, 41, 37, 36, 35];
  expect(listExams().map((e) => e.id)).toEqual(rounds.flatMap((r) => [`${r}-topik1`, `${r}-topik2`]));
});

describe.each(listExams().map((e) => [e.id, e] as [string, Exam]))("dữ liệu đề %s", (_id, e) => {
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
});
