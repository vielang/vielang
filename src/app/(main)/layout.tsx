import { type ReactNode } from 'react';
import { MainShell } from '@/components/shared/MainShell';

export default function MainLayout({ children }: { children: ReactNode }) {
  return <MainShell>{children}</MainShell>;
}
