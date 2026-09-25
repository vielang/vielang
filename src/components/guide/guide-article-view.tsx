"use client";

import { useCallback, useEffect, useRef } from "react";
import Link from "next/link";
import { ArrowLeft, ChevronRight, Info, List, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import {
  SCHOOL_FACT_LABELS,
  checklistId,
  formatSchoolFact,
  formatUpdated,
  type GuideArticle,
  type GuideSection,
} from "@/lib/guide";
import { useGuideChecklistStore } from "@/lib/guide-checklist-store";

const EMPTY: string[] = [];

/** Cùng bộ class `prose` với bài học IT, thêm phần riêng cho checklist ở globals.css. */
const PROSE_CLASS =
  "prose prose-base dark:prose-invert max-w-none prose-headings:font-heading prose-headings:scroll-mt-20 " +
  "prose-table:text-sm prose-a:break-words";

/**
 * Nhắc kiểm tra lại, đặt ngay đầu bài chứ không để cuối: người ta đọc tới
 * mục hồ sơ là đi làm luôn, ít ai kéo xuống tận chân trang.
 */
const DISCLAIMER: Record<GuideSection["id"], string> = {
  visa: "Quy định visa và lệ phí thay đổi thường xuyên. Kiểm tra lại tại HiKorea (1345) hoặc Đại sứ quán trước khi nộp hồ sơ.",
  truong: "Học phí và điều kiện tuyển sinh đổi theo từng kỳ. Kiểm tra lại trên trang tuyển sinh của trường trước khi nộp.",
  "viec-lam": "Mức lương và quy định lao động đổi theo năm. Khi có tranh chấp, hỏi đường dây nóng 1350 hoặc trung tâm hỗ trợ lao động nước ngoài.",
};

export function GuideArticleView({
  section,
  article,
}: {
  section: GuideSection;
  article: GuideArticle;
}) {
  const id = checklistId(section.id, article.slug);
  const hasHydrated = useGuideChecklistStore((s) => s.hasHydrated);
  const checked = useGuideChecklistStore((s) => s.checked[id] ?? EMPTY);
  const toggle = useGuideChecklistStore((s) => s.toggle);
  const reset = useGuideChecklistStore((s) => s.reset);
  const body = useRef<HTMLDivElement>(null);

  // Ô tick nằm trong HTML dựng sẵn lúc build, không phải phần tử React —
  // đồng bộ trạng thái vào DOM sau mỗi lần đổi. Chờ nạp xong localStorage để
  // không vẽ ô trống rồi mới nhảy sang đã tick.
  useEffect(() => {
    if (!hasHydrated || !body.current) return;
    for (const input of body.current.querySelectorAll<HTMLInputElement>("input[data-check]")) {
      input.checked = checked.includes(input.dataset.check ?? "");
    }
  }, [hasHydrated, checked]);

  // Nghe sự kiện `change` GỐC của trình duyệt chứ không dùng `onChange` của
  // React: ô tick nằm trong HTML dựng sẵn, React không quản nó nên không
  // sinh `onChange` cho nó — bấm vào không có gì xảy ra.
  const onChange = useCallback(
    (e: Event) => {
      const target = e.target;
      if (target instanceof HTMLInputElement && target.dataset.check) {
        toggle(id, target.dataset.check);
      }
    },
    [id, toggle]
  );
  useEffect(() => {
    const el = body.current;
    if (!el) return;
    el.addEventListener("change", onChange);
    return () => el.removeEventListener("change", onChange);
  }, [onChange]);

  const done = hasHydrated ? checked.length : 0;
  const facts = Object.entries(SCHOOL_FACT_LABELS).filter(([key]) => article.facts[key]);
  const toc = article.headings.filter((h) => h.level === 2 && h.text !== "Nguồn");

  return (
    <article className="mx-auto flex w-full max-w-2xl flex-col gap-6">
      {/* Quay về đúng tab của mục đang đọc — "Cẩm nang" trơn sẽ mở tab đầu
          (Visa) dù người ta đang đọc bài về trường. */}
      <Link
        href={`/cam-nang/${section.id}`}
        className="inline-flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="size-4" aria-hidden />
        Cẩm nang · {section.title}
      </Link>

      <header className="-mt-2 flex flex-col gap-2">
        {article.facts.code && (
          <span className="w-fit rounded-md bg-primary/10 px-2 py-0.5 font-mono text-sm font-semibold text-primary">
            {article.facts.code}
          </span>
        )}
        <h1 className="font-heading text-2xl font-semibold tracking-tight">{article.title}</h1>
        {/* Tiêu đề thường đã kèm tên Hàn trong ngoặc — đừng nhắc lại ngay dưới. */}
        {article.facts.nameKo && !article.title.includes(article.facts.nameKo) && (
          <p className="font-korean -mt-1 text-sm text-muted-foreground">{article.facts.nameKo}</p>
        )}
        {article.summary && <p className="text-muted-foreground">{article.summary}</p>}
        <p className="text-xs text-muted-foreground tabular-nums">
          Cập nhật {formatUpdated(article.updated)}
        </p>
      </header>

      <p className="flex gap-2 rounded-xl border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm">
        <Info className="mt-0.5 size-4 shrink-0 text-amber-600 dark:text-amber-400" aria-hidden />
        <span>{DISCLAIMER[section.id]}</span>
      </p>

      {facts.length > 0 && (
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 rounded-xl border border-border px-4 py-3 text-sm">
          {facts.map(([key, label]) => (
            <div key={key} className="contents">
              <dt className="text-muted-foreground">{label}</dt>
              <dd className={key === "nameKo" ? "font-korean" : undefined}>
                {formatSchoolFact(key, article.facts[key])}
              </dd>
            </div>
          ))}
          {article.facts.website && (
            <div className="contents">
              <dt className="text-muted-foreground">Website</dt>
              <dd className="min-w-0 truncate">
                <a
                  href={article.facts.website}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary underline-offset-4 hover:underline"
                >
                  {article.facts.website.replace(/^https?:\/\//, "").replace(/\/$/, "")}
                </a>
              </dd>
            </div>
          )}
        </dl>
      )}

      {article.checks > 0 && (
        <div className="flex items-center gap-3 rounded-xl border border-border bg-muted/40 px-4 py-3 text-sm">
          <span className="shrink-0 tabular-nums">
            Hồ sơ: <strong>{done}</strong>/{article.checks} đã chuẩn bị
          </span>
          <Progress value={(done / article.checks) * 100} className="h-1 flex-1" />
          {done > 0 && (
            <Button
              variant="ghost"
              size="icon"
              className="size-7 shrink-0"
              onClick={() => reset(id)}
              aria-label="Bỏ tick toàn bộ checklist"
              title="Bỏ tick toàn bộ"
            >
              <RotateCcw className="size-3.5" aria-hidden />
            </Button>
          )}
        </div>
      )}

      {toc.length > 2 && (
        <details className="group rounded-xl border border-border bg-muted/40 px-4 py-3">
          <summary className="flex cursor-pointer list-none items-center gap-1.5 text-sm text-muted-foreground">
            <List className="size-4" aria-hidden />
            Nội dung bài · {toc.length} mục
            <ChevronRight className="ml-auto size-4 transition-transform group-open:rotate-90" aria-hidden />
          </summary>
          <ol className="mt-2 flex flex-col gap-1 text-sm">
            {toc.map((h, i) => (
              <li key={h.id} className="flex gap-2">
                <span className="w-5 shrink-0 text-right text-muted-foreground tabular-nums">{i + 1}.</span>
                <a href={`#${h.id}`} className="text-muted-foreground hover:text-foreground">
                  {h.text}
                </a>
              </li>
            ))}
          </ol>
        </details>
      )}

      <div
        ref={body}
        className={PROSE_CLASS}
        dangerouslySetInnerHTML={{ __html: article.html }}
      />
    </article>
  );
}
