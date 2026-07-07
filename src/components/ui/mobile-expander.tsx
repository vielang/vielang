'use client';

import React, { useEffect, useId, useState } from 'react';
import { ChevronDown } from 'lucide-react';

/**
 * Mobile-first disclosure: collapsed by default on small screens, always
 * expanded at ≥md so desktop reads the section inline. Controlled by JS
 * state with `aria-expanded` + `aria-controls` for proper a11y semantics —
 * native <details> overrides via UA stylesheet are too fragile to skin
 * consistently across browsers.
 *
 *   <MobileExpander title="Itinerary" defaultOpen={false} mdOpen>
 *     <Itinerary />
 *   </MobileExpander>
 *
 * `lazy` (default true) defers rendering children until the panel has been
 * opened at least once. Useful for heavy embeds (Google Maps iframe) — the
 * first paint stays fast and the embed only loads when the user asks for it.
 */
export interface MobileExpanderProps {
  title: React.ReactNode;
  subtitle?: React.ReactNode;
  icon?: React.ReactNode;
  defaultOpen?: boolean;
  /** Force open at ≥md (default true). */
  mdOpen?: boolean;
  /** Defer rendering children until first open (default true). */
  lazy?: boolean;
  className?: string;
  children: React.ReactNode;
}

function useIsDesktop(breakpointPx = 768): boolean {
  // SSR-safe initial value: assume mobile so the first paint matches the
  // collapsed state. Once hydrated, the effect flips to the real value.
  const [isDesktop, setIsDesktop] = useState(false);
  useEffect(() => {
    if (typeof window === 'undefined') return;
    const mq = window.matchMedia(`(min-width: ${breakpointPx}px)`);
    const update = () => setIsDesktop(mq.matches);
    update();
    mq.addEventListener('change', update);
    return () => mq.removeEventListener('change', update);
  }, [breakpointPx]);
  return isDesktop;
}

export function MobileExpander({
  title,
  subtitle,
  icon,
  defaultOpen = false,
  mdOpen = true,
  lazy = true,
  className = '',
  children,
}: MobileExpanderProps) {
  const isDesktop = useIsDesktop();
  const forceOpen = mdOpen && isDesktop;
  const [open, setOpen] = useState(defaultOpen);
  // Track if the user has ever opened it — controls lazy mount. On desktop
  // with mdOpen, the children are needed immediately.
  const [hasOpened, setHasOpened] = useState(defaultOpen || (mdOpen && isDesktop));
  const panelId = useId();
  const effectiveOpen = forceOpen || open;

  // Keep `hasOpened` in sync when viewport shifts to ≥md and the panel becomes
  // forced-open — otherwise the panel would mount empty on the first desktop
  // render after a mobile-resize.
  useEffect(() => {
    if (forceOpen) setHasOpened(true);
  }, [forceOpen]);

  const handleToggle = () => {
    if (forceOpen) return;
    if (!open) setHasOpened(true);
    setOpen((v) => !v);
  };

  return (
    <div
      className={`overflow-hidden rounded-xl border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900 ${className}`}
    >
      <button
        type="button"
        onClick={handleToggle}
        aria-expanded={effectiveOpen}
        aria-controls={panelId}
        // Desktop with force-open: button is non-interactive (header only).
        // Removing the role keeps the cursor/focus behavior natural.
        {...(forceOpen ? { tabIndex: -1, 'aria-disabled': true } : {})}
        className={`focus-visible:ring-primary/40 flex w-full items-center gap-3 p-4 text-left select-none focus-visible:ring-2 focus-visible:outline-none md:p-5 ${
          forceOpen ? 'cursor-default' : 'cursor-pointer'
        }`}
      >
        {icon && (
          <span className="bg-primary/10 text-primary flex size-8 shrink-0 items-center justify-center rounded-md">
            {icon}
          </span>
        )}
        <div className="min-w-0 flex-1">
          <p className="text-brand dark:text-accent-warm truncate font-serif text-sm font-bold md:text-base">
            {title}
          </p>
          {subtitle && (
            <p className="mt-0.5 truncate text-[11px] text-slate-500 dark:text-slate-400">
              {subtitle}
            </p>
          )}
        </div>
        {!forceOpen && (
          <ChevronDown
            className={`size-4 shrink-0 text-slate-400 transition-transform duration-200 ${
              effectiveOpen ? 'rotate-180' : ''
            }`}
            aria-hidden="true"
          />
        )}
      </button>
      <div
        id={panelId}
        hidden={!effectiveOpen}
        className="border-t border-slate-100 px-4 pt-1 pb-4 md:px-5 md:pb-5 dark:border-slate-800"
      >
        {!lazy || hasOpened ? children : null}
      </div>
    </div>
  );
}
