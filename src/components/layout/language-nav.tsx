"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";

const LANGUAGES = [
  { href: "/", label: "Tiếng Hàn" },
  { href: "/en", label: "Tiếng Anh" },
] as const;

/**
 * Chuyển đổi giữa các mảng giáo trình theo ngôn ngữ (hiện tại: KIIP tiếng Hàn
 * và khung tiếng Anh sắp bổ sung). So khớp pathname theo tiền tố vì mỗi mảng
 * có route con riêng (`/books/[id]`, `/en/books/[id]`...).
 */
export function LanguageNav() {
  const pathname = usePathname();

  return (
    <nav className="flex items-center gap-1 rounded-lg bg-muted p-1 text-sm">
      {LANGUAGES.map(({ href, label }) => {
        const isActive =
          href === "/" ? pathname === "/" : pathname.startsWith(href);
        return (
          <Link
            key={href}
            href={href}
            aria-current={isActive ? "page" : undefined}
            className={cn(
              "rounded-md px-3 py-1.5 font-medium transition-colors",
              isActive
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            )}
          >
            {label}
          </Link>
        );
      })}
    </nav>
  );
}
