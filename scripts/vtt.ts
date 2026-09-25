/**
 * Đọc phụ đề tự động của YouTube (`--write-auto-subs`) dạng "roll-up": mỗi
 * cụm hiện lại câu TRƯỚC rồi ghép thêm từng chữ mới qua thẻ
 * `<00:00:05.279><c> ...</c>`, tạo hiệu ứng gõ dần. Parse thẳng ra danh sách
 * cue kiểu vậy sẽ trùng lặp gần như mọi dòng — cần gộp lại thành 1 câu/cue.
 *
 * Cách gộp: mỗi cụm chỉ lấy DÒNG CUỐI (câu đang được gõ dần). Cụm sau mà
 * dòng cuối vẫn bắt đầu bằng câu đang gộp -> vẫn cùng một câu, cập nhật bản
 * dài hơn. Khác hẳn -> câu mới, chốt câu cũ lại (bắt đầu = lúc câu cũ xuất
 * hiện lần đầu, kết thúc = lúc cụm này bắt đầu).
 */
export interface Cue {
  s: number;
  e: number;
  t: string;
}

function toSeconds(ts: string): number {
  const m = /^(\d+):(\d{2}):(\d{2})\.(\d{3})$/.exec(ts.trim());
  if (!m) return 0;
  const [, h, min, sec, ms] = m;
  return Number(h) * 3600 + Number(min) * 60 + Number(sec) + Number(ms) / 1000;
}

function stripTags(line: string): string {
  return line.replace(/<[^>]*>/g, "").trim();
}

interface RawCue {
  start: number;
  end: number;
  lines: string[];
}

function parseRawCues(vtt: string): RawCue[] {
  const blocks = vtt.replace(/\r\n/g, "\n").split(/\n\n+/);
  const cues: RawCue[] = [];
  const timeRe = /^(\d+:\d{2}:\d{2}\.\d{3})\s*-->\s*(\d+:\d{2}:\d{2}\.\d{3})/;
  for (const block of blocks) {
    const lines = block.split("\n");
    const timeLine = lines.find((l) => timeRe.test(l));
    if (!timeLine) continue;
    const m = timeRe.exec(timeLine)!;
    const start = toSeconds(m[1]);
    const end = toSeconds(m[2]);
    const textLines = lines
      .slice(lines.indexOf(timeLine) + 1)
      .map(stripTags)
      .filter(Boolean);
    cues.push({ start, end, lines: textLines });
  }
  return cues;
}

export function parseVtt(vtt: string): Cue[] {
  const rawCues = parseRawCues(vtt);
  const out: Cue[] = [];
  let curStart = 0;
  let curText = "";
  let lastEnd = 0;

  for (const cue of rawCues) {
    lastEnd = cue.end;
    const candidate = cue.lines[cue.lines.length - 1] ?? "";
    if (!candidate) continue;
    if (curText && candidate.startsWith(curText)) {
      curText = candidate;
      continue;
    }
    if (curText && curText.startsWith(candidate)) {
      continue;
    }
    if (curText) out.push({ s: curStart, e: cue.start, t: curText });
    curStart = cue.start;
    curText = candidate;
  }
  if (curText) out.push({ s: curStart, e: lastEnd, t: curText });
  return out;
}
