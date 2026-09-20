"use client";

import { useCaptureInstallPrompt } from "@/lib/install-store";

/**
 * Không vẽ gì — chỉ ngồi ở root layout để nghe `beforeinstallprompt` ngay từ
 * lúc vào app. Xem `lib/install-store.ts` để biết vì sao không thể đợi tới
 * lúc người dùng mở trang hướng dẫn.
 */
export function InstallPromptCapture() {
  useCaptureInstallPrompt();
  return null;
}
