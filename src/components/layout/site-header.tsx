"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { NavigationMenu } from "radix-ui";
import { BookOpenText, ChevronDown, Download, MoreHorizontal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { ThemeToggle } from "@/components/layout/theme-toggle";
import { cn } from "@/lib/utils";
import { NAV_TABS, activeTab, type NavTab } from "@/lib/nav";

/**
 * Header.
 *
 * Máy tính: logo + tab chính; tab có mục con (Thư viện, Luyện thi, Cẩm nang)
 * mở menu thả xuống khi rê chuột hoặc bấm, liệt kê mục con kèm một dòng mô tả
 * — nội dung nhiều lên thì chọn thẳng từ header, khỏi vào trang rồi mới chọn.
 *
 * Điện thoại: tab chính nằm ở thanh dưới đáy (`TabBar`), nên header dành chỗ
 * cho MỤC CON của tab đang mở (Tiếng Hàn | Tiếng Anh | IT…) thay cho logo.
 * Tab không có mục con thì hiện tên tab; trang ngoài các tab thì hiện logo.
 * Hàng tab trong trang (`SegmentedNav`) vì thế ẩn trên điện thoại.
 */
export function SiteHeader() {
  const pathname = usePathname();
  const current = activeTab(pathname);

  return (
    // Lề trên theo vùng an toàn: thêm app vào màn hình chính trên iOS thì
    // app chạy toàn màn hình, không chừa thì chữ nằm dưới thanh trạng thái.
    <header className="sticky top-0 z-40 border-b border-border/60 bg-background/80 pt-[env(safe-area-inset-top)] backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 max-w-5xl items-center justify-between gap-2 px-4 lg:gap-4">
        <div className="flex min-w-0 items-center gap-4">
          <Link
            href="/"
            className={cn(
              "shrink-0 items-center gap-2 font-semibold tracking-tight",
              current ? "hidden lg:flex" : "flex"
            )}
          >
            <BookOpenText className="size-5 text-primary" aria-hidden />
            VieTopik
          </Link>
          {current && <MobileSubnav tab={current} pathname={pathname} />}
          <DesktopNav current={current} pathname={pathname} />
        </div>

        <div className="flex shrink-0 items-center gap-1">
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

/** Điện thoại: mục con của tab đang mở, cùng kiểu với hàng tab trong trang. */
function MobileSubnav({ tab, pathname }: { tab: NavTab; pathname: string }) {
  if (!tab.children?.length) {
    return <span className="truncate font-semibold tracking-tight lg:hidden">{tab.label}</span>;
  }
  const active = tab.activeChild?.(pathname);
  return (
    <nav
      aria-label={tab.label}
      className="flex min-w-0 overflow-x-auto text-sm [scrollbar-width:none] lg:hidden [&::-webkit-scrollbar]:hidden"
    >
      <span className="flex shrink-0 gap-0.5 rounded-lg bg-muted p-0.5">
        {tab.children.map((child) => (
          <Link
            key={child.href}
            href={child.href}
            aria-current={child.href === active ? "page" : undefined}
            className={cn(
              "shrink-0 rounded-md px-3 py-1.5 transition-colors",
              child.href === active
                ? "bg-background font-medium shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {child.label}
          </Link>
        ))}
      </span>
    </nav>
  );
}

const TOP_ITEM =
  "inline-flex items-center gap-1 rounded-md px-2.5 py-1.5 transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring";

/**
 * Máy tính: tab chính; tab có mục con mở menu thả xuống (Radix NavigationMenu
 * lo rê chuột / bấm / bàn phím / Esc). Chọn một mục thì menu tự đóng.
 */
function DesktopNav({ current, pathname }: { current: NavTab | undefined; pathname: string }) {
  return (
    <NavigationMenu.Root aria-label="Điều hướng chính" className="relative hidden lg:block">
      <NavigationMenu.List className="flex items-center gap-1 text-sm">
        {NAV_TABS.map((tab) => {
          const isCurrent = tab === current;
          const tone = isCurrent
            ? "bg-muted font-medium text-foreground"
            : "text-muted-foreground hover:text-foreground data-[state=open]:text-foreground";
          if (!tab.children?.length) {
            return (
              <NavigationMenu.Item key={tab.href}>
                <NavigationMenu.Link asChild active={isCurrent}>
                  <Link
                    href={tab.href}
                    aria-current={isCurrent ? "page" : undefined}
                    className={cn(TOP_ITEM, tone)}
                  >
                    {tab.label}
                  </Link>
                </NavigationMenu.Link>
              </NavigationMenu.Item>
            );
          }
          const active = isCurrent ? tab.activeChild?.(pathname) : undefined;
          return (
            <NavigationMenu.Item key={tab.href} className="relative">
              <NavigationMenu.Trigger className={cn(TOP_ITEM, "group", tone)}>
                {tab.label}
                <ChevronDown
                  className="size-3.5 opacity-60 transition-transform group-data-[state=open]:rotate-180"
                  aria-hidden
                />
              </NavigationMenu.Trigger>
              <NavigationMenu.Content className="absolute top-full left-0 z-50 mt-2 w-72 rounded-xl border bg-popover p-1.5 text-popover-foreground shadow-lg">
                <ul className="flex flex-col">
                  {tab.children.map((child) => (
                    <li key={child.href}>
                      <NavigationMenu.Link asChild active={child.href === active}>
                        <Link
                          href={child.href}
                          aria-current={child.href === active ? "page" : undefined}
                          className={cn(
                            "block rounded-lg px-3 py-2 transition-colors outline-none hover:bg-muted focus-visible:bg-muted",
                            child.href === active && "bg-muted"
                          )}
                        >
                          <span className="block text-sm font-medium">{child.label}</span>
                          {child.description && (
                            <span className="mt-0.5 block text-xs text-muted-foreground">
                              {child.description}
                            </span>
                          )}
                        </Link>
                      </NavigationMenu.Link>
                    </li>
                  ))}
                </ul>
              </NavigationMenu.Content>
            </NavigationMenu.Item>
          );
        })}
      </NavigationMenu.List>
    </NavigationMenu.Root>
  );
}
