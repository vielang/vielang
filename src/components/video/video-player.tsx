"use client";

import { useEffect, useRef, useState } from "react";
import { Languages } from "lucide-react";
import { Toggle } from "@/components/ui/toggle";
import { activeCue, videoPosterUrl, videoUrl, type VideoLesson } from "@/lib/videos";

/**
 * Phụ đề tiếng Hàn đè lên video (luôn hiện — video vốn đã có sẵn), phụ đề
 * tiếng Việt (dịch tự động) hiện thành 1 dòng RIÊNG bên dưới, bật/tắt bằng
 * nút Toggle. Cả hai chữ tự vẽ bằng React theo `currentTime` của video, chứ
 * KHÔNG dùng `<track>`/TextTrack của trình duyệt — R2 chưa bật CORS nên
 * track phụ đề nguồn ngoài dễ bị trình duyệt âm thầm không hiện (xem
 * `lib/videos.ts`).
 */
export function VideoPlayer({ lesson }: { lesson: VideoLesson }) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [koText, setKoText] = useState("");
  const [viText, setViText] = useState("");
  const [showVi, setShowVi] = useState(true);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    function onTimeUpdate() {
      if (!video) return;
      const t = video.currentTime;
      setKoText(activeCue(lesson.koCues, t)?.t ?? "");
      setViText(activeCue(lesson.viCues, t)?.t ?? "");
    }
    video.addEventListener("timeupdate", onTimeUpdate);
    return () => video.removeEventListener("timeupdate", onTimeUpdate);
  }, [lesson]);

  return (
    <div className="flex flex-col gap-3">
      <div className="relative overflow-hidden rounded-xl bg-black">
        <video
          ref={videoRef}
          src={videoUrl(lesson.id)}
          poster={videoPosterUrl(lesson.id)}
          controls
          playsInline
          preload="metadata"
          className="aspect-video w-full"
        />
        {koText && (
          <div className="pointer-events-none absolute inset-x-0 bottom-3 flex justify-center px-3 sm:bottom-4">
            <span className="font-korean max-w-[92%] rounded bg-black/70 px-2.5 py-1 text-center text-base leading-snug font-medium text-white sm:text-lg">
              {koText}
            </span>
          </div>
        )}
      </div>

      <div className="flex items-start justify-between gap-3">
        <p className="min-h-10 flex-1 text-sm leading-snug text-muted-foreground">
          {showVi ? viText : null}
        </p>
        <Toggle
          pressed={showVi}
          onPressedChange={setShowVi}
          disabled={!lesson.hasVi}
          variant="outline"
          className="shrink-0"
          aria-label="Bật/tắt phụ đề tiếng Việt"
        >
          <Languages className="size-4" aria-hidden />
          Phụ đề Việt
        </Toggle>
      </div>
      {!lesson.hasVi && (
        <p className="-mt-2 text-xs text-muted-foreground">Tập này chưa có phụ đề tiếng Việt.</p>
      )}
    </div>
  );
}
