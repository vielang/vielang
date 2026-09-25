"use client";

import { useEffect } from "react";
import { create } from "zustand";

/**
 * Sự kiện Chrome bắn ra khi app đủ điều kiện cài. Chưa có trong lib DOM
 * chuẩn vì mới chỉ Chromium hỗ trợ.
 */
interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>;
  userChoice: Promise<{ outcome: "accepted" | "dismissed" }>;
}

export type Platform = "ios" | "android" | "desktop";

interface InstallState {
  /**
   * Lời mời cài của Chrome, nếu đã bắt được. Có nó thì cài bằng một nút bấm
   * thật, không cần đọc hướng dẫn.
   */
  event: BeforeInstallPromptEvent | null;
  /** Đang chạy dưới dạng app đã cài, không phải trong tab trình duyệt. */
  installed: boolean;
  setEvent: (e: BeforeInstallPromptEvent | null) => void;
  setInstalled: (v: boolean) => void;
}

export const useInstallStore = create<InstallState>((set) => ({
  event: null,
  installed: false,
  setEvent: (event) => set({ event }),
  setInstalled: (installed) => set({ installed }),
}));

/**
 * Bắt `beforeinstallprompt` ngay từ lúc vào app.
 *
 * Phải nghe từ đầu chứ không đợi tới lúc mở trang hướng dẫn: Chrome bắn sự
 * kiện này một lần, rất sớm, và nếu không ai gọi `preventDefault()` thì nó
 * trôi mất — lúc đó chỉ còn cách chỉ người dùng tự vào menu.
 *
 * Gọi ở root layout, xem `components/install-prompt-capture.tsx`.
 */
export function useCaptureInstallPrompt(): void {
  const setEvent = useInstallStore((s) => s.setEvent);
  const setInstalled = useInstallStore((s) => s.setInstalled);

  useEffect(() => {
    setInstalled(isStandalone());

    function onPrompt(e: Event) {
      // Chặn thanh mời cài mặc định của Chrome để tự quyết chỗ và lúc mời —
      // đổi lại thì phải giữ sự kiện lại mà dùng sau.
      e.preventDefault();
      setEvent(e as BeforeInstallPromptEvent);
    }
    function onInstalled() {
      setEvent(null);
      setInstalled(true);
    }

    window.addEventListener("beforeinstallprompt", onPrompt);
    window.addEventListener("appinstalled", onInstalled);
    return () => {
      window.removeEventListener("beforeinstallprompt", onPrompt);
      window.removeEventListener("appinstalled", onInstalled);
    };
  }, [setEvent, setInstalled]);
}

/** Đang mở dưới dạng app đã cài chứ không phải tab trình duyệt. */
function isStandalone(): boolean {
  if (typeof window === "undefined") return false;
  return (
    window.matchMedia("(display-mode: standalone)").matches ||
    // Safari trên iOS không theo chuẩn display-mode, dùng cờ riêng.
    (navigator as Navigator & { standalone?: boolean }).standalone === true
  );
}

/**
 * Đoán hệ điều hành để đưa đúng hướng dẫn lên trước.
 *
 * iPadOS 13 trở đi khai user-agent y hệt máy Mac — phân biệt bằng số điểm
 * chạm, vì Mac thật thì không có màn cảm ứng.
 */
export function detectPlatform(userAgent: string, maxTouchPoints: number): Platform {
  if (/iphone|ipod|ipad/i.test(userAgent)) return "ios";
  if (/macintosh/i.test(userAgent) && maxTouchPoints > 1) return "ios";
  if (/android/i.test(userAgent)) return "android";
  return "desktop";
}

/** Hệ điều hành của thiết bị đang dùng. Ngoài trình duyệt thì coi như máy bàn. */
export function currentPlatform(): Platform {
  if (typeof navigator === "undefined") return "desktop";
  return detectPlatform(navigator.userAgent, navigator.maxTouchPoints ?? 0);
}
