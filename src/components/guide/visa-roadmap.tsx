import { Fragment } from "react";
import Link from "next/link";
import { ChevronRight } from "lucide-react";
import { VISA_PATHS, articleHref, getArticle } from "@/lib/guide";

/**
 * Sơ đồ lộ trình visa ở đầu tab Visa: "từ visa này đi tiếp được tới đâu" —
 * câu người dùng hỏi nhiều nhất mà danh sách bài rời rạc không trả lời được.
 *
 * Mỗi ô là một link tới bài của visa đó. Bước nào chưa có bài thì vẫn hiện
 * mã nhưng không bấm được. Lối chuyển chỉ lấy từ VISA_PATHS (đều đã có trong
 * bài, kèm nguồn) — điều kiện cụ thể nằm trong từng bài, ở đây chỉ là bản đồ.
 */
export function VisaRoadmap() {
  return (
    <section aria-labelledby="visa-roadmap" className="flex flex-col gap-2.5 rounded-xl bg-muted/50 px-4 py-3">
      <div className="flex items-baseline justify-between gap-2">
        <h2 id="visa-roadmap" className="text-sm font-semibold">
          Lộ trình thường gặp
        </h2>
        <span className="text-xs text-muted-foreground">Bấm vào mã để xem điều kiện</span>
      </div>
      <ul className="flex flex-col gap-2">
        {VISA_PATHS.map((path) => (
          <li key={path.title} className="flex items-center gap-2">
            <span className="w-14 shrink-0 text-xs text-muted-foreground">{path.title}</span>
            {/* Cuộn ngang khi dài hơn màn hình (lộ trình du học có 5 bước). */}
            <span className="-mr-4 flex min-w-0 items-center gap-0.5 overflow-x-auto pr-4 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
              {path.steps.map((step, i) => {
                const exists = getArticle(step.sectionId, step.slug) !== undefined;
                const chip = "shrink-0 rounded-md px-1.5 py-1 text-xs font-semibold";
                return (
                  <Fragment key={step.code}>
                    {i > 0 && <ChevronRight className="size-3 shrink-0 text-muted-foreground/60" aria-hidden />}
                    {exists ? (
                      <Link
                        href={articleHref(step.sectionId, step.slug)}
                        className={`${chip} bg-background text-primary shadow-xs transition-colors hover:bg-primary hover:text-primary-foreground`}
                      >
                        {step.code}
                      </Link>
                    ) : (
                      <span className={`${chip} text-muted-foreground`}>{step.code}</span>
                    )}
                  </Fragment>
                );
              })}
            </span>
          </li>
        ))}
      </ul>
      <p className="text-xs text-muted-foreground">
        Mỗi bước có điều kiện riêng (bằng cấp, thời gian ở Hàn, thu nhập) — sơ đồ chỉ cho biết đường đi.
      </p>
    </section>
  );
}
