'use client';

import { createContext, useCallback, useContext, useState, type ReactNode } from 'react';

export interface SiteSettings {
  id: 'main';
  brandPrimary: string;
  brandAccent: string;
  companyName: string;
  addressKR: string;
  addressVN: string;
  addressEN: string;
  phone: string;
  email: string;
  facebookUrl: string;
  instagramUrl: string;
  youtubeUrl: string;
  copyrightText: string;
  // Legal compliance — required by the Toss merchant review, which checks the
  // footer carries the full 사업자 정보 block matching the business license:
  // 상호 / 대표 / 사업자등록번호 / 통신판매업신고 / 주소 / 고객센터 / 이메일.
  // All optional in schema for backward-compat; each row hidden when empty.
  businessRegistrationNumber?: string; // 사업자등록번호 — Mã số đăng ký kinh doanh
  representativeName?: string; // 대표자명 — Tên người đại diện
  mailOrderSalesNumber?: string; // 통신판매업 신고번호 — Số đăng ký bán hàng qua mạng
}

// Defaults match the post-migration seed so the UI has values to show
// before the network fetch resolves (avoids flash of empty footer).
const DEFAULTS: SiteSettings = {
  id: 'main',
  brandPrimary: 'Vie',
  brandAccent: 'Lang',
  companyName: 'VieLang',
  addressKR: '',
  addressVN: '',
  addressEN: '',
  phone: '',
  email: 'hello@vielang.com',
  facebookUrl: '',
  instagramUrl: '',
  youtubeUrl: '',
  copyrightText: 'VieLang. All rights reserved.',
  businessRegistrationNumber: '',
  representativeName: '',
  mailOrderSalesNumber: '',
};

interface ContextValue {
  settings: SiteSettings;
  refetch: () => Promise<void>;
  // setSettings is exposed for the admin editor's optimistic update — every
  // other caller should treat the context as read-only.
  setSettings: (next: SiteSettings) => void;
}

const SiteSettingsContext = createContext<ContextValue>({
  settings: DEFAULTS,
  refetch: async () => {
    /* no-op */
  },
  setSettings: () => {
    /* no-op */
  },
});

export function SiteSettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<SiteSettings>(DEFAULTS);

  // No network fetch during the pivot — the /api/site-settings endpoint was
  // torn down with the golf-era admin CMS. Static DEFAULTS suffice until the
  // admin site-settings editor is rebuilt (Phase 6).
  const refetch = useCallback(async () => {
    /* no-op */
  }, []);

  return (
    <SiteSettingsContext.Provider value={{ settings, refetch, setSettings }}>
      {children}
    </SiteSettingsContext.Provider>
  );
}

export const useSiteSettings = () => useContext(SiteSettingsContext);
