"use client";

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { syncAcrossTabs } from "@/lib/cross-tab-sync";

/**
 * Tuỳ chọn khi xem video — chọn một lần, áp dụng cho mọi tập (lưu
 * localStorage, cùng khuôn với `reader-prefs-store.ts`).
 *
 * Mặc định hướng tới người mới: hiện cả hai dòng phụ đề, không tự dừng. Người
 * đã khá thì tắt/che tiếng Việt để tự nghe hiểu trước, rồi mới chạm để xem
 * bản dịch — che chứ không tắt hẳn, vì bí thì vẫn cần tra được ngay.
 */
export const SUBTITLE_SIZE_COUNT = 4;
export const PLAYBACK_RATES = [0.5, 0.75, 1, 1.25, 1.5] as const;

export interface VideoPrefs {
  /** Phụ đề tiếng Hàn đè trên video. */
  subKo: boolean;
  /** Phụ đề tiếng Việt đè trên video. */
  subVi: boolean;
  /** 0 (nhỏ) … `SUBTITLE_SIZE_COUNT - 1` (rất lớn). */
  subSize: number;
  /** Làm mờ tiếng Việt (phụ đề lẫn transcript), chạm/rê chuột mới hiện. */
  blurVi: boolean;
  playbackRate: number;
  /** Tự dừng ở cuối mỗi câu — để kịp đọc, nhắc lại theo (shadowing). */
  autoPause: boolean;
  /** Bảng transcript cạnh/dưới video. */
  showTranscript: boolean;
}

export const DEFAULT_VIDEO_PREFS: VideoPrefs = {
  subKo: true,
  subVi: true,
  subSize: 1,
  blurVi: false,
  playbackRate: 1,
  autoPause: false,
  showTranscript: true,
};

interface VideoPrefsState extends VideoPrefs {
  set: (patch: Partial<VideoPrefs>) => void;
  reset: () => void;
}

export const useVideoPrefsStore = create<VideoPrefsState>()(
  persist(
    (set) => ({
      ...DEFAULT_VIDEO_PREFS,
      set: (patch) => set(patch),
      reset: () => set(DEFAULT_VIDEO_PREFS),
    }),
    {
      name: "kiip-video-prefs-v1",
      storage: createJSONStorage(() => localStorage),
    }
  )
);

syncAcrossTabs(useVideoPrefsStore);
