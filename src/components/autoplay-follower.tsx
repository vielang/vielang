"use client";

import { useEffect, useRef } from "react";
import { usePathname, useRouter } from "next/navigation";
import { getSpreadAnchor } from "@/lib/chapters";
import { stopAutoplay, useAutoplayStore } from "@/lib/autoplay-player";

/**
 * Nửa "lật trang" của nghe tự động — nửa phát tiếng ở `lib/autoplay-player`.
 *
 * Đặt ở layout gốc chứ không trong `ReaderView`: trình đọc bị dựng lại mỗi
 * lần lật trang, còn thứ này phải sống xuyên suốt, kể cả lúc bấm bắt đầu từ
 * trang chi tiết sách (chưa vào trình đọc).
 */
export function AutoplayFollower() {
  const router = useRouter();
  const pathname = usePathname();
  // Đọc đường dẫn qua ref: effect lật trang không được chạy lại mỗi lần
  // đổi đường dẫn (xem dưới). Effect này đứng trước nên luôn cập nhật kịp.
  const pathRef = useRef(pathname);
  useEffect(() => {
    pathRef.current = pathname;
  }, [pathname]);

  const bookId = useAutoplayStore((s) => s.bookId);
  const index = useAutoplayStore((s) => s.index);

  // Sang bài mới thì lật tới trang của nó, nếu trang đó chưa hiện. Chỉ chạy
  // khi ĐỔI BÀI: người dùng tự lật đi xem trang khác thì bài đang phát vẫn
  // phát tiếp, tới bài sau mới đưa về.
  useEffect(() => {
    if (!bookId) return;
    const { queue, visiblePages, double } = useAutoplayStore.getState();
    const page = queue[index]?.page;
    if (page === undefined) return;
    const inReader = pathRef.current.startsWith(`/read/${bookId}/`);
    if (inReader && visiblePages.includes(page)) return;
    const dest = double ? getSpreadAnchor(bookId, page) : page;
    router.push(`/read/${bookId}/${dest}`);
  }, [bookId, index, router]);

  // Rời khỏi trình đọc của sách đang nghe thì dừng. So với đường dẫn TRƯỚC
  // để không dừng ngay lúc vừa bấm bắt đầu ở trang chi tiết sách — lúc đó
  // vẫn đang đứng ngoài trình đọc, chờ lật vào.
  const prevPath = useRef(pathname);
  useEffect(() => {
    if (prevPath.current === pathname) return;
    prevPath.current = pathname;
    const current = useAutoplayStore.getState().bookId;
    if (current && !pathname.startsWith(`/read/${current}/`)) stopAutoplay();
  }, [pathname]);

  return null;
}
