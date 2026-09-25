import type { Metadata } from "next";
import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { GUIDE_SECTIONS, sectionArticles } from "@/lib/guide";
import { GuideArticleList } from "@/components/guide/guide-article-list";

export const metadata: Metadata = {
  title: "Cẩm nang",
  description: "Visa, trường đại học và việc làm cho người Việt ở Hàn Quốc.",
};

/** Mỗi mục bày sẵn vài bài đầu — đủ để thấy mục đó có gì mà trang không dài lê thê. */
const PREVIEW = 3;

export default function GuidePage() {
  return (
    <div className="flex flex-col gap-10">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Cẩm nang sống ở Hàn</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Visa, chọn trường và đi làm — tổng hợp từ nguồn chính thức, mỗi bài ghi rõ ngày cập nhật
          và nguồn để bạn đối chiếu.
        </p>
      </div>

      {GUIDE_SECTIONS.map((section) => {
        const articles = sectionArticles(section.id);
        if (articles.length === 0) return null;
        return (
          <section key={section.id} className="flex flex-col gap-3">
            <div className="flex items-end justify-between gap-4">
              <div>
                <h2 className="text-lg font-semibold">{section.title}</h2>
                <p className="text-sm text-muted-foreground">{section.description}</p>
              </div>
              {articles.length > PREVIEW && (
                <Link
                  href={`/cam-nang/${section.id}`}
                  className="inline-flex shrink-0 items-center gap-1 text-sm text-primary hover:underline"
                >
                  Tất cả {articles.length}
                  <ArrowRight className="size-3.5" aria-hidden />
                </Link>
              )}
            </div>
            <GuideArticleList sectionId={section.id} articles={articles.slice(0, PREVIEW)} />
          </section>
        );
      })}
    </div>
  );
}
