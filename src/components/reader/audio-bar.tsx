"use client";

import { useState } from "react";
import type { AudioTrack } from "@/lib/audio";
import { cn } from "@/lib/utils";

export function AudioBar({
  tracks,
  visible,
}: {
  tracks: AudioTrack[];
  visible: boolean;
}) {
  const [activeType, setActiveType] = useState(tracks[0]?.type);
  if (tracks.length === 0) return null;

  const active = tracks.find((t) => t.type === activeType) ?? tracks[0];

  return (
    <div
      className={cn(
        "fixed inset-x-0 bottom-16 z-20 flex flex-col items-center gap-2 px-3 transition-all duration-200",
        visible ? "opacity-100" : "pointer-events-none opacity-0"
      )}
    >
      <div className="flex w-full max-w-sm flex-col gap-2 rounded-xl bg-black/70 p-2.5 backdrop-blur">
        {tracks.length > 1 && (
          <div className="flex justify-center gap-1.5">
            {tracks.map((t) => (
              <button
                key={t.type}
                type="button"
                onClick={() => setActiveType(t.type)}
                className={cn(
                  "rounded-full px-2.5 py-1 text-xs text-white transition-colors",
                  t.type === active.type
                    ? "bg-primary"
                    : "bg-white/10 hover:bg-white/20"
                )}
              >
                {t.label}
              </button>
            ))}
          </div>
        )}
        {/* key={active.url} để reset trạng thái phát khi đổi track/trang */}
        <audio key={active.url} controls src={active.url} className="h-9 w-full" />
      </div>
    </div>
  );
}
