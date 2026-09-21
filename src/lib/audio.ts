import {
  resolvePageAudio,
  getTrackLabel,
  getAudioPages as getTextbookAudioPages,
  resolveWorkbookPageAudio,
  getWorkbookAudioPages,
  resolveEnglishPageAudio,
  getEnglishAudioPages,
  type AudioTrackType,
} from "@/lib/audio-config";

export interface AudioTrack {
  /** Định danh duy nhất trong danh sách track của 1 trang — dùng làm key + chọn track đang phát. */
  type: string;
  label: string;
  url: string;
}

/**
 * Tiền tố cùng origin, chuyển tiếp sang R2 bởi `rewrites()` trong
 * `next.config.ts`. Giống hệt đường của ảnh trang.
 *
 * Tên `/img` là do lịch sử — nó ra đời lúc mới chỉ có ảnh đi qua. Giữ
 * nguyên chứ không đổi thành `/media` cho đẹp: đường dẫn này là KHOÁ CACHE
 * của những cuốn người dùng đã tải về máy, đổi một chữ là mấy cuốn đó không
 * khớp nữa và họ phải tải lại từ đầu. Cái tên hơi lệch nghĩa rẻ hơn nhiều.
 *
 * Vì sao phải cùng origin: `sw.js` bỏ qua mọi request khác origin, và quan
 * trọng hơn — `cache.put()` TỪ CHỐI bản trả về `opaque` (status 0), mà
 * fetch sang r2.dev không CORS thì chỉ nhận được opaque. Tức là trỏ thẳng
 * R2 thì không tài nào tải audio về máy được.
 */
const MEDIA_PREFIX = "/img";

function audioBaseUrl(): string {
  return MEDIA_PREFIX;
}

function fileNameFor(lesson: number | null, type: AudioTrackType): string {
  if (type === "intro") return "0.mp3";
  return `${lesson}-${type}.mp3`;
}

/** Danh sách track audio gắn với 1 trang cụ thể — gộp cả textbook và workbook. */
export function getPageAudio(bookId: string, page: number): AudioTrack[] {
  const textbookTracks = resolvePageAudio(bookId, page).map(({ lesson, type }) => ({
    type,
    label: getTrackLabel(type),
    url: `${audioBaseUrl()}/books/${bookId}/audio/${fileNameFor(lesson, type)}`,
  }));
  if (textbookTracks.length > 0) return textbookTracks;

  const englishTracks = resolveEnglishPageAudio(bookId, page).map((trackId) => ({
    type: trackId,
    label: trackId,
    url: `${audioBaseUrl()}/books/${bookId}/audio/${trackId}.mp3`,
  }));
  if (englishTracks.length > 0) return englishTracks;

  return resolveWorkbookPageAudio(bookId, page).map((trackNum, i) => ({
    type: `workbook-${i + 1}`,
    label: `Bài nghe ${i + 1}`,
    url: `${audioBaseUrl()}/books/${bookId}/audio/track${String(trackNum).padStart(2, "0")}.mp3`,
  }));
}

export function hasAudio(bookId: string, page: number): boolean {
  return getPageAudio(bookId, page).length > 0;
}

/** Danh sách số trang có audio của 1 sách (textbook hoặc workbook). */
export function getAudioPages(bookId: string): number[] {
  const textbookPages = getTextbookAudioPages(bookId);
  if (textbookPages.length > 0) return textbookPages;
  const englishPages = getEnglishAudioPages(bookId);
  if (englishPages.length > 0) return englishPages;
  return getWorkbookAudioPages(bookId);
}
