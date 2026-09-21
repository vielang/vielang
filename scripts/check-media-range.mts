/**
 * Kiểm máy chủ có trả ĐÚNG khoảng byte được xin hay không.
 *
 * Vì sao cần một phép kiểm riêng: thẻ <audio> tải file theo từng khoảng
 * (`Range`). Nếu máy chủ trả 206 nhưng là khoảng KHÁC khoảng được xin, trình
 * duyệt xin đoạn tiếp theo, nhận lại đoạn nó đã có, rồi xin lại — quay vòng
 * mãi. Người dùng thấy đúng một cái vòng xoay không bao giờ dứt, và KHÔNG có
 * lỗi nào bắn ra: mã trả về vẫn là 206, vẫn "thành công".
 *
 * Đúng lỗi đó đã xảy ra thật: cache biên của Vercel giữ lại bản trả lời của
 * lần xin `bytes=0-1` đầu tiên rồi đem 2 byte ấy trả cho mọi khoảng khác.
 * Không test nào trong máy bắt được, vì nó chỉ xuất hiện khi đã có một tầng
 * cache thật đứng trước.
 *
 * Chạy sau mỗi lần deploy:
 *   npx tsx scripts/check-media-range.mts https://www.vietopik.com
 */
import { readFileSync } from "node:fs";
import { BOOKS } from "../src/lib/books";
import { getPageAudio } from "../src/lib/audio";

/**
 * Nạp `.env.local` bằng tay: script này chạy ngoài Next nên không được
 * Next nạp hộ, mà thiếu biến gốc R2 thì URL audio dựng ra sẽ cụt và cả
 * phép kiểm báo 404 hàng loạt — sai lè nhưng trông rất giống lỗi thật.
 */
if (!process.env.NEXT_PUBLIC_IMAGE_BASE_URL) {
  try {
    const text = readFileSync(".env.local", "utf8");
    for (const raw of text.split(String.fromCharCode(10))) {
      const line = raw.trim();
      const at = line.indexOf("=");
      if (at > 0 && !line.startsWith("#")) {
        process.env[line.slice(0, at).trim()] ??= line.slice(at + 1).trim();
      }
    }
  } catch {
    /* không có file thì thôi, kiểm dưới sẽ báo */
  }
}

const base = process.argv[2]?.replace(/\/$/, "") ?? "";

/**
 * URL audio nay là địa chỉ R2 tuyệt đối (xem `lib/audio.ts`), nên `base`
 * chỉ còn dùng cho những đường vẫn tương đối — ví dụ muốn kiểm luôn một
 * bản deploy thay vì kiểm thẳng R2.
 */
const resolve = (u: string) => (u.startsWith("http") ? u : base + u);
// Không bắt buộc truyền địa chỉ: URL audio nay đã là tuyệt đối. Truyền vào
// khi muốn kiểm những đường còn tương đối của một bản deploy cụ thể.

/** Vài URL audio có thật, trải đều các sách. */
function sampleUrls(perBook = 2): string[] {
  const out: string[] = [];
  for (const book of BOOKS) {
    const seen = new Set<string>();
    for (let p = 1; p <= book.totalPages && seen.size < perBook; p++) {
      for (const t of getPageAudio(book.id, p)) seen.add(t.url);
    }
    out.push(...[...seen].slice(0, perBook));
  }
  return out;
}

interface Problem {
  url: string;
  asked: string;
  got: string;
  note: string;
}

async function checkOne(url: string): Promise<Problem[]> {
  const problems: Problem[] = [];

  // Bước 1: hỏi cỡ file bằng đúng cú thăm dò mà trình duyệt gửi đầu tiên.
  const probe = await fetch(resolve(url), { headers: { Range: "bytes=0-1" } });
  const total = Number(/\/(\d+)$/.exec(probe.headers.get("content-range") ?? "")?.[1]);
  if (!total) {
    problems.push({ url, asked: "bytes=0-1", got: probe.headers.get("content-range") ?? `HTTP ${probe.status}`, note: "không đọc được cỡ file" });
    return problems;
  }

  // Bước 2: xin những khoảng KHÁC nhau. Đây là chỗ cache hỏng lộ mặt — nếu
  // mọi khoảng đều trả về cùng một đoạn thì tức là đang phát lại bản cache.
  const mid = Math.floor(total / 2);
  const cases = [
    { asked: `bytes=2-`, wantStart: 2, wantEnd: total - 1 },
    { asked: `bytes=${mid}-${mid + 99}`, wantStart: mid, wantEnd: mid + 99 },
    { asked: `bytes=${total - 10}-`, wantStart: total - 10, wantEnd: total - 1 },
  ];

  for (const c of cases) {
    const res = await fetch(resolve(url), { headers: { Range: c.asked } });
    const got = res.headers.get("content-range") ?? "";
    const want = `bytes ${c.wantStart}-${c.wantEnd}/${total}`;
    if (res.status !== 206) {
      problems.push({ url, asked: c.asked, got: `HTTP ${res.status}`, note: "không phải 206" });
    } else if (got !== want) {
      problems.push({ url, asked: c.asked, got, note: `đáng lẽ phải là ${want}` });
    }
  }
  return problems;
}

const urls = sampleUrls();
console.log(`Kiểm ${urls.length} file audio trên ${base}\n`);

let bad = 0;
for (const url of urls) {
  const problems = await checkOne(url);
  if (problems.length === 0) {
    console.log(`  ok   ${url}`);
    continue;
  }
  bad++;
  console.log(`  LỖI  ${url}`);
  for (const p of problems) console.log(`         xin ${p.asked} -> nhận ${p.got}  (${p.note})`);
}

console.log();
if (bad > 0) {
  console.log(`${bad}/${urls.length} file trả sai khoảng byte — thẻ <audio> sẽ quay vòng mãi.`);
  process.exit(1);
}
console.log(`Cả ${urls.length} file đều trả đúng khoảng được xin.`);
