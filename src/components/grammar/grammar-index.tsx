"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { Search, X } from "lucide-react";
import type { GrammarEntry } from "@/lib/page-grammar";
import { hasEnoughQuery, searchGrammar } from "@/lib/grammar-search";

/**
 * Ô tra cứu ngữ pháp, đặt ngay đầu trang thư viện.
 *
 * Thay chỗ của tiêu đề "Thư viện": tiêu đề đó chỉ nhắc lại thứ người dùng
 * vừa bấm vào để tới đây, còn ô này làm được việc. Dòng gợi ý bên dưới giữ
 * lại phần thông tin mà tiêu đề từng mang (đang có gì, bao nhiêu).
 *
 * Danh sách CHỈ hiện khi đã gõ đủ (xem `MIN_QUERY_LENGTH`). Đổ sẵn cả bảy
 * chục mục ra đây là đẩy tụt lưới sách xuống — mà phần lớn người mở thư
 * viện là để đọc tiếp, không phải để tra ngữ pháp.
 *
 * Lọc ngay trên máy chứ không gọi máy chủ: cả kho chỉ hơn bảy chục mục, đã
 * nằm sẵn trong bundle. Gọi mạng cho chừng đó dữ liệu vừa chậm hơn vừa hỏng
 * hẳn khi mất mạng — mà mất mạng là lúc người ta đang ngồi trên tàu điện
 * giở sách ra xem.
 */
export function GrammarIndex({
  entries,
  bookTitles,
}: {
  entries: GrammarEntry[];
  /** bookId -> tên hiển thị, dựng sẵn ở phía máy chủ. */
  bookTitles: Record<string, string>;
}) {
  const [query, setQuery] = useState("");
  const ready = hasEnoughQuery(query);
  const results = useMemo(
    () => (ready ? searchGrammar(entries, query) : []),
    [entries, query, ready]
  );

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <Search
          className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground"
          aria-hidden
        />
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Tra cứu ngữ pháp — tiếng Hàn hoặc tiếng Việt…"
          aria-label="Tra cứu ngữ pháp"
          className="focus-visible:ring-ring w-full rounded-lg border border-border bg-background py-2.5 pr-9 pl-9 text-sm focus-visible:ring-2 focus-visible:outline-none"
        />
        {query !== "" && (
          <button
            type="button"
            onClick={() => setQuery("")}
            aria-label="Xoá từ khoá"
            className="absolute top-1/2 right-2 -translate-y-1/2 rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            <X className="size-4" aria-hidden />
          </button>
        )}
      </div>

      {!ready ? (
        <p className="text-sm text-muted-foreground">
          {entries.length} điểm ngữ pháp trong Sơ cấp 1 và Sơ cấp 2, kèm nghĩa
          tiếng Việt và câu ví dụ.
        </p>
      ) : (
        <>
          <p className="text-sm text-muted-foreground" aria-live="polite">
            {results.length} kết quả cho “{query}”
          </p>

          {results.length === 0 ? (
            <p className="rounded-lg border border-dashed border-border px-4 py-6 text-center text-sm text-muted-foreground">
              Không tìm thấy điểm ngữ pháp nào. Thử gõ tiếng Hàn (vd{" "}
              <span className="font-korean">이에요</span>) hoặc tiếng Việt (vd
              “điều kiện”).
            </p>
          ) : (
            <ul className="flex flex-col gap-2">
              {results.map((entry) => (
                <li key={`${entry.bookId}:${entry.id}`}>
                  {/* Cả thẻ là một liên kết: mục đích ở đây là ĐI TỚI đúng
                      trang trong sách, nên đừng bắt người ta nhắm vào một
                      chữ "xem" nhỏ xíu. */}
                  <Link
                    href={`/read/${entry.bookId}/${entry.page}`}
                    className="focus-visible:ring-ring block rounded-lg border border-border px-4 py-3 transition-colors hover:border-primary/60 hover:bg-muted/50 focus-visible:ring-2 focus-visible:outline-none"
                  >
                    <div className="flex items-baseline justify-between gap-3">
                      <span className="font-korean font-semibold tracking-tight">
                        {entry.title}
                      </span>
                      <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
                        {bookTitles[entry.bookId] ?? entry.bookId} · tr.{" "}
                        {entry.page}
                      </span>
                    </div>

                    <p className="mt-1 text-sm">{entry.vi}</p>

                    {/* Ví dụ mờ hơn một bậc: lướt danh sách thì mắt phải bắt
                        được tiêu đề và nghĩa trước, ví dụ chỉ để xác nhận. */}
                    <p className="font-korean mt-2 text-sm text-muted-foreground">
                      {entry.exKo}
                    </p>
                    <p className="text-xs text-muted-foreground/80">
                      {entry.exVi}
                    </p>
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  );
}
