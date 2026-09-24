import { getAudioPages, getPageAudio, type AudioTrack } from "@/lib/audio";

/** Một bài trong danh sách nghe tự động — track kèm trang chứa nó. */
export interface AutoplayItem extends AudioTrack {
  page: number;
}

/**
 * Toàn bộ bài nghe của 1 sách theo đúng thứ tự trang, trong trang thì theo
 * thứ tự track đã khai báo (S → L → P với giáo trình).
 *
 * Bỏ track trùng URL: sách tiếng Anh có track in ở 2 trang liền nhau (vd
 * "2.16" ở cả trang 18 và 19 của en-pre-intermediate) — nghe 2 lần liền là
 * thừa. Giữ lần xuất hiện đầu.
 */
export function buildPlaylist(bookId: string): AutoplayItem[] {
  const seen = new Set<string>();
  const items: AutoplayItem[] = [];
  for (const page of getAudioPages(bookId)) {
    for (const track of getPageAudio(bookId, page)) {
      if (seen.has(track.url)) continue;
      seen.add(track.url);
      items.push({ page, ...track });
    }
  }
  return items;
}

/**
 * Vị trí bắt đầu khi bật nghe tự động giữa chừng: bài đầu tiên nằm ở trang
 * này hoặc sau đó. Đã qua bài cuối thì quay về đầu sách.
 */
export function startIndexFor(queue: AutoplayItem[], page: number): number {
  const i = queue.findIndex((item) => item.page >= page);
  return i === -1 ? 0 : i;
}
