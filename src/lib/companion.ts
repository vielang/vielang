/**
 * Giọng "người bạn đồng hành" của My page: một lời chào, một câu nhận xét
 * đúng lúc — thay cho một bức tường số liệu.
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
import { addDays, dayKey, isStudyDay, type DayStats } from "@/lib/activity";
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
    body: "Bắt đầu nhẹ nhàng thôi — mở bài đang học dở và đọc vài phút.",
  };
}
