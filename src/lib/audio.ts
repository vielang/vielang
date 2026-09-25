import { mediaOriginBase } from "@/lib/books";
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
 * Bài nghe trỏ THẲNG vào R2, không đi qua `/img/...` như ảnh trang.
 *
 * Đi qua proxy cùng origin nghe hợp lý hơn, và đã từng làm thế — nhưng nó
 * hỏng nặng: thẻ <audio> tải file theo từng KHOẢNG byte (`Range`), mà cache
 * biên của Vercel lấy khoá cache chỉ theo đường dẫn, không tính header
 * `Range`. Nó giữ lại bản trả lời của khoảng ĐẦU TIÊN — đúng 2 byte thăm
 * dò — rồi đem 2 byte ấy trả cho mọi khoảng xin sau đó.
 *
 * Hỏng kiểu tệ nhất: mã vẫn 206, vẫn coi như thành công, không lỗi nào bắn
 * ra. Trình duyệt xin đoạn tiếp, nhận lại đoạn nó đã có, xin lại, lại nhận
 * đúng thứ đó — người dùng chỉ thấy một vòng xoay không bao giờ dứt, trên
 * MỌI thiết bị. Đã thử vá bằng `headers()` trong next.config để tắt cache
 * CDN: KHÔNG ăn thua, vì `headers()` không áp lên đường đã `rewrites()` ra
 * ngoài (đã dựng bản production ra kiểm).
 *
 * R2 thì trả đúng khoảng được xin, luôn luôn. Nên đường ngắn nhất là đừng
 * để ai đứng giữa.
 *
 * Cái mất: R2 chưa bật CORS, nên `fetch` từ trang nhận về bản `opaque` và
 * không cất vào cache được — tải sách về máy sẽ KHÔNG kèm bài nghe nữa
 * (phần tải bỏ qua êm, không làm hỏng cả lượt). Bật CORS cho bucket là
 * phần này tự sống lại, không phải sửa dòng nào.
 *
 * Kiểm lại sau mỗi lần deploy: `npm run check-media`.
 */
function audioBaseUrl(): string {
  return mediaOriginBase();
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

/** Danh sách số trang có audio của 1 sách (textbook hoặc workbook). */
export function getAudioPages(bookId: string): number[] {
  const textbookPages = getTextbookAudioPages(bookId);
  if (textbookPages.length > 0) return textbookPages;
  const englishPages = getEnglishAudioPages(bookId);
  if (englishPages.length > 0) return englishPages;
  return getWorkbookAudioPages(bookId);
}
