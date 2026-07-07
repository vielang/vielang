'use client';

import Link from 'next/link';
import { BrandMark } from './BrandMark';
import { Mail } from 'lucide-react';
import { useLang, useSiteSettings } from '@/contexts';

// Lucide-react ships no brand icons — inline minimal monochrome SVG so we
// don't ship a second icon library just for socials.
const FacebookIcon = (props: React.SVGProps<SVGSVGElement>) => (
  <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
    <path d="M22 12a10 10 0 1 0-11.563 9.875v-6.984h-2.54V12h2.54V9.797c0-2.506 1.492-3.89 3.776-3.89 1.094 0 2.238.195 2.238.195v2.46h-1.26c-1.243 0-1.63.772-1.63 1.563V12h2.773l-.443 2.89h-2.33v6.985A10.002 10.002 0 0 0 22 12Z" />
  </svg>
);

const InstagramIcon = (props: React.SVGProps<SVGSVGElement>) => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2}
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden="true"
    {...props}
  >
    <rect width="20" height="20" x="2" y="2" rx="5" />
    <path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z" />
    <line x1="17.5" y1="6.5" x2="17.51" y2="6.5" />
  </svg>
);

const YoutubeIcon = (props: React.SVGProps<SVGSVGElement>) => (
  <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
    <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.546 15.568V8.432L15.818 12l-6.273 3.568z" />
  </svg>
);

// Kept intentionally small. Golf-era Toss legal block + payment logo strip +
// business-license section are gone — they were compliance requirements for
// a Korean payment gateway we aren't shipping.
type Group = {
  eyebrowVn: string;
  eyebrowEn: string;
  links: { labelVn: string; labelEn: string; href: string }[];
};

const GROUPS: Group[] = [
  {
    eyebrowVn: 'Sản phẩm',
    eyebrowEn: 'Product',
    links: [
      { labelVn: 'Tìm giáo viên', labelEn: 'Find a tutor', href: '/tutors' },
      { labelVn: 'Tin tức', labelEn: 'News', href: '/news' },
    ],
  },
  {
    eyebrowVn: 'Hỗ trợ',
    eyebrowEn: 'Support',
    links: [
      { labelVn: 'Liên hệ', labelEn: 'Contact', href: '/contact' },
      { labelVn: 'FAQ', labelEn: 'FAQ', href: '/faq' },
    ],
  },
  {
    eyebrowVn: 'Pháp lý',
    eyebrowEn: 'Legal',
    links: [
      { labelVn: 'Điều khoản', labelEn: 'Terms', href: '/terms' },
      { labelVn: 'Bảo mật', labelEn: 'Privacy', href: '/privacy' },
      { labelVn: 'Hoàn tiền', labelEn: 'Refund policy', href: '/refund' },
    ],
  },
];

export function Footer() {
  const { lang } = useLang();
  const { settings } = useSiteSettings();
  const year = new Date().getFullYear();

  const socials = [
    { Icon: FacebookIcon, href: settings.facebookUrl, label: 'Facebook' },
    { Icon: InstagramIcon, href: settings.instagramUrl, label: 'Instagram' },
    { Icon: YoutubeIcon, href: settings.youtubeUrl, label: 'YouTube' },
  ].filter((s) => s.href);

  const tagline =
    lang === 'VN'
      ? 'Học tiếng Anh 1-1 với giáo viên chất lượng — bất cứ lúc nào, ở bất cứ đâu.'
      : 'Live 1-on-1 English tutoring with certified teachers — anytime, anywhere.';

  return (
    <footer className="border-t border-slate-900 bg-slate-950 text-slate-400">
      <div className="mx-auto max-w-7xl px-4 py-12 md:px-6 md:py-16">
        <div className="grid grid-cols-1 gap-10 md:grid-cols-[2fr_1fr_1fr_1fr]">
          {/* Brand */}
          <div className="max-w-sm space-y-4">
            <Link
              href="/"
              className="focus-ring inline-flex items-center gap-2.5 rounded-md"
              aria-label="VieLang home"
            >
              <BrandMark className="text-accent-warm size-9" aria-label="" />
              <span className="text-lg font-bold tracking-tight text-white">
                Vie<span className="text-accent-warm">Lang</span>
              </span>
            </Link>
            <p className="text-sm leading-relaxed text-slate-400">{tagline}</p>
            {settings.email && (
              <a
                href={`mailto:${settings.email}`}
                className="focus-ring -my-1 inline-flex items-center gap-2 rounded-md py-1.5 text-sm text-slate-300 transition-colors hover:text-white"
              >
                <Mail className="text-accent-warm size-4" aria-hidden />
                {settings.email}
              </a>
            )}
            {socials.length > 0 && (
              <div className="flex gap-2 pt-1">
                {socials.map(({ Icon, href, label }) => (
                  <a
                    key={label}
                    href={href}
                    target="_blank"
                    rel="noopener noreferrer"
                    aria-label={label}
                    className="focus-ring inline-flex size-9 items-center justify-center rounded-lg bg-slate-800/60 text-slate-400 transition-colors hover:bg-slate-800 hover:text-white"
                  >
                    <Icon className="size-4" />
                  </a>
                ))}
              </div>
            )}
          </div>

          {GROUPS.map((g) => (
            <div key={g.eyebrowEn} className="space-y-3">
              <h4 className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase">
                {lang === 'VN' ? g.eyebrowVn : g.eyebrowEn}
              </h4>
              {/* -space-y-1 counter-flows the extra vertical padding on each
                  Link so the visual footer height stays roughly the same
                  while the tap target grows from ~17px to ~40px — enough to
                  clear the "close enough to 44" WCAG 2.2 bar without
                  ballooning the layout. */}
              <ul className="-space-y-1">
                {g.links.map((l) => (
                  <li key={l.href}>
                    <Link
                      href={l.href}
                      className="focus-ring -mx-2 inline-block rounded-md px-2 py-2 text-sm text-slate-400 transition-colors hover:text-white"
                    >
                      {lang === 'VN' ? l.labelVn : l.labelEn}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-12 flex flex-col items-start justify-between gap-2 border-t border-slate-900 pt-6 sm:flex-row sm:items-center">
          <p className="text-xs text-slate-500">
            © {year} VieLang. {lang === 'VN' ? 'Bảo lưu mọi quyền.' : 'All rights reserved.'}
          </p>
          <p className="text-xs text-slate-500">
            {lang === 'VN'
              ? 'Được xây dựng cho học viên Việt Nam.'
              : 'Built for Vietnamese learners.'}
          </p>
        </div>
      </div>
    </footer>
  );
}
