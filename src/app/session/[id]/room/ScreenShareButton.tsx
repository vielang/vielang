'use client';

import { useState } from 'react';
import { Loader2, MonitorUp, MonitorX } from 'lucide-react';
import { toast } from 'sonner';
import { useLocalParticipant } from '@livekit/components-react';
import { Button } from '@/components/ui/button';
import { useLang } from '@/contexts';

const COPY = {
  VN: {
    share: 'Chia sẻ màn hình',
    stop: 'Dừng chia sẻ',
    failed: 'Chia sẻ màn hình thất bại',
    refused: 'Trình duyệt của bạn đã từ chối yêu cầu.',
  },
  EN: {
    share: 'Share screen',
    stop: 'Stop sharing',
    failed: 'Screen share failed',
    refused: 'Your browser refused the request.',
  },
} as const;

type RoomRole = 'tutor' | 'admin' | 'student';

function parseRole(metadata: string | undefined): RoomRole {
  if (!metadata) return 'student';
  try {
    const parsed = JSON.parse(metadata) as { role?: unknown };
    return parsed.role === 'tutor' || parsed.role === 'admin' ? parsed.role : 'student';
  } catch {
    return 'student';
  }
}

/**
 * One-tap screen share toggle for tutors and admins.
 *
 * Renders nothing for students — the token endpoint already restricts
 * `canPublishSources` to camera + microphone for them, so bypassing this
 * component (e.g. via devtools) would still be rejected by LiveKit. This is
 * purely a UI gate.
 *
 * The user picks the surface (screen / window / tab) via the browser's own
 * picker; NotAllowedError / AbortError from that picker means "user changed
 * their mind" and stays silent. Any other error toasts, since it usually
 * means we asked for permission that isn't granted or the browser refused
 * (some mobile Safari builds still reject getDisplayMedia entirely).
 *
 * Stacked above HostControlsPanel (both fixed to the bottom-right) so both
 * host-only affordances share the same corner without overlapping the chat
 * button on the left or the reactions pill in the centre.
 */
export function ScreenShareButton() {
  const { lang } = useLang();
  const lc = COPY[lang];
  const { localParticipant, isScreenShareEnabled } = useLocalParticipant();
  const [pending, setPending] = useState(false);

  const role = parseRole(localParticipant?.metadata);
  if (role !== 'tutor' && role !== 'admin') return null;

  const toggle = async () => {
    if (!localParticipant) return;
    setPending(true);
    try {
      // audio:true also captures system audio when the surface is a tab and
      // the user ticks "share audio" — silently ignored on window/screen where
      // the browser doesn't offer that option.
      await localParticipant.setScreenShareEnabled(!isScreenShareEnabled, { audio: true });
    } catch (err) {
      const name = (err as Error)?.name;
      // User cancelled the surface picker — not an error worth surfacing.
      if (name !== 'NotAllowedError' && name !== 'AbortError') {
        toast.error(lc.failed, {
          description: (err as Error)?.message || lc.refused,
        });
      }
    } finally {
      setPending(false);
    }
  };

  const label = isScreenShareEnabled ? lc.stop : lc.share;

  return (
    <Button
      variant={isScreenShareEnabled ? 'destructive' : 'default'}
      onClick={toggle}
      disabled={pending}
      className="fixed right-4 bottom-40 z-40 rounded-full shadow-lg sm:right-6"
      aria-label={label}
      aria-pressed={isScreenShareEnabled}
    >
      {pending ? (
        <Loader2 className="animate-spin" />
      ) : isScreenShareEnabled ? (
        <MonitorX />
      ) : (
        <MonitorUp />
      )}
      <span className="hidden sm:inline">{label}</span>
    </Button>
  );
}
