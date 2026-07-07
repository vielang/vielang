import { cn } from '@/lib/utils';

/**
 * VieLang brand mark — inline SVG so each caller can tint it with a
 * `text-*` Tailwind class via `currentColor`. The static `public/brand-v.svg`
 * still exists for the favicon and PWA manifest (browser chrome + iOS home
 * screen render SVGs as raster images, so currentColor doesn't work there).
 *
 * Shape: a downward "V" stroke with a small amber accent dot underneath.
 * The dot is the only splash of accent colour — everything else inherits
 * from the parent so the mark reads naturally on any background.
 */
export function BrandMark({
  className,
  accent = true,
  'aria-label': ariaLabel = 'VieLang',
}: {
  className?: string;
  /** Show the amber accent dot beneath the V. Turn off for very small
   *  sizes (<24px) where the dot loses definition. */
  accent?: boolean;
  'aria-label'?: string;
}) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 512 512"
      role="img"
      aria-label={ariaLabel}
      className={cn('shrink-0', className)}
    >
      <path
        d="M 144 152 L 256 384 L 368 152"
        stroke="currentColor"
        strokeWidth={56}
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      {accent && <circle cx={256} cy={440} r={18} fill="#F59E0B" />}
    </svg>
  );
}
