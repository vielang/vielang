"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { NAV_TABS, activeTab, hidesTabBar } from "@/lib/nav";

/**
 * Thanh điều hướng dưới đáy màn hình ĐIỆN THOẠI: bốn tab có chữ, tab đang
 * mở được tô — thay cho hàng icon không nhãn trên header, để luôn biết mình
 * đang ở đâu và đi đâu. Máy tính dùng nav trong header (xem `SiteHeader`).
 *
 * Trả về cả khoảng trống bằng chiều cao thanh, để nội dung cuối trang không
 * bị che. Màn đang làm bài thì ẩn hẳn (xem `hidesTabBar`).
 */
export function TabBar() {
  const pathname = usePathname();
  if (hidesTabBar(pathname)) return null;
  const current = activeTab(pathname);

  return (
    <>
      <div aria-hidden className="h-[max(4rem,calc(3.5rem+env(safe-area-inset-bottom)))] lg:hidden" />
      <nav
        aria-label="Điều hướng chính"
        className="fixed inset-x-0 bottom-0 z-40 border-t border-border bg-background/95 backdrop-blur lg:hidden"
      >
        <ul className="mx-auto flex max-w-md items-stretch pb-[env(safe-area-inset-bottom)]">
          {NAV_TABS.map((tab) => {
            const isActive = tab === current;
            return (
              <li key={tab.href} className="flex-1">
                <Link
                  href={tab.href}
                  aria-current={isActive ? "page" : undefined}
                  className={cn(
                    "flex flex-col items-center gap-0.5 py-1.5 text-[0.7rem] transition-colors",
                    isActive ? "font-medium text-foreground" : "text-muted-foreground"
                  )}
                >
                  <tab.icon className={cn("size-5", isActive && "text-primary")} aria-hidden />
                  {tab.label}
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>
    </>
  );
}
