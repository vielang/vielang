import raw from "../../content/videos/PLOBbhydezQbfUHGKsM8q9DUyiOFWCRzOw.json";
import { mediaOriginBase } from "@/lib/books";

/**
 * "Học tiếng Hàn qua video" (`/video`) — 10 tập đầu của một series phim Hàn
 * (tải bằng yt-dlp, xem `scripts/prepare-video.ts`), sống trong Thư viện
 * tiếng Hàn cạnh bộ sách KIIP.
 *
 * Video + ảnh bìa nằm trên R2, phụ đề (tiếng Hàn có sẵn, tiếng Việt tự dịch
 * bằng YouTube) đã gộp sẵn thành cue {s, e, t} ngay trong file JSON này —
 * không parse .vtt lúc runtime, và không cần bật CORS cho R2 (xem lý do
 * dùng URL R2 trực tiếp thay vì `<track>` ở `components/reader/*` phần
 * audio: phụ đề chữ tự vẽ bằng React, không phải TextTrack của trình duyệt).
 *
 * Import TĨNH (không fs.readFileSync) — cùng lý do với `lib/exams.ts`.
 */
export interface WordSpan {
  t: string;
  at: number;
}

export interface VideoCue {
  s: number;
  e: number;
  t: string;
  /** Mốc thời gian từng chữ trong câu — để overlay tô sáng dần lúc đọc tới
   * (karaoke), lấy từ thẻ `<c>` gốc của YouTube. Xem `scripts/vtt.ts`. */
  words: WordSpan[];
}

export interface VideoLesson {
  id: string;
  order: number;
  episode: number;
  part: number;
  showTitle: string;
  durationSec: number | null;
  hasVi: boolean;
  koCues: VideoCue[];
  viCues: VideoCue[];
}

const PLAYLIST_ID = "PLOBbhydezQbfUHGKsM8q9DUyiOFWCRzOw";

const VIDEO_LESSONS = raw as VideoLesson[];

export function getVideoLessons(): VideoLesson[] {
  return VIDEO_LESSONS;
}

export function getVideoLesson(id: string): VideoLesson | undefined {
  return VIDEO_LESSONS.find((v) => v.id === id);
}

/** Tập trước/sau trong danh sách — để trang phát có nút chuyển tập liền mạch. */
export function adjacentVideoLessons(id: string): { prev?: VideoLesson; next?: VideoLesson } {
  const i = VIDEO_LESSONS.findIndex((v) => v.id === id);
  if (i < 0) return {};
  return { prev: VIDEO_LESSONS[i - 1], next: VIDEO_LESSONS[i + 1] };
}

export function videoLessonTitle(v: VideoLesson): string {
  return `Tập ${v.episode} · Phần ${v.part}`;
}

/** Trỏ THẲNG vào R2 (không qua rewrite cùng origin) — thẻ <video> tải theo
 * khoảng byte (Range) để tua, mà cache biên chỉ khoá theo đường dẫn, không
 * tính header Range. Cùng lý do và cùng cách với `lib/audio.ts`. */
export function videoUrl(id: string): string {
  return `${mediaOriginBase()}/videos/${PLAYLIST_ID}/${id}.mp4`;
}

export function videoPosterUrl(id: string): string {
  return `${mediaOriginBase()}/videos/${PLAYLIST_ID}/${id}.jpg`;
}

export function formatVideoDuration(sec: number | null): string | undefined {
  if (sec === null) return undefined;
  const total = Math.round(sec);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}

/** Cue đang phát tại thời điểm `t` (giây) — tìm tuyến tính, danh sách cue
 * mỗi tập chỉ vài trăm mục nên không cần nhị phân. */
export function activeCue(cues: VideoCue[], t: number): VideoCue | undefined {
  return cues.find((c) => t >= c.s && t < c.e);
}
