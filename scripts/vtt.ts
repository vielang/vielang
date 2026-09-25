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
 *
 * Đúng thẻ `<c>` đó cũng là mốc thời gian TỪNG CHỮ trong câu — giữ lại luôn
 * (field `words`) để overlay video tô sáng dần từng chữ đã đọc tới (karaoke),
 * thay vì chỉ hiện nguyên câu tĩnh.
 */
export interface WordSpan {
  t: string;
  at: number;
}

export interface Cue {
  s: number;
  e: number;
  t: string;
  words: WordSpan[];
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

/** Tách 1 dòng đã gõ dần (còn nguyên thẻ `<c>`) thành từng chữ kèm mốc bắt đầu. */
function tokenizeWords(rawLine: string, cueStart: number): WordSpan[] {
  const parts = rawLine.split(/<(\d+:\d{2}:\d{2}\.\d{3})>/);
  const words: WordSpan[] = [];
  let at = cueStart;
  for (let i = 0; i < parts.length; i++) {
    if (i % 2 === 0) {
      const text = stripTags(parts[i]);
      if (text) words.push({ t: text, at });
    } else {
      at = toSeconds(parts[i]);
    }
  }
  return words;
}

interface RawCue {
  start: number;
  end: number;
  rawLines: string[];
  cleanLines: string[];
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
    const rawLines = lines.slice(lines.indexOf(timeLine) + 1);
    cues.push({ start, end, rawLines, cleanLines: rawLines.map(stripTags) });
  }
  return cues;
}

export function parseVtt(vtt: string): Cue[] {
  const rawCues = parseRawCues(vtt);
  const out: Cue[] = [];
  let curStart = 0;
  let curText = "";
  let curRawLine = "";
  let lastEnd = 0;

  const flush = (end: number) => {
    if (!curText) return;
    out.push({ s: curStart, e: end, t: curText, words: tokenizeWords(curRawLine, curStart) });
  };

  for (const cue of rawCues) {
    lastEnd = cue.end;
    const idx = cue.cleanLines.length - 1;
    const candidate = idx >= 0 ? cue.cleanLines[idx] : "";
    if (!candidate) continue;
    if (curText && candidate.startsWith(curText)) {
      curText = candidate;
      curRawLine = cue.rawLines[idx];
      continue;
    }
    if (curText && curText.startsWith(candidate)) {
      continue;
    }
    flush(cue.start);
    curStart = cue.start;
    curText = candidate;
    curRawLine = cue.rawLines[idx];
  }
  flush(lastEnd);
  return out;
}
