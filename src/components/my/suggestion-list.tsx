"use client";

import Link from "next/link";
import { BookOpen, ChevronRight, ListChecks, RotateCcw } from "lucide-react";
import type { Suggestion } from "@/lib/companion";

const ICONS = { grammar: BookOpen, redo: RotateCcw, quiz: ListChecks } as const;

/** Vài việc nên làm tiếp — mỗi việc một dòng bấm được, không phải bảng số. */
export function SuggestionList({ items }: { items: Suggestion[] }) {
  return (
    <ul className="flex flex-col gap-2">
      {items.map((s) => {
        const Icon = ICONS[s.kind];
        return (
          <li key={s.href + s.kind}>
            <Link
              href={s.href}
              className="group flex items-center gap-3 rounded-xl border border-border p-3 transition-colors hover:bg-muted/60"
            >
              <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-muted">
                <Icon className="size-4 text-foreground/80" aria-hidden />
              </span>
              <span className="min-w-0 flex-1">
                <span className="font-korean block truncate text-sm font-medium">{s.title}</span>
                <span className="block truncate text-xs text-muted-foreground">{s.detail}</span>
              </span>
              <ChevronRight
                className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                aria-hidden
              />
            </Link>
          </li>
        );
      })}
    </ul>
  );
}
