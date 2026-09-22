/**
 * Giọng "người bạn đồng hành" của My page: một lời chào, một câu nhận xét
 * đúng lúc, và vài việc nên làm tiếp — thay cho một bức tường số liệu.
 *
 * Nguyên tắc giọng văn:
 * - Ngắn. Một câu tiêu đề, một câu thân. Người học mở trang để biết "giờ làm
 *   gì", không phải để đọc báo cáo.
 * - Khích lệ, không trách. Nghỉ lâu thì chào mừng quay lại, không đếm số
 *   ngày đã bỏ.
 * - Luôn kèm MỘT việc cụ thể làm được ngay.
 *
 * Hàm thuần — nhận `now` từ ngoài vào để test được mọi khung giờ.
 */
import type { Book } from "@/lib/books";
import { addDays, dayKey, isStudyDay, type DayStats } from "@/lib/activity";
import type { Ability } from "@/lib/ability";

const LEVEL_VI: Record<string, string> = {
  초급1: "Sơ cấp 1",
  초급2: "Sơ cấp 2",
  중급1: "Trung cấp 1",
  중급2: "Trung cấp 2",
};

/**
 * Tên ngắn cho danh sách chật chỗ: "Bài tập Sơ cấp 1" thay vì "Sách bài tập –
 * Văn hóa Xã hội Hàn Quốc, Tập 1". Tên đầy đủ dài tới mức trên điện thoại nó
 * chiếm hết dòng và đẩy mất số trang — đúng phần người ta cần nhìn.
 */
export function shortBookName(book: Book): string {
  const level = book.levelLabelKo ? LEVEL_VI[book.levelLabelKo] : undefined;
  if (!level) return book.titleVi;
  return `${book.kind === "workbook" ? "Bài tập" : "Giáo trình"} ${level}`;
}

export function greeting(now: Date): string {
  const h = now.getHours();
  if (h < 11) return "Chào buổi sáng";
  if (h < 14) return "Chào buổi trưa";
  if (h < 18) return "Chào buổi chiều";
  return "Chào buổi tối";
}

export interface CompanionInput {
  days: Record<string, DayStats>;
  now: Date;
  streak: number;
  weekMinutes: number;
  goal: number;
}

export interface CompanionMessage {
  title: string;
  body: string;
}

/** Số ngày kể từ lần học gần nhất (0 = hôm nay), `null` nếu chưa từng học. */
export function daysSinceLastStudy(days: Record<string, DayStats>, now: Date): number | null {
  for (let i = 0; i <= 365; i++) {
    if (isStudyDay(days[dayKey(addDays(now, -i))])) return i;
  }
  return null;
}

export function companionMessage({
  days,
  now,
  streak,
  weekMinutes,
  goal,
}: CompanionInput): CompanionMessage {
  const todayMinutes = Math.floor((days[dayKey(now)]?.activeMs ?? 0) / 60_000);
  const since = daysSinceLastStudy(days, now);

  if (since === 0) {
    if (weekMinutes >= goal) {
      return {
        title: "Bạn đã đạt mục tiêu tuần rồi!",
        body: `${weekMinutes} phút tuần này, vượt mức ${goal} phút đã đặt. Giữ nhịp này nhé.`,
      };
    }
    return {
      title: `Hôm nay bạn đã học ${todayMinutes} phút`,
      body:
        streak > 1
          ? `${streak} ngày liên tiếp rồi — cứ đều đặn thế này là tiến bộ nhanh lắm.`
          : "Mỗi ngày một chút là đủ. Mai mình học tiếp nhé.",
    };
  }
  if (streak > 0) {
    return {
      title: `Giữ chuỗi ${streak} ngày nhé`,
      body: "Hôm nay chỉ cần vài phút là chuỗi vẫn tiếp tục.",
    };
  }
  if (since !== null && since <= 7) {
    return {
      title: "Học tiếp nào",
      body: "10 phút hôm nay là đủ để ôn lại những gì bạn vừa học.",
    };
  }
  return {
    title: "Mừng bạn quay lại!",
    body: "Bắt đầu nhẹ nhàng thôi — mở trang đang học dở và đọc một trang.",
  };
}

export interface Suggestion {
  /** Việc cần làm, một dòng. */
  title: string;
  /** Vì sao / ở đâu, một dòng nhỏ. */
  detail: string;
  href: string;
  /** Mã phân loại — để UI chọn biểu tượng. */
  kind: "grammar" | "redo" | "quiz";
}

export interface QuizWrongLite {
  bookId: string;
  page: number;
  where: string;
}

/**
 * Tối đa `limit` việc nên làm tiếp, mỗi loại một việc, xếp theo ích lợi:
 * ôn điểm ngữ pháp yếu nhất (có giải thích để đọc lại) → làm lại bài tự chấm
 * sai nhiều nhất → làm lại câu quiz sai. Mỗi loại chỉ lấy MỘT: ba gợi ý cùng
 * một kiểu thì chẳng khác gì một danh sách dài.
 */
export function buildSuggestions(
  ability: Ability,
  quizWrong: QuizWrongLite[],
  books: readonly Book[],
  limit = 3
): Suggestion[] {
  const name = (id: string) => {
    const book = books.find((b) => b.id === id);
    return book ? shortBookName(book) : id;
  };
  const out: Suggestion[] = [];

  const weakest = ability.weakGrammar[0];
  if (weakest) {
    out.push({
      kind: "grammar",
      title: `Ôn lại ngữ pháp ${weakest.entry?.title ?? weakest.heading}`,
      detail: weakest.entry?.vi ?? `${name(weakest.bookId)} · trang ${weakest.page}`,
      href: weakest.entry
        ? `/read/${weakest.entry.bookId}/${weakest.entry.page}`
        : `/read/${weakest.bookId}/${weakest.page}`,
    });
  }

  // Bài thuộc điểm ngữ pháp vừa gợi ý thì bỏ qua — khỏi nhắc hai lần một chuyện.
  const redo = ability.redo.find(
    (r) => !(weakest && r.bookId === weakest.bookId && r.page === weakest.page)
  );
  if (redo) {
    out.push({
      kind: "redo",
      title: `Làm lại bài ${redo.section}`,
      detail: `${name(redo.bookId)} · trang ${redo.page} · lần trước ${
        redo.grade === 0 ? "sai nhiều" : "sai vài câu"
      }`,
      href: `/read/${redo.bookId}/${redo.page}`,
    });
  }

  const quiz = quizWrong[0];
  if (quiz) {
    out.push({
      kind: "quiz",
      title: "Làm lại câu quiz còn sai",
      detail: `${name(quiz.bookId)} · trang ${quiz.page} · ${quiz.where}`,
      href: `/read/${quiz.bookId}/${quiz.page}`,
    });
  }

  return out.slice(0, limit);
}
