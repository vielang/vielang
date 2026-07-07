'use client';

import { useRouter, usePathname, useSearchParams } from 'next/navigation';
import { useCallback, useState, useTransition } from 'react';
import { ChevronsUpDown, SlidersHorizontal, X } from 'lucide-react';
import { useLang } from '@/contexts';
import { FilterPill } from '@/components/shared/FilterPill';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Sheet,
  SheetContent,
  SheetTrigger,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetFooter,
  SheetClose,
} from '@/components/ui/sheet';

const SPECIALTIES = ['IELTS', 'Business', 'Kids', 'Speaking', 'Interview', 'General'];

const SORTS: { key: string; labels: Record<'VN' | 'EN', string> }[] = [
  { key: 'rating', labels: { VN: 'Đánh giá cao nhất', EN: 'Top rated' } },
  { key: 'price_asc', labels: { VN: 'Giá tăng dần', EN: 'Price: low to high' } },
  { key: 'price_desc', labels: { VN: 'Giá giảm dần', EN: 'Price: high to low' } },
  { key: 'experience', labels: { VN: 'Nhiều kinh nghiệm', EN: 'Most experienced' } },
  { key: 'newest', labels: { VN: 'Mới nhất', EN: 'Newest' } },
];

const DEFAULT_SORT = 'rating';

/**
 * URL-driven filter bar. Two layouts share the same state:
 *
 *   • Desktop (md+): specialty pills + sort <select> inline. Same as before.
 *   • Mobile: a compact row with `[Filters (N)]` sheet trigger + a truncated
 *     sort chip. Full options live inside a bottom sheet — plenty of room for
 *     tap-safe pills and readable sort labels without cramping the discovery
 *     grid above the fold.
 *
 * Filters apply the moment a pill is tapped (URL query param → router.replace
 * → parent Server Component refetches). No "Apply" button is needed; the
 * sheet's "Done" is a convenience close, and "Reset" clears everything.
 */
export function TutorFilterBar() {
  const { lang } = useLang();
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const [pending, startTransition] = useTransition();
  const [sheetOpen, setSheetOpen] = useState(false);

  const currentSpecialty = searchParams.get('specialty') || '';
  const currentSort = searchParams.get('sort') || DEFAULT_SORT;

  const activeCount = (currentSpecialty ? 1 : 0) + (currentSort !== DEFAULT_SORT ? 1 : 0);

  const setParam = useCallback(
    (key: string, value: string | null) => {
      const params = new URLSearchParams(searchParams);
      if (value) params.set(key, value);
      else params.delete(key);
      const qs = params.toString();
      startTransition(() => {
        router.replace(qs ? `${pathname}?${qs}` : pathname, { scroll: false });
      });
    },
    [pathname, router, searchParams],
  );

  const resetAll = useCallback(() => {
    startTransition(() => {
      router.replace(pathname, { scroll: false });
    });
  }, [pathname, router]);

  const currentSortLabel =
    SORTS.find((s) => s.key === currentSort)?.labels[lang] ?? SORTS[0].labels[lang];

  return (
    <div className={`space-y-4 ${pending ? 'opacity-60' : ''} transition-opacity`}>
      {/* ------------------------------------------------------------------
          Desktop layout (md+) — untouched behaviour from the original bar.
          ------------------------------------------------------------------ */}
      <div className="hidden md:block">
        <div className="flex flex-wrap items-end gap-4">
          <div className="min-w-[240px] flex-1 space-y-1.5">
            <p className="type-eyebrow">{lang === 'VN' ? 'Chuyên môn' : 'Specialty'}</p>
            <div className="flex flex-wrap gap-1.5">
              <FilterPill
                active={!currentSpecialty}
                onClick={() => setParam('specialty', null)}
                label={lang === 'VN' ? 'Tất cả' : 'All'}
              />
              {SPECIALTIES.map((s) => (
                <FilterPill
                  key={s}
                  active={currentSpecialty === s}
                  onClick={() => setParam('specialty', s)}
                  label={s}
                />
              ))}
            </div>
          </div>

          <div className="shrink-0 space-y-1.5">
            <span className="type-eyebrow">{lang === 'VN' ? 'Sắp xếp' : 'Sort by'}</span>
            <Select value={currentSort} onValueChange={(v) => setParam('sort', v as string)}>
              <SelectTrigger className="h-9 w-48 text-xs font-semibold">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SORTS.map((s) => (
                  <SelectItem key={s.key} value={s.key} className="text-xs">
                    {s.labels[lang]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      {/* ------------------------------------------------------------------
          Mobile layout — filter sheet trigger + inline sort chip + active
          filter chips row (dismissable). Kept dense so it doesn't push the
          tutor grid below the fold on a 375px viewport.
          ------------------------------------------------------------------ */}
      <div className="space-y-2 md:hidden">
        <div className="flex items-center gap-2">
          <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
            <SheetTrigger
              render={
                <Button
                  variant="outline"
                  size="sm"
                  className="focus-ring h-10 gap-1.5 rounded-full px-4"
                  aria-label={lang === 'VN' ? 'Mở bộ lọc' : 'Open filters'}
                />
              }
            >
              <SlidersHorizontal className="size-4" />
              <span>{lang === 'VN' ? 'Bộ lọc' : 'Filters'}</span>
              {activeCount > 0 && (
                <span className="bg-brand ml-1 inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[10px] font-bold text-white tabular-nums">
                  {activeCount}
                </span>
              )}
            </SheetTrigger>

            <SheetContent
              side="bottom"
              className="pb-safe max-h-[85dvh] rounded-t-2xl border-0 p-0"
            >
              <SheetHeader className="border-b border-slate-100 pt-5 pb-3 dark:border-slate-800">
                {/* Drag-handle affordance — same visual as the Dialog primitive
                    so users learn a single pattern for both sheets. */}
                <div
                  aria-hidden="true"
                  className="bg-muted-foreground/30 mx-auto -mt-3 mb-3 h-1 w-10 rounded-full"
                />
                <SheetTitle>{lang === 'VN' ? 'Bộ lọc giáo viên' : 'Filter tutors'}</SheetTitle>
                <SheetDescription>
                  {lang === 'VN'
                    ? 'Áp dụng ngay khi bạn chọn — không cần bấm nút.'
                    : 'Choices apply as you tap — no need to hit apply.'}
                </SheetDescription>
              </SheetHeader>

              <div className="flex-1 space-y-6 overflow-y-auto p-4">
                <section className="space-y-2">
                  <h3 className="type-eyebrow">{lang === 'VN' ? 'Chuyên môn' : 'Specialty'}</h3>
                  <div className="flex flex-wrap gap-2">
                    <FilterPill
                      size="md"
                      active={!currentSpecialty}
                      onClick={() => setParam('specialty', null)}
                      label={lang === 'VN' ? 'Tất cả' : 'All'}
                    />
                    {SPECIALTIES.map((s) => (
                      <FilterPill
                        key={s}
                        size="md"
                        active={currentSpecialty === s}
                        onClick={() => setParam('specialty', s)}
                        label={s}
                      />
                    ))}
                  </div>
                </section>

                <section className="space-y-2">
                  <h3 className="type-eyebrow">{lang === 'VN' ? 'Sắp xếp' : 'Sort by'}</h3>
                  {/* Sort as tap-friendly radios inside the sheet — the desktop
                      <select> shrinks readability on a 375px screen. */}
                  <div role="radiogroup" className="flex flex-col gap-1">
                    {SORTS.map((s) => {
                      const active = currentSort === s.key;
                      return (
                        <button
                          key={s.key}
                          type="button"
                          role="radio"
                          aria-checked={active}
                          onClick={() => setParam('sort', s.key)}
                          className={`focus-ring tap-min flex items-center justify-between rounded-lg border px-4 text-sm font-medium transition-colors ${
                            active
                              ? 'border-brand bg-brand/5 text-brand dark:bg-brand/15'
                              : 'border-slate-200 bg-white text-slate-700 hover:border-slate-300 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200'
                          }`}
                        >
                          <span>{s.labels[lang]}</span>
                          {active && <span className="text-brand text-base">•</span>}
                        </button>
                      );
                    })}
                  </div>
                </section>
              </div>

              <SheetFooter className="border-t border-slate-100 dark:border-slate-800">
                <div className="flex gap-2">
                  <Button
                    variant="ghost"
                    className="flex-1"
                    onClick={resetAll}
                    disabled={activeCount === 0}
                  >
                    {lang === 'VN' ? 'Đặt lại' : 'Reset'}
                  </Button>
                  <SheetClose
                    render={<Button className="flex-1">{lang === 'VN' ? 'Xong' : 'Done'}</Button>}
                  />
                </div>
              </SheetFooter>
            </SheetContent>
          </Sheet>

          {/* Inline sort chip — one tap opens the sheet at the sort section.
              Truncated on narrow screens so long labels don't push the filter
              button off-canvas. */}
          <button
            type="button"
            onClick={() => setSheetOpen(true)}
            className="focus-ring flex h-10 min-w-0 flex-1 items-center justify-between gap-2 rounded-full border border-slate-200 bg-white px-4 text-xs font-semibold text-slate-700 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200"
          >
            <span className="truncate">{currentSortLabel}</span>
            <ChevronsUpDown className="size-3.5 shrink-0 text-slate-400" />
          </button>
        </div>

        {/* Active filter chips — one-tap remove without opening the sheet. */}
        {activeCount > 0 && (
          <div className="no-scrollbar flex gap-1.5 overflow-x-auto">
            {currentSpecialty && (
              <RemovableChip
                label={currentSpecialty}
                onRemove={() => setParam('specialty', null)}
                aria={lang === 'VN' ? 'Bỏ chuyên môn' : 'Remove specialty'}
              />
            )}
            {currentSort !== DEFAULT_SORT && (
              <RemovableChip
                label={currentSortLabel}
                onRemove={() => setParam('sort', null)}
                aria={lang === 'VN' ? 'Bỏ sắp xếp' : 'Remove sort'}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function RemovableChip({
  label,
  onRemove,
  aria,
}: {
  label: string;
  onRemove: () => void;
  aria: string;
}) {
  return (
    <span className="bg-brand/10 text-brand dark:bg-brand/20 inline-flex h-8 shrink-0 items-center gap-1 rounded-full pr-1 pl-3 text-xs font-semibold">
      {label}
      <button
        type="button"
        onClick={onRemove}
        aria-label={aria}
        className="focus-ring hover:bg-brand/20 inline-flex size-6 items-center justify-center rounded-full"
      >
        <X className="size-3.5" />
      </button>
    </span>
  );
}
