# Kế hoạch Migration lên chuẩn Công nghiệp — VieLang

> **Dự án:** VieLang — platform học tiếng Anh 1‑1 qua video call
> **Base:** `D:\dev\Self\vielang-v2` (Next.js 16 App Router + Supabase SSR + LiveKit + Tailwind v4 + shadcn/ui)
> **Commit tham chiếu:** `9af55ee feat(group-sessions): schema + API + homepage rework`
> **Ngày lập:** 2026-07-04
> **Bổ trợ:** đọc song song với `VIELANG_ROADMAP.md` (P0–P10 MVP) — file này là **wrap layer công nghiệp**, không thay thế.

---

## Mục lục

- [Phần A — Audit tình trạng hiện tại](#phần-a--audit-tình-trạng-hiện-tại)
  - [A.1 Ma trận trạng thái](#a1-ma-trận-trạng-thái)
  - [A.2 Gap so với chuẩn công nghiệp](#a2-gap-so-với-chuẩn-công-nghiệp)
- [Phần B — Kế hoạch Migration Backend/Foundation](#phần-b--kế-hoạch-migration-backendfoundation)
  - [M1 — Foundation Hardening](#m1--foundation-hardening-23-ngày)
  - [M2 — Quality & Testing](#m2--quality--testing-45-ngày)
  - [M3 — Observability & Security](#m3--observability--security-34-ngày)
  - [M4 — Product Completion](#m4--product-completion-57-ngày)
  - [M5 — Production Deploy](#m5--production-deploy-12-ngày)
  - [M6 — Documentation & Handover](#m6--documentation--handover-1-ngày)
- [Phần C — Migration UI/UX Responsive](#phần-c--migration-uiux-responsive)
  - [C.1 Audit UI/UX hiện tại](#c1-audit-uiux-hiện-tại)
  - [C.2 Design tokens & foundation](#c2-design-tokens--foundation)
  - [C.3 Breakpoint strategy chuẩn công nghiệp](#c3-breakpoint-strategy-chuẩn-công-nghiệp)
  - [C.4 Mobile ≤375px — checklist tối ưu](#c4-mobile-375px--checklist-tối-ưu)
  - [C.5 Large screen ≥1440px — checklist tối ưu](#c5-large-screen-1440px--checklist-tối-ưu)
  - [C.6 Component-by-component migration](#c6-component-by-component-migration)
  - [C.7 A11y & motion sensitivity](#c7-a11y--motion-sensitivity)
  - [C.8 Performance & Web Vitals](#c8-performance--web-vitals)
  - [C.9 DoD & sign-off](#c9-dod--sign-off)
- [Phần D — Rủi ro & Mitigation](#phần-d--rủi-ro--mitigation)
- [Phần E — Timeline tổng hợp](#phần-e--timeline-tổng-hợp)

---

## Phần A — Audit tình trạng hiện tại

### A.1 Ma trận trạng thái

| Layer                               | % Hoàn thành | Ghi chú                                                                                               |
| ----------------------------------- | ------------ | ----------------------------------------------------------------------------------------------------- |
| Domain pivot (golf → VieLang)       | ~92%         | Còn di sản: `StarRating.tsx`, `validate.ts`, chuỗi golf trong `translations.ts`, KR trong `error.tsx` |
| DB schema                           | 95%          | 3 migrations sạch, **RLS tắt** (chấp nhận vì service_role route handlers)                             |
| Auth (Supabase SSR + Google + demo) | 90%          | X‑Demo‑User đã gate prod, thiếu MFA/rate‑limit login                                                  |
| API route handlers                  | 90%          | 21 endpoint, đều `authenticate()` + Zod + `apiError()`                                                |
| Public routes                       | 95%          | Tutor discovery, sessions, my‑page, my‑sessions đủ                                                    |
| Admin dashboard                     | **40%**      | 4/10 sections (thiếu Tutors/Courses/Banners/Content/Reviews/Settings)                                 |
| Tutor dashboard                     | 90%          | 5/5 sections, thiếu MyCourses + MyReviews                                                             |
| Booking/availability                | 85%          | Chạy được, **VN timezone hardcode UTC+7**                                                             |
| LiveKit                             | 70%          | Token + webhook + docker‑dev đủ, **thiếu component wrapper**                                          |
| Group sessions                      | 80%          | Schema + API + homepage widget xong                                                                   |
| PWA                                 | 60%          | `sw.js` còn cache `/api/stay-and-play`, `/api/golf-tours`; thiếu install prompt + offline banner      |
| i18n VN/EN                          | 80%          | ~1600 dòng, còn lẫn key golf & legacy Kakao                                                           |
| Design system                       | 85%          | 31 primitives shadcn, tokens indigo/amber, **không có tailwind.config.\*** (v4 CSS-only)              |
| A11y                                | 70%          | 92 aria‑\* trên 38 files, thiếu review Breadcrumbs/FilterPill/Dialog                                  |
| Testing                             | **0%**       | Không có test framework                                                                               |
| CI/CD                               | **0%**       | Không có `.github/workflows/`, `vercel.json`, `Caddyfile` prod                                        |
| Observability                       | **0%**       | Chỉ `console.error()`, không Sentry, không structured logger                                          |
| Security headers                    | 10%          | Chỉ COOP/COEP `/game/*` (di sản), thiếu CSP/HSTS                                                      |
| Performance                         | 60%          | `optimizePackageImports` bật, image `remotePatterns` an toàn, **thiếu next/font**, bundle analyzer    |
| Docs                                | 70%          | SETUP.md, ROADMAP, CLAUDE.md; thiếu ADR/runbook                                                       |

### A.2 Gap so với chuẩn công nghiệp

**Blocker cho production**

1. Không có testing (unit + e2e) — mọi refactor đều mù.
2. Không có Sentry/observability — prod fail không ai biết.
3. Không CI (typecheck/lint/test/build gate trước merge).
4. Không có RLS + audit log (dữ liệu học viên PII cần trace).
5. Không có prod deployment artifact (`vercel.json`, `Caddyfile`, `docker-compose.prod.yml`).
6. Không có rate limit trên auth/booking API.
7. `tsconfig strict=false` — miss type bug.
8. Không có prettier + husky + lint‑staged.

**Debt cần trả**

- SW cache invalidate golf routes.
- Cleanup `validate.ts`, `translations.ts`, `error.tsx` KR copy.
- Xoá `/owner` route + `StarRating.tsx`.
- Missing admin sections (Tutors/Courses/Banners/Content/Reviews/Settings).
- LiveKit wrapper (`<VideoRoom session={} />`).

**Enterprise gap**

- Multi‑timezone (hardcode UTC+7).
- Notification pipeline (email/SMS reminder T-24h, T-1h).
- Payment gateway (VNPay/Stripe).
- Recording / whiteboard / group class UX.
- Feature flags, backup, SLA/error‑budget dashboard.

---

## Phần B — Kế hoạch Migration Backend/Foundation

Chia làm **4 track song song**, gói vào **6 milestone**.

| Track                                  | Priority | Effort   |
| -------------------------------------- | -------- | -------- |
| **T1 — Foundation Hardening** (M1)     | P0       | 2‑3 ngày |
| **T2 — Quality & Testing** (M2)        | P0       | 4‑5 ngày |
| **T3 — Observability & Security** (M3) | P0       | 3‑4 ngày |
| **T4 — Product Completion** (M4)       | P1       | 5‑7 ngày |

---

### M1 — Foundation Hardening (2‑3 ngày)

**Mục tiêu:** repo sạch legacy, mọi commit đi qua gate CI.

#### M1.1 Legacy cleanup (0.5 ngày)

- Xoá `src/app/owner/`, `src/components/customer/StarRating.tsx`.
- Xoá `src/lib/validate.ts` → thay bằng zod.
- Refactor `src/app/(main)/error.tsx`: bỏ chuỗi KR "문제가 발생했습니다", dùng `translations.ts`.
- `src/lib/translations.ts`: xoá key `kakaoModalTitle`, `continueWithKakao/Naver`, `teeTime`, `stayPlay`, `elitePass`, `golfTours`, `priceList`.
- `public/sw.js`: cập nhật `RUNTIME_ROUTES` (bỏ `/api/stay-and-play`, `/api/golf-tours`), thêm `/api/tutors`, `/api/sessions` (GET public). Bump `CACHE_VERSION='v2'`.
- `next.config.ts`: bỏ COOP/COEP `/game/*`.

#### M1.2 TypeScript strict (0.5 ngày)

- Bật `"strict": true`, `"noUncheckedIndexedAccess": true`, `"exactOptionalPropertyTypes": true`.
- `npx supabase gen types typescript --local > src/types/database.ts`.
- Wrap Supabase queries với `.returns<Database['public']['Tables']['users']['Row'][]>()`.
- Fix lỗi từng nhóm (dự kiến 30‑80 issue quanh Supabase response typing).

#### M1.3 Formatter + hooks (0.5 ngày)

- `prettier`, `eslint-config-prettier`, `@typescript-eslint/*`, `eslint-plugin-tailwindcss`.
- `.prettierrc.json`: `{ "semi": true, "singleQuote": true, "printWidth": 100 }`.
- `eslint.config.mjs` (flat): next/core‑web‑vitals + prettier + strict‑boolean‑expressions.
- `husky` + `lint-staged`:
  ```json
  {
    "*.{ts,tsx}": ["prettier --write", "eslint --fix"],
    "*.{md,json,css}": ["prettier --write"]
  }
  ```
- `.husky/commit-msg`: Conventional Commits qua `@commitlint/cli`.

#### M1.4 CI pipeline (0.5‑1 ngày)

- `.github/workflows/ci.yml`: matrix Node 20 — `install → lint → typecheck → build`.
- `.github/workflows/pr-preview.yml`: Vercel Preview trigger.
- `vercel.json`:
  ```json
  { "buildCommand": "next build", "framework": "nextjs", "regions": ["sin1"] }
  ```

**DoD M1:** `git push` chạy CI xanh; PR không pass lint/type/build sẽ block.

---

### M2 — Quality & Testing (4‑5 ngày)

**Mục tiêu:** coverage ≥ 60% cho `src/lib/` + `src/app/api/`, smoke E2E happy path.

#### M2.1 Vitest setup (0.5 ngày)

- `vitest`, `@vitest/coverage-v8`, `jsdom`, `@testing-library/react`, `@testing-library/jest-dom`, `msw`.
- `vitest.config.ts` alias `@/*`, `setupFiles`.
- Scripts: `test`, `test:watch`, `test:coverage`.

#### M2.2 Unit tests business logic (1.5 ngày)

- `src/lib/booking.test.ts`: `getAvailableSlots` — no availability, overlapping session, boundary DST, past slot filter.
- `src/lib/schemas/session.test.ts`: discriminated union private vs group, reject capacity, reschedule action refine.
- `src/lib/promo.test.ts`: percent vs fixed, clamp negative, expired, per‑user usage.
- `src/lib/auth-server.test.ts`: authenticate() branches (cookie/bearer/demo/prod‑block), requireRole, requireOwnership admin bypass.
- `src/lib/api-errors.test.ts`: response shape, no leak stack in prod.

#### M2.3 Component tests (1 ngày)

- `LoginPage.test.tsx`: invalid submit → error UI, Google OAuth click.
- `session/BookingDialog.test.tsx`: step flow, slot disable, submit payload shape.
- `admin/PendingTutorsSection.test.tsx`: approve/reject optimistic UI.

#### M2.4 E2E Playwright (1.5 ngày)

- `@playwright/test`, config 3 project (chromium, mobile‑chrome, mobile‑safari).
- Global setup seed DB qua `POST /api/seed`.
- `e2e/`:
  - `auth.spec.ts` — email login redirect theo role.
  - `student-book-session.spec.ts` — browse → detail → book → `/my-sessions`.
  - `tutor-availability.spec.ts` — tutor set slot → student thấy.
  - `admin-approve-tutor.spec.ts` — pending → approve → public list.
  - `video-room-token.spec.ts` — mock LiveKit, verify token gated by join window.
- CI job `e2e` chạy Supabase local via `supabase start`.

#### M2.5 Coverage gate

- CI fail nếu coverage `src/lib/**` + `src/app/api/**` < 60%.
- Badge trong README.

**DoD M2:** `npm test` xanh, Playwright 5 flow chính, PR gate cả unit + e2e.

---

### M3 — Observability & Security (3‑4 ngày)

#### M3.1 Structured logging (0.5 ngày)

- `pino` + `pino-pretty` (dev).
- `src/lib/logger.ts`: singleton, `logger.child({ traceId })` per request qua `middleware.ts`.
- Thay tất cả `console.error()` bằng `logger.error({ err, scope }, msg)`.
- Vercel log drains → Axiom/Datadog optional.

#### M3.2 Sentry (0.5 ngày)

- `npx @sentry/wizard@latest -i nextjs`.
- Config `sentry.{client,server,edge}.config.ts`.
- Env `SENTRY_DSN`, `SENTRY_AUTH_TOKEN` (upload source map).
- Wrap `apiError()`: `Sentry.captureException(err, { tags: { scope } })`.
- `error.tsx` client: `Sentry.captureException` + user feedback dialog.

#### M3.3 Rate limiting (0.5 ngày)

- `@upstash/ratelimit` + Upstash Redis (hoặc reuse LiveKit droplet Redis).
- `src/lib/rate-limit.ts` — token bucket.
- Apply:
  - `POST /api/sessions` — 5/min per user.
  - `POST /api/reviews` — 3/min per user.
  - `/api/livekit/token` — 10/min per user.
  - `/auth/callback`, login — 20/min per IP.
- Response 429 + `Retry-After`.

#### M3.4 RLS enable + audit log (1 ngày)

Migration `20260710_enable_rls.sql`:

```sql
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
CREATE POLICY "user reads self" ON users FOR SELECT USING (supabase_uid = auth.uid());
CREATE POLICY "public reads approved tutors" ON tutor_profiles FOR SELECT USING (is_approved = true);
-- Policies cho sessions, reviews (student/tutor tự thấy)
```

- Table `audit_log(id, user_id, action, entity, entity_id, before, after, ip, ua, created_at)`.
- Middleware wrap mutations: session create/patch, role change, tutor approve.
- Test: chạy anon key song song service key, verify không xem chéo.

#### M3.5 Security headers (0.5 ngày)

- `next.config.ts` headers:
  ```
  Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' *.supabase.co;
    connect-src 'self' *.supabase.co wss://livekit.vielang.com;
    img-src 'self' data: https:; frame-ancestors 'none'
  Strict-Transport-Security: max-age=63072000; includeSubDomains; preload
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: camera=(self), microphone=(self), geolocation=()
  ```
- Verify Supabase cookie `secure=true, httpOnly=true, sameSite=lax`.

#### M3.6 Health & readiness (0.5 ngày)

- `/api/health` mở rộng: check Supabase, Redis, LiveKit HTTP ping.
- `/api/ready` — 200 chỉ khi tất cả deps OK.
- Vercel cron / Uptime Kuma ping.

**DoD M3:** Sentry nhận event thật; RLS ON; `securityheaders.com` A+.

---

### M4 — Product Completion (5‑7 ngày)

#### M4.1 Admin sections còn thiếu (2 ngày)

- `TutorsSection.tsx` — list, ban/enable, filter search.
- `CoursesSection.tsx` — CRUD, publish/unpublish, filter theo tutor.
- `BannersManager.tsx` — CRUD banner homepage.
- `ContentSection.tsx` — news CRUD + static pages (Tiptap).
- `ReviewsSection.tsx` — moderation (hide/delete).
- `SettingsSection.tsx` — brand, hotline, chat toggle.
- API: `GET /api/admin/tutors`, CRUD `courses`, `banners`, `news`, `reviews`, `site-settings`.

#### M4.2 Tutor dashboard hoàn thiện (0.5 ngày)

- `MyCoursesSection.tsx`, `MyReviewsSection.tsx`.

#### M4.3 LiveKit component wrapper (1 ngày)

- `src/components/video/VideoRoom.tsx`:
  ```tsx
  <LiveKitRoom token={} serverUrl={} onDisconnected={} onError={}>
    <VideoConference />
    <RoomAudioRenderer />
    <ChatToggle />
    <StartAudio />
  </LiveKitRoom>
  ```
- Pre‑join device check + reconnect UX + fallback outside_window (403).

#### M4.4 Notifications pipeline (1.5 ngày)

- Resend SDK + React Email.
- Templates: SessionConfirmation, Reminder24h, Reminder1h, ReviewRequest.
- Cron `/api/cron/reminders` (Vercel Cron, Bearer `CRON_SECRET`).
- Table `notification_log(id, session_id, kind, sent_at, provider_id)`.

#### M4.5 i18n cleanup + timezone (0.5 ngày)

- Grep `translations.ts` xoá key golf‑only.
- `src/lib/booking.ts` thay hardcode UTC+7 bằng `user.timezone` qua `date-fns-tz`.

#### M4.6 Fonts + performance (0.5 ngày)

- `next/font/google` cho Inter/Manrope.
- `@next/bundle-analyzer` — script `analyze`: `ANALYZE=true next build`.
- Target Lighthouse Performance ≥ 90, PWA installable.

#### M4.7 Payment placeholder (1 ngày, optional post‑MVP)

- Nếu ship charge: VNPay hoặc Stripe SDK, table `payments`, webhook, status `awaiting_payment` → `confirmed`.
- Nếu chưa: scaffold + feature flag `PAYMENTS_ENABLED=false`.

**DoD M4:** Admin quản lý được toàn bộ; video call reconnect; reminder gửi thật; Lighthouse A.

---

### M5 — Production Deploy (1‑2 ngày)

- `docker-compose.prod.yml` + `Caddyfile` cho DO droplet (P9 roadmap).
- `vercel.json` regions `sin1`, headers CSP đã thêm ở M3.
- `.github/workflows/deploy-prod.yml`: on push `main` → tests → Vercel action deploy.
- Supabase auto‑backup snapshot bật (paid tier).
- LiveKit droplet monitoring: `docker stats` + Uptime Kuma.
- `docs/RUNBOOK.md` cover 5 incident: LiveKit down, Supabase quota, booking double, TURN/ICE fail, Vercel edge outage.

**DoD M5:** smoke test prod full E2E; runbook cover 5 kịch bản.

---

### M6 — Documentation & Handover (1 ngày)

- `docs/ADR/`:
  - ADR‑001 Next.js + Supabase (vs Nest+Postgres).
  - ADR‑002 LiveKit self‑host (vs Cloud).
  - ADR‑003 RLS + service_role coexistence.
  - ADR‑004 Vercel + DO split deploy.
- `docs/API.md`: OpenAPI 3.1 sinh từ zod (`zod-to-openapi`).
- `docs/CONTRIBUTING.md`: branching, commit convention, PR template.
- `.github/PULL_REQUEST_TEMPLATE.md`, `.github/ISSUE_TEMPLATE/{bug,feature}.md`.
- README refresh: badges (CI, coverage, license).

---

## Phần C — Migration UI/UX Responsive

**Mục tiêu:** UI clean + chuẩn công nghiệp, chạy mượt trên **màn hình lớn (≥1440px, 4K)** và **điện thoại nhỏ (≤375px — iPhone SE, Galaxy S8)**.

### C.1 Audit UI/UX hiện tại

| Mục                     | Trạng thái | Chi tiết                                                          |
| ----------------------- | ---------- | ----------------------------------------------------------------- |
| Breakpoint strategy     | ⚠          | Default Tailwind, không custom, no container queries              |
| Layout shell            | ✅         | Header sticky, mobile hamburger (Sheet) tại `Header.tsx:237-316`  |
| Homepage                | ✅         | Grid progression 1→2→3 col ổn                                     |
| Discovery filters       | ⚠          | Top bar OK, không collapse thành sheet mobile                     |
| Tutor detail            | ⚠          | Không có sticky bottom CTA mobile                                 |
| Booking dialog          | ❌         | Dialog centered, không full‑screen mobile                         |
| My‑sessions tabs        | ⚠          | Không xác định wrap vs scroll                                     |
| Video room              | ❌         | Không fullscreen lock, không landscape hint                       |
| Admin tables            | ⚠          | `overflow-x-auto`, không có card fallback mobile                  |
| Tutor availability grid | ❌         | 7×48 slots — khó mobile                                           |
| Login page              | ✅         | Stack mobile OK                                                   |
| Typography scale        | ✅         | 8 utility classes, nhưng **không dùng clamp()**                   |
| Touch targets           | ❌         | Button h‑9/h‑10 (36‑40px) — **dưới 44‑48px chuẩn**                |
| Container width         | ⚠          | `max-w-7xl` (1280px), thiếu tối ưu ≥1440px                        |
| Density/spacing         | ✅         | Card padding + section rhythm ổn                                  |
| A11y mobile             | ⚠          | Skip link + focus rings OK; input không explicit ≥16px (iOS zoom) |
| Motion                  | ❌         | Không respect `prefers-reduced-motion`                            |
| Dark mode               | ✅         | `next-themes` setup                                               |
| Images                  | ❌         | `next/image` thiếu `sizes` prop, không aspect ratio placeholder   |
| PWA install             | ❌         | Manifest có, thiếu install prompt component                       |
| Safe area (notch)       | ⚠          | Chỉ ChatWidget dùng `safe-area-inset-bottom`                      |

---

### C.2 Design tokens & foundation

**C.2.1 Tạo `tailwind.config.ts` (v4 CSS‑only vẫn cần cho breakpoint override + plugin):**

```ts
export default {
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    screens: {
      xs: '375px', // iPhone SE
      sm: '640px',
      md: '768px',
      lg: '1024px',
      xl: '1280px',
      '2xl': '1440px', // laptop chuẩn
      '3xl': '1920px', // desktop 1080p
      '4xl': '2560px', // 4K/ultrawide
    },
    container: {
      center: true,
      padding: { DEFAULT: '1rem', md: '1.5rem', lg: '2rem' },
      screens: { '2xl': '1360px', '3xl': '1600px', '4xl': '1800px' },
    },
  },
  plugins: [require('@tailwindcss/typography'), require('tailwindcss-safe-area')],
};
```

**C.2.2 Typography scale fluid với `clamp()`** — thay fixed break trong `globals.css`:

```css
.type-hero {
  font-size: clamp(2rem, 5vw + 1rem, 4.5rem);
  line-height: 1.05;
  letter-spacing: -0.02em;
}
.type-page {
  font-size: clamp(1.5rem, 2.5vw + 0.5rem, 2.25rem);
  line-height: 1.15;
}
.type-section {
  font-size: clamp(1.25rem, 1.5vw + 0.5rem, 1.75rem);
  line-height: 1.2;
}
.type-title {
  font-size: clamp(1rem, 1vw + 0.5rem, 1.25rem);
  line-height: 1.3;
}
.type-body {
  font-size: clamp(0.95rem, 0.5vw + 0.75rem, 1rem);
  line-height: 1.6;
  max-width: 65ch;
}
.type-muted {
  font-size: 0.875rem;
  line-height: 1.5;
}
.type-eyebrow {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.type-kpi {
  font-size: clamp(1.75rem, 3vw + 0.5rem, 2.75rem);
  font-weight: 700;
}
```

**C.2.3 Touch target tokens:**

```css
:root {
  --tap-min: 44px; /* iOS HIG */
  --tap-comfy: 48px; /* Material */
  --input-h: 44px; /* prevent iOS zoom, min 16px font */
  --input-h-lg: 52px; /* primary CTA */
}
```

**C.2.4 Safe area utilities:**

```css
.pb-safe {
  padding-bottom: max(1rem, env(safe-area-inset-bottom));
}
.pt-safe {
  padding-top: max(1rem, env(safe-area-inset-top));
}
.px-safe {
  padding-left: max(1rem, env(safe-area-inset-left));
  padding-right: max(1rem, env(safe-area-inset-right));
}
.h-dvh-safe {
  min-height: 100dvh;
}
```

**C.2.5 Motion sensitivity:**

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

---

### C.3 Breakpoint strategy chuẩn công nghiệp

**Nguyên tắc mobile‑first, 4 tầng thiết bị:**

| Tầng          | Range     | Design target           | Layout pattern                         |
| ------------- | --------- | ----------------------- | -------------------------------------- |
| **Phone S**   | 320‑374   | iPhone SE (1st)         | 1 col, sticky bottom CTA, sheet dialog |
| **Phone M/L** | 375‑639   | iPhone 12+, Galaxy S    | 1 col, tab bar navigation              |
| **Tablet**    | 640‑1023  | iPad mini, Galaxy Tab   | 2 col, side sheet filter               |
| **Laptop**    | 1024‑1439 | MacBook 13", laptop 14" | 3 col cards, top nav + filter bar      |
| **Desktop**   | 1440‑1919 | iMac 24"                | 3‑4 col, sidebar dashboard             |
| **Wide/4K**   | ≥1920     | 4K, ultrawide           | max‑w container, focus reading col     |

**Container queries** dùng cho **Card component + Dashboard section** (không phụ thuộc viewport):

```tsx
<section className="@container">
  <div className="grid gap-4 @sm:grid-cols-2 @lg:grid-cols-3">...</div>
</section>
```

---

### C.4 Mobile ≤375px — checklist tối ưu

#### C.4.1 Layout & navigation

- [ ] `Header.tsx`: hamburger + logo + role avatar; nav items collapse vào Sheet (đã có, verify `xs` breakpoint).
- [ ] Thêm **bottom tab bar** cho authenticated user (`/`, `/tutors`, `/my-sessions`, `/my-page`) — component `BottomNav.tsx`, `safe-area-inset-bottom`.
- [ ] `MainShell.tsx`: `min-h-dvh` thay `min-h-screen` (iOS Safari address bar chip).
- [ ] Toàn bộ trang: `pb-[calc(4rem+env(safe-area-inset-bottom))]` khi có BottomNav.

#### C.4.2 Dialog → Sheet fallback mobile

- [ ] `src/components/ui/dialog.tsx`: wrap responsive:
  ```tsx
  const isMobile = useMediaQuery('(max-width: 640px)');
  return isMobile ? <Sheet side="bottom">{children}</Sheet> : <Dialog>{children}</Dialog>;
  ```
- [ ] Áp dụng cho `BookingDialog`, `ReviewDialog`, `RescheduleDialog`, `MaterialsDialog`.
- [ ] Sheet drag handle + snap points (50%, 100%) qua `vaul` (đã cài).

#### C.4.3 Touch targets

- [ ] Button variant `size="sm"` bỏ h‑8, đổi thành h‑10 (40px) — **chỉ cho pill/tag**, không cho tap chính.
- [ ] Default button `h-11` (44px), `size="lg"` `h-13` (52px).
- [ ] Icon‑only button `size="icon"` = 44×44px (`h-11 w-11`), padding tăng.
- [ ] Input `h-11` + `text-base` (16px) — chặn iOS auto‑zoom.
- [ ] Calendar day cell: min 44×44px, gap 4px.

#### C.4.4 Tutor list / discovery

- [ ] `TutorFilterBar`: mobile fold vào **filter sheet** (`<Sheet>`), trigger là button "Filters (3)".
- [ ] Filter chip active hiển thị dưới bar, clear‑all button.
- [ ] Sort chuyển từ inline dropdown sang icon button mở sheet.
- [ ] Card list: full width, `space-y-3`, avatar 56×56 (không 72 như desktop).

#### C.4.5 Tutor detail

- [ ] Hero + book CTA: sticky bottom bar `<div class="fixed bottom-0 inset-x-0 pb-safe bg-background/95 backdrop-blur border-t">`.
- [ ] Info sections: tab bar (About / Courses / Availability / Reviews).
- [ ] Availability preview: compact 3‑day carousel thay 7‑day grid.

#### C.4.6 Booking flow

- [ ] Sheet full‑height (`h-[95dvh]`), header sticky, footer sticky action (Next / Confirm).
- [ ] Progress dots + step name.
- [ ] Calendar: single month, swipe navigation qua `embla-carousel` (đã cài).
- [ ] Time slot grid: 2 col mobile (thay 3 col desktop), button min 44px.

#### C.4.7 My‑sessions

- [ ] Tabs: horizontal scroll với snap (`overflow-x-auto scroll-snap-x`), active tab centered.
- [ ] Card mobile: avatar + name + status badge top row; time + course row 2; action buttons row 3.
- [ ] Countdown chip mờ dần khi > 24h.

#### C.4.8 Video room

- [ ] `min-h-dvh` full screen, `overflow-hidden`.
- [ ] Landscape hint banner (rotate icon) nếu portrait — dùng `matchMedia('(orientation: portrait)')`.
- [ ] Control bar: bottom safe‑area, `env(safe-area-inset-bottom)`.
- [ ] Video tiles: 1 col portrait, 2 col landscape, PiP self‑view drag.
- [ ] Fallback overlay khi mất mạng: "Reconnecting…" toast + retry.

#### C.4.9 Admin/tutor dashboard mobile

- [ ] Sidebar `-translate-x-full` mặc định, backdrop overlay khi mở (đã có `AdminShell:41-44`).
- [ ] KPI cards grid: 2 col mobile, gap-3, `text-2xl` số (không quá to).
- [ ] Data table → **card list mobile** (component `<ResponsiveTable>` với `<div className="hidden md:block">table</div>` + `<div className="md:hidden">cards</div>`).
- [ ] Availability grid tutor: mobile chuyển sang **list per day** (Monday: 09:00, 10:00, 14:00 — chips).

#### C.4.10 Form UX mobile

- [ ] Label trên input, không placeholder‑only.
- [ ] `inputMode` chuẩn: `numeric` cho price, `email` cho email, `tel` cho phone.
- [ ] `autoComplete` chuẩn: `email`, `current-password`, `new-password`, `one-time-code`.
- [ ] Error text `text-sm text-destructive mt-1`.
- [ ] Submit button full‑width mobile, sticky footer khi form dài.

---

### C.5 Large screen ≥1440px — checklist tối ưu

#### C.5.1 Container & max‑width

- [ ] Container tối đa `1600px` cho content, `1360px` cho reading (news/course detail).
- [ ] Wide screen ≥ 1920px: **không stretch content**, giữ max‑w + auto margin.
- [ ] 4K: consider 2‑col article layout (main + sidebar) thay 1 col dài.

#### C.5.2 Layout patterns desktop

- [ ] Homepage hero: 2‑col split (headline + illustration/mockup) ≥ `2xl`.
- [ ] Tutor list: 4‑col grid ≥ `2xl`, sticky sidebar filter (không top bar).
- [ ] Tutor detail: 2‑col split (main 66% + book/summary 33% sticky).
- [ ] Admin dashboard: sidebar 240px fixed + main + optional right rail (KPI/quick actions).
- [ ] Video room: 3‑col (participant list + main video + chat panel) khi `2xl`.

#### C.5.3 Typography desktop

- [ ] Hero clamp cap ở 4.5rem (`clamp(2rem, 5vw+1rem, 4.5rem)`) — không quá lớn 4K.
- [ ] Body max‑width `65ch` — chống line dài khó đọc.
- [ ] Heading letter‑spacing tighten (`-0.02em`).

#### C.5.4 Density & spacing

- [ ] Section vertical rhythm: `py-24 3xl:py-32`.
- [ ] Grid gap: `gap-6 3xl:gap-8`.
- [ ] Card padding: `p-6 3xl:p-8`.

#### C.5.5 Data density

- [ ] Admin tables: hiển thị nhiều cột hơn ≥ `2xl` (add columns: `updated_at`, `last_login`, `action_menu`).
- [ ] Bulk actions bar khi select nhiều rows.
- [ ] Pagination + page size selector (10/25/50/100) mặc định 25 desktop, 10 mobile.

#### C.5.6 Multitasking desktop

- [ ] Keyboard shortcuts (đã có `cmdk` — Cmd+K global search).
- [ ] Right‑click context menu cho admin CRUD (dropdown‑menu shadcn).
- [ ] Split view ready (admin + preview khi edit course/news).

#### C.5.7 High‑DPI assets

- [ ] `next/image` với `sizes` chính xác:
  ```tsx
  <Image sizes="(max-width: 640px) 100vw, (max-width: 1440px) 50vw, 33vw" />
  ```
- [ ] Ảnh hero 2× cho retina, format AVIF/WebP.
- [ ] Icon SVG (đã lucide‑react).

---

### C.6 Component-by-component migration

Bảng migration cụ thể, sort theo mức độ impact:

| #   | Component                                     | Vấn đề hiện tại                        | Fix cần làm                                          | Effort |
| --- | --------------------------------------------- | -------------------------------------- | ---------------------------------------------------- | ------ |
| 1   | `ui/dialog.tsx`                               | Fixed centered, mobile khó dùng        | Responsive Dialog↔Sheet                              | 0.5d   |
| 2   | `ui/button.tsx`                               | h‑9/h‑10 dưới 44px                     | Bump default h‑11, size="lg" h‑13, size="icon" 44×44 | 0.25d  |
| 3   | `ui/input.tsx`                                | text-sm 14px → iOS auto‑zoom           | h‑11 + text-base (16px)                              | 0.25d  |
| 4   | `session/BookingDialog.tsx`                   | Multi‑step không mobile‑friendly       | Wrap Sheet mobile, sticky footer next/back           | 0.75d  |
| 5   | `shared/Header.tsx`                           | OK desktop; thiếu bottom nav mobile    | Thêm `BottomNav.tsx` cho auth user                   | 0.5d   |
| 6   | `home/HomeContent.tsx`                        | Grid tốt, hero fixed                   | Clamp typography, 2‑col hero ≥ `2xl`                 | 0.5d   |
| 7   | `tutor/TutorFilterBar.tsx`                    | Top bar mobile chật                    | Filter Sheet trigger, chip active list               | 0.5d   |
| 8   | `(main)/tutors/[id]/page.tsx`                 | CTA book không sticky mobile           | Sticky bottom bar + tab sections                     | 0.75d  |
| 9   | `(main)/my-sessions/page.tsx`                 | Tabs không xác định wrap               | Horizontal scroll snap + card list                   | 0.5d   |
| 10  | `session/[id]/room` (`RoomClient.tsx`)        | Không fullscreen, không landscape hint | `min-h-dvh`, orientation banner, safe‑area controls  | 1d     |
| 11  | `admin/AdminShell.tsx`                        | Sidebar OK; tables scroll‑x            | `ResponsiveTable` primitive card fallback            | 0.75d  |
| 12  | `admin/sections/UsersSection.tsx`             | Table only                             | Card mobile + table desktop                          | 0.5d   |
| 13  | `admin/sections/SessionsSection.tsx`          | Table only                             | Card mobile + filter sheet                           | 0.5d   |
| 14  | `tutor/sections/TutorAvailabilitySection.tsx` | Grid 7×48 khó mobile                   | List per day + drag desktop                          | 1d     |
| 15  | `customer/LoginPage.tsx`                      | Stack OK; thiếu 16px input             | Input h‑11 + text-base                               | 0.25d  |
| 16  | `shared/Footer.tsx`                           | OK                                     | Verify safe‑area bottom                              | 0.1d   |
| 17  | `shared/ChatWidget.tsx`                       | Có safe‑area riêng                     | Sync với BottomNav positioning                       | 0.25d  |
| 18  | `ui/calendar.tsx`                             | Day cell nhỏ                           | Cell min 44×44, swipe navigation                     | 0.5d   |
| 19  | `ui/table.tsx`                                | Primitive OK                           | Thêm variant `<TableMobile>` card                    | 0.5d   |
| 20  | `ui/sheet.tsx`                                | Cần drag handle mobile                 | Snap points + drag handle (vaul integration)         | 0.5d   |

**Tổng effort UI/UX:** ~10 dev‑days.

---

### C.7 A11y & motion sensitivity

- [ ] Skip‑link (`Header.tsx:73-78`) — giữ, verify contrast.
- [ ] Focus‑visible ring 2px `ring-offset-2`, contrast ≥ 3:1 với background.
- [ ] `aria-live` region cho toast (`sonner` default có, verify).
- [ ] `<dialog>` semantic + `aria-modal` (Radix default).
- [ ] Landmarks: `<header>`, `<main>`, `<nav>`, `<aside>`, `<footer>` — audit missing.
- [ ] Table `<caption>` + `<th scope>` cho screen reader.
- [ ] Form `<label htmlFor>` — không dùng placeholder thay label.
- [ ] `prefers-reduced-motion` — CSS global đã ở C.2.5, motion/react disable qua `MotionConfig`:
  ```tsx
  <MotionConfig reducedMotion="user">{children}</MotionConfig>
  ```
- [ ] Contrast audit: text on primary `#4F46E5` — trắng OK, foreground trên `accent-warm` amber cần check.
- [ ] Test screen reader: NVDA + VoiceOver iOS, tối thiểu 3 flow (login, book, review).
- [ ] Keyboard‑only nav qua toàn app: tab order, Escape đóng modal, Enter submit.

---

### C.8 Performance & Web Vitals

Target thiết bị:

| Metric | Mobile (Moto G4 3G) | Desktop | Ghi chú                                   |
| ------ | ------------------- | ------- | ----------------------------------------- |
| LCP    | < 2.5s              | < 1.8s  | Hero image priority + fetchpriority       |
| INP    | < 200ms             | < 100ms | React 19 concurrent, no long tasks        |
| CLS    | < 0.1               | < 0.05  | Aspect ratio placeholder mọi image        |
| FCP    | < 1.8s              | < 1s    | Font preload, critical CSS inline         |
| TTI    | < 3.8s              | < 2s    | Code split route + dynamic import dialogs |

**Actions:**

- [ ] `next/font/google` Inter + Manrope, `display: swap`, `preload: true`.
- [ ] Homepage `getStaticProps` (ISR revalidate 60s) — không SSR.
- [ ] Tutor list ISR + client‑side filter (không refetch mọi filter).
- [ ] `dynamic(() => import('...'), { ssr: false })` cho: `BookingDialog`, `VideoRoom`, `TipTapEditor`, `LiveKitRoom`.
- [ ] `@next/bundle-analyzer` — gate size: `main` < 200KB gzip, `admin` chunk < 300KB.
- [ ] Preload critical routes trên homepage (Link prefetch).
- [ ] Image: `<Image priority sizes="..." placeholder="blur" />` cho above‑fold.
- [ ] Service Worker (`sw.js`): pre‑cache `/`, `/tutors`, `/login`, `/offline`. Stale‑while‑revalidate cho `/api/tutors`, `/api/sessions`.
- [ ] Lighthouse CI gate ≥ 90 (Performance + A11y + BestPractice + SEO).

---

### C.9 DoD & sign‑off

- [ ] `tailwind.config.ts` có breakpoint `xs`/`2xl`/`3xl`/`4xl` + container width.
- [ ] Typography scale dùng `clamp()`, verify visual ở 320px / 768px / 1440px / 2560px.
- [ ] Tất cả Dialog quan trọng có Sheet fallback mobile (5 dialog trong bảng C.6).
- [ ] Bottom nav xuất hiện cho authenticated user ≤ md breakpoint.
- [ ] Toàn bộ button interactive ≥ 44×44px (script test qua Playwright query).
- [ ] Input ≥ 16px font size (không iOS zoom).
- [ ] Safe‑area class áp dụng cho: header top, footer bottom, video room controls, sheet content.
- [ ] `prefers-reduced-motion` respected — verify với DevTools emulate.
- [ ] `next/image` mọi ảnh có `sizes` prop + aspect ratio.
- [ ] Table admin có card fallback mobile.
- [ ] Tutor availability có list mode mobile.
- [ ] Video room fullscreen + landscape hint verified trên Safari iOS + Chrome Android.
- [ ] Lighthouse Mobile ≥ 90 tất cả trục.
- [ ] Screenshot regression test qua Playwright: 5 route × 4 viewport (375/768/1440/2560) = 20 snapshot.
- [ ] Real device test tối thiểu: iPhone SE, iPhone 15, Galaxy S8, iPad, MacBook 13", Desktop 27" 4K.

---

## Phần D — Rủi ro & Mitigation

| Rủi ro                                   | Impact          | Mitigation                                                     |
| ---------------------------------------- | --------------- | -------------------------------------------------------------- |
| Bật strict TS phá vỡ 100 chỗ             | Bug rơi vãi     | Bật theo file `// @ts-strict-mode` trước, migrate 1 PR/module  |
| RLS enable rớt request frontend          | Data không load | Deploy staging trước, wrap service_role vs anon test song song |
| Playwright flaky CI                      | PR chậm         | Pin browser version, upload trace on fail, retry 2             |
| Sentry đội cost noisy                    | Chi phí         | `beforeSend` filter, sample rate 0.2                           |
| LiveKit UDP bị NAT VN chặn               | Video fail      | TURN over TCP 7881 fallback + test 4G thật                     |
| Migration RLS quên policy → leak         | Data leak       | Test suite anon key verify không xem chéo                      |
| Dialog→Sheet migration làm vỡ UX desktop | Regression      | Feature flag `MOBILE_SHEET=true`, rollout ≤ md breakpoint      |
| Bottom nav che content                   | UX              | Add `pb-[calc(4rem+safe-area)]` global cho auth layout         |
| Clamp typography quá to 4K               | Xấu             | Cap max ở clamp (4.5rem hero) + test 2560px                    |
| Cache SW cũ giữ golf routes              | User thấy 404   | Bump `CACHE_VERSION`, force reload on activate                 |

---

## Phần E — Timeline tổng hợp

| Milestone                   | Effort (dev‑days) | Ship‑blocker | Ghi chú                                      |
| --------------------------- | ----------------- | ------------ | -------------------------------------------- |
| M1 Foundation               | 2‑3               | ✅           | Không có = mọi commit sau rủi ro             |
| M2 Testing                  | 4‑5               | ✅           | Không có = không dám refactor                |
| M3 Observability + Security | 3‑4               | ✅           | RLS + Sentry + CSP tối thiểu cho PII prod    |
| M4 Product completion       | 5‑7               | ⚠            | Admin sections + LiveKit UI cần MVP          |
| **C UI/UX Responsive**      | **~10**           | ✅           | Chạy song song M4, share component với admin |
| M5 Deploy                   | 1‑2               | ✅           | Trước go‑live đầu                            |
| M6 Docs                     | 1                 | ❌           | Sau go‑live được                             |

**Tổng:** 26‑32 dev‑days. Với 2 engineer song song (1 backend + 1 frontend/UI): **~14‑17 calendar‑days**.

**Đề xuất thứ tự tuần đầu:**

1. Day 1‑2: M1 (foundation) — engineer cả hai cùng làm cleanup + strict TS + CI.
2. Day 3‑5: M2.1‑M2.2 (Vitest + unit test business logic) — engineer backend.
3. Day 3‑5: C.2‑C.3 (design tokens + breakpoint config) — engineer frontend.
4. Day 6+: parallel M2.3‑M2.4 (E2E) + C.4 (mobile checklist) + C.6 component migration.
5. Day 10+: M3 (observability + security) + C.5 (large screen) + C.7 (a11y).
6. Cuối: M4 product completion + M5 deploy + M6 docs.

**Sign‑off criteria cho "chuẩn công nghiệp":**

- ✅ CI xanh trên mọi PR (lint/type/test/build).
- ✅ Coverage ≥ 60% `src/lib/**` + `src/app/api/**`.
- ✅ Sentry + structured log active.
- ✅ RLS ON với policy đầy đủ.
- ✅ Security headers `securityheaders.com` A+.
- ✅ Lighthouse Mobile + Desktop ≥ 90 (Perf/A11y/BP/SEO).
- ✅ Real device test iPhone SE + Galaxy S8 + iPad + Desktop 4K.
- ✅ Runbook cover 5 incident scenarios.
- ✅ ADR + OpenAPI docs.

---

_File này song hành với `VIELANG_ROADMAP.md` (P0–P10 MVP). Roadmap tập trung feature, file này tập trung industrial quality + responsive UX._
