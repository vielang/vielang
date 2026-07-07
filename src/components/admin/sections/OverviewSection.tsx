'use client';

import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts';
import { CalendarClock, GraduationCap, Star, TrendingUp, UserCheck, Users } from 'lucide-react';
import { formatVnd } from '@/lib/format';
import type { AdminKpis, PendingTutor } from '@/lib/supabase';
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@/components/ui/chart';
import { Card } from '@/components/ui/card';
import { cn } from '@/lib/utils';

interface Props {
  kpis: AdminKpis;
  pending: PendingTutor[];
}

const CHART_CONFIG = {
  count: {
    label: 'Sessions',
    theme: {
      light: '#4f46e5',
      dark: '#818cf8',
    },
  },
} satisfies ChartConfig;

type Accent = 'blue' | 'brand' | 'amber' | 'yellow' | 'emerald' | 'slate';

const ACCENT: Record<Accent, { wrap: string; icon: string }> = {
  blue: {
    wrap: 'bg-blue-50 dark:bg-blue-950/30',
    icon: 'text-blue-600 dark:text-blue-400',
  },
  brand: {
    wrap: 'bg-brand/10 dark:bg-brand/15',
    icon: 'text-brand dark:text-indigo-300',
  },
  amber: {
    wrap: 'bg-amber-50 dark:bg-amber-950/30',
    icon: 'text-amber-600 dark:text-amber-400',
  },
  yellow: {
    wrap: 'bg-yellow-50 dark:bg-yellow-950/30',
    icon: 'text-yellow-600 dark:text-yellow-400',
  },
  emerald: {
    wrap: 'bg-emerald-50 dark:bg-emerald-950/30',
    icon: 'text-emerald-600 dark:text-emerald-400',
  },
  slate: {
    wrap: 'bg-slate-100 dark:bg-slate-800',
    icon: 'text-slate-500 dark:text-slate-400',
  },
};

export function OverviewSection({ kpis, pending }: Props) {
  const pendingCount = pending.length;

  return (
    <div className="space-y-8">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Overview</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          Live snapshot of the VieLang platform.
        </p>
      </header>

      {/* Primary KPIs */}
      <section className="space-y-3">
        <p className="type-eyebrow">Platform</p>
        <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
          <KpiCard
            icon={<Users className="size-4" />}
            label="Students"
            value={kpis.studentsTotal}
            accent="blue"
          />
          <KpiCard
            icon={<GraduationCap className="size-4" />}
            label="Tutors"
            value={kpis.tutorsTotal}
            hint={`${kpis.tutorsApproved} approved`}
            accent="brand"
          />
          <KpiCard
            icon={<UserCheck className="size-4" />}
            label="Pending approvals"
            value={pendingCount}
            accent={pendingCount > 0 ? 'amber' : 'slate'}
            urgent={pendingCount > 0}
          />
          <KpiCard
            icon={<Star className="size-4" />}
            label="Avg rating"
            value={kpis.ratingAvg > 0 ? kpis.ratingAvg.toFixed(1) : '—'}
            accent="yellow"
          />
        </div>
      </section>

      {/* Sessions */}
      <section className="space-y-3">
        <p className="type-eyebrow">Sessions</p>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
          <KpiCard
            icon={<CalendarClock className="size-4" />}
            label="Today"
            value={kpis.sessionsToday}
            accent="slate"
          />
          <KpiCard
            icon={<CalendarClock className="size-4" />}
            label="This week"
            value={kpis.sessionsWeek}
            accent="slate"
          />
          <KpiCard
            icon={<CalendarClock className="size-4" />}
            label="This month"
            value={kpis.sessionsMonth}
            accent="slate"
            className="col-span-2 sm:col-span-1"
          />
        </div>
      </section>

      {/* Revenue */}
      <section className="space-y-3">
        <p className="type-eyebrow">Revenue</p>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <KpiCard
            icon={<TrendingUp className="size-4" />}
            label="This week"
            value={`${formatVnd(kpis.revenueWeekVnd)}₫`}
            hint="Completed sessions only"
            accent="emerald"
          />
          <KpiCard
            icon={<TrendingUp className="size-4" />}
            label="This month"
            value={`${formatVnd(kpis.revenueMonthVnd)}₫`}
            hint="Completed sessions only"
            accent="emerald"
          />
        </div>
      </section>

      {/* 30-day area chart */}
      <section>
        <Card className="gap-0 py-0">
          <div className="border-b border-slate-100 p-5 dark:border-slate-800">
            <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
              Sessions per day — last 30 days
            </h2>
            <p className="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
              Every booked session that touched the calendar, regardless of status.
            </p>
          </div>
          <div className="p-5">
            <ChartContainer config={CHART_CONFIG} className="h-56 w-full">
              <AreaChart
                data={kpis.sessionsPerDay}
                margin={{ top: 5, right: 10, left: -20, bottom: 0 }}
              >
                <defs>
                  <linearGradient id="countFill" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--color-count)" stopOpacity={0.18} />
                    <stop offset="95%" stopColor="var(--color-count)" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid
                  strokeDasharray="3 3"
                  stroke="currentColor"
                  className="text-slate-100 dark:text-slate-800"
                  strokeOpacity={0.8}
                />
                <XAxis
                  dataKey="date"
                  tickFormatter={(v: string) => v.slice(5)}
                  tickLine={false}
                  axisLine={false}
                  fontSize={10}
                  tickMargin={6}
                />
                <YAxis tickLine={false} axisLine={false} fontSize={10} allowDecimals={false} />
                <ChartTooltip content={<ChartTooltipContent indicator="line" />} />
                <Area
                  type="monotone"
                  dataKey="count"
                  stroke="var(--color-count)"
                  strokeWidth={2}
                  fill="url(#countFill)"
                  dot={false}
                  activeDot={{ r: 4 }}
                />
              </AreaChart>
            </ChartContainer>
          </div>
        </Card>
      </section>
    </div>
  );
}

function KpiCard({
  icon,
  label,
  value,
  hint,
  accent = 'slate',
  urgent = false,
  className,
}: {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  hint?: string;
  accent?: Accent;
  urgent?: boolean;
  className?: string;
}) {
  const { wrap, icon: iconColor } = ACCENT[accent];
  return (
    <Card
      className={cn('gap-0 py-0', urgent && 'ring-amber-300/70 dark:ring-amber-700/50', className)}
    >
      <div className="space-y-3 p-4">
        <div className="flex items-start justify-between gap-2">
          <div
            className={cn(
              'flex size-9 shrink-0 items-center justify-center rounded-lg',
              wrap,
              iconColor,
            )}
          >
            {icon}
          </div>
          {urgent && (
            <span className="inline-flex items-center gap-1 rounded-full bg-amber-100 px-2 py-0.5 text-[10px] font-bold text-amber-700 dark:bg-amber-950/40 dark:text-amber-400">
              <span className="size-1.5 animate-pulse rounded-full bg-amber-500" aria-hidden />
              Action needed
            </span>
          )}
        </div>
        <div>
          <p className="type-kpi text-slate-900 dark:text-slate-100">{value}</p>
          <p className="type-eyebrow mt-1">{label}</p>
          {hint && <p className="mt-0.5 text-[11px] text-slate-500 dark:text-slate-400">{hint}</p>}
        </div>
      </div>
    </Card>
  );
}
