/**
 * Năng lực người học, tính từ hai nguồn trên trình duyệt:
 *
 * 1. TỰ CHẤM sau khi xem đáp án sách (đúng hết / sai vài câu / sai nhiều) —
 *    phủ cả tám cuốn.
 * 2. Quiz có chấm tự động của Sơ cấp 1.
 *
 * Đây là số người học TỰ BÁO, không phải điểm thi — My page nói rõ điều đó.
 * Vì vậy chỉ đưa ra nhận xét tương đối (kỹ năng nào yếu hơn, điểm ngữ pháp
 * nào hay sai), không quy ra "trình độ" hay "đủ sức thi".
 */
import type { GradeValue, SelfGrade } from "@/lib/activity-store";
import { getPageAnswers } from "@/lib/page-answers";
import { getGrammarPages, type GrammarEntry } from "@/lib/page-grammar";
import { getPageQuiz, isCorrect, isGradable } from "@/lib/quiz";
import type { PageAnswers } from "@/lib/quiz-store";
import { SKILLS, skillOfSection, textbookGrammarFor, grammarHeading, type Skill } from "@/lib/skills";

/** Dưới ngần này lượt chấm thì chưa kết luận gì về một kỹ năng. */
export const MIN_SAMPLES = 5;

export interface SkillScore {
  skill: Skill;
  vi: string;
  /** Số bài/câu đã chấm thuộc kỹ năng này. */
  count: number;
  /** Trung bình 0–1; `null` khi chưa có lượt nào. */
  score: number | null;
}

export interface WeakGrammar {
  /** Điểm ngữ pháp trong giáo trình, nếu ghép được — có nghĩa và trang giải thích. */
  entry: GrammarEntry | null;
  /** Tên như in trong sách bài tập, dùng khi không ghép được. */
  heading: string;
  count: number;
  score: number;
  /** Trang bài tập để làm lại. */
  bookId: string;
  page: number;
}

export interface RedoItem {
  bookId: string;
  page: number;
  section: string;
  grade: GradeValue;
}

export interface Ability {
  skills: SkillScore[];
  weakGrammar: WeakGrammar[];
  redo: RedoItem[];
  /** Tổng số lượt đã chấm (cả hai nguồn). */
  total: number;
}

interface Sample {
  skill: Skill | null;
  value: number;
}

export function computeAbility(
  grades: Record<string, SelfGrade>,
  quizPages: Record<string, PageAnswers>
): Ability {
  const samples: Sample[] = [];
  const grammar = new Map<string, WeakGrammar & { sum: number }>();
  const redo: RedoItem[] = [];

  for (const [key, { grade }] of Object.entries(grades)) {
    // "bookId:page:answerKeyId" — bookId có thể chứa gạch ngang, không chứa ":".
    const [bookId, pageStr, id] = key.split(":");
    const page = Number(pageStr);
    const answerKey = getPageAnswers(bookId, page).find((k) => k.id === id);
    if (!answerKey) continue; // dữ liệu đáp án đổi id — bỏ qua lượt chấm cũ

    samples.push({ skill: skillOfSection(answerKey.section), value: grade });
    if (grade < 1) redo.push({ bookId, page, section: answerKey.section, grade });

    const heading = grammarHeading(answerKey.section);
    if (heading) {
      const entry = textbookGrammarFor(bookId, page, answerKey.section);
      const groupKey = entry ? `${entry.bookId}:${entry.id}` : `${bookId}:${heading}`;
      const prev = grammar.get(groupKey);
      if (prev) {
        prev.count++;
        prev.sum += grade;
        // Giữ trang có lượt chấm thấp nhất làm chỗ gợi ý làm lại.
        if (grade < prev.score) Object.assign(prev, { bookId, page, score: grade });
      } else {
        grammar.set(groupKey, { entry, heading, count: 1, sum: grade, score: grade, bookId, page });
      }
    }
  }

  for (const [key, { answers, checked }] of Object.entries(quizPages)) {
    const [bookId, pageStr] = key.split(":");
    const page = Number(pageStr);
    const onGrammarPage = getGrammarPages(bookId).includes(page);
    for (const section of getPageQuiz(bookId, page)) {
      // Bài luyện trên trang ngữ pháp chỉ đánh số ("1", "2") — vẫn là ngữ pháp.
      const skill =
        skillOfSection(section.title) ??
        (onGrammarPage && /^\d+$/.test(section.title.trim()) ? "문법" : null);
      for (const item of section.items) {
        if (!isGradable(item) || !checked.includes(item.id)) continue;
        samples.push({ skill, value: isCorrect(item, answers[item.id]) ? 1 : 0 });
      }
    }
  }

  const skills = SKILLS.map(({ id, vi }) => {
    const mine = samples.filter((s) => s.skill === id);
    return {
      skill: id,
      vi,
      count: mine.length,
      score: mine.length ? mine.reduce((n, s) => n + s.value, 0) / mine.length : null,
    };
  });

  const weakGrammar = [...grammar.values()]
    .map(({ sum, ...g }) => ({ ...g, score: sum / g.count }))
    .filter((g) => g.score < 0.75)
    .sort((a, b) => a.score - b.score || b.count - a.count);

  redo.sort((a, b) => a.grade - b.grade || a.bookId.localeCompare(b.bookId) || a.page - b.page);

  return { skills, weakGrammar, redo, total: samples.length };
}
