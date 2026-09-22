/**
 * Đề thi TOPIK để luyện trong tab "Luyện thi" (`/exam`).
 *
 * Nguồn: dịch vụ "토픽 기출문제 풀어보기" của topik.go.kr — câu hỏi, lựa chọn,
 * đoạn văn là CHỮ (đã làm sạch HTML, xem `lib/exam-html`), tranh/biểu đồ là
 * ảnh riêng, kèm đáp án và điểm từng câu. `scripts/import-topik.ts` chuyển bộ
 * dữ liệu gốc thành `content/exams/<id>.json` theo đúng kiểu dưới đây.
 *
 * Chữ thật thay vì ảnh cắt từ PDF (bản đầu): nét ở mọi cỡ chữ, hợp chế độ
 * tối, bấm thẳng vào lựa chọn được, và sau này tra cứu/dịch được.
 *
 * Import TĨNH từng đề (xem `EXAMS`), cùng lý do với `lib/page-answers.ts`:
 * Output File Tracing của Next không lần được file đọc qua path dựng động.
 */
import e35 from "../../content/exams/35-topik1.json";
import e36 from "../../content/exams/36-topik1.json";
import e37 from "../../content/exams/37-topik1.json";
import e41 from "../../content/exams/41-topik1.json";
import e47 from "../../content/exams/47-topik1.json";
import e52 from "../../content/exams/52-topik1.json";
import e60 from "../../content/exams/60-topik1.json";
import e64 from "../../content/exams/64-topik1.json";
import e83 from "../../content/exams/83-topik1.json";
import e91 from "../../content/exams/91-topik1.json";
import e96 from "../../content/exams/96-topik1.json";
import e102 from "../../content/exams/102-topik1.json";
import { mediaOriginBase } from "@/lib/books";
import {
  examAssetBase,
  isImage,
  type Content,
  type Exam,
  type ExamGroup,
  type ExamLevel,
  type ExamQuestion,
  type ExamSection,
  type SectionId,
} from "@/lib/exam-types";

export * from "@/lib/exam-types";

const EXAMS = [e102, e96, e91, e83, e64, e60, e52, e47, e41, e37, e36, e35] as unknown as Exam[];

export function listExams(): Exam[] {
  return [...EXAMS].sort((a, b) => b.round - a.round || a.level.localeCompare(b.level));
}

export function getExam(id: string): Exam | undefined {
  return EXAMS.find((e) => e.id === id);
}

export function examTitle(exam: Exam): string {
  return `${exam.level} · Kỳ ${exam.round}`;
}

export function sectionVi(id: SectionId): string {
  return id === "listening" ? "Nghe" : id === "reading" ? "Đọc" : "Viết";
}

export function totalMinutes(exam: Exam): number {
  return exam.sections.reduce((n, s) => n + s.minutes, 0);
}

export function maxScore(exam: Exam): number {
  return exam.sections.reduce((n, s) => n + s.questions.reduce((m, q) => m + q.points, 0), 0);
}

/** Khối "※ [a~b]" chứa câu `no`. */
export function groupOf(section: ExamSection, no: number): ExamGroup | undefined {
  return section.groups.find((g) => no >= g.from && no <= g.to);
}

/**
 * Các đoạn cần phát để NGHE LẠI RIÊNG một câu: hội thoại chung của khối (nếu
 * có và đoạn của câu không chứa sẵn nó) rồi đến đoạn của câu. Rỗng khi đề
 * chưa có mốc thời gian từng câu — lúc đó giao diện cho nghe cả bài.
 */
export function questionAudio(section: ExamSection, q: ExamQuestion): [number, number][] {
  if (!q.audio) return [];
  const d = groupOf(section, q.no)?.dialogue;
  if (d && q.audio[0] >= d[1]) return [d, q.audio];
  return [q.audio];
}

/**
 * Đề câu hỏi chỉ là "(4점)" thì không cần hiện: số điểm đã có ở tiêu đề câu,
 * còn câu nghe thì nội dung nằm trong file nghe.
 */
export function isPointsOnly(c: Content): boolean {
  return !isImage(c) && /^\s*(\(\d+점\))?\s*$/.test(c.html);
}

// ---------------------------------------------------------------------------
// Chấm điểm

export type Answers = Record<number, number>;

export interface SectionScore {
  id: SectionId;
  score: number;
  max: number;
  correct: number;
  total: number;
}

export interface ExamScore {
  sections: SectionScore[];
  score: number;
  max: number;
  level: string | null;
}

/**
 * Ngưỡng cấp theo quy định TOPIK (tổng điểm các phần):
 * TOPIK I (200 điểm): 1급 ≥ 80, 2급 ≥ 140.
 * TOPIK II (300 điểm): 3급 ≥ 120, 4급 ≥ 150, 5급 ≥ 190, 6급 ≥ 230.
 */
const LEVEL_CUTS: Record<ExamLevel, [number, string][]> = {
  "TOPIK I": [
    [140, "Cấp 2"],
    [80, "Cấp 1"],
  ],
  "TOPIK II": [
    [230, "Cấp 6"],
    [190, "Cấp 5"],
    [150, "Cấp 4"],
    [120, "Cấp 3"],
  ],
};

export function levelFor(examLevel: ExamLevel, score: number): string | null {
  return LEVEL_CUTS[examLevel].find(([cut]) => score >= cut)?.[1] ?? null;
}

export function scoreExam(exam: Exam, answers: Answers): ExamScore {
  const sections = exam.sections.map((s) => {
    let score = 0;
    let correct = 0;
    for (const q of s.questions) {
      if (answers[q.no] === q.answer) {
        score += q.points;
        correct++;
      }
    }
    return {
      id: s.id,
      score,
      max: s.questions.reduce((n, q) => n + q.points, 0),
      correct,
      total: s.questions.length,
    };
  });
  const score = sections.reduce((n, s) => n + s.score, 0);
  return {
    sections,
    score,
    max: sections.reduce((n, s) => n + s.max, 0),
    level: levelFor(exam.level, score),
  };
}

// ---------------------------------------------------------------------------
// Tài nguyên (ảnh, file nghe)

export function examAssetUrl(exam: Exam, path: string): string {
  return `${examAssetBase(exam.assetDir)}/${path}`;
}

/**
 * File nghe trỏ THẲNG R2 ở production — cùng lý do với bài nghe của sách
 * (xem chú thích dài ở `lib/audio.ts`): cache biên của Vercel trả sai khoảng
 * byte (`Range`) qua rewrite, audio quay mãi không phát. Lúc dev thì lấy file
 * trong `public/`, dev server trả `Range` đúng.
 */
export function examAudioUrl(exam: Exam, file: string): string {
  if (process.env.NODE_ENV !== "production") return examAssetUrl(exam, file);
  return `${mediaOriginBase()}/exams/${exam.assetDir}/${file}`;
}
