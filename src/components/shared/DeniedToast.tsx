'use client';

import { useEffect, useRef } from 'react';
import { useSearchParams, useRouter, usePathname } from 'next/navigation';
import { toast } from 'sonner';
import { useLang } from '@/contexts';

/**
 * Listens for `?denied=1` on the URL and surfaces a localized toast — fired
 * once per visit, then the param is scrubbed from the URL so a back-button
 * doesn't re-trigger it. Used as a destination signal from server-component
 * guards in /admin and /owner that redirect to home when an authenticated
 * user without the right role lands there.
 */
export function DeniedToast() {
  const params = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();
  const { lang } = useLang();
  const firedRef = useRef(false);

  useEffect(() => {
    if (firedRef.current) return;
    if (params.get('denied') !== '1') return;
    firedRef.current = true;

    toast.error(
      lang === 'VN'
        ? 'Bạn không có quyền truy cập trang này'
        : 'You do not have permission to access that page',
    );

    // Strip the marker so a refresh / back-nav doesn't re-toast.
    const next = new URLSearchParams(params.toString());
    next.delete('denied');
    const qs = next.toString();
    router.replace(qs ? `${pathname}?${qs}` : pathname, { scroll: false });
  }, [params, lang, router, pathname]);

  return null;
}
