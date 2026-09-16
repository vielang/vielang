import Link from "next/link";
import { BookOpenText } from "lucide-react";
import { LanguageNav } from "@/components/layout/language-nav";
import { ThemeToggle } from "@/components/layout/theme-toggle";

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 border-b border-border/60 bg-background/80 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 max-w-5xl items-center justify-between gap-4 px-4">
        <Link
          href="/"
          className="flex shrink-0 items-center gap-2 font-semibold tracking-tight"
        >
          <BookOpenText className="size-5 text-primary" aria-hidden />
          <span>KIIP Reader</span>
        </Link>
        <div className="flex items-center gap-2">
          <LanguageNav />
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}
