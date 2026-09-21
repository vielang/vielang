import Link from "next/link";
import { Bookmark, BookOpen, BookOpenText, Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LanguageNav } from "@/components/layout/language-nav";
import { ThemeToggle } from "@/components/layout/theme-toggle";

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 border-b border-border/60 bg-background/80 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 max-w-5xl items-center justify-between gap-4 px-4">
        <div className="flex min-w-0 items-center gap-3">
          <Link
            href="/"
            className="flex shrink-0 items-center gap-2 font-semibold tracking-tight"
          >
            <BookOpenText className="size-5 text-primary" aria-hidden />
            {/* Màn điện thoại nhường chỗ cho nav ngôn ngữ và các nút bên
                phải — chỉ giữ lại logo. Dùng `sr-only` chứ không phải
                `hidden`: ẩn hẳn thì liên kết này mất luôn tên gọi, vì cái
                icon đã `aria-hidden`. */}
            <span className="sr-only sm:not-sr-only">VieTopik</span>
          </Link>
          <span className="h-4 w-px shrink-0 bg-border" aria-hidden />
          <LanguageNav />
        </div>
        <div className="flex items-center gap-1">
          {/* Chữ ở màn rộng, chỉ icon ở màn hẹp — header còn phải chừa chỗ
              cho nav ngôn ngữ bên trái. */}
          <Button asChild variant="ghost" size="sm" className="hidden sm:inline-flex">
            <Link href="/install">
              <Download className="size-4" aria-hidden />
              Tải app
            </Link>
          </Button>
          <Button
            asChild
            variant="ghost"
            size="icon"
            className="sm:hidden"
            aria-label="Cài app vào máy"
            title="Cài app vào máy"
          >
            <Link href="/install">
              <Download className="size-5" aria-hidden />
            </Link>
          </Button>
          {/* Chỉ icon ở mọi cỡ màn hình: header đã chật vì nav ngôn ngữ bên
              trái, mà "Tải app" đã chiếm suất hiện chữ rồi. */}
          <Button
            asChild
            variant="ghost"
            size="icon"
            aria-label="Tra cứu ngữ pháp"
            title="Tra cứu ngữ pháp"
          >
            <Link href="/grammar">
              <BookOpen className="size-5" aria-hidden />
            </Link>
          </Button>
          <Button
            asChild
            variant="ghost"
            size="icon"
            aria-label="Trang đã đánh dấu"
            title="Trang đã đánh dấu"
          >
            <Link href="/bookmarks">
              <Bookmark className="size-5" aria-hidden />
            </Link>
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}
