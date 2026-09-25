"use client";

import { useCallback, useEffect, useRef } from "react";
import { BadgeCheck, Building2, Globe, GraduationCap, Info, Languages, MapPin, RotateCcw, Wallet, type LucideIcon } from "lucide-react";
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
import { ARTICLE_PROSE_BASE, ArticleTocAside, ArticleTocMobile } from "@/components/layout/article-toc";
import { BackLink } from "@/components/layout/back-link";

const EMPTY: string[] = [];

/** Cùng bộ class `prose` gốc với bài học IT; checklist có CSS riêng ở globals.css. */
const PROSE_CLASS = ARTICLE_PROSE_BASE + " prose-a:break-words";

/**
 * Nhắc kiểm tra lại — một dòng chữ nhỏ ngay dưới ngày cập nhật, không phải
 * một cái khung màu: đặt ở đầu bài vì người ta đọc tới hồ sơ là đi làm luôn,
 * nhưng không được to tới mức chiếm cả màn hình đầu.
 */
const DISCLAIMER: Record<GuideSection["id"], string> = {
  visa: "Quy định và lệ phí có thể đổi. Đối chiếu HiKorea (1345) hoặc Đại sứ quán trước khi nộp.",
  truong: "Học phí, điều kiện đổi theo từng kỳ. Xem lại trang tuyển sinh của trường trước khi nộp.",
  "viec-lam": "Quy định lao động đổi theo năm. Có tranh chấp, hỏi đường dây nóng 1350.",
};

const FACT_ICONS: Record<string, LucideIcon> = {
  city: MapPin,
  kind: Building2,
  topik: GraduationCap,
  tuition: Wallet,
  certified: BadgeCheck,
  languageSchool: Languages,
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
  // Tên Hàn thường đã nằm trong tiêu đề ("Đại học Korea (고려대학교)") — khỏi
  // lặp lại trong lưới thông số.
  const facts = Object.entries(SCHOOL_FACT_LABELS).filter(
    ([key]) => article.facts[key] && !(key === "nameKo" && article.title.includes(article.facts.nameKo))
  );
  const toc = article.headings.filter((h) => h.level === 2 && h.text !== "Nguồn");

  return (
    // Màn rộng: bài ở giữa, mục lục dính ở cột phải. Màn hẹp: một cột, mục
    // lục gập lại thành một dòng — như trang bài học IT.
    <div className="mx-auto w-full max-w-2xl lg:grid lg:max-w-none lg:grid-cols-[minmax(0,42rem)_12rem] lg:justify-center lg:gap-10">
      <article className="flex min-w-0 flex-col gap-5">
        {/* Quay về đúng tab của mục đang đọc — "Cẩm nang" trơn sẽ mở tab đầu
            (Visa) dù người ta đang đọc bài về trường. */}
        <BackLink href={`/cam-nang/${section.id}`}>Cẩm nang · {section.title}</BackLink>

        <header className="-mt-1 flex flex-col gap-2">
          {article.facts.code && (
            <span className="w-fit rounded-md bg-primary/10 px-2 py-0.5 text-sm font-semibold text-primary">
              {article.facts.code}
            </span>
          )}
          <h1 className="font-heading text-2xl font-semibold leading-tight tracking-tight">{article.title}</h1>
          {article.summary && <p className="text-muted-foreground">{article.summary}</p>}
          <p className="flex items-start gap-1.5 text-xs text-muted-foreground">
            <Info className="mt-px size-3.5 shrink-0" aria-hidden />
            <span>
              <span className="tabular-nums">Cập nhật {formatUpdated(article.updated)}.</span>{" "}
              {DISCLAIMER[section.id]}
            </span>
          </p>
        </header>

        {facts.length > 0 && (
          <dl className="grid grid-cols-2 gap-x-4 gap-y-3 border-y border-border/60 py-4 text-sm">
            {facts.map(([key, label]) => {
              const Icon = FACT_ICONS[key];
              return (
                <div key={key} className="flex min-w-0 flex-col gap-0.5">
                  <dt className="flex items-center gap-1.5 text-xs text-muted-foreground">
                    {Icon && <Icon className="size-3.5 shrink-0" aria-hidden />}
                    {label}
                  </dt>
                  <dd className={key === "nameKo" ? "font-korean" : "font-medium"}>
                    {formatSchoolFact(key, article.facts[key])}
                  </dd>
                </div>
              );
            })}
            {article.facts.website && (
              <div className="col-span-2 flex min-w-0 flex-col gap-0.5">
                <dt className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Globe className="size-3.5 shrink-0" aria-hidden />
                  Website
                </dt>
                <dd className="min-w-0 truncate">
                  <a
                    href={article.facts.website}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="font-medium text-primary underline-offset-4 hover:underline"
                  >
                    {article.facts.website.replace(/^https?:\/\//, "").replace(/\/$/, "")}
                  </a>
                </dd>
              </div>
            )}
          </dl>
        )}

        {article.checks > 0 && (
          <div className="flex items-center gap-3 rounded-lg bg-muted/60 px-3 py-2.5 text-sm">
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

        {toc.length > 2 && <ArticleTocMobile headings={toc} />}

        <div ref={body} className={PROSE_CLASS} dangerouslySetInnerHTML={{ __html: article.html }} />
      </article>

      {toc.length > 2 && <ArticleTocAside headings={toc} />}
    </div>
  );
}
