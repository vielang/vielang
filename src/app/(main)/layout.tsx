import { SiteHeader } from "@/components/layout/site-header";
import { TabBar } from "@/components/layout/tab-bar";

/**
 * Layout cho các trang "bình thường" (thư viện, chi tiết sách) — có header
 * và (trên điện thoại) thanh tab dưới đáy.
 * Trang đọc (`/read/[bookId]/[page]`) nằm ngoài group này để full-screen,
 * không có header, tối đa không gian đọc.
 */
export default function MainLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-full flex-1 flex-col">
      <SiteHeader />
      <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-6">
        {children}
      </main>
      <TabBar />
    </div>
  );
}
