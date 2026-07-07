# VieLang — Detailed Implementation Roadmap

**Project:** VieLang — English Learning Platform
**Domain:** vielang.com
**Base:** Pivot từ codebase Next.js golf (VinaRounding), giữ nguyên framework — thay domain logic
**Deploy:** Vercel (Next.js full stack) + DigitalOcean droplet (LiveKit + Redis, docker)
**Timeline:** ~13-18 ngày cho MVP (P0 → P10) — tiết kiệm ~2 ngày vì không cần migrate framework

---

## Kiến trúc tổng thể

```
┌──────────────────────────────────────────────────────┐
│  Vercel — Full Next.js 16 App                        │
│  • vielang.com                                       │
│  • SSR pages + Route Handlers (/api/*)               │
│  • Includes /api/livekit/token, /api/livekit/webhook │
└───────────────┬──────────────────────┬──────────────┘
                │ HTTPS                 │ HTTPS webhook
                ▼                       ▼
      ┌──────────────────┐   ┌──────────────────────────┐
      │  Supabase        │   │  DigitalOcean Droplet    │
      │  (Auth + DB)     │   │  (Docker Compose)         │
      │  SG region       │   │  ┌────────────────────┐  │
      └──────────────────┘   │  │ Caddy (HTTPS)      │  │
                              │  │ livekit.vielang.com│  │
                              │  └─────────┬──────────┘  │
                              │  ┌─────────▼──────────┐  │
                              │  │ livekit-server     │  │
                              │  │ SFU + TURN         │  │
                              │  │ 7881/tcp,          │  │
                              │  │ 50000-60000/udp    │  │
                              │  └─────────┬──────────┘  │
                              │  ┌─────────▼──────────┐  │
                              │  │ redis (signaling)  │  │
                              │  └────────────────────┘  │
                              └──────────────────────────┘
```

**Vì sao chỉ LiveKit trên DO?** LiveKit là stateful server dài hạn với WebRTC/UDP — không hợp với serverless. Vercel handle Next.js (SSR + API routes) tốt hơn nhiều. Còn LiveKit chỉ cần 1 droplet nhỏ chạy docker.

**Actors:**

| Role               | Quyền                                                                                     |
| ------------------ | ----------------------------------------------------------------------------------------- |
| **User (Student)** | Xem tutor, book session, join video call, review, xem materials                           |
| **Tutor**          | Quản lý availability + session của mình. UI giống admin nhưng data filter theo `tutor_id` |
| **Admin**          | Full quyền: duyệt tutor apply, quản lý users/courses/content/banners/news                 |

---

## Phase 0 — Cleanup golf codebase (2 phần)

### Phase 0a (DONE — 2026-07-02)

- [x] Rename `vinarounding` → `vielang` trong `package.json`, `manifest.ts`, `layout.tsx`, `sw.js`
- [x] Đổi brand default trong `SiteSettingsContext`, `Header`, `Footer`, `BrandedLoader`, `LoginPage`
- [x] Đổi localStorage keys `vinarounding_*` → `vielang_*`
- [x] Xóa OAuth Kakao/Naver: `app/auth/kakao/*`, `app/auth/naver/*`
- [x] Strip Kakao/Naver JWT branch trong `auth-server.ts`
- [x] Strip Kakao/Naver imports + handlers trong `auth-client.ts`, `LoginPage.tsx`
- [x] Strip Kakao chat button + modal trong `ChatWidget.tsx`
- [x] Rewrite `CLAUDE.md` phản ánh Next.js stack
- [x] `npm run typecheck` passes

### Phase 0b (NEXT — 1-2 ngày)

**Xóa golf-domain routes + components + API:**

**Customer routes (`src/app/(main)/`):**

- `cart/`, `checkout/`, `elite-pass/`, `golf-tours/`, `pricing/`
- Giữ và refactor sau: `courses/`, `news/`, `my-page/`, `contact/`, `faq/`, `privacy/`, `terms/`, `refund/`

**Components (`src/components/customer/`):**

- `CartPage.tsx`, `CheckoutPage.tsx`, `ElitePass.tsx`, `PriceListPage.tsx`,
  `StayPlayDetail.tsx`, `TeeTimeGrid.tsx`, `CourseDetail.tsx`, `CourseDirectory.tsx`,
  `BannerSlider.tsx`, `Testimonials.tsx`, `NewsDetail.tsx`, `BookingDialogHost.tsx`,
  `MyPageContent.tsx`, `RescheduleDialog.tsx`
- Giữ: `LoginPage.tsx` (đã refactor), `StarRating.tsx`
- `src/components/booking/BookingDialog.tsx` — golf-specific, xóa
- `src/components/course/CourseDetailRoute.tsx` — xóa
- `src/components/stayplay/` — xóa toàn bộ
- `src/components/home/HomeContent.tsx` — rewrite phase 3
- `src/components/news/` — giữ (refactor content)

**API Route Handlers (`src/app/api/`):**

- Xóa: `tee-times/`, `stay-and-play/`, `elite-pass/`, `golf-tours/`, `promo/`,
  `promo-codes/`, `pricing/`, `payments/` (Toss), `webhooks/` (Toss webhook)
- Giữ và refactor: `courses/`, `bookings/` (→ sessions), `news/`, `banners/`,
  `reviews/`, `users/`, `chat/`, `upload/`, `translate/`, `leads/`, `health/`,
  `admin/`, `site-settings/`, `static-pages/`, `seed/`, `migrate-owner-ids/`

**Contexts:**

- Xóa: `CartContext.tsx`, `BookingContext.tsx`
- `data/CombosContext.tsx`, `data/PromoCodesContext.tsx` — xóa
- Giữ: `AuthContext`, `LangContext`, `SiteSettingsContext`, `DataContext`,
  `data/{Courses,Bookings,News,Banners,Reviews}Context` (rename + adjust P1)

**Data + config:**

- Xóa: `supabase-schema.sql`, `supabase-payments-migration.sql`, `supabase/migrations/*.sql`
  (sẽ tạo lại schema mới ở P1)
- `.env.example` — xóa Kakao/Naver/Toss vars, thêm LiveKit vars
- Dependencies: `npm uninstall @tosspayments/tosspayments-sdk @google/genai`
  (Toss = Korean payment không cần; Gemini translate không cần cho VieLang MVP)

**Docs cũ (xóa hết trừ `VIELANG_ROADMAP.md`):**

- `BRD-golf.md`, `PHASE4_TOSS_INTEGRATION.md`, `SHADCN_REFACTOR_PLAN.md`,
  `logic_owners.md`, `booking-lifecycle.md`, `payment-flow.md`, `promo-rules.md`,
  `validation-matrix.md`, `api-permissions.md`, `auth-model.md`,
  `business-logic.md`, `i18n-currency.md`, `prompt.md`

**Scripts (`scripts/`):**

- Xóa: `test-toss-payment.mjs`, `seed-owners-and-courses.mjs`, `seed-region-owners.sql`
- Giữ: `generate-icons.mjs` (PWA icons)

**Cleanup i18n:** Xóa key golf-specific trong `translations.ts` (kakaoModalTitle, teeTime, stayPlay, elitePass, golfTours, price list...). Giữ shell keys chung.

### Phase 0b — Definition of Done

- [ ] `(main)/` chỉ còn routes VieLang-friendly (hoặc placeholder)
- [ ] Trang chủ `/` render placeholder "VieLang — Coming Soon"
- [ ] `npm run typecheck` pass
- [ ] `npm run dev` chạy được, homepage load OK
- [ ] `npm run build` pass
- [ ] Login (Email + Google) vẫn hoạt động
- [ ] `docs/` chỉ còn `VIELANG_ROADMAP.md`
- [ ] Git commit: `chore(pivot): remove golf domain code — keep VieLang skeleton`

---

## Phase 1 — Supabase schema mới (1 ngày)

**Mục tiêu:** Reset schema sang VieLang entities. Tạo project Supabase mới hoàn toàn (không migrate data cũ).

### 1.1 Supabase project mới

- Tạo trên supabase.com — region **Southeast Asia (Singapore)**
- Cập nhật `.env.local`:
  ```
  NEXT_PUBLIC_SUPABASE_URL=https://<new-project>.supabase.co
  NEXT_PUBLIC_SUPABASE_ANON_KEY=...
  SUPABASE_SERVICE_ROLE_KEY=...
  ```
- Trong Vercel: cập nhật env vars tương ứng

### 1.2 Schema SQL (`supabase/migrations/00000000000001_vielang_init.sql`)

```sql
-- users: linked to Supabase Auth via supabase_uid
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  supabase_uid UUID UNIQUE,
  email TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user','tutor','admin')),
  avatar TEXT,
  bio TEXT,
  timezone TEXT DEFAULT 'Asia/Ho_Chi_Minh',
  native_lang TEXT DEFAULT 'vi',
  learning_lang TEXT DEFAULT 'en',
  enabled BOOLEAN DEFAULT TRUE,
  disabled_reason TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- tutor_profiles: 1-1 với users role=tutor
CREATE TABLE tutor_profiles (
  user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  hourly_rate_vnd INTEGER NOT NULL,
  intro_video_url TEXT,
  intro_video_thumbnail TEXT,
  specialties TEXT[] DEFAULT '{}',
  years_experience INTEGER DEFAULT 0,
  certifications TEXT[] DEFAULT '{}',
  languages_spoken TEXT[] DEFAULT '{}',
  is_approved BOOLEAN DEFAULT FALSE,
  approved_at TIMESTAMPTZ,
  rating_avg NUMERIC(2,1) DEFAULT 0,
  session_count INTEGER DEFAULT 0
);

CREATE TABLE courses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title_vn TEXT NOT NULL,
  title_en TEXT NOT NULL,
  description_vn TEXT,
  description_en TEXT,
  level TEXT CHECK (level IN ('A1','A2','B1','B2','C1','C2')),
  category TEXT,
  image TEXT,
  tutor_id UUID REFERENCES users(id) ON DELETE SET NULL,
  price_vnd INTEGER NOT NULL,
  duration_min INTEGER DEFAULT 60,
  is_published BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE availability (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tutor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  weekday SMALLINT NOT NULL CHECK (weekday BETWEEN 0 AND 6),
  start_time TIME NOT NULL,
  end_time TIME NOT NULL,
  UNIQUE (tutor_id, weekday, start_time)
);

CREATE TABLE sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  student_id UUID NOT NULL REFERENCES users(id),
  tutor_id UUID NOT NULL REFERENCES users(id),
  course_id UUID REFERENCES courses(id) ON DELETE SET NULL,
  scheduled_at TIMESTAMPTZ NOT NULL,
  duration_min INTEGER NOT NULL DEFAULT 60,
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending','confirmed','live','completed','cancelled','no_show')),
  livekit_room_name TEXT UNIQUE,
  price_vnd INTEGER NOT NULL,
  student_notes TEXT,
  tutor_notes TEXT,
  cancelled_by UUID REFERENCES users(id),
  cancelled_reason TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_sessions_tutor_date ON sessions(tutor_id, scheduled_at);
CREATE INDEX idx_sessions_student ON sessions(student_id);

CREATE TABLE reviews (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID UNIQUE NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  student_id UUID NOT NULL REFERENCES users(id),
  tutor_id UUID NOT NULL REFERENCES users(id),
  rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
  comment TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE materials (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  type TEXT CHECK (type IN ('pdf','video','link','image')),
  url TEXT NOT NULL,
  order_index INTEGER DEFAULT 0
);

CREATE TABLE banners (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title_vn TEXT, title_en TEXT,
  subtitle_vn TEXT, subtitle_en TEXT,
  image TEXT NOT NULL,
  link TEXT,
  order_index INTEGER DEFAULT 0,
  is_active BOOLEAN DEFAULT TRUE
);

CREATE TABLE news (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title_vn TEXT NOT NULL, title_en TEXT NOT NULL,
  content_vn TEXT, content_en TEXT,
  category TEXT,
  image TEXT,
  published_at TIMESTAMPTZ DEFAULT NOW(),
  is_published BOOLEAN DEFAULT FALSE
);
```

### 1.3 Seed data initial (`src/app/api/seed/route.ts`)

- 1 admin: `admin@vielang.com`
- 3 tutors demo: `alice@vielang.com`, `bob@vielang.com`, `chi@vielang.com`
  — mỗi tutor có profile + availability + 1-2 courses
- 2 students demo
- 5 courses (IELTS Speaking, Business English, Kids English, TOEIC, General B1)
- 3 banners, 2 news

### 1.4 Server helpers cần rewrite

- `src/lib/supabase.ts` — reset (chỉ giữ service_role client + basic helpers)
- `src/lib/seed-data.ts` — rewrite full sang VieLang entities
- `src/lib/schemas/` — thêm zod schema cho `Course`, `Session`, `Review`, `TutorProfile`, `Availability`
- `src/contexts/data/*` — rewrite `useResource.ts` để hit endpoints VieLang

### 1.5 Definition of Done

- [ ] Supabase project mới chạy, query bằng SQL Editor OK
- [ ] `POST /api/seed` populate demo data thành công
- [ ] `SELECT * FROM courses` trả 5 rows
- [ ] Homepage list được 3 tutors demo (kể cả UI vẫn placeholder)

---

## Phase 2 — Auth (0.5 ngày)

**Mục tiêu:** Auth flow đã có sẵn (Supabase SSR + Google). Chỉ cần adjust cho VieLang.

### 2.1 Refactor

- `src/app/auth/callback/route.ts` — sau khi exchange code, upsert `users` row với
  `supabase_uid`, `email`, `name`, default `role='user'` (đã có logic
  `resolveOrCreateSocialUser` — rename + update)
- Bỏ code liên quan legacy `ownerId` region trong callback
- 3 demo login button trên `LoginPage`:
  - `student@demo.com` → role=user
  - `tutor@demo.com` → role=tutor
  - `admin@demo.com` → role=admin
  - Set cookie `vielang_demo_user` (đã update middleware)
- Routing sau login (trong `AuthContext.tsx`):
  - `user` → `/`
  - `tutor` → `/tutor`
  - `admin` → `/admin`

### 2.2 Middleware

- Đã update `src/middleware.ts` để protect `/tutor/*` (Phase 0a)
- Không cần thêm gì

### 2.3 Definition of Done

- [ ] Google OAuth login working end-to-end
- [ ] Email/password login working
- [ ] 3 demo login button work + set correct role
- [ ] Redirect sau login đúng role

---

## Phase 3 — Landing + Tutor discovery (2 ngày)

### 3.1 Homepage (`src/app/(main)/page.tsx` + `HomeContent.tsx`)

- Hero: title bilingual, CTA "Find a Tutor"
- Section 1: Banners carousel (dùng `embla-carousel` đã có)
- Section 2: Featured Tutors (top 6 rating) — grid card
- Section 3: Courses category grid (IELTS/Business/Kids/TOEIC/General)
- Section 4: How it works (3 bước)
- Section 5: Testimonials (từ reviews rating 5)
- Section 6: News/Blog latest 3

### 3.2 Tutor list (`src/app/(main)/tutors/page.tsx`)

- Filter query params: `?level=B2&category=IELTS&sort=rating&page=1`
- Server component fetches via Supabase, SSR-first paint
- Client filter UI updates URL, triggers re-fetch
- Card: avatar, name, specialty tags, rating, price/hour, "View Profile"

### 3.3 Tutor detail (`src/app/(main)/tutors/[id]/page.tsx`)

- Hero: avatar, name, rating, specialty tags
- About: bio, years exp, certifications, languages
- Intro video embed
- Courses của tutor
- Availability preview 7 ngày tới (calendar)
- Reviews list (paginated)
- CTA: "Book a Session" (chưa login → redirect `/login?redirect=...`)

### 3.4 API Route Handlers mới

```
src/app/api/tutors/route.ts               → GET tutors (filter + sort + paginate)
src/app/api/tutors/[id]/route.ts          → GET tutor detail
src/app/api/tutors/[id]/courses/route.ts  → GET courses của tutor
src/app/api/tutors/[id]/reviews/route.ts  → GET reviews của tutor
```

### 3.5 Definition of Done

- [ ] Homepage responsive, animation mượt
- [ ] Tutor list filter/sort updates URL + refetches
- [ ] Tutor detail hiển thị đủ info + reviews thật

---

## Phase 4 — Booking flow (3 ngày)

### 4.1 Availability calculation

Server helper `src/lib/booking.ts`:

```typescript
export async function getAvailableSlots(tutorId: string, from: Date, to: Date) {
  // 1. Fetch recurring availability rows
  // 2. Generate slots per day in range
  // 3. Subtract sessions with status IN (pending, confirmed) trong range
  // 4. Return { date, startTime, endTime, durationMin }[]
}
```

### 4.2 Booking UI (dialog)

- `src/components/session/BookingDialog.tsx` (thay thế `BookingDialog.tsx` cũ)
- Step 1: chọn ngày (calendar 14 ngày tới, disabled=slot=0)
- Step 2: chọn giờ (list slot trống ngày đó)
- Step 3: chọn course (dropdown courses của tutor)
- Step 4: notes optional + confirm
- Success: redirect `/my-sessions?highlight=<id>`

### 4.3 API `POST /api/sessions`

```typescript
// src/app/api/sessions/route.ts
export async function POST(req: NextRequest) {
  const auth = await authenticate(req);
  if ('response' in auth) return auth.response;

  const body = await req.json();
  const parsed = createSessionSchema.parse(body); // zod

  // Validate tutor.is_approved, slot còn trống (conflict check với FOR UPDATE)
  // Insert với livekit_room_name = `session-${uuid}`, status='pending'
  return NextResponse.json({ session });
}
```

Concurrency: transaction PostgreSQL — nếu Supabase JS chưa support, dùng RPC function.

### 4.4 MyPage — Student view (`src/app/(main)/my-sessions/page.tsx`)

- 3 tabs: Upcoming / Completed / Cancelled
- Card: tutor avatar+name, date+time (theo `user.timezone`),
  course title, status badge, action buttons:
  - `pending`/`confirmed`: "Join Class" (bật khi < 15ph trước giờ), "Reschedule", "Cancel"
  - `completed`: "Leave Review" (nếu chưa review)
- Countdown live cho session sắp diễn ra

### 4.5 API `PATCH /api/sessions/[id]`

- Cancel: student hoặc tutor hoặc admin
- Reschedule: chỉ khi > 24h trước giờ
- Chỉ update field cho phép (status, scheduled_at, notes) — không cho update tutor_id/student_id/id

### 4.6 Definition of Done

- [ ] Book session end-to-end thành công
- [ ] Không book được 2 sessions trùng slot
- [ ] MyPage hiển thị đúng phân tab
- [ ] Cancel/reschedule hoạt động với confirmation modal

---

## Phase 5 — LiveKit integration (2-3 ngày)

**Mục tiêu:** Video call hoạt động local + prod, phân quyền theo session.

### 5.1 Docker Compose local (`docker-compose.dev.yml`)

```yaml
services:
  livekit:
    image: livekit/livekit-server:latest
    command: --config /etc/livekit.yaml
    ports:
      - '7880:7880'
      - '7881:7881'
      - '50000-60100:50000-60100/udp'
    volumes: ['./livekit.yaml:/etc/livekit.yaml']
    depends_on: [redis]
  redis:
    image: redis:7-alpine
    ports: ['6379:6379']
```

**`livekit.yaml`:**

```yaml
port: 7880
rtc:
  tcp_port: 7881
  port_range_start: 50000
  port_range_end: 60100
  use_external_ip: false
keys:
  devkey: devsecret_at_least_32_chars_long_xxxxxxxxxxx
redis:
  address: redis:6379
```

Chạy: `docker compose -f docker-compose.dev.yml up`

### 5.2 Token endpoint (`src/app/api/livekit/token/route.ts`)

```typescript
import { NextRequest, NextResponse } from 'next/server';
import { AccessToken } from 'livekit-server-sdk';
import { authenticate } from '@/lib/auth-server';
import { supabase } from '@/lib/supabase';

export async function POST(req: NextRequest) {
  const auth = await authenticate(req);
  if ('response' in auth) return auth.response;

  const { session_id } = await req.json();
  const { data: session, error } = await supabase
    .from('sessions')
    .select('*')
    .eq('id', session_id)
    .single();
  if (error || !session) return NextResponse.json({ error: 'not found' }, { status: 404 });

  // Only participants can join
  const uid = auth.user.id;
  if (uid !== session.student_id && uid !== session.tutor_id && auth.user.role !== 'admin') {
    return NextResponse.json({ error: 'forbidden' }, { status: 403 });
  }

  // Join window: 15min before → duration+15min after
  const start = new Date(session.scheduled_at).getTime();
  const end = start + session.duration_min * 60_000;
  const now = Date.now();
  if (now < start - 15 * 60_000 || now > end + 15 * 60_000) {
    return NextResponse.json({ error: 'outside_window' }, { status: 400 });
  }

  const token = new AccessToken(process.env.LIVEKIT_API_KEY!, process.env.LIVEKIT_API_SECRET!, {
    identity: uid,
    name: auth.user.email,
    ttl: '2h',
  });
  token.addGrant({
    room: session.livekit_room_name,
    roomJoin: true,
    canPublish: true,
    canSubscribe: true,
    canPublishData: true,
  });

  // Bump status to 'live' on first join (idempotent — CHECK is null OR pending)
  await supabase
    .from('sessions')
    .update({ status: 'live' })
    .eq('id', session.id)
    .in('status', ['pending', 'confirmed']);

  return NextResponse.json({
    token: await token.toJwt(),
    url: process.env.NEXT_PUBLIC_LIVEKIT_WS_URL,
  });
}
```

### 5.3 Room page (`src/app/session/[id]/room/page.tsx`)

Client component dùng `@livekit/components-react`:

```tsx
'use client';
import { LiveKitRoom, VideoConference, RoomAudioRenderer } from '@livekit/components-react';
import '@livekit/components-styles';

export default function SessionRoom({ params }: { params: { id: string } }) {
  // fetch token via POST /api/livekit/token
  // render <LiveKitRoom token={token} serverUrl={url} connect video audio>
  //   <VideoConference /><RoomAudioRenderer />
  // </LiveKitRoom>
}
```

### 5.4 Webhook (`src/app/api/livekit/webhook/route.ts`)

- Verify HMAC signature với API key
- Events: `participant_joined`, `participant_left`, `room_finished`
- On `room_finished`: nếu session.status='live' → set 'completed'

### 5.5 Definition of Done

- [ ] `docker compose -f docker-compose.dev.yml up` chạy LiveKit + Redis
- [ ] Student và Tutor cùng join room, thấy nhau
- [ ] Chat + screen share hoạt động
- [ ] Người không phải participant KHÔNG lấy được token (403)
- [ ] Sau khi room_finished → session.status = completed

---

## Phase 6 — Admin Dashboard (2 ngày)

**Mục tiêu:** Rewrite `src/components/admin/AdminDashboard.tsx` cho VieLang.

### 6.1 Sections mới

`src/components/admin/sections/`:

- `OverviewSection.tsx` — KPI cards + Recharts (đã có Recharts)
- `UsersSection.tsx` — list/filter, ban/promote
- `TutorApprovalsSection.tsx` — queue pending tutor apply
- `TutorsSection.tsx` — all tutors
- `CoursesSection.tsx` — CRUD
- `SessionsSection.tsx` — all sessions, filter, cancel/refund manual
- `BannersManager.tsx` — giữ (đã đủ tốt, chỉ đổi labels)
- `ContentSection.tsx` — news + static pages editor
- `ReviewsSection.tsx` — moderation
- `SettingsSection.tsx` — giữ (site settings), bỏ trường liên quan golf

### 6.2 KPI Overview

- Total users (student + tutor separate)
- Sessions today / week / month
- Revenue (VND) week/month (dù chưa charge — accumulate `sum(price_vnd) where status=completed`)
- Pending tutor approvals count
- Average rating overall
- Chart: sessions per day (last 30 days)

### 6.3 API endpoints mới

```
GET  /api/admin/kpis
GET  /api/admin/users?role=&search=&page=
PUT  /api/admin/users/[id]/role
GET  /api/admin/tutors/pending
PUT  /api/admin/tutors/[id]/approve
PUT  /api/admin/tutors/[id]/reject
GET  /api/admin/sessions?status=&from=&to=
```

### 6.4 Definition of Done

- [ ] Admin login → `/admin` với KPI đúng
- [ ] Duyệt tutor pending → tutor xuất hiện public list
- [ ] CRUD course + banner + news
- [ ] Xem list sessions toàn hệ thống

---

## Phase 7 — Tutor Dashboard (1-2 ngày)

**Mục tiêu:** Rename `/owner` → `/tutor`, adjust data source.

### 7.1 Routes

- `src/app/tutor/[section]/page.tsx` — mirror của admin nhưng data filter theo `tutor_id = req.user.id`
- Xóa `src/app/owner/` sau khi migrate

### 7.2 Sections

`src/components/tutor/sections/`:

- `OverviewSection.tsx` — KPI cá nhân
- `AvailabilitySection.tsx` — editor lịch rảnh recurring (grid 7×time slots)
- `SessionsSection.tsx` — upcoming / history
- `MyCoursesSection.tsx` — CRUD courses mình tạo
- `MaterialsSection.tsx` — materials cho courses của mình
- `MyReviewsSection.tsx` — read-only, xem reviews student
- `ProfileSection.tsx` — edit bio, intro video, rate, specialties, certs

### 7.3 Availability Editor

- Grid 7 days × time slots (30min steps từ 06:00-22:00)
- Click drag → add availability row
- Delete slot → xóa row
- Preview: "Học viên sẽ thấy X slots/tuần rảnh"

### 7.4 Definition of Done

- [ ] Tutor login → `/tutor` dashboard
- [ ] Set availability → student thấy slot
- [ ] Manage sessions cá nhân
- [ ] KHÔNG thấy được data của tutor khác (test kỹ ownership check)

---

## Phase 8 — Reviews + Materials (1 ngày)

### 8.1 Review flow

- Sau session completed → student thấy nút "Leave Review" trong `/my-sessions`
- Modal: rating stars, comment textarea
- `POST /api/reviews` — validate: chỉ student của session, chỉ 1 review/session
- Trigger DB: update `tutor_profiles.rating_avg` và `session_count`

### 8.2 Materials

- Trong course detail (admin/tutor view): upload materials (PDF, video URL, external link)
- Student booked course → thấy materials list trong session detail
- Access: chỉ student có session confirmed/completed với course

### 8.3 Definition of Done

- [ ] Review submit thành công, hiển thị trên tutor page
- [ ] Rating avg update đúng
- [ ] Materials attach vào course, student truy cập được

---

## Phase 9 — Docker + Caddy (LiveKit prod) (1 ngày)

### 9.1 docker-compose.yml (prod trên DO)

```yaml
services:
  caddy:
    image: caddy:2-alpine
    ports: ['80:80', '443:443']
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
  livekit:
    image: livekit/livekit-server:latest
    command: --config /etc/livekit.yaml
    volumes: ['./livekit.yaml:/etc/livekit.yaml']
    ports:
      - '7881:7881'
      - '50000-60100:50000-60100/udp'
    depends_on: [redis]
    restart: unless-stopped
  redis:
    image: redis:7-alpine
    volumes: [redis_data:/data]
    restart: unless-stopped

volumes: { caddy_data: {}, caddy_config: {}, redis_data: {} }
```

### 9.2 Caddyfile

```
livekit.vielang.com {
  reverse_proxy livekit:7880
}
```

### 9.3 livekit.yaml (prod)

```yaml
port: 7880
rtc:
  tcp_port: 7881
  port_range_start: 50000
  port_range_end: 60100
  use_external_ip: true
keys:
  <API_KEY>: <API_SECRET_at_least_32_chars>
redis:
  address: redis:6379
webhook:
  api_keys: [<API_KEY>]
  urls:
    - https://vielang.com/api/livekit/webhook
```

### 9.4 Definition of Done

- [ ] SSH lên droplet, `docker compose up -d` chạy toàn stack
- [ ] `curl https://livekit.vielang.com/` returns healthy
- [ ] Từ Next.js prod (Vercel) gọi vào livekit.vielang.com OK
- [ ] Webhook Vercel nhận event từ LiveKit

---

## Phase 10 — Deploy Production (0.5-1 ngày)

### 10.1 Vercel (Next.js app)

- GitHub push → Vercel auto-deploy
- Env vars trên Vercel:
  ```
  NEXT_PUBLIC_SUPABASE_URL, NEXT_PUBLIC_SUPABASE_ANON_KEY, SUPABASE_SERVICE_ROLE_KEY
  LIVEKIT_API_KEY, LIVEKIT_API_SECRET
  NEXT_PUBLIC_LIVEKIT_WS_URL=wss://livekit.vielang.com
  NEXT_PUBLIC_SITE_URL=https://vielang.com
  ```
- Domain: `vielang.com` + `www.vielang.com` → Vercel
- Auto deploy on push to `main`

### 10.2 DO Droplet (LiveKit)

- Droplet: Ubuntu 22.04, 2vCPU / 4GB RAM (start size)
- Install docker + docker-compose
- DNS: `livekit.vielang.com` → droplet IP (A record)
- Firewall UFW:
  - 22/tcp (SSH), 80/tcp, 443/tcp (Caddy)
  - 7881/tcp (LiveKit TCP fallback)
  - 50000-60100/udp (LiveKit media)
- Clone repo, tạo `.env` prod, `docker compose up -d`
- Caddy tự lấy HTTPS cert Let's Encrypt

### 10.3 Supabase Auth prod config

- Google OAuth Console: add authorized redirect `https://vielang.com/auth/callback`
- Supabase Auth Settings: allowed redirect URLs list

### 10.4 Smoke test checklist

- [ ] `https://vielang.com` load OK
- [ ] Login Google từ prod domain
- [ ] Book session end-to-end
- [ ] Video call giữa 2 người ở 2 mạng khác nhau (thử mobile 4G)
- [ ] LiveKit logs không có TURN/ICE failure
- [ ] SSL A+ trên ssllabs

---

## Ngoài roadmap (Post-MVP)

Sau khi MVP live:

- **Payment gateway** — VNPay + Stripe (currently free/manual)
- **Email/SMS notifications** — Resend (session confirm, reminders T-24h)
- **Group classes** — 1 tutor + nhiều student trong 1 LiveKit room
- **Recording** — LiveKit egress record session để student review sau
- **Whiteboard** — Excalidraw / TLDraw embed trong room
- **Mobile app** — React Native share types + API
- **Realtime notifications** — Supabase Realtime subscriptions
- **AI assistant** — Post-session summary, pronunciation feedback

---

## Rủi ro & Mitigation

| Rủi ro                                          | Impact                   | Mitigation                                                       |
| ----------------------------------------------- | ------------------------ | ---------------------------------------------------------------- |
| LiveKit UDP bị chặn ở mạng học viên (VN mobile) | Video không kết nối      | TURN over TCP fallback (7881), test trên 4G Vietnam              |
| Timezone bug (VN/US/KR khác nhau)               | Book sai giờ             | Store UTC ở DB, format client-side theo `user.timezone`          |
| Double-book race condition                      | 2 student book cùng slot | DB constraint + `SELECT FOR UPDATE` trong RPC function           |
| Vercel serverless timeout 10s                   | Booking flow chậm        | Booking API < 3s; heavy work (email) → background                |
| DO droplet bandwidth cho video                  | Cost hoặc throttle       | Monitor egress, upscale khi cần hoặc switch LiveKit Cloud        |
| Supabase RLS misconfig                          | Data leak                | Server-only routes với service_role key — không dùng RLS cho MVP |

---

## Checklist tổng bàn giao MVP

- [ ] P0-P10 DoD tick xong
- [ ] Test E2E: student đăng ký → book → join video → review
- [ ] Admin duyệt được tutor mới
- [ ] Tutor set availability + dạy session
- [ ] Responsive mobile (iOS Safari + Android Chrome)
- [ ] Landing page SEO cơ bản (meta tags, OG tags)
- [ ] `CLAUDE.md` reflect production stack
- [ ] `README` có hướng dẫn dev setup + deploy
- [ ] Backup Supabase auto snapshot
