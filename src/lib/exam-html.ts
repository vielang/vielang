/**
 * Làm sạch HTML của đề TOPIK (lấy từ dịch vụ "토픽 기출문제 풀어보기" của
 * topik.go.kr) trước khi đưa vào app — chạy LÚC NHẬP ĐỀ (scripts/import-topik),
 * không phải lúc hiển thị.
 *
 * Chỉ cho qua đúng những gì đề thật dùng: <br>, <b>, <u>, khung viền quanh
 * đoạn văn (<div class="question_outline"> → <div class="exam-box">) và ảnh
 * nằm trong đoạn văn (đường dẫn được đổi sang tài nguyên của app). Mọi thẻ
 * khác bị bỏ, chỉ giữ chữ; thuộc tính (style, onclick…) không bao giờ qua.
 *
 * Bẫy cần nhớ: chỉ dẫn in "<보기>" — chữ Hàn trong ngoặc nhọn, KHÔNG phải thẻ.
 * Nên chỉ coi là thẻ khi sau "<" là tên chữ Latin; mọi "<" còn lại đổi thành
 * "&lt;" để trình duyệt không hiểu nhầm.
 */

const ALLOWED = new Set(["br", "b", "u", "div", "img"]);

export function sanitizeExamHtml(html: string, assetBase: string): string {
  const src = html
    // Trang gốc hay viết "&nbsp" thiếu dấu chấm phẩy.
    .replace(/&nbsp;?/g, " ");
  let out = "";
  let i = 0;
  while (i < src.length) {
    const ch = src[i];
    if (ch !== "<") {
      out += ch === ">" ? "&gt;" : ch;
      i++;
      continue;
    }
    const m = /^<(\/?)([a-zA-Z][a-zA-Z0-9]*)\b([^>]*)>/.exec(src.slice(i));
    if (!m) {
      out += "&lt;";
      i++;
      continue;
    }
    const [whole, closing, rawName, attrs] = m;
    const name = rawName.toLowerCase();
    i += whole.length;
    if (!ALLOWED.has(name)) continue;
    if (name === "br") {
      out += "<br>";
    } else if (name === "img") {
      const s = /src=['"]([^'"]+)['"]/.exec(attrs)?.[1] ?? "";
      const alt = /alt=['"]([^'"]*)['"]/.exec(attrs)?.[1] ?? "";
      const file = s.split("/").pop() ?? "";
      if (file) out += `<img src="${assetBase}/images/${webpName(file)}" alt="${escapeAttr(alt)}" loading="lazy">`;
    } else if (name === "div") {
      out += closing ? "</div>" : `<div class="exam-box">`;
    } else {
      out += closing ? `</${name}>` : `<${name}>`;
    }
  }
  return out.trim();
}

/** "102_1_15_1.png" → "102_1_15_1.webp" — ảnh được đổi sang WebP lúc nhập. */
export function webpName(file: string): string {
  return file.replace(/\.(png|jpe?g|gif)$/i, ".webp");
}

function escapeAttr(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}

/** Mọi thẻ còn lại trong HTML đã làm sạch — test dùng để chắc không lọt thẻ lạ. */
export function tagsIn(html: string): string[] {
  return [...html.matchAll(/<\/?([a-zA-Z][a-zA-Z0-9]*)\b/g)].map((m) => m[1].toLowerCase());
}
