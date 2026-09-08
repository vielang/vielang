import { resolvePageAudio, getTrackLabel, type AudioTrackType } from "@/lib/audio-config";

export { getAudioPages } from "@/lib/audio-config";

export interface AudioTrack {
  type: AudioTrackType;
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

/** Danh sách track audio (0-2) gắn với 1 trang cụ thể. */
export function getPageAudio(bookId: string, page: number): AudioTrack[] {
  return resolvePageAudio(bookId, page).map(({ lesson, type }) => ({
    type,
    label: getTrackLabel(type),
    url: `${audioBaseUrl()}/books/${bookId}/audio/${fileNameFor(lesson, type)}`,
  }));
}

export function hasAudio(bookId: string, page: number): boolean {
  return getPageAudio(bookId, page).length > 0;
}
