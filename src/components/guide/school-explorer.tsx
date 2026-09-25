"use client";

import { useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  schoolRegion,
  schoolTopik,
  schoolTuitionFrom,
  type GuideArticle,
  type SchoolRegion,
} from "@/lib/guide";
import { GuideArticleList } from "@/components/guide/guide-article-list";

type TopikFilter = "all" | "0" | "2" | "3" | "4";
type Sort = "name" | "tuition";

const REGIONS: SchoolRegion[] = ["Seoul", "Gyeonggi – Incheon", "Tỉnh khác"];

const SELECT =
  "h-9 w-full rounded-lg bg-muted/60 px-2.5 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring";

/**
 * Danh sách trường kèm bộ lọc: vùng, loại trường, mức TOPIK tối thiểu; sắp
 * theo tên hoặc học phí thấp nhất. Mọi tiêu chí suy từ thông số sẵn có trong
 * phần khai báo của bài — thêm trường mới là tự có mặt trong bộ lọc.
 *
 * Dùng `<select>` gốc chứ không phải hàng nút: trên điện thoại nó mở bảng
 * chọn của hệ điều hành, gọn hơn và quen tay hơn ba hàng nút xếp chồng.
 *
 * Lọc TOPIK theo mức CHUNG của trường ("Cấp 3 (ngành nghệ thuật: cấp 2)" ->
 * 3). Trường không ghi mức TOPIK thì vẫn hiện — chưa rõ không có nghĩa là
 * không đạt.
 */
export function SchoolExplorer({ articles }: { articles: GuideArticle[] }) {
  const [region, setRegion] = useState<"all" | SchoolRegion>("all");
  const [kind, setKind] = useState("all");
  const [topik, setTopik] = useState<TopikFilter>("all");
  const [sort, setSort] = useState<Sort>("name");

  const kinds = useMemo(
    () => [...new Set(articles.map((a) => a.facts.kind).filter(Boolean))].sort(),
    [articles]
  );

  const shown = useMemo(() => {
    const list = articles.filter((a) => {
      if (region !== "all" && schoolRegion(a.facts.city) !== region) return false;
      if (kind !== "all" && a.facts.kind !== kind) return false;
      if (topik !== "all") {
        const need = schoolTopik(a.facts.topik);
        if (need !== null && need > Number(topik)) return false;
      }
      return true;
    });
    if (sort === "tuition") {
      // Trường chưa rõ học phí xếp cuối.
      list.sort(
        (a, b) =>
          (schoolTuitionFrom(a.facts.tuition) ?? Infinity) - (schoolTuitionFrom(b.facts.tuition) ?? Infinity)
      );
    }
    return list;
  }, [articles, region, kind, topik, sort]);

  const filtered = region !== "all" || kind !== "all" || topik !== "all";

  function reset() {
    setRegion("all");
    setKind("all");
    setTopik("all");
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          Vùng
          <select className={SELECT} value={region} onChange={(e) => setRegion(e.target.value as typeof region)}>
            <option value="all">Tất cả</option>
            {REGIONS.map((r) => (
              <option key={r} value={r}>
                {r}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          TOPIK hiện có
          <select className={SELECT} value={topik} onChange={(e) => setTopik(e.target.value as TopikFilter)}>
            <option value="all">Không lọc</option>
            <option value="0">Chưa có TOPIK</option>
            <option value="2">Cấp 2</option>
            <option value="3">Cấp 3</option>
            <option value="4">Cấp 4</option>
          </select>
        </label>
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          Loại trường
          <select className={SELECT} value={kind} onChange={(e) => setKind(e.target.value)}>
            <option value="all">Tất cả</option>
            {kinds.map((k) => (
              <option key={k} value={k}>
                {k}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          Sắp xếp
          <select className={SELECT} value={sort} onChange={(e) => setSort(e.target.value as Sort)}>
            <option value="name">Theo tên</option>
            <option value="tuition">Học phí thấp trước</option>
          </select>
        </label>
      </div>

      <p className="flex items-center justify-between gap-2 text-xs text-muted-foreground tabular-nums">
        <span>
          {filtered ? `${shown.length}/${articles.length} trường khớp bộ lọc` : `${articles.length} trường`}
        </span>
        {filtered && (
          <Button variant="ghost" size="sm" className="h-7 px-2 text-xs" onClick={reset}>
            Bỏ lọc
          </Button>
        )}
      </p>

      {shown.length > 0 ? (
        <GuideArticleList sectionId="truong" articles={shown} />
      ) : (
        <p className="py-12 text-center text-sm text-muted-foreground">
          Không có trường nào khớp bộ lọc.{" "}
          <button type="button" onClick={reset} className="text-primary underline-offset-4 hover:underline">
            Bỏ lọc
          </button>
        </p>
      )}
    </div>
  );
}
