"use client";

import { useDeferredValue, useMemo, useState } from "react";
import Link from "next/link";
import { Search, X } from "lucide-react";
import type { GrammarEntry } from "@/lib/page-grammar";
import {
  MAX_VISIBLE_RESULTS,
  MIN_QUERY_LENGTH,
  hasEnoughQuery,
  searchGrammar,
} from "@/lib/grammar-search";

/**
 * Ô tra cứu ngữ pháp, đặt ngay đầu trang thư viện.
 *
 * Thay chỗ của tiêu đề "Thư viện": tiêu đề đó chỉ nhắc lại thứ người dùng
 * vừa bấm vào để tới đây, còn ô này làm được việc. Dòng gợi ý bên dưới giữ
 * lại phần thông tin mà tiêu đề từng mang (đang có gì, bao nhiêu).
 *
 * Danh sách CHỈ hiện khi đã gõ đủ (xem `hasEnoughQuery`). Đổ sẵn cả bảy chục
 * mục ra đây là đẩy tụt lưới sách xuống — mà phần lớn người mở thư viện là
 * để đọc tiếp, không phải để tra ngữ pháp.
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

  // Bàn phím tiếng Hàn (và gõ Telex tiếng Việt) ghép chữ dần dần: gõ "이에"
  // thì ô lần lượt mang "ㅇ" → "이" → "이ㅇ" → "이에".
  //
  // Vẫn LỌC trên mấy bước dở dang đó, chỉ không kêu "không tìm thấy". Bản
  // trước tạm ngưng lọc cho tới `compositionend` và đó là một canh bạc:
  // nhiều bàn phím Hàn trên Android giữ nguyên một mạch ghép chữ cho tới khi
  // gõ dấu cách, nên ô sẽ đứng im suốt cả từ — đúng cái lỗi người dùng đã
  // báo. Ngưng lọc cũng chẳng bảo vệ được bộ gõ: ô là input có kiểm soát,
  // mỗi phím vẫn dựng lại component y như cũ dù có lọc hay không.
  const [composing, setComposing] = useState(false);

  // Lọc chạy sau một nhịp so với việc gõ: gõ luôn mượt, danh sách theo sau.
  // Với 72 mục thì gần như cùng nhịp, nhưng đây là thứ giữ cho ô không khựng
  // khi kho ngữ pháp lớn dần.
  //
  // Mọi thứ hiển thị đều tính từ `deferred`, kể cả dòng đếm: lấy số kết quả
  // của nhịp này mà ghép với từ khoá của nhịp sau thì ra câu sai lè kiểu
  // "0 kết quả cho 이에요".
  // Tên sách lấy từ chính dữ liệu chứ không chép tay vào câu giới thiệu:
  // soạn thêm ngữ pháp cho một quyển nữa mà quên sửa câu này thì người
  // dùng đọc được một câu sai, và không có gì báo lỗi cả.
  const books = useMemo(
    () =>
      [...new Set(entries.map((e) => e.bookId))]
        .map((id) => bookTitles[id] ?? id)
        .join(", "),
    [entries, bookTitles]
  );

  const deferred = useDeferredValue(query);
  const ready = hasEnoughQuery(deferred);
  const typed = [...deferred.trim()].length;

  const matches = useMemo(
    () => (hasEnoughQuery(deferred) ? searchGrammar(entries, deferred) : []),
    [entries, deferred]
  );
  const results = matches.slice(0, MAX_VISIBLE_RESULTS);
  const hidden = matches.length - results.length;

  // Đang ghép chữ mà chưa khớp gì thì im lặng: "이ㅇ" là nửa chừng của "이에",
  // báo không tìm thấy ngay lúc đó chỉ làm nhấp nháy giữa lúc người ta gõ.
  const quiet = composing && matches.length === 0;

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
          onCompositionStart={() => setComposing(true)}
          onCompositionEnd={(e) => {
            setComposing(false);
            // Lấy giá trị từ chính sự kiện: `onChange` của nhịp cuối có thể
            // chạy SAU sự kiện này tuỳ trình duyệt, nên đọc ở đây mới chắc
            // chữ vừa ghép xong được đem đi lọc.
            setQuery(e.currentTarget.value);
          }}
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
        // Gõ được một ký tự mà giao diện không đổi gì thì trông y như hỏng —
        // người dùng không có cách nào đoán ra là cần gõ thêm. Nói thẳng ra.
        // Chỉ chữ Latinh mới rơi vào đây: một chữ Hàn đã đủ để tra.
        <p className="text-sm text-muted-foreground" aria-live="polite">
          {typed === 0
            ? `${entries.length} điểm ngữ pháp trong ${books}, kèm nghĩa tiếng Việt và câu ví dụ.`
            : `Gõ thêm ${MIN_QUERY_LENGTH - typed} ký tự nữa để tra cứu.`}
        </p>
      ) : quiet ? null : (
        <>
          <p className="text-sm text-muted-foreground" aria-live="polite">
            {matches.length} kết quả cho “{deferred.trim()}”
            {hidden > 0 && ` — hiện ${results.length} mục đầu, gõ thêm cho hẹp lại`}
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
