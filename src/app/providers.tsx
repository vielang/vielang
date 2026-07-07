'use client';

import { MotionConfig } from 'motion/react';
import { ThemeProvider } from 'next-themes';
import { LangProvider, AuthProvider, SiteSettingsProvider } from '@/contexts';
import { Toaster } from '@/components/ui/sonner';
import { TooltipProvider } from '@/components/ui/tooltip';
import { ServiceWorkerRegister } from '@/components/pwa/ServiceWorkerRegister';
import type { AppUser } from '@/contexts/AuthContext';

export function Providers({
  initialUser,
  children,
}: {
  initialUser?: AppUser | null;
  children: React.ReactNode;
}) {
  return (
    <ThemeProvider attribute="class" defaultTheme="light" enableSystem disableTransitionOnChange>
      <MotionConfig reducedMotion="user">
        <LangProvider>
          <SiteSettingsProvider>
            <AuthProvider initialUser={initialUser}>
              <TooltipProvider delay={150}>{children}</TooltipProvider>
              <Toaster position="top-right" richColors closeButton />
              <ServiceWorkerRegister />
            </AuthProvider>
          </SiteSettingsProvider>
        </LangProvider>
      </MotionConfig>
    </ThemeProvider>
  );
}
