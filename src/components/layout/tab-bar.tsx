"use client";

import Link from "next/link";
import { useLayoutEffect } from "react";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { NAV_TABS, activeTab, hidesTabBar } from "@/lib/nav";

/**
 * Thanh điều hướng dưới đáy màn hình ĐIỆN THOẠI: các tab có chữ, tab đang
 * mở được tô — thay cho hàng icon không nhãn trên header, để luôn biết mình
 * đang ở đâu và đi đâu. Máy tính dùng nav trong header (xem `SiteHeader`).
 *
 * Trả về cả khoảng trống bằng chiều cao thanh, để nội dung cuối trang không
 * bị che. Màn đang làm bài thì ẩn hẳn (xem `hidesTabBar`).
 */
export function TabBar() {
  const pathname = usePathname();

  // Gắn lại cờ PWA mà script trong <head> đã gắn (xem layout.tsx): ở chế độ
  // dev, Strict Mode dựng lại <html> và xoá mất thuộc tính không có trong
  // JSX. Bản build thật thì script là đủ, dòng này không đổi gì.
  useLayoutEffect(() => {
    if ((navigator as Navigator & { standalone?: boolean }).standalone) {
      document.documentElement.setAttribute("data-standalone", "");
    }
  }, []);

  if (hidesTabBar(pathname)) return null;
  const current = activeTab(pathname);

  return (
    <>
      {/* Đệm đáy theo `--standalone-safe-bottom` chứ không lấy thẳng
          env(safe-area-inset-bottom): trong Safari con số đó đổi theo thanh
          công cụ của trình duyệt — xem globals.css. */}
      <div aria-hidden className="h-[max(4rem,calc(3.5rem+var(--standalone-safe-bottom)))] lg:hidden" />
      <nav
        aria-label="Điều hướng chính"
        className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-background/95 backdrop-blur lg:hidden"
      >
        <ul className="mx-auto flex max-w-md items-stretch pb-(--standalone-safe-bottom)">
          {NAV_TABS.map((tab) => {
            const isActive = tab === current;
            return (
              <li key={tab.href} className="flex-1">
                {/* Chiều cao CỐ ĐỊNH, chữ một dòng với chiều cao dòng cố định:
                    trước đây thanh cao theo chữ, và trên iPhone nó cao vọt lên
                    ở tab Cẩm nang / Đánh dấu — hai nhãn có dấu chồng ("ẩ",
                    "ấ") cao nhất, lại in đậm khi đang mở. Màn hẹp (320px, hoặc
                    phóng to hiển thị) thì "Góc học tập" xuống hai dòng — nay
                    chữ co nhẹ theo bề ngang màn (tối thiểu 10px) để vừa ô. Cố
                    định thì thanh cao như nhau ở mọi trang, mọi máy. */}
                <Link
                  href={tab.href}
                  aria-current={isActive ? "page" : undefined}
                  className={cn(
                    "flex h-13 flex-col items-center justify-center gap-0.5 text-[clamp(0.625rem,2.9vw,0.7rem)] transition-colors",
                    isActive ? "font-medium text-foreground" : "text-muted-foreground"
                  )}
                >
                  <tab.icon className={cn("size-5 shrink-0", isActive && "text-primary")} aria-hidden />
                  <span className="max-w-full truncate px-0.5 leading-4">{tab.label}</span>
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>
    </>
  );
}
