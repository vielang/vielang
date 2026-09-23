"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { BookOpenText, Download, MoreHorizontal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { ThemeToggle } from "@/components/layout/theme-toggle";
import { cn } from "@/lib/utils";
import { NAV_TABS, activeTab } from "@/lib/nav";

/**
 * Header: máy tính là logo + nav chính bằng CHỮ; điện thoại chỉ còn logo và
 * nút "⋯" cho mấy việc ít dùng (tải app, sáng/tối) — điều hướng chính nằm ở
 * thanh tab dưới đáy (xem `TabBar`), nơi ngón tay với tới dễ hơn và có chữ
 * nên không phải đoán icon.
 */
export function SiteHeader() {
  const pathname = usePathname();
  const current = activeTab(pathname);

  return (
    <header className="sticky top-0 z-40 border-b border-border/60 bg-background/80 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 max-w-5xl items-center justify-between gap-4 px-4">
        <div className="flex min-w-0 items-center gap-4">
          {/* Điện thoại chỉ còn tên app: đang ở đâu thì thanh tab dưới đáy
              đã tô rõ, ghi lại tên trang ở đây là lặp với tiêu đề trang. */}
          <Link href="/" className="flex shrink-0 items-center gap-2 font-semibold tracking-tight">
            <BookOpenText className="size-5 text-primary" aria-hidden />
            VieTopik
          </Link>
          <nav aria-label="Điều hướng chính" className="hidden items-center gap-1 text-sm lg:flex">
            {NAV_TABS.map((tab) => (
              <Link
                key={tab.href}
                href={tab.href}
                aria-current={tab === current ? "page" : undefined}
                className={cn(
                  "rounded-md px-2.5 py-1.5 transition-colors",
                  tab === current
                    ? "bg-muted font-medium text-foreground"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                {tab.label}
              </Link>
            ))}
          </nav>
        </div>

        <div className="flex items-center gap-1">
          <Button asChild variant="ghost" size="sm" className="hidden lg:inline-flex">
            <Link href="/install">
              <Download className="size-4" aria-hidden />
              Tải app
            </Link>
          </Button>
          <span className="hidden lg:block">
            <ThemeToggle />
          </span>

          {/* Điện thoại: gom việc ít dùng vào một nút, header khỏi thành dãy icon. */}
          <Popover>
            <PopoverTrigger asChild>
              <Button variant="ghost" size="icon" aria-label="Thêm" className="lg:hidden">
                <MoreHorizontal className="size-5" aria-hidden />
              </Button>
            </PopoverTrigger>
            <PopoverContent align="end" className="w-52 p-1">
              <Button asChild variant="ghost" size="sm" className="w-full justify-start">
                <Link href="/install">
                  <Download className="size-4" aria-hidden />
                  Tải app về máy
                </Link>
              </Button>
              <div className="flex items-center justify-between gap-2 px-2.5 py-1 text-[0.8rem]">
                Giao diện sáng/tối
                <ThemeToggle />
              </div>
            </PopoverContent>
          </Popover>
        </div>
      </div>
    </header>
  );
}
