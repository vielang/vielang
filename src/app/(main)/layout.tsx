import { SiteHeader } from "@/components/layout/site-header";
import { TabBar } from "@/components/layout/tab-bar";

/**
 * Layout chung: header và (trên điện thoại) thanh tab dưới đáy.
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
