"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { LANGUAGES, languageHref } from "@/lib/languages";
import { cn } from "@/lib/utils";

/**
 * Chọn thư viện theo ngôn ngữ (KIIP tiếng Hàn, tiếng Anh, …) — nút chọn đặt
 * ở đầu trang thư viện, vì nó chỉ áp dụng cho thư viện. Nguồn ngôn ngữ lấy
 * từ lib/languages.ts: thêm ngôn ngữ mới ở đó là có thêm mục, khỏi sửa ở đây.
 */
export function LanguageNav() {
  const pathname = usePathname();

  return (
    <nav aria-label="Ngôn ngữ" className="flex gap-0.5 self-start rounded-lg bg-muted p-0.5 text-sm">
      {LANGUAGES.map((language) => {
        const href = languageHref(language.slug);
        const isActive = href === "/" ? pathname === "/" : pathname.startsWith(href);
        return (
          <Link
            key={language.code}
            href={href}
            aria-current={isActive ? "page" : undefined}
            className={cn(
              "rounded-md px-3 py-1.5 transition-colors",
              isActive ? "bg-background font-medium shadow-sm" : "text-muted-foreground hover:text-foreground"
            )}
          >
            {language.label}
          </Link>
        );
      })}
    </nav>
  );
}
