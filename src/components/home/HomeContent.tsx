'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useRef, useState, useMemo } from 'react';
import { motion } from 'motion/react';
import {
  ArrowUpRight,
  Search,
  CalendarPlus,
  PlayCircle,
  Target,
  Briefcase,
  Baby,
  MessagesSquare,
  Mic,
  Sparkles,
  Users,
  Star,
  GraduationCap,
  Clock,
  X,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useLang } from '@/contexts';
import { TutorCard } from '@/components/tutor/TutorCard';
import { GroupSessionCard } from '@/components/session/GroupSessionCard';
import { FilterPill } from '@/components/shared/FilterPill';
import { useRecentSearches } from '@/hooks/use-recent-searches';
import type { Tutor } from '@/lib/types';
import type { SessionListItem } from '@/lib/supabase';
import type { GroupLevel } from '@/lib/types';

const RECENT_SEARCH_KEY = 'vielang_recent_sessions';

export interface HomeStats {
  tutorsApproved: number;
  sessionsCompleted: number;
  ratingAvg: number;
}

interface Props {
  featuredTutors: Tutor[];
  upcomingSessions: SessionListItem[];
  stats: HomeStats;
}

type LevelFilter = GroupLevel | 'all';

const CATEGORIES: {
  key: string;
  Icon: React.ComponentType<any>;
  labels: Record<'VN' | 'EN', string>;
}[] = [
  { key: 'IELTS', Icon: Target, labels: { VN: 'Luyện IELTS', EN: 'IELTS Prep' } },
  { key: 'Business', Icon: Briefcase, labels: { VN: 'Business English', EN: 'Business English' } },
  { key: 'Kids', Icon: Baby, labels: { VN: 'Cho trẻ em', EN: 'For Kids' } },
  { key: 'General', Icon: MessagesSquare, labels: { VN: 'Giao tiếp', EN: 'Conversation' } },
  { key: 'Interview', Icon: Mic, labels: { VN: 'Phỏng vấn', EN: 'Job Interview' } },
  { key: 'Speaking', Icon: Sparkles, labels: { VN: 'Kỹ năng Nói', EN: 'Speaking' } },
];

const formatCount = (n: number) => new Intl.NumberFormat('en-US').format(n);

export function HomeContent({ featuredTutors, upcomingSessions, stats }: Props) {
  const { lang } = useLang();
  const router = useRouter();
  const [query, setQuery] = useState('');
  const [level, setLevel] = useState<LevelFilter>('all');
  const [focused, setFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const {
    items: recentSearches,
    push: pushRecent,
    clear: clearRecent,
  } = useRecentSearches(RECENT_SEARCH_KEY);

  // Keyboard shortcut: `/` focuses the hero search from anywhere on `/`,
  // Escape blurs it. Skipped when the user is already typing in an input,
  // textarea or contenteditable — otherwise `/` would eat text as they type.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName?.toLowerCase();
      const isTyping = tag === 'input' || tag === 'textarea' || target?.isContentEditable === true;
      if (e.key === '/' && !isTyping) {
        e.preventDefault();
        inputRef.current?.focus();
        return;
      }
      if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        inputRef.current?.blur();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  const copy = {
    VN: {
      hero: 'Học tiếng Anh 1-1',
      accent: 'qua video call',
      sub: 'Đặt buổi luyện nói cộng đồng miễn phí hoặc đặt lịch 1-1 với giáo viên có chứng chỉ — tất cả trên trình duyệt.',
      searchPlaceholder: 'Tìm buổi luyện nói theo chủ đề, level, host...',
      searchAria: 'Tìm buổi luyện nói',
      searchButton: 'Tìm',
      recentTitle: 'Tìm kiếm gần đây',
      recentClear: 'Xóa',
      levelAll: 'Mọi cấp độ',
      levelA1: 'A1–A2',
      levelB1: 'B1–B2',
      levelC1: 'C1–C2',
      trustTutors: 'giáo viên',
      trustSessions: 'buổi đã dạy',
      trustRating: 'điểm trung bình',
      sessionsTitle: 'Sắp diễn ra',
      sessionsSub: 'Các buổi trò chuyện cộng đồng miễn phí — chọn phòng và tham gia.',
      viewAllSessions: 'Xem tất cả',
      categories: 'Chuyên đề học',
      featured: 'Giáo viên nổi bật',
      viewAll: 'Xem tất cả',
      how: 'Bắt đầu trong 3 bước',
      s1t: 'Tìm buổi phù hợp',
      s1d: 'Chọn chủ đề, level, khung giờ tiện cho bạn.',
      s2t: 'Đặt lịch',
      s2d: 'Xác nhận chỗ trong vài giây, không cần thanh toán.',
      s3t: 'Vào lớp',
      s3d: 'Tham gia phòng học video ngay trên trình duyệt.',
    },
    EN: {
      hero: 'Learn English 1-on-1',
      accent: 'live on video',
      sub: 'Drop into free community speaking rooms or book 1-on-1 with certified tutors — right in your browser.',
      searchPlaceholder: 'Search sessions by topic, level, host...',
      searchAria: 'Search sessions',
      searchButton: 'Search',
      recentTitle: 'Recent searches',
      recentClear: 'Clear',
      levelAll: 'All levels',
      levelA1: 'A1–A2',
      levelB1: 'B1–B2',
      levelC1: 'C1–C2',
      trustTutors: 'tutors',
      trustSessions: 'lessons taught',
      trustRating: 'average rating',
      sessionsTitle: 'Upcoming sessions',
      sessionsSub: 'Free community speaking rooms — pick a slot and hop in.',
      viewAllSessions: 'View all',
      categories: 'Learning topics',
      featured: 'Featured tutors',
      viewAll: 'View all',
      how: 'Start in 3 steps',
      s1t: 'Find a session',
      s1d: 'Pick a topic, level and time that works for you.',
      s2t: 'Reserve a seat',
      s2d: 'Confirm your spot in seconds — no payment needed.',
      s3t: 'Join & speak',
      s3d: 'Enter the live video room from any browser.',
    },
  } as const;
  const t = copy[lang];

  const levelOptions: { key: LevelFilter; label: string }[] = [
    { key: 'all', label: t.levelAll },
    { key: 'a1_a2', label: t.levelA1 },
    { key: 'b1_b2', label: t.levelB1 },
    { key: 'c1_c2', label: t.levelC1 },
  ];

  const navigateToSessions = (q: string, lvl: LevelFilter) => {
    const params = new URLSearchParams();
    const trimmed = q.trim();
    if (trimmed) {
      params.set('q', trimmed);
      pushRecent(trimmed);
    }
    if (lvl !== 'all') params.set('level', lvl);
    const qs = params.toString();
    router.push(qs ? `/sessions?${qs}` : '/sessions');
  };

  const submitSearch = (e?: React.FormEvent) => {
    e?.preventDefault();
    navigateToSessions(query, level);
  };

  const pickRecent = (value: string) => {
    setQuery(value);
    setFocused(false);
    navigateToSessions(value, level);
  };

  // Dropdown visibility: only when the input is focused, the query is empty
  // (so we don't cover live typing) and we actually have something to show.
  const showRecent = focused && query.trim() === '' && recentSearches.length > 0;

  const steps = [
    { n: 1, Icon: Search, title: t.s1t, desc: t.s1d },
    { n: 2, Icon: CalendarPlus, title: t.s2t, desc: t.s2d },
    { n: 3, Icon: PlayCircle, title: t.s3t, desc: t.s3d },
  ];

  const sessionPreview = upcomingSessions.slice(0, 3);

  const trustPills = useMemo(
    () =>
      [
        stats.tutorsApproved > 0 && {
          icon: GraduationCap,
          value: formatCount(stats.tutorsApproved),
          label: t.trustTutors,
        },
        stats.sessionsCompleted > 0 && {
          icon: Users,
          value: `${formatCount(stats.sessionsCompleted)}+`,
          label: t.trustSessions,
        },
        stats.ratingAvg > 0 && {
          icon: Star,
          value: stats.ratingAvg.toFixed(1),
          label: t.trustRating,
        },
      ].filter(Boolean) as { icon: any; value: string; label: string }[],
    [stats, t],
  );

  return (
    <div className="space-y-20 md:space-y-28">
      {/* Hero — single-purpose search: find a session. Everything else on the
          page is browsing; this is the primary action. */}
      <motion.section
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-8 pt-6 text-center"
      >
        <div className="mx-auto max-w-3xl space-y-5">
          <h1 className="text-brand dark:text-accent-warm font-serif text-4xl leading-[1.05] font-bold tracking-tight sm:text-5xl md:text-6xl">
            {t.hero}
            <br />
            <span className="italic">{t.accent}</span>
          </h1>
          <p className="mx-auto max-w-xl text-base leading-relaxed text-slate-600 sm:text-lg dark:text-slate-300">
            {t.sub}
          </p>
        </div>

        {/* Session search bar. Search icon + input + level select + submit. On
            mobile the button collapses to the icon so the row stays on one
            line without wrapping. Wrapped in a relative container so the
            recent-searches dropdown can anchor to it. */}
        <div className="relative mx-auto w-full max-w-2xl">
          <form
            onSubmit={submitSearch}
            role="search"
            aria-label={t.searchAria}
            className="focus-within:ring-brand/30 flex w-full items-stretch gap-2 rounded-2xl border border-slate-200 bg-white p-1.5 shadow-sm ring-1 ring-slate-100 transition focus-within:ring-2 dark:border-slate-800 dark:bg-slate-900 dark:ring-slate-800"
          >
            <div className="relative flex flex-1 items-center">
              <Search
                className="pointer-events-none absolute left-3 size-4 text-slate-400"
                aria-hidden
              />
              <input
                ref={inputRef}
                type="search"
                role="combobox"
                aria-autocomplete="list"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onFocus={() => setFocused(true)}
                // relatedTarget lets a click on a dropdown item register before
                // we tear the dropdown down.
                onBlur={(e) => {
                  const next = e.relatedTarget as HTMLElement | null;
                  if (next?.closest('[data-recent-searches]')) return;
                  setFocused(false);
                }}
                placeholder={t.searchPlaceholder}
                aria-label={t.searchAria}
                aria-expanded={showRecent}
                aria-controls="hero-recent-searches"
                aria-keyshortcuts="/"
                className="h-11 w-full rounded-xl bg-transparent pr-10 pl-9 text-sm text-slate-900 placeholder:text-slate-400 focus:outline-none dark:text-slate-100"
              />
              {/* Keyboard shortcut hint. Hidden on touch (no keyboard),
                  hidden once the input is focused (kbd is already engaged),
                  hidden while typing (would collide with the value). */}
              {!focused && !query && (
                <kbd
                  aria-hidden
                  className="pointer-events-none absolute right-2 hidden h-5 items-center rounded border border-slate-200 bg-slate-50 px-1.5 font-mono text-[10px] font-semibold text-slate-500 sm:inline-flex dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400"
                >
                  /
                </kbd>
              )}
            </div>
            <div className="hidden items-center gap-1.5 sm:flex">
              <div className="h-5 w-px bg-slate-200 dark:bg-slate-700" aria-hidden />
              <Select value={level} onValueChange={(v) => setLevel(v as LevelFilter)}>
                <SelectTrigger
                  aria-label="Level filter"
                  className="h-11 w-36 rounded-xl border-slate-100 bg-slate-50 text-xs font-semibold text-slate-700 focus-visible:ring-0 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-200"
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {levelOptions.map((o) => (
                    <SelectItem key={o.key} value={o.key} className="text-xs">
                      {o.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <Button
              type="submit"
              aria-label={t.searchButton}
              className="h-11 shrink-0 rounded-xl px-4 sm:px-5"
            >
              <Search className="size-4 sm:hidden" aria-hidden />
              <span className="hidden sm:inline">{t.searchButton}</span>
            </Button>
          </form>

          {showRecent && (
            <div
              id="hero-recent-searches"
              data-recent-searches
              role="listbox"
              aria-label={t.recentTitle}
              className="absolute top-full right-0 left-0 z-20 mt-2 overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-lg dark:border-slate-800 dark:bg-slate-900"
            >
              <div className="flex items-center justify-between border-b border-slate-100 px-4 py-2 text-left dark:border-slate-800">
                <span className="text-[11px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                  {t.recentTitle}
                </span>
                <button
                  type="button"
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => {
                    clearRecent();
                    inputRef.current?.focus();
                  }}
                  className="hover:text-brand text-[11px] font-semibold text-slate-500"
                >
                  {t.recentClear}
                </button>
              </div>
              <ul>
                {recentSearches.map((r) => (
                  <li key={r}>
                    <button
                      type="button"
                      role="option"
                      aria-selected={false}
                      // onMouseDown fires before the input's onBlur — using it
                      // (with preventDefault) means the click on the item picks
                      // the value before the dropdown collapses.
                      onMouseDown={(e) => {
                        e.preventDefault();
                        pickRecent(r);
                      }}
                      className="hover:bg-brand/5 flex w-full items-center gap-2 px-4 py-2.5 text-left text-sm text-slate-700 transition-colors dark:text-slate-200 dark:hover:bg-slate-800/60"
                    >
                      <Clock className="size-3.5 shrink-0 text-slate-400" aria-hidden />
                      <span className="truncate">{r}</span>
                      <X
                        className="ml-auto size-3 text-slate-300 opacity-0 transition-opacity group-hover:opacity-100"
                        aria-hidden
                      />
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>

        {/* Level picker on mobile — the inline select above is hidden < sm. */}
        <div className="mx-auto flex max-w-2xl flex-wrap justify-center gap-1.5 sm:hidden">
          {levelOptions.map((o) => (
            <FilterPill
              key={o.key}
              active={level === o.key}
              onClick={() => setLevel(o.key)}
              label={o.label}
            />
          ))}
        </div>

        {trustPills.length > 0 && (
          <div className="flex flex-wrap items-center justify-center gap-x-8 gap-y-2 pt-2 text-xs text-slate-500 dark:text-slate-400">
            {trustPills.map((p, i) => (
              <span key={i} className="inline-flex items-center gap-1.5">
                <p.icon className="text-brand dark:text-accent-warm size-4 shrink-0" />
                <span className="font-semibold text-slate-800 tabular-nums dark:text-slate-100">
                  {p.value}
                </span>
                {p.label}
              </span>
            ))}
          </div>
        )}
      </motion.section>

      {/* Upcoming sessions preview */}
      {sessionPreview.length > 0 && (
        <section className="space-y-6">
          <div className="flex items-end justify-between gap-4">
            <div>
              <h2 className="font-serif text-xl font-bold text-slate-900 md:text-2xl dark:text-slate-100">
                {t.sessionsTitle}
              </h2>
              <p className="mt-1 max-w-xl text-sm text-slate-500 dark:text-slate-400">
                {t.sessionsSub}
              </p>
            </div>
            <Link
              href="/sessions"
              className="focus-ring text-brand dark:text-accent-warm -mx-2 -my-2 inline-flex shrink-0 items-center gap-1 rounded-md px-2 py-2 text-xs font-semibold hover:underline"
            >
              {t.viewAllSessions}
              <ArrowUpRight className="size-3" />
            </Link>
          </div>
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {sessionPreview.map((s, i) => (
              <GroupSessionCard key={s.id} session={s} index={i} />
            ))}
          </div>
        </section>
      )}

      {/* Categories — light, subtle tile row rather than heavy cards. Each tile
          links into the tutors listing filtered by specialty (categories are a
          tutor concept, not a session concept). */}
      <section className="space-y-6">
        <div className="flex items-end justify-between gap-4">
          <h2 className="font-serif text-xl font-bold text-slate-900 md:text-2xl dark:text-slate-100">
            {t.categories}
          </h2>
        </div>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
          {CATEGORIES.map((c) => (
            <Link
              key={c.key}
              href={`/tutors?specialty=${c.key}`}
              className="focus-ring group hover:border-brand/40 flex flex-col items-center gap-2 rounded-2xl border border-slate-200 bg-white p-4 transition-all hover:-translate-y-0.5 hover:shadow-md dark:border-slate-800 dark:bg-slate-900"
            >
              <div className="bg-brand/10 text-brand dark:bg-brand/15 group-hover:bg-brand flex size-11 items-center justify-center rounded-xl transition-colors group-hover:text-white">
                <c.Icon className="size-5" />
              </div>
              <span className="text-center text-xs font-semibold text-slate-700 dark:text-slate-200">
                {c.labels[lang]}
              </span>
            </Link>
          ))}
        </div>
      </section>

      {/* Featured tutors */}
      {featuredTutors.length > 0 && (
        <section className="space-y-6">
          <div className="flex items-end justify-between gap-4">
            <h2 className="font-serif text-xl font-bold text-slate-900 md:text-2xl dark:text-slate-100">
              {t.featured}
            </h2>
            <Link
              href="/tutors"
              className="focus-ring text-brand dark:text-accent-warm -mx-2 -my-2 inline-flex items-center gap-1 rounded-md px-2 py-2 text-xs font-semibold hover:underline"
            >
              {t.viewAll}
              <ArrowUpRight className="size-3" />
            </Link>
          </div>
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {featuredTutors.map((tt, i) => (
              <TutorCard key={tt.id} tutor={tt} index={i} />
            ))}
          </div>
        </section>
      )}

      {/* How it works */}
      <section className="space-y-6">
        <h2 className="text-center font-serif text-xl font-bold text-slate-900 md:text-2xl dark:text-slate-100">
          {t.how}
        </h2>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
          {steps.map((s, i) => (
            <motion.div
              key={s.n}
              // Non-zero initial opacity so a slow-JS device or a
              // fast-scroll capture never lands on a blank frame — the
              // stagger reads as a subtle fade-in rather than a
              // "did-the-page-break?" pause.
              initial={{ opacity: 0.35, y: 8 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-80px' }}
              transition={{ delay: i * 0.08 }}
              className="relative space-y-3 overflow-hidden rounded-2xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900"
            >
              {/* Decorative step number — purely visual, clips to the card edge */}
              <span
                aria-hidden
                className="pointer-events-none absolute -top-3 right-3 font-serif text-[7rem] leading-none font-bold text-slate-100 select-none dark:text-slate-800"
              >
                {s.n}
              </span>
              <div className="bg-brand/10 dark:bg-brand/20 text-brand flex size-11 items-center justify-center rounded-xl">
                <s.Icon className="size-5" />
              </div>
              <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100">
                {s.title}
              </h3>
              <p className="text-sm leading-relaxed text-slate-500 dark:text-slate-400">{s.desc}</p>
            </motion.div>
          ))}
        </div>
      </section>
    </div>
  );
}
