'use client';

import { useState } from 'react';
import Link from 'next/link';
import { motion, AnimatePresence } from 'motion/react';
import {
  ArrowLeft,
  BookOpen,
  CalendarClock,
  CalendarDays,
  LayoutDashboard,
  User as UserIcon,
} from 'lucide-react';
import { TutorOverviewSection } from './sections/TutorOverviewSection';
import { TutorSessionsSection } from './sections/TutorSessionsSection';
import { TutorAvailabilitySection } from './sections/TutorAvailabilitySection';
import { TutorMaterialsSection } from './sections/TutorMaterialsSection';
import { TutorProfileSection } from './sections/TutorProfileSection';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
} from '@/components/ui/sidebar';
import type { SessionListItem, TutorKpis } from '@/lib/supabase';
import type { Availability, Course, TutorProfile, User } from '@/lib/types';

interface Props {
  user: User;
  profile: TutorProfile;
  kpis: TutorKpis;
  sessions: SessionListItem[];
  availability: Availability[];
  courses: Course[];
}

type SectionKey = 'overview' | 'sessions' | 'availability' | 'materials' | 'profile';

const NAV: {
  key: SectionKey;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  badge?: (p: Props) => number | undefined;
}[] = [
  { key: 'overview', label: 'Overview', icon: LayoutDashboard },
  {
    key: 'sessions',
    label: 'Sessions',
    icon: CalendarClock,
    badge: (p) => p.sessions.filter((s) => s.status === 'pending').length || undefined,
  },
  { key: 'availability', label: 'Availability', icon: CalendarDays },
  { key: 'materials', label: 'Materials', icon: BookOpen },
  { key: 'profile', label: 'Profile', icon: UserIcon },
];

const formatBadge = (n: number) => (n > 99 ? '99+' : String(n));

export function TutorShell(props: Props) {
  const [active, setActive] = useState<SectionKey>('overview');
  const activeNav = NAV.find((n) => n.key === active);
  // Initials fallback for the sidebar avatar. Empty name gets a `?` glyph
  // so the tile never renders blank.
  const initial = (props.user.name?.trim()[0] || '?').toUpperCase();

  return (
    <SidebarProvider>
      <Sidebar variant="inset" collapsible="icon">
        <SidebarHeader>
          <div className="flex items-center gap-2 px-2 py-1.5">
            <div className="bg-brand flex size-8 shrink-0 items-center justify-center rounded-lg font-serif font-bold text-white">
              {initial}
            </div>
            <div className="min-w-0 group-data-[collapsible=icon]:hidden">
              <p className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                Tutor
              </p>
              <p className="text-brand dark:text-accent-warm truncate font-serif text-sm font-bold">
                {props.user.name || 'Unnamed tutor'}
              </p>
            </div>
          </div>
        </SidebarHeader>

        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupContent>
              <SidebarMenu>
                {NAV.map((n) => {
                  const isActive = active === n.key;
                  const badge = n.badge?.(props);
                  const Icon = n.icon;
                  return (
                    <SidebarMenuItem key={n.key}>
                      <SidebarMenuButton
                        isActive={isActive}
                        tooltip={n.label}
                        onClick={() => setActive(n.key)}
                      >
                        <Icon />
                        <span>{n.label}</span>
                      </SidebarMenuButton>
                      {typeof badge === 'number' && (
                        <SidebarMenuBadge className="text-amber-700 dark:text-amber-300">
                          {formatBadge(badge)}
                        </SidebarMenuBadge>
                      )}
                    </SidebarMenuItem>
                  );
                })}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>

        <SidebarFooter>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton tooltip="Back to site" render={<Link href="/" />}>
                <ArrowLeft />
                <span>Back to site</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarFooter>

        <SidebarRail />
      </Sidebar>

      <SidebarInset>
        <header className="bg-background/80 sticky top-0 z-10 flex h-14 shrink-0 items-center gap-3 border-b border-slate-200/60 px-4 backdrop-blur md:px-6 dark:border-slate-800/60">
          <SidebarTrigger className="-ml-1" />
          <div className="min-w-0">
            <p className="type-eyebrow leading-none">Tutor</p>
            <p className="mt-0.5 truncate text-sm font-semibold text-slate-700 dark:text-slate-200">
              {activeNav?.label}
            </p>
          </div>
        </header>

        <div className="min-w-0 flex-1 p-4 md:p-6">
          <AnimatePresence mode="wait">
            <motion.div
              key={active}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.18 }}
            >
              {active === 'overview' && (
                <TutorOverviewSection
                  kpis={props.kpis}
                  profile={props.profile}
                  sessions={props.sessions}
                />
              )}
              {active === 'sessions' && <TutorSessionsSection sessions={props.sessions} />}
              {active === 'availability' && (
                <TutorAvailabilitySection initial={props.availability} />
              )}
              {active === 'materials' && <TutorMaterialsSection courses={props.courses} />}
              {active === 'profile' && (
                <TutorProfileSection user={props.user} profile={props.profile} />
              )}
            </motion.div>
          </AnimatePresence>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
