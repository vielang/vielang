import { describe, expect, it } from "vitest";
import { existsSync } from "node:fs";
import path from "node:path";
import { getExam, isImage, isPointsOnly, levelFor, listExams, questionAudio, scoreExam, type Exam } from "./exams";
import { tagsIn } from "./exam-html";

const exam = getExam("102-topik1")!;

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
    const all = Object.fromEntries(exam.sections.flatMap((s) => s.questions.map((q) => [q.no, q.answer])));
    const r = scoreExam(exam, all);
    expect(r.score).toBe(200);
    expect(r.level).toBe("Cấp 2");
  });

  it("câu bỏ trống hay sai thì không có điểm, điểm tính theo đúng hệ số từng câu", () => {
    // Câu 1 (4 điểm) đúng, câu 3 (3 điểm) sai, còn lại bỏ trống.
    const r = scoreExam(exam, { 1: 1, 3: 1 });
    expect(r.score).toBe(4);
    expect(r.sections[0]).toMatchObject({ score: 4, correct: 1, total: 30 });
  });
});

describe("đề câu hỏi chỉ ghi điểm", () => {
  it("không hiện lại — số điểm đã có ở tiêu đề câu", () => {
    expect(isPointsOnly({ html: "(4점)" })).toBe(true);
    expect(isPointsOnly({ html: "" })).toBe(true);
    expect(isPointsOnly({ html: "여자가 왜 이 이야기를 하고 있는지 고르십시오. (3점)" })).toBe(false);
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
    const old = getExam("35-topik1")!.sections[0];
    expect(questionAudio(old, old.questions[0])).toEqual([]);
  });

  it("câu đơn thì chỉ một đoạn", () => {
    const listening = exam.sections[0];
    expect(questionAudio(listening, listening.questions[0])).toHaveLength(1);
  });
});

/** Soát dữ liệu MỌI đề — đề nhập sau này cũng phải qua được. */
it("đủ 12 kỳ TOPIK I", () => {
  expect(listExams().map((e) => e.round)).toEqual([102, 96, 91, 83, 64, 60, 52, 47, 41, 37, 36, 35]);
});

describe.each(listExams().map((e) => [e.id, e] as [string, Exam]))("dữ liệu đề %s", (_id, e) => {
  it("số câu liền mạch, đáp án 1–4, mỗi phần đủ 100 điểm", () => {
    const nos = e.sections.flatMap((s) => s.questions.map((q) => q.no));
    expect(nos).toEqual(nos.map((_, i) => nos[0] + i));
    for (const s of e.sections) {
      expect(s.questions.every((q) => [1, 2, 3, 4].includes(q.answer))).toBe(true);
      expect(s.questions.reduce((n, q) => n + q.points, 0)).toBe(100);
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
    const refs = JSON.stringify(e).match(/images\/[^"\\]+?\.webp/g) ?? [];
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
