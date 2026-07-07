'use client';

import { useEffect } from 'react';
import { toast } from 'sonner';
import { useLocalParticipant } from '@livekit/components-react';
import { useLang } from '@/contexts';

const COPY = {
  VN: {
    micOn: 'Bật mic',
    micOff: 'Tắt mic',
    camOn: 'Bật camera',
    camOff: 'Tắt camera',
  },
  EN: {
    micOn: 'Mic on',
    micOff: 'Mic off',
    camOn: 'Camera on',
    camOff: 'Camera off',
  },
} as const;

/**
 * Ignore key events fired while the user is typing — chat input, host
 * search fields, forms. Otherwise pressing "m" mid-message would mute the
 * mic instead of typing an "m".
 */
function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
}

/**
 * Global room shortcuts, matching what Zoom/Meet users already reach for.
 *
 *   M  — toggle microphone
 *   C  — toggle camera
 *
 * No shortcut for Leave (Esc etc.) — booting yourself with a stray keypress
 * is a nasty footgun and the existing top-bar button is one click away.
 * No push-to-talk (Space) — Space also scrolls, so a hold-to-talk interaction
 * fights the page and needs its own opt-in surface; deferred to a later PR.
 *
 * Modifier-held presses (Ctrl/Alt/Cmd) pass through so system shortcuts
 * like Cmd+M (minimise) and Ctrl+C (copy) keep working.
 */
export function RoomShortcuts() {
  const { lang } = useLang();
  const lc = COPY[lang];
  const { localParticipant, isMicrophoneEnabled, isCameraEnabled } = useLocalParticipant();

  useEffect(() => {
    if (!localParticipant) return;
    const handler = (e: KeyboardEvent) => {
      if (e.altKey || e.ctrlKey || e.metaKey) return;
      if (isEditableTarget(e.target)) return;
      const k = e.key.toLowerCase();
      if (k === 'm') {
        e.preventDefault();
        const next = !isMicrophoneEnabled;
        void localParticipant.setMicrophoneEnabled(next).catch(() => {});
        toast(next ? lc.micOn : lc.micOff, { duration: 1200 });
      } else if (k === 'c') {
        e.preventDefault();
        const next = !isCameraEnabled;
        void localParticipant.setCameraEnabled(next).catch(() => {});
        toast(next ? lc.camOn : lc.camOff, { duration: 1200 });
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [localParticipant, isMicrophoneEnabled, isCameraEnabled, lc]);

  return null;
}
