'use client';

import { useState } from 'react';
import Link from 'next/link';
import {
  ArrowLeft,
  BookOpen,
  CalendarClock,
  GraduationCap,
  LayoutDashboard,
  Star,
  UserCheck,
  Users,
} from 'lucide-react';
import { OverviewSection } from './sections/OverviewSection';
import { SessionsSection } from './sections/SessionsSection';
import { UsersSection } from './sections/UsersSection';
import { PendingTutorsSection } from './sections/PendingTutorsSection';
import { TutorsSection } from './sections/TutorsSection';
import { CoursesSection } from './sections/CoursesSection';
import { ReviewsSection } from './sections/ReviewsSection';
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
import type {
  AdminKpis,
  CourseForAdmin,
  PendingTutor,
  ReviewForAdmin,
  SessionListItem,
} from '@/lib/supabase';
import type { User } from '@/lib/types';

interface Props {
  kpis: AdminKpis;
  users: User[];
  sessions: SessionListItem[];
  pendingTutors: PendingTutor[];
  courses: CourseForAdmin[];
  reviews: ReviewForAdmin[];
  /** Signed-in admin's user id — used to grey out the self-demote option. */
  currentUserId: string;
}

type SectionKey = 'overview' | 'sessions' | 'users' | 'tutors' | 'pending' | 'courses' | 'reviews';

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
    badge: (p) => p.sessions.length || undefined,
  },
  { key: 'users', label: 'Users', icon: Users, badge: (p) => p.users.length || undefined },
  {
    key: 'tutors',
    label: 'Tutors',
    icon: GraduationCap,
    badge: (p) => p.users.filter((u) => u.role === 'tutor').length || undefined,
  },
  {
    key: 'pending',
    label: 'Pending tutors',
    icon: UserCheck,
    badge: (p) => p.pendingTutors.length || undefined,
  },
  {
    key: 'courses',
    label: 'Courses',
    icon: BookOpen,
    badge: (p) => p.courses.length || undefined,
  },
  {
    key: 'reviews',
    label: 'Reviews',
    icon: Star,
    badge: (p) => p.reviews.length || undefined,
  },
];

const formatBadge = (n: number) => (n > 99 ? '99+' : String(n));

export function AdminShell(props: Props) {
  const [active, setActive] = useState<SectionKey>('overview');
  const activeNav = NAV.find((n) => n.key === active);

  return (
    <SidebarProvider>
      <Sidebar variant="inset" collapsible="icon">
        <SidebarHeader>
          <div className="flex items-center gap-2 px-2 py-1.5">
            <div className="bg-brand flex size-8 shrink-0 items-center justify-center rounded-lg font-serif font-bold text-white">
              V
            </div>
            <div className="min-w-0 group-data-[collapsible=icon]:hidden">
              <p className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                Admin
              </p>
              <p className="text-brand dark:text-accent-warm truncate font-serif text-sm font-bold">
                VieLang HQ
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
                        <SidebarMenuBadge>{formatBadge(badge)}</SidebarMenuBadge>
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
            <p className="type-eyebrow leading-none">Admin</p>
            <p className="mt-0.5 truncate text-sm font-semibold text-slate-700 dark:text-slate-200">
              {activeNav?.label}
            </p>
          </div>
        </header>

        <div className="min-w-0 flex-1 p-4 md:p-6">
          {/* Instant swap between sections — animating the whole panel makes
              admin work feel sluggish. Individual widgets can still animate. */}
          <div key={active}>
            {active === 'overview' && (
              <OverviewSection kpis={props.kpis} pending={props.pendingTutors} />
            )}
            {active === 'sessions' && <SessionsSection sessions={props.sessions} />}
            {active === 'users' && (
              <UsersSection users={props.users} currentUserId={props.currentUserId} />
            )}
            {active === 'tutors' && (
              <TutorsSection users={props.users} currentUserId={props.currentUserId} />
            )}
            {active === 'pending' && <PendingTutorsSection pending={props.pendingTutors} />}
            {active === 'courses' && <CoursesSection courses={props.courses} />}
            {active === 'reviews' && <ReviewsSection reviews={props.reviews} />}
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
