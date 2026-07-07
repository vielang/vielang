# CLAUDE.md — VieLang Project Reference

> **Snapshot:** 2026-07-04. The pivot from the old golf booking codebase is
> essentially done — schema is fully VieLang-native, all customer-facing golf
> routes are removed, LiveKit video calls (M1–M5) are landed. What remains is
> deployment prep and small quality-of-life polish. See
> `docs/VIELANG_ROADMAP.md` for the phase plan.

---

## 1. Product

**VieLang** — nền tảng học tiếng Anh 1-on-1 và nhóm nhỏ qua video call.

| Actor              | Description                                                                                             |
| ------------------ | ------------------------------------------------------------------------------------------------------- |
| **User (Student)** | Browse tutors, book 1-on-1 sessions, join group free-talk rooms, leave reviews, access course materials |
| **Tutor**          | Sets weekly availability, publishes courses, teaches sessions, sees earnings/rating in own dashboard    |
| **Admin**          | Approves tutor applications, manages users/courses/reviews, watches KPIs                                |

Two session shapes coexist in one `sessions` table:

- **private (1-on-1)**: `student_id` set, `capacity=1`, tied to a booked course
- **group**: `student_id` NULL, `topic_*`, `level`, `capacity>1`, joiners tracked in `session_participants`

Domain: **vielang.com**. Live-video: **LiveKit self-hosted via Docker**.
Deploy target: **Vercel (Next.js)** + **DigitalOcean droplet (LiveKit + Redis)**.

---

## 2. Tech Stack (actual)

| Layer         | Technology                                                                     |
| ------------- | ------------------------------------------------------------------------------ |
| Framework     | **Next.js 16** (App Router, RSC), React 19, TypeScript 6                       |
| UI            | Tailwind CSS v4, **shadcn/ui** (base-ui + Radix), lucide-react, motion/react   |
| Auth          | **Supabase Auth (SSR cookies)** — Email + Google OAuth (PKCE)                  |
| DB            | **Supabase PostgreSQL** — VieLang-native schema (see §4)                       |
| Video         | **LiveKit** self-hosted — `@livekit/components-react` + `livekit-server-sdk`   |
| Forms         | react-hook-form + zod, shared `useDialogForm` hook                             |
| Rich text     | Tiptap 3 (+ `reactjs-tiptap-editor`)                                           |
| Email         | **Resend** + `@react-email/components` (templates in `src/lib/emails/`)        |
| Rate-limit    | **Upstash Redis** (falls back to in-memory sliding window if unconfigured)     |
| Observability | **Sentry** (`@sentry/nextjs`), Pino logger                                     |
| Toasts        | sonner                                                                         |
| DnD           | @dnd-kit                                                                       |
| Charts        | recharts                                                                       |
| Testing       | **Vitest** (unit) + **Playwright** (e2e)                                       |
| Formatting    | Prettier + prettier-plugin-tailwindcss, ESLint flat config, lint-staged, husky |

**Not here (don't expect it):** No Vite. No Express `server.ts`. No Firebase.
No Kakao/Naver OAuth. No Toss/golf payments. No React Query — pages fetch in
Server Components, mutations use direct `fetch()` / supabase client.

---

## 3. Directory layout

```
vielang-v2/
├── CLAUDE.md                       # this file
├── docs/
│   ├── VIELANG_ROADMAP.md          # phase plan (P0-P10)
│   ├── SETUP.md                    # local dev setup
│   ├── AUDIT_2026-07.md            # backend audit notes
│   ├── UI_AUDIT_2026-07.md         # UI/UX audit notes
│   └── INDUSTRIAL_MIGRATION_PLAN.md
├── docker/
│   ├── docker-compose.dev.yml      # local LiveKit + Redis
│   ├── livekit.yaml
│   └── README.md
├── supabase/
│   ├── config.toml                 # project_id: "vielang"
│   └── migrations/                 # 8 SQL migrations, all VieLang-native
├── scripts/
│   ├── generate-pwa-icons.mjs      # (icons now single SVG, may be vestigial)
│   └── setup-storage.mjs           # Supabase Storage bucket init
├── public/
│   ├── brand-v.svg                 # SINGLE brand mark — feeds every icon slot
│   └── sw.js                       # hand-written service worker
├── src/
│   ├── middleware.ts               # auth cookie refresh + /admin, /tutor gate
│   ├── app/
│   │   ├── layout.tsx              # SSR user hydration; auto-favicon via icon.svg
│   │   ├── icon.svg                # Next auto-serves as favicon
│   │   ├── providers.tsx           # Lang → SiteSettings → Auth
│   │   ├── manifest.ts, robots.ts, sitemap.ts
│   │   ├── (main)/                 # public routes: home, tutors, sessions, my-sessions, news, faq, contact, legal
│   │   ├── admin/page.tsx          # monolithic AdminShell (KPIs, users, sessions, pending tutors, courses, reviews)
│   │   ├── tutor/page.tsx          # monolithic TutorShell (KPIs, sessions, availability, materials, profile)
│   │   ├── session/[id]/room/      # LiveKit video room page
│   │   ├── login/, auth/callback/
│   │   ├── offline/, not-found.tsx, error.tsx
│   │   └── api/                    # Route Handlers (see §5)
│   ├── components/
│   │   ├── ui/                     # ~34 shadcn primitives
│   │   ├── shared/                 # Header, Footer, BottomNav, BrandedLoader, ChatWidget, RichTextEditor, ThemeToggle, ...
│   │   ├── admin/                  # AdminShell + sections/ (Overview, Sessions, Users, PendingTutors, Courses, Tutors, Reviews)
│   │   ├── tutor/                  # TutorShell + sections/ (Overview, Sessions, Availability, Materials, Profile, MyCourses, MyReviews) + TutorCard, TutorFilterBar, StickyBookBar
│   │   ├── session/                # BookingDialog, RescheduleDialog, ReviewDialog, MaterialsDialog, SessionCard, GroupSessionCard
│   │   │   └── room/               # RoomMessageBubble, TerminalScreen (error/ended/denied)
│   │   ├── auth/                   # LoginPage split: LoginPanel, SignupPanel, ForgotPasswordPanel, BrandingSide, SocialButtons, DemoLoginRow, auth-alerts
│   │   ├── home/                   # HomeContent
│   │   ├── pwa/                    # ServiceWorkerRegister
│   │   └── skeletons/              # ListPageSkeleton
│   ├── contexts/                   # AuthContext, LangContext, SiteSettingsContext (all wired in providers.tsx)
│   ├── hooks/                      # use-dialog-form, use-mobile, use-recent-searches
│   └── lib/
│       ├── auth-client.ts          # browser: Google + email auth
│       ├── auth-server.ts          # authenticate(), requireRole()
│       ├── auth/                   # get-current-user.ts, types.ts
│       ├── supabase.ts             # service-role singleton + ~50 query functions
│       ├── supabase/               # client.ts, server.ts, middleware.ts (@supabase/ssr)
│       ├── livekit-admin.ts        # server-side RoomServiceClient (mute, remove, perms)
│       ├── schemas/                # zod: auth, forms, tutor, session, material, review
│       ├── emails/                 # React email templates (SessionConfirmation, SessionReminder, ReviewRequest)
│       ├── email.ts, logger.ts, audit.ts, rate-limit.ts, sanitize.ts
│       ├── translations.ts, i18n-*.ts
│       ├── currency.ts, csv.ts, image-upload.ts, api-errors.ts
│       ├── types.ts                # 20+ domain interfaces, 8 enum aliases
│       └── seed-data.ts
└── (no .github/workflows — CI/CD wired outside the repo)
```

**No `src/types/`** — all domain types live in `src/lib/types.ts`.
**No `src/components/customer/`, `booking/`, `course/`, `news/`, `stayplay/`,
`owner/`** — golf-era dirs and the old owner dashboard are gone.

---

## 4. Data model (Supabase)

Schema is **fully VieLang-native**. No golf tables remain. RLS is enabled with
default DENY on every user-visible table; **all writes go through the service-
role client in Route Handlers** — RLS is defense-in-depth, not the primary gate.

Migrations, applied in order:

| #   | Migration                                  | What it lands                                                                                                                                        |
| --- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `20260702000001_vielang_init.sql`          | Core: `users`, `tutor_profiles`, `courses`, `availability`, `sessions`, `reviews`, `materials`, `banners`, `news` + `refresh_tutor_rating()` trigger |
| 2   | `20260702180000_fix_review_trigger.sql`    | Rating trigger DELETE-safe (COALESCE(NEW, OLD)) + backfill                                                                                           |
| 3   | `20260703000000_group_sessions.sql`        | Extend `sessions` (type, topic_*, level, capacity, cover_emoji) + `session_participants` table                                                       |
| 4   | `20260710000000_rls_and_audit.sql`         | Turn on RLS; add `audit_log` for sensitive mutations                                                                                                 |
| 5   | `20260710010000_notification_log.sql`      | Email delivery audit (`notification_log`)                                                                                                            |
| 6   | `20260711000000_session_admission.sql`     | Waiting room: `sessions.require_admission` + `session_admissions` (XOR admitted/denied)                                                              |
| 7   | `20260712000000_chat_and_attendance.sql`   | `session_messages` (client_id dedupe), `session_attendance` (join/leave events, `duration_sec` GENERATED)                                            |
| 8   | `20260713000000_fix_message_client_id.sql` | Make `client_id NOT NULL` so `ON CONFLICT` dedupe works                                                                                              |

**Key entity notes:**

- `sessions.status`: `pending → confirmed → live → completed` (or `cancelled` /
  `no_show`). Bumped by API endpoints and the LiveKit webhook.
- `sessions.livekit_room_name`: unique per session; the token endpoint mints
  against it.
- `session_participants` is for **group** sessions; private sessions use
  `sessions.student_id` directly. Don't dual-write.
- `session_attendance` has a GENERATED `duration_sec` column — never write it.
- `notification_log` is the append-only email audit; write it even when Resend
  is unconfigured (delivered=false, reason='dev_no_key').

---

## 5. API surface (`src/app/api/`)

Grouped by domain — full endpoint list:

**Session lifecycle**

- `POST /api/sessions` — create private or group session
- `GET  /api/sessions` — list (scoped by role: student=own bookings, tutor=own teaching, admin=all)
- `PATCH /api/sessions/[id]` — cancel / reschedule / confirm
- `GET  /api/sessions/[id]/messages` — session chat feed
- `POST /api/sessions/[id]/attendance` — record join/leave events

**LiveKit**

- `POST /api/livekit/token` — mint access token; enforces **join-window (±15 min)**, **capacity gate** (counts open `session_attendance`), and **waiting-room gating** when `require_admission=true`
- `GET  /api/livekit/token/refresh` — refresh before 2h TTL expires
- `POST /api/livekit/admit` — host admits waiting participant
- `POST /api/livekit/moderate` — host mutes / ejects
- `POST /api/livekit/webhook` — LiveKit → server; `room_finished` bumps `sessions.status='completed'`

**Tutors + availability**

- `GET  /api/tutors`, `GET /api/tutors/[id]`, `GET /api/tutors/[id]/courses`, `GET /api/tutors/[id]/reviews`, `GET /api/tutors/[id]/availability`
- `PATCH /api/tutor/profile`
- `GET/POST /api/availability`, `GET/PUT /api/availability/[id]`

**Reviews, courses, materials**

- `POST /api/reviews`, `GET /api/reviews`
- `GET /api/courses/[id]/materials`, `GET /api/materials/[id]`

**Admin**

- `PUT /api/admin/tutors/[id]/approve`, `.../reject`
- `PUT /api/admin/users/[id]/role`, `.../enabled`
- `PATCH/DELETE /api/admin/courses/[id]`
- `DELETE /api/admin/reviews/[id]`

**Cron (Vercel, every 10 min)**

- `GET /api/cron/reminders` — 24h + 1h reminder emails
- `GET /api/cron/no-show` — mark missed sessions (`status='no_show'`) if the
  room never entered `live` by grace period
- Both require `Authorization: Bearer $CRON_SECRET`

**System**

- `GET /api/health`, `GET /api/ready`, `POST /api/seed` (dev)
- `GET /api/users/me`

---

## 6. Auth flow

Cookie-based Supabase SSR. `authenticate()` in `src/lib/auth-server.ts` tries,
in order:

1. `@supabase/ssr` cookie session (default path)
2. `Authorization: Bearer <supabase_access_token>` (legacy clients)
3. `X-Demo-User` header — **dev/E2E only**, blocked in production

Google OAuth uses PKCE via `/auth/callback/route.ts`. Middleware
(`src/middleware.ts`) refreshes cookies on every request and redirects
unauthenticated users away from `/admin/*` and `/tutor/*`; the page's Server
Component then does the final **role** check.

Dev bypass: setting `ALLOW_DEMO_USER=true` locally allows a `vielang_demo_user`
cookie to seed a fake session — never enable in production.

---

## 7. Session / video-call feature (M1–M5, landed)

The most-recently-built area. Room lives at `/session/[id]/room`.

- **Pre-join lobby**: `/my-sessions/[id]` page shows session card + "Join now"
- **Waiting room**: If `sessions.require_admission=true`, non-host tokens ship
  with metadata `state='waiting'` (no publish/subscribe until host admits)
- **Host controls**: tutor/admin sees waiting drawer + mute/eject; chat feed;
  attendance counter drives the capacity gate
- **Adaptive stream / reconnect banner / quality bars / emoji reactions**
  (see commit `07d182d feat(M5)`)
- **No-show cron** marks sessions the room never went live for
- **Webhook** finalizes attendance + status when the LiveKit room ends

For token minting, always route through `POST /api/livekit/token` — never
mint on the client. The server-side `livekit-admin.ts` wraps
`RoomServiceClient` for mute / remove / permissions changes.

---

## 8. Frontend conventions

- **State model**: server-first. Pages fetch in Server Components (usually a
  `Promise.all` of `src/lib/supabase.ts` helpers). Client mutations go through
  Route Handlers via `fetch()`. **No React Query.**
- **Forms**: every dialog uses `useDialogForm` (`src/hooks/use-dialog-form.ts`)
  — react-hook-form + zod + sonner toast, with a `'handled'` sentinel so
  handlers can silence the default error toast when they want custom UX.
- **i18n**: `useLang()` from `LangContext` returns `'VN' | 'EN'`. Copy lives
  either in `src/lib/translations.ts` (global keys) or inline `COPY = { VN, EN }`
  objects in components. KR was removed 2026-07-02.
- **Brand icon**: single source `public/brand-v.svg` + `src/app/icon.svg`. Any
  brand-mark `<Image>` should point at `/brand-v.svg` with `unoptimized`. No
  PNG icon set, no `generate-icons` step.
- **Focus ring / a11y**: shared `.focus-ring` utility class; keep it on every
  interactive element.

---

## 9. Environment

Runtime + local dev vars (see `.env.example` for the full list):

- **Supabase**: `SUPABASE_URL`, `SUPABASE_SERVICE_ROLE_KEY` (server),
  `NEXT_PUBLIC_SUPABASE_URL`, `NEXT_PUBLIC_SUPABASE_ANON_KEY` (client)
- **LiveKit**: `LIVEKIT_API_KEY`, `LIVEKIT_API_SECRET`,
  `NEXT_PUBLIC_LIVEKIT_WS_URL` (dev: `ws://localhost:7880` from docker-compose)
- **Email**: `RESEND_API_KEY` (empty = dev no-op, still logs), `EMAIL_FROM`,
  `EMAIL_REPLY_TO`
- **Rate limit**: `UPSTASH_REDIS_REST_URL/TOKEN` (empty = in-memory fallback);
  `RATE_LIMIT_DISABLED=true` only for CI/E2E
- **Cron**: `CRON_SECRET`
- **Observability**: `SENTRY_DSN`, `NEXT_PUBLIC_SENTRY_DSN`, `LOG_LEVEL`
- **Site**: `NEXT_PUBLIC_SITE_URL`

---

## 10. Local dev

```bash
npm install
cp .env.example .env.local          # fill in Supabase + LiveKit keys

# Optional: start local LiveKit + Redis
docker compose -f docker/docker-compose.dev.yml up -d

npm run dev                         # http://localhost:3000
npm run typecheck                   # tsc --noEmit
npm run lint                        # ESLint flat config
npm run test                        # Vitest unit tests
npm run e2e                         # Playwright (needs e2e:install once)
npm run build                       # next build
```

`husky` + `lint-staged` run prettier + eslint on staged `.{ts,tsx,js,jsx,mjs,cjs}`
files at commit time. Don't skip hooks (`--no-verify`) unless there's a real
reason — CI (wired outside the repo by the operator) runs the same checks.

---

## 11. Deployment

- **Frontend + API**: `vercel.json` still targets `sin1` (Singapore) if you
  push to Vercel, but CI/CD is intentionally not committed — the operator
  wires their own pipeline. `/api/cron/reminders` and `/api/cron/no-show`
  are ordinary Route Handlers gated by `Authorization: Bearer $CRON_SECRET`;
  point any external scheduler (systemd timer, GitHub Actions cron, a
  separate cron container, another platform's scheduled job) at them on the
  10-minute cadence the handlers expect.
- **DigitalOcean droplet** (LiveKit + Redis): the docker-compose file in
  `docker/` is the same shape as prod — Redis in front of Upstash-style rate
  limiting when configured, LiveKit server for WebRTC. `docker/livekit.yaml`
  pins `--node-ip` for the Windows Docker Desktop dev case; prod overrides
  with the droplet's public IP.
- **Supabase**: hosted project, region `ap-southeast-1` planned per roadmap.
  Migrations applied via `supabase db push`.

---

## 12. What's actually left

Golf legacy is essentially gone from the runtime — the schema, routes, and
components are all VieLang-native. The remaining work (per
`docs/VIELANG_ROADMAP.md`) is production hardening: real Supabase project
provisioning, LiveKit prod deployment, CI/CD wiring, and continuing
UI/product polish. If you're building on top of anything and can't tell
whether it's current, `git log --oneline -30` beats guessing — recent commits
(M1–M5 session feature, U2–U6 UI industrialization, session audits) show the
current direction.
