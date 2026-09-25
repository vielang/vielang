import type { Metadata } from "next";
import { Clapperboard } from "lucide-react";
import { PageHeader } from "@/components/layout/page-header";
import { EmptyState } from "@/components/layout/empty-state";
import { VideoCard } from "@/components/video/video-card";
import { getVideoLessons } from "@/lib/videos";

export const metadata: Metadata = { title: "Học tiếng Hàn qua video" };

/**
 * Danh sách video của Thư viện tiếng Hàn (`/video`) — phim có sẵn phụ đề
 * tiếng Hàn, bật/tắt được phụ đề tiếng Việt khi xem (xem `/video/[id]`).
 * Nội dung nạp qua `scripts/prepare-video.ts`, có thể chưa đủ 10 tập nếu
 * video còn đang tải — trang vẫn hiện những tập đã sẵn sàng.
 */
export default function VideoLibraryPage() {
  const lessons = getVideoLessons();

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Học tiếng Hàn qua video"
        subtitle="Xem theo thứ tự tập — phụ đề tiếng Hàn luôn hiện, bật thêm phụ đề tiếng Việt khi cần."
      />

      {lessons.length === 0 ? (
        <EmptyState
          dashed
          icon={Clapperboard}
          title="Sắp có video"
          description="Video đang được chuẩn bị, quay lại sau nhé."
        />
      ) : (
        <div className="grid min-w-0 grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4">
          {lessons.map((lesson) => (
            <VideoCard key={lesson.id} lesson={lesson} />
          ))}
        </div>
      )}
    </div>
  );
}
