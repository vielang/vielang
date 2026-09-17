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

let warnedMissingBase = false;

function audioBaseUrl(): string {
  const base = process.env.NEXT_PUBLIC_IMAGE_BASE_URL;
  if (!base) {
    if (!warnedMissingBase) {
      console.warn(
        "⚠ Thiếu biến môi trường NEXT_PUBLIC_IMAGE_BASE_URL — audio sẽ không " +
          "tải được. Xem web/.env.local.example."
      );
      warnedMissingBase = true;
    }
    return "";
  }
  return base.replace(/\/$/, "");
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
