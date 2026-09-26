import { SiteHeader } from "@/components/layout/site-header";
import { TabBar } from "@/components/layout/tab-bar";

/**
 * Layout cho trang xem video (`/video/[id]`) — tách khỏi `(main)` vì cần bố
 * cục kiểu YouTube: trên điện thoại video tràn hết bề ngang (không có lề
 * `px-4`), trên máy tính rộng hơn hẳn khung `max-w-5xl` để đặt video lớn
 * cạnh cột transcript. URL giữ nguyên — route group không thêm vào đường dẫn.
 */
export default function WatchLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-full flex-1 flex-col">
      <SiteHeader />
      <main className="mx-auto w-full max-w-[1800px] flex-1 lg:px-6 lg:py-6">{children}</main>
      <TabBar />
    </div>
  );
}
