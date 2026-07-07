import { type ReactNode, Suspense } from 'react';
import { Header } from '@/components/shared/Header';
import { Footer } from '@/components/shared/Footer';
import { BottomNav } from '@/components/shared/BottomNav';
import { DeniedToast } from '@/components/shared/DeniedToast';

export function MainShell({ children }: { children: ReactNode }) {
  return (
    // min-h-dvh (vs min-h-screen) matters on iOS Safari — the address bar
    // collapse/expand shifts 100vh, causing a jump on scroll. dvh tracks the
    // visible viewport in real time.
    <div className="bg-background text-foreground flex min-h-dvh flex-col font-sans">
      <Header />

      <main
        id="main-content"
        // Container widths tuned per breakpoint tier:
        //   xl (1280) — laptop default; matches the current max-w-7xl.
        //   2xl (1440) — bumps to 1360px so wide MacBooks get one more grid col.
        //   3xl (1920) — caps at 1600px; wide desktops stop stretching text.
        //   4xl (2560) — 1800px; 4K screens keep line-length readable.
        // Vertical rhythm scales with the same idea.
        //
        // Extra bottom padding on mobile keeps content clear of the fixed
        // BottomNav (4rem tall + safe-area). Desktop resets since BottomNav
        // is hidden on md+.
        className="3xl:max-w-[1600px] 3xl:py-16 4xl:max-w-[1800px] 4xl:py-20 mx-auto w-full max-w-7xl flex-1 px-4 py-6 pb-[calc(4.5rem+env(safe-area-inset-bottom))] md:px-6 md:py-12 md:pb-12 2xl:max-w-[1360px]"
      >
        {children}
      </main>

      <Footer />

      <BottomNav />
      <Suspense fallback={null}>
        <DeniedToast />
      </Suspense>
    </div>
  );
}
