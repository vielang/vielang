"use client";

import { useDeferredValue, useMemo, useState } from "react";
import Link from "next/link";
import { Search, X } from "lucide-react";
import type { GrammarEntry } from "@/lib/page-grammar";
import {
  MAX_VISIBLE_RESULTS,
  MIN_QUERY_LENGTH,
  findMatch,
  hasEnoughQuery,
  searchGrammar,
} from "@/lib/grammar-search";

/**
 * Tô đậm đúng đoạn đã khớp trong một dòng chữ.
 *
 * Gõ `이` ra hơn chục mục mà nhìn thẻ nào cũng như thẻ nào thì người ta phải
 * tự dò bằng mắt xem vì sao nó lọt vào đây. Tô lên là trả lời luôn câu đó.
 *
 * CHỈ tô ở tiêu đề và phần nghĩa, vì đó mới là hai trường được đem đi so
 * khớp. Tô cả vào câu ví dụ là nói dối người dùng về cách ô này hoạt động.
 */
function Mark({ text, query }: { text: string; query: string }) {
  const at = findMatch(text, query);
  if (!at) return <>{text}</>;

  const chars = [...text];
  return (
    <>
      {chars.slice(0, at[0]).join("")}
      <mark className="rounded-[3px] bg-primary/15 px-px text-inherit">
        {chars.slice(at[0], at[1]).join("")}
      </mark>
      {chars.slice(at[1]).join("")}
    </>
  );
}

/**
 * Ô tra cứu ngữ pháp, đặt ngay đầu trang thư viện.
 *
 * Thay chỗ của tiêu đề "Thư viện": tiêu đề đó chỉ nhắc lại thứ người dùng
 * vừa bấm vào để tới đây, còn ô này làm được việc. Dòng gợi ý bên dưới giữ
 * lại phần thông tin mà tiêu đề từng mang (đang có gì, bao nhiêu).
 *
 * Danh sách CHỈ hiện khi đã gõ đủ (xem `hasEnoughQuery`). Đổ sẵn cả trăm
 * mục ra đây là đẩy tụt lưới sách xuống — mà phần lớn người mở thư viện là
 * để đọc tiếp, không phải để tra ngữ pháp.
 *
 * Lọc ngay trên máy chứ không gọi máy chủ: cả kho chỉ hơn trăm mục, đã nằm
 * sẵn trong bundle. Gọi mạng cho chừng đó dữ liệu vừa chậm hơn vừa hỏng hẳn
 * khi mất mạng — mà mất mạng là lúc người ta đang ngồi trên tàu điện giở
 * sách ra xem.
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
  //
  // Mọi thứ hiển thị đều tính từ `deferred`, kể cả dòng đếm và phần tô đậm:
  // lấy số kết quả của nhịp này ghép với từ khoá của nhịp sau thì ra câu sai
  // lè kiểu "0 kết quả cho 이에요".
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

  // Tên sách lấy từ chính dữ liệu chứ không chép tay vào câu giới thiệu:
  // soạn thêm ngữ pháp cho một quyển nữa mà quên sửa câu này thì người dùng
  // đọc được một câu sai, và không có gì báo lỗi cả.
  const books = useMemo(() => {
    // `entries` đã xếp theo thứ tự sách nên `Set` giữ đúng thứ tự đó, và
    // phần tử đầu với phần tử cuối chính là hai đầu dải. Liệt kê đủ bốn
    // cái tên thì câu dài lê thê mà chẳng nói thêm được gì.
    const names = [...new Set(entries.map((e) => e.bookId))].map(
      (id) => bookTitles[id] ?? id
    );
    return names.length === 1
      ? `trong ${names[0]}`
      : `từ ${names[0]} đến ${names.at(-1)}`;
  }, [entries, bookTitles]);

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <Search
          className="pointer-events-none absolute top-1/2 left-3.5 size-4 -translate-y-1/2 text-muted-foreground/70"
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
          onKeyDown={(e) => {
            // Esc xoá từ khoá, phản xạ quen của mọi ô tìm kiếm. Tự xử thay
            // vì để trình duyệt lo: hành vi mặc định của input[type=search]
            // mỗi nơi một kiểu, và có nơi không đụng tới state của React.
            if (e.key === "Escape" && query !== "") {
              e.preventDefault();
              setQuery("");
            }
          }}
          placeholder="Tra cứu ngữ pháp…"
          aria-label="Tra cứu ngữ pháp"
          // Bộ gõ tiếng Hàn và tiếng Việt đều khổ vì mấy thứ này: tự động
          // viết hoa chữ đầu, tự sửa chính tả, gợi ý từ cũ.
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          enterKeyHint="search"
          // Dùng `focus` chứ không `focus-visible`: bấm chuột vào ô tìm
          // kiếm mà không thấy gì đổi thì không rõ đã gõ được hay chưa.
          // Với thẻ kết quả bên dưới thì ngược lại, `focus-visible` mới
          // đúng — viền chỉ nên hiện khi đi bằng bàn phím.
          //
          // Cao 48px cho vừa đầu ngón tay. Nút xoá của WebKit bị ẩn vì đã có
          // nút xoá riêng — để cả hai thì góc phải có hai dấu X chồng nhau.
          // Viền sáng lúc bấm vào vẽ VÀO TRONG (`inset-ring`): vẽ ra ngoài thì
          // ô rộng hơn mọi thẻ bên dưới vài px, nhìn như lệch khung.
          className="focus:inset-ring-ring/25 h-12 w-full rounded-xl border border-border bg-background pr-11 pl-10 text-[15px] transition-colors outline-none placeholder:text-muted-foreground/70 focus:border-primary/50 focus:inset-ring-2 [&::-webkit-search-cancel-button]:hidden"
        />
        {query !== "" && (
          <button
            type="button"
            onClick={() => setQuery("")}
            aria-label="Xoá từ khoá"
            className="absolute top-1/2 right-1.5 flex size-9 -translate-y-1/2 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <X className="size-4" aria-hidden />
          </button>
        )}
      </div>

      {!ready ? (
        // Gõ được một ký tự mà giao diện không đổi gì thì trông y như hỏng —
        // người dùng không có cách nào đoán ra là cần gõ thêm. Nói thẳng ra.
        // Chỉ chữ Latinh mới rơi vào đây: một chữ Hàn đã đủ để tra.
        <p className="px-1 text-xs text-muted-foreground" aria-live="polite">
          {typed === 0
            ? `${entries.length} điểm ngữ pháp ${books}, kèm nghĩa tiếng Việt và câu ví dụ.`
            : `Gõ thêm ${MIN_QUERY_LENGTH - typed} ký tự nữa để tra cứu.`}
          {typed === 0 && (
            <>
              {" "}
              <Link href="/ngu-phap" className="text-primary underline-offset-2 hover:underline">
                Xem tất cả
              </Link>
            </>
          )}
        </p>
      ) : quiet ? null : results.length === 0 ? (
        <div className="px-4 py-6 text-center" aria-live="polite">
          <p className="text-sm text-muted-foreground">
            Không có điểm ngữ pháp nào khớp “{deferred.trim()}”.
          </p>
          <p className="mt-1 text-xs text-muted-foreground/75">
            Thử tên đuôi câu bằng tiếng Hàn (vd{" "}
            <span className="font-korean">이에요</span>) hoặc nghĩa tiếng Việt
            (vd “điều kiện”).
          </p>
        </div>
      ) : (
        <>
          <p className="px-1 text-xs text-muted-foreground" aria-live="polite">
            {matches.length} kết quả
            {hidden > 0 && ` — hiện ${results.length} mục hợp nhất`}
          </p>

          <ul className="flex flex-col gap-2">
            {results.map((entry) => (
              <li key={`${entry.bookId}:${entry.id}`}>
                {/* Cả thẻ là một liên kết: mục đích ở đây là ĐI TỚI đúng
                    trang trong sách, nên đừng bắt người ta nhắm vào một
                    chữ "xem" nhỏ xíu. */}
                <Link
                  href={`/read/${entry.bookId}/${entry.page}`}
                  className="focus-visible:ring-ring/40 block rounded-xl border border-border/70 px-4 py-3 transition-colors hover:border-primary/40 hover:bg-muted/40 focus-visible:ring-2 focus-visible:outline-none"
                >
                  <div className="flex items-baseline justify-between gap-3">
                    <span className="font-korean font-semibold tracking-tight">
                      <Mark text={entry.title} query={deferred} />
                    </span>
                    <span className="shrink-0 text-[11px] text-muted-foreground/80 tabular-nums">
                      {bookTitles[entry.bookId] ?? entry.bookId} · tr.{" "}
                      {entry.page}
                    </span>
                  </div>

                  <p className="mt-1 text-sm leading-relaxed">
                    <Mark text={entry.vi} query={deferred} />
                  </p>

                  {/* Ví dụ lùi vào sau một vạch mờ: lướt danh sách thì mắt
                      phải bắt được tiêu đề và nghĩa trước, ví dụ chỉ để xác
                      nhận mình chọn đúng mục. */}
                  <div className="mt-2.5 border-l-2 border-border/80 pl-3">
                    <p className="font-korean text-[13px] text-muted-foreground">
                      {entry.exKo}
                    </p>
                    <p className="mt-0.5 text-xs text-muted-foreground/75">
                      {entry.exVi}
                    </p>
                  </div>
                </Link>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}
