"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { LANGUAGES, languageHref } from "@/lib/languages";
import { cn } from "@/lib/utils";

/**
 * Chuyển đổi giữa các thư viện theo ngôn ngữ (KIIP tiếng Hàn, tiếng Anh sắp
 * bổ sung, ...) — đứng cạnh logo trong header như 1 nav chữ thường, không
 * phải nhóm nút. Nguồn ngôn ngữ lấy từ lib/languages.ts, thêm ngôn ngữ mới
 * ở đó là nav này tự có thêm mục, khỏi sửa gì ở đây.
 */
export function LanguageNav() {
  const pathname = usePathname();

  return (
    <nav className="flex items-center gap-3 text-sm">
      {LANGUAGES.map((language, i) => {
        const href = languageHref(language.slug);
        const isActive = href === "/" ? pathname === "/" : pathname.startsWith(href);
        return (
          <span key={language.code} className="flex items-center gap-3">
            {i > 0 && (
              <span className="text-border" aria-hidden>
                /
              </span>
            )}
            <Link
              href={href}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "transition-colors",
                isActive
                  ? "font-medium text-foreground"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              {language.label}
            </Link>
          </span>
        );
      })}
    </nav>
  );
}
