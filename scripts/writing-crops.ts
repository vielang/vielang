/**
 * Cắt ảnh phần viết TOPIK II theo TỪNG CÂU (dùng trong `import-topik`).
 *
 * Bộ dữ liệu gốc chỉ có ảnh cả trang: trang đề 1 chứa câu 51–52, trang 2
 * chứa 53–54; trang đáp án là một bảng 51–54 (có kỳ tràn sang trang thứ
 * hai). Hiện cả trang thì người học phải tự tìm câu của mình — nên cắt ra:
 *
 * - Trang đề: chia theo các dải trắng ngang. Trang 1 = [tiêu đề] [chỉ dẫn +
 *   câu 51 …] [khung câu 52] [số trang]. Trang 2 = [chạy đầu trang] [câu 53
 *   …] [câu 54 …] ["* 원고지 쓰기의 예"] [lưới ô] [dòng hết giờ] [số trang];
 *   câu 54 bắt đầu ở dòng chữ sát lề trái thứ hai.
 * - Trang đáp án: tách theo đường kẻ ngang chạy suốt bề rộng bảng; bỏ dòng
 *   tiêu đề bảng (dòng đầu ở trang đầu), các dòng còn lại lần lượt là 51, 52,
 *   53, 54.
 *
 * Bố cục lệch khỏi quy luật trên thì ném lỗi — thà dừng nhập còn hơn cắt sai.
 */
import sharp from "sharp";

interface Block {
  y0: number;
  y1: number;
  xmin: number;
  xmax: number;
}
export interface Crop {
  top: number;
  bottom: number;
  left: number;
  right: number;
}

const INK = 180;

async function grey(file: string) {
  const { data, info } = await sharp(file).greyscale().raw().toBuffer({ resolveWithObject: true });
  return { data, W: info.width, H: info.height };
}

/** Các khối nội dung cách nhau bởi ≥ `gap` hàng trắng. */
function blocksOf(data: Buffer, W: number, H: number, gap = 10): Block[] {
  const out: Block[] = [];
  let cur: Block | null = null;
  let blank = 0;
  for (let y = 0; y < H; y++) {
    let xmin = W;
    let xmax = -1;
    for (let x = 0; x < W; x++) {
      if (data[y * W + x] < INK) {
        if (x < xmin) xmin = x;
        xmax = x;
      }
    }
    if (xmax >= 0) {
      if (!cur) cur = { y0: y, y1: y, xmin, xmax };
      cur.y1 = y;
      cur.xmin = Math.min(cur.xmin, xmin);
      cur.xmax = Math.max(cur.xmax, xmax);
      blank = 0;
    } else if (cur && ++blank >= gap) {
      out.push(cur);
      cur = null;
    }
  }
  if (cur) out.push(cur);
  return out;
}

function span(blocks: Block[], pad = 14, W = Infinity, H = Infinity): Crop {
  return {
    top: Math.max(0, Math.min(...blocks.map((b) => b.y0)) - pad),
    bottom: Math.min(H, Math.max(...blocks.map((b) => b.y1)) + pad),
    left: Math.max(0, Math.min(...blocks.map((b) => b.xmin)) - pad),
    right: Math.min(W, Math.max(...blocks.map((b) => b.xmax)) + pad),
  };
}

const isPageNumber = (b: Block, W: number) => b.y1 - b.y0 < 26 && b.xmin > W * 0.4 && b.xmax < W * 0.6;

/** Trang đề 1 → vùng câu 51 (kèm lời chỉ dẫn [51~52]) và câu 52. */
export async function cropPage1(file: string): Promise<{ 51: Crop; 52: Crop }> {
  const { data, W, H } = await grey(file);
  let b = blocksOf(data, W, H);
  if (isPageNumber(b[b.length - 1], W)) b = b.slice(0, -1);
  const [title, ...rest] = b;
  if (title.y1 - title.y0 < 70 || title.xmin < W * 0.15) throw new Error(`${file}: không thấy khung tiêu đề`);
  const q52 = rest[rest.length - 1];
  const q51 = rest.slice(0, -1);
  if (q52.y1 - q52.y0 < 100 || q51.length < 2) throw new Error(`${file}: bố cục trang 1 lạ`);
  return { 51: span(q51, 14, W, H), 52: span([q52], 14, W, H) };
}

/** Trang đề 2 → vùng câu 53 và câu 54 (bỏ mẫu 원고지 và dòng hết giờ). */
export async function cropPage2(file: string): Promise<{ 53: Crop; 54: Crop }> {
  const { data, W, H } = await grey(file);
  let b = blocksOf(data, W, H);
  if (isPageNumber(b[b.length - 1], W)) b = b.slice(0, -1);
  // Chạy đầu trang ("제102회 한국어능력시험 …", lệch phải) + đường kẻ dưới nó.
  while (b.length && (b[0].xmin > W * 0.4 || b[0].y1 - b[0].y0 < 5)) b = b.slice(1);
  // Từ dưới lên: dòng "제1교시 … 끝났습니다", lưới ô mẫu, dòng "* 원고지 쓰기의 예".
  const [label, grid, footer] = b.slice(-3);
  if (footer.y1 - footer.y0 < 40 || grid.y1 - grid.y0 < 60 || label.y1 - label.y0 > 30) {
    throw new Error(`${file}: không thấy phần 원고지 / dòng hết giờ ở cuối trang`);
  }
  const body = b.slice(0, -3);
  const margin = body[0].xmin;
  const starts = body.filter((x) => x.xmin <= margin + 8 && x.y1 - x.y0 < 30);
  if (starts.length < 2) throw new Error(`${file}: không tách được câu 53 / 54`);
  const i54 = body.indexOf(starts[1]);
  return { 53: span(body.slice(0, i54), 14, W, H), 54: span(body.slice(i54), 14, W, H) };
}

/**
 * Các trang đáp án phần viết → vùng từng dòng 51–54: `{ page, crop }` với
 * `page` là chỉ số trong danh sách trang truyền vào.
 */
export async function cropAnswerKey(files: string[]): Promise<Record<51 | 52 | 53 | 54, { page: number; crop: Crop }>> {
  const rows: { page: number; crop: Crop }[] = [];
  for (let p = 0; p < files.length; p++) {
    const { data, W, H } = await grey(files[p]);
    // Bề rộng bảng: hàng có nhiều mực nhất (đường kẻ ngang dài nhất).
    let best = 0;
    const count = (y: number) => {
      let n = 0;
      for (let x = 0; x < W; x++) if (data[y * W + x] < INK) n++;
      return n;
    };
    const counts = Array.from({ length: H }, (_, y) => count(y));
    for (const c of counts) best = Math.max(best, c);
    const rules: number[] = [];
    for (let y = 0; y < H; y++) {
      if (counts[y] >= best * 0.85 && (rules.length === 0 || y - rules[rules.length - 1] > 4)) rules.push(y);
    }
    // Khung bảng: mép trái/phải của đường kẻ dài nhất.
    const yLong = counts.indexOf(best);
    let left = 0;
    let right = W - 1;
    while (left < W && data[yLong * W + left] >= INK) left++;
    while (right > 0 && data[yLong * W + right] >= INK) right--;
    let header = p === 0; // bảng bắt đầu ở trang đầu bằng dòng tiêu đề (번호 / 모범답안 / 배점)
    for (let i = 0; i + 1 < rules.length; i++) {
      const top = rules[i];
      const bottom = rules[i + 1];
      if (bottom - top < 20) continue; // khe giữa đường kẻ đôi
      if (header) {
        header = false;
        continue;
      }
      rows.push({ page: p, crop: { top: top - 2, bottom: bottom + 3, left: left - 2, right: right + 3 } });
    }
  }
  if (rows.length !== 4) throw new Error(`${files[0]}: tìm được ${rows.length} dòng đáp án viết (cần 4)`);
  return { 51: rows[0], 52: rows[1], 53: rows[2], 54: rows[3] };
}

/** Cắt `crop` từ `file`, lề trắng quanh, ghi WebP. */
export async function writeCrop(file: string, crop: Crop, out: string, pad = 16) {
  await sharp(file)
    .extract({ left: crop.left, top: crop.top, width: crop.right - crop.left, height: crop.bottom - crop.top })
    .extend({ top: pad, bottom: pad, left: pad, right: pad, background: "#ffffff" })
    .flatten({ background: "#ffffff" })
    .webp({ quality: 85 })
    .toFile(out);
}
