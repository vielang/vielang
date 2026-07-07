# VieLang — UI + Core Feature Polish Audit

**Date:** 2026-07-02
**Purpose:** Freeze feature work and take every existing surface up to industry-professional quality. No new features until the shipped ones look and behave like they belong in Preply/Cambly/Notion company.

**Scope:** 10 core surfaces + shared foundations. Batched into 5 focused execution PRs (§5).

---

## 1. Foundational issues (touch every surface)

These are cross-cutting and worth fixing first. Every polish batch below assumes these are done.

### 1.1 Button consistency

**Problem:** ~30 hand-rolled `<button>` elements use ad-hoc Tailwind classes (`h-9 px-3 rounded-lg bg-brand text-white text-xs font-semibold ...`) instead of the shadcn `<Button>` primitive. Focus rings look different across the app, hover states diverge, disabled opacity varies.

**Fix:**

- Audit shadcn `Button` variants (default/outline/secondary/ghost/destructive/link + sizes xs/sm/default/lg/icon).
- Replace hand-rolled buttons where variant matches. Where a bespoke look is needed (e.g. status pills), leave the pill but stop calling it a button.
- Consistent focus ring: `focus-visible:ring-3 ring-ring/50 border-ring` (already in Button primitive).

### 1.2 Focus + keyboard nav

**Problem:** Focus rings inconsistent because of the button issue above. Some interactive elements (custom cards, filter pills) are `<button type="button">` but have no visible focus. Tab order not verified.

**Fix:**

- Standardize `focus-visible:` classes via `focus-ring` utility in globals.css.
- Add `focus-visible:ring-2 focus-visible:ring-brand/40` to filter pills, session cards' clickable regions, star picker.
- Verify tab order on booking dialog + admin/tutor sidebars.

### 1.3 Typography scale

**Problem:** No documented scale. `text-2xl md:text-3xl font-serif font-bold` used inconsistently. Some section headers h1, some h2, admin overview cards use serif for the number (unusual).

**Fix:** Define scale in `globals.css` as CSS custom props + document in AUDIT:

| Role              | Class                                                                  |
| ----------------- | ---------------------------------------------------------------------- |
| Display / hero h1 | `text-4xl sm:text-5xl md:text-6xl font-serif font-bold tracking-tight` |
| Page h1           | `text-2xl md:text-3xl font-serif font-bold tracking-tight`             |
| Section h2        | `text-lg md:text-xl font-semibold tracking-tight`                      |
| Card title        | `text-sm font-semibold`                                                |
| Body              | `text-sm text-slate-700 leading-relaxed`                               |
| Muted             | `text-xs text-slate-500`                                               |
| Label / eyebrow   | `text-[10px] font-semibold uppercase tracking-wider text-slate-500`    |
| KPI number        | `text-2xl font-sans font-bold tabular-nums` (not serif)                |

### 1.4 Spacing rhythm

**Problem:** Section spacing hops between `space-y-4`, `space-y-6`, `space-y-8`. No clear rule.

**Fix:** Use a small vocabulary:

- Card interior: `p-5` (or `p-4` on mobile).
- Between sections: `space-y-8 md:space-y-12`.
- Between cards in a stack: `space-y-3`.
- Within a card: `space-y-4`.

### 1.5 State handling — loading, empty, error

**Problem:** Loading is inconsistent (some Loader2 spinner, some plain text, some nothing). Empty states show plain text without an illustration or clear next action. Error states throw `console.error` and rely on toast.

**Fix:**

- Create `<EmptyState icon label description action />` reusable component (already partially exists — reuse everywhere).
- Loading: standardize on branded Loader + optional skeleton for tables/cards.
- Errors: never bare "Something went wrong" — always cite what failed + a retry.

### 1.6 Motion budget

**Problem:** Motion is used tastefully in most places but the ReviewDialog star scales `hover:scale-110` (jarring), TutorCard fade-in delay caps at 320ms which is fine, but AdminShell tab transitions animate on every keystroke tab click (feels heavy).

**Fix:** Rule of thumb — motion for spatial transitions (drawer open, dialog appear, list append), not for hover feedback. Hover = color/border/shadow only.

### 1.7 Localization / KR debt

**Problem:** `translations.ts` is 1022 lines, ~40% KR strings that we don't ship. Header language dropdown lists KR. Footer legal block uses KR labels (from Toss compliance golf era).

**Fix:**

- Trim `translations.ts` to VN + EN only. Drop KR keys.
- Remove KR from Header language selector.
- Rewrite Footer legal block: drop the whole Toss-legacy 사업자 information section (it was for Korean payment gateway compliance we're not doing).
- Remove `L(kr, vn, en)` helper.

### 1.8 Dead code + golf legacy strings

- Footer's `SHOW_PRICING_LINK` flag + golf-tours nav links → delete (routes gone).
- ChatWidget mounted on every page but points to `/api/chat/*` routes we deleted. Either gate behind `NEXT_PUBLIC_ENABLE_CHAT=true` or remove entirely.
- SiteSettingsContext legacy fields (`businessRegistrationNumber`, `mailOrderSalesNumber`, `representativeName`, `addressKR`) → drop from interface, drop from footer.

### 1.9 Sonner (toast) styling

**Problem:** Default sonner theme doesn't match brand. `toast.success` uses green (fine), `toast.error` red (fine), but plain `toast('…')` uses stark black on light bg with system font.

**Fix:** In `providers.tsx` `<Toaster>`: set `theme="system"`, `richColors`, `position="top-right"` already OK. Add `toastOptions.classNames` mapping to shadcn `--card/--foreground/--border` tokens so toasts feel like cards.

### 1.10 Skeleton loaders

**Problem:** Only `ListPageSkeleton` component exists; not used by real pages. Server components block on Supabase — first paint is fine but a slow query stalls the whole shell.

**Fix:** For each server component that awaits >1 query, wrap the body in `<Suspense fallback={<SectionSkeleton />}>` where the fallback approximates final layout. This is Batch 4 work — noted here.

### 1.11 Accessibility

- Add `<html lang={lang}>` — currently hardcoded `lang="en"` in RootLayout despite VN default.
- Star picker in ReviewDialog: wrap in `role="radiogroup" aria-label="Rating"`; each star `role="radio" aria-checked={n===rating}`.
- Booking Dialog: on step change, `aria-live="polite"` announces "Step 2 of 4 — pick a date".
- Tables in admin: add `<caption>` (sr-only) + `<th scope="col">`.
- Color contrast: `text-slate-400` on `bg-white` is ~4.5:1 borderline WCAG AA. Prefer `text-slate-500` for muted text.

---

## 2. Per-surface deep dive

### 2.1 Header

**Current state:** Wordmark + 3 nav links + language dropdown + theme toggle + user dropdown. Mobile: hamburger sheet with same content.

**Issues:**

- Brand wordmark: `<span>VIELANG</span>` hard-uppercase in Playfair serif reads odd. Real product logos usually use a lockup (mark + wordmark image) not typed caps.
- Slogan (`{t('slogan')}`) shows a KR/golf-era string ("골프 예약의 새로운 기준" or empty). Delete or replace with "Live 1-on-1 English tutoring" in current lang.
- Nav has "Courses" tab pointing to `/courses` — but `/courses` doesn't exist as a list route. The Header link 404s when clicked. Either build a courses list or drop the tab.
- Language dropdown lists KR (per §1.7).
- User dropdown label uses `t('myPage')` which returns Korean characters when lang=KR.
- Mobile sheet header shows just "Menu" — could show user name + role badge for signed-in state.
- The tinted-brand top-of-page gradient stripe (added in P11) is 1px too thin visually.

**Priority fixes:**

- Trim to Tutors + News + (optional Courses when route exists).
- Remove KR, replace slogan with real EN/VN copy.
- Standardize dropdown item paddings (currently vary between 8 and 12 px).

### 2.2 Footer

**Current state:** Grand golf-era footer with brand, links, payment methods, business-license block.

**Issues:**

- Bg is `bg-brand` (indigo) — heavy for a footer. Preply/Notion use a subtle slate footer, not full brand color.
- Payment method logos (Visa/Mastercard/JCB) shown but we don't take payments yet. Misleading.
- Business-license section is Korean tax compliance legacy — irrelevant.
- SiteSettingsContext defaults are empty so most rows render nothing → footer feels sparse anyway.
- Social icons hidden when `settings.facebookUrl` empty, which is always.
- Legal footer strip only has copyright; missing terms/privacy/refund quick links even though `supportLinks` array includes them (they render but visually stacked with the golf-nav).

**Priority fixes:**

- Rewrite footer as slate-950 background, 3-column grid: brand + description, product (Tutors/Courses/News), company (About/Contact/FAQ/Terms/Privacy).
- Drop payment methods, business-license section.
- Bottom bar: copyright + language switcher (moved from header — or left in header, chosen once).

### 2.3 Home (`/`)

**Current state:** Hero + banners + categories emoji grid + featured tutors + how-it-works.

**Issues:**

- Hero: `text-4xl sm:text-5xl md:text-6xl` hero copy in Playfair Display. Bold + italic accent line is elegant but feels wedding-invite, not tech. Consider swapping serif italic to sans + softer weight.
- Feature triad (Certified Tutors / Live Video / Flexible Booking) below hero: fine visually but the icons live in an `emerald-100` bubble even after color migration (Batch 1 didn't sweep this).
- Banners carousel: gradient overlay is fine; CTA button "Learn more" is white-on-brand-text which reads well.
- Categories row uses emoji (🎯 💼 🎈…) — feels amateur. Replace with Lucide + subtle brand tint bubble.
- Featured tutors: TutorCard is decent, but the "From X₫/hour" label uses `text-brand` — that's now indigo which looks louder than intended on a price. Use `text-slate-900` for the number, `text-slate-500` for the "From" label.
- How-it-works: 3 cards with numbers. Numbers use `bg-brand` circle — good. Card body content lacks visual weight; no icons.
- Missing: testimonials section (roadmap called for it, deferred), news preview.
- No SEO structured data.

**Priority fixes:**

- Kill emojis in categories (P.batch2).
- Rebalance featured-tutor pricing hierarchy.
- Add icons to how-it-works cards.
- Consider swap serif italic accent → sans (subjective — test both).

### 2.4 Tutor discovery (`/tutors`)

**Current state:** Page header + filter bar + grid.

**Issues:**

- Filter bar has 2 rows: specialty + sort. On mobile these wrap fine but hierarchically the sort feels equal to specialty which is wrong (sort should be less prominent).
- No result count animation on filter change — page just re-renders without transition.
- No `?page=` pagination; will fall apart past ~30 rows.
- TutorCard has `View profile` button + implicit whole-card clickable region → double-clickable, bad a11y. Pick one: whole card link OR button.
- Card avatar ring is `ring-indigo-100` after Batch 0 fix — good.
- Session count on card ("0 sessions") shows "0" for new tutors — reads as a negative. Show only when > 0, else omit the row.
- No favorite / bookmark. Not adding, but the card layout doesn't leave room for a heart icon later; note for future.

**Priority fixes:**

- Sort should be a small dropdown, not equal pills.
- Card = single Link, remove nested button.
- Hide "0 sessions" for new tutors.
- Add empty state illustration for no-match filters.

### 2.5 Tutor detail (`/tutors/[id]`)

**Current state:** Hero (avatar + name + rating + specialties) + bio + certifications/languages + courses grid + availability + reviews. Sticky "Book a session" card on right.

**Issues:**

- Hero is dense — avatar, name, years, sessions, rating, specialties all crammed in top 200px. Could breathe.
- Certifications & Languages rendered as `<dl>` grid cards — nice but "Certifications" and "Speaks" boxes on empty tutor look barren.
- Courses grid: shows image + title + level pill + description + duration/category + price. Price alignment inconsistent (rows with long titles push the price up).
- Availability preview: weekday × slot windows, works. Missing timezone indicator.
- Reviews section: rating stars line + date + comment + reviewer name — after Phase 8 fix. Good.
- Sticky sidebar "Book a session": price + button — good. No secondary info like "usually replies within 2h" that Preply/Cambly use as trust signal.
- No "report profile" or "share profile" affordance.
- No breadcrumb (Home → Tutors → Alice).

**Priority fixes:**

- Add breadcrumb.
- Add timezone label on availability ("Times in Vietnam GMT+7 · shown in your local: [browser tz]").
- Course card: consistent height, aligned price.
- Sidebar: add response-time placeholder.

### 2.6 Booking dialog

**Current state:** 4-step (course → date → time → confirm+notes).

**Issues:**

- No progress indicator. User doesn't know how many steps left.
- Header shows `DialogTitle` "Book a session with {tutorName}" fixed across all steps; `DialogDescription` changes based on step. Descriptions read as instructions ("Pick a course to start with.") — could be more delightful.
- Course step: cards use `bg-emerald-50` on selected (was fixed by Batch 0). Missing "recommended for beginners" badge or level filter.
- Date step: shows only days with slots. Weekends might be far apart visually.
- Time step: grid of times. No timezone label. No "next day" navigation.
- Confirm step: dl-style rows. Good density. Notes textarea has placeholder but no char count.
- Back button labeled "Back" — inconsistent with the 4-step numeric hint would be nicer.
- On slot_taken 409, dialog auto-refetches and drops user back to time step but keeps date state — good. Should show a subtle toast pointing at "This slot was just taken".
- Success redirect to `/my-sessions?highlight=…` — good.

**Priority fixes:**

- Add step indicator (1/4, 2/4, 3/4, 4/4) top of dialog.
- Char count on notes (e.g. 47/1000).
- Time step: timezone label at bottom.

### 2.7 My sessions (`/my-sessions`)

**Current state:** 3 tabs (upcoming/completed/cancelled). SessionCard shows tutor avatar + name + course + when + notes + action buttons.

**Issues:**

- Tab underline animates position — good. Count badges: consistent but pending tab has no color emphasis on tutor view when there are pending.
- SessionCard is dense; on mobile the action row (Join / Leave review / Materials / Cancel) wraps to a second line awkwardly.
- No countdown: session in 4 minutes shows same time as session in 4 hours.
- "Join class" button disabled when outside window — but no explanation ("Available 15 min before start"). Users click it and get nothing.
- "Cancel" button confirmation uses `<AlertDialog>` — good. But the "cancelled_by" info isn't surfaced on the cancelled tab.
- No batch operations (select multiple, mass cancel) — probably fine to skip.
- Empty state: text + "Browse tutors" link. Illustrations missing.
- Highlight border ring `ring-2 ring-indigo-100` — nice.
- No filter by tutor or date range even on tutor's view of "my sessions" — could hurt tutors with 20+ per week.

**Priority fixes:**

- Collapse action buttons into a "more" popover at narrow widths.
- Add countdown for sessions starting within 30 min.
- Tooltip on disabled Join: "Available 15 min before start".
- Empty state gets a Lucide icon in a rounded background.

### 2.8 Video room (`/session/[id]/room`)

**Current state:** Server guard → RoomClient fetches token → LiveKit prefab room.

**Issues:**

- Loading state: `<Loader2 />` + "Preparing your session…" — fine.
- Error state: title + description + retry button. Good copy.
- The room itself is 100% LiveKit prefab. No custom footer with "End session" that returns cleanly, no branded header, no clear "Recording" indicator (once we do recording).
- On successful join, `<LiveKitRoom>` fills the screen but there's no way to see the session context (course name, other participant name) inside the room.
- Disconnect fires `router.push('/my-sessions')` — no "how did it go?" prompt to leave a review inline.

**Priority fixes:**

- Add a persistent thin top bar inside the room: course title + tutor/student name + "End session" button.
- On disconnect, if session status became completed and user is student, show ReviewDialog inline (client state, no route change).

### 2.9 Admin dashboard (`/admin`)

**Current state:** Sidebar (Overview / Sessions / Users / Pending tutors) + section content. AnimatePresence between sections.

**Issues:**

- Sidebar transitions with motion — feels heavy. Just fade in main content.
- Overview KPI cards use `font-serif` for the numbers — unusual for dashboard UX. Use sans + tabular-nums.
- Recharts line chart: fine visually. Missing axis labels + no comparison ("vs previous 30 days").
- Sessions table: no pagination. No column sort. No inline actions (view detail).
- Users table: role dropdown allows self-demote — server blocks it but UI should also gray out the option for `you`.
- Pending Tutors cards: nicely designed. Approve/Reject buttons both use full width — should be primary (Approve) + secondary (Reject) hierarchy.
- No global search across everything.
- No `/admin/settings` (implicit in shell but no route).

**Priority fixes:**

- KPI numbers → sans + tabular.
- Sessions table: pagination controls + column sort headers.
- Approve/Reject visual hierarchy.
- Grey out `admin` option in "Change role" when target === current admin.

### 2.10 Tutor dashboard (`/tutor`)

**Current state:** Sidebar (Overview / Sessions / Availability / Materials / Profile) + section content.

**Issues:**

- Sidebar badge on Sessions tab counts _pending_ — good.
- Overview: "Next 5 upcoming sessions" list is dense but readable.
- Approval banner (amber): fine copy but no next-step link ("Complete your profile to speed up approval").
- Availability: **form-based add slot** — visually low-fidelity vs the drag-select grid Preply uses. This is the biggest gap.
- Materials: table-of-links style. Type icons are good. No "preview" of media, no reorder.
- Profile: one big form. Comma-separated arrays for specialties/certifications feel low-quality vs a chip input.
- No "Preview my public page" link — should link out to `/tutors/[id]` so tutor sees exactly what students see.

**Priority fixes:**

- Add "Preview public profile" link in Profile section.
- Availability: replace form with a visual weekly grid (Preply-style click-drag). This is Batch 3 or 4 sized.
- Materials list: add drag-handle for reorder using `@dnd-kit` (already in deps).

### 2.11 Dialogs

**Review Dialog:**

- Star picker `hover:scale-110` — jarring, kill.
- No "5 star? Awesome!" feedback on high rating.
- Comment textarea has no char count.
- Post button doesn't disable when rating === 0 (default is 5, so N/A).

**Materials Dialog:**

- Icon per type — good.
- Empty state: "No materials for this course yet — ask your tutor to add some." — good.
- Error state: `no_access` copy is student-friendly. Other errors show raw code — improve.

**Booking Dialog:** covered §2.6.

**Cancel confirmation (AlertDialog):**

- Title "Cancel this session?" — good.
- Description mentions notifying other party — but we don't email yet.
- Action button says "Cancel session" — the label collision with "Cancel" cancel-button (dismiss) causes brief confusion. Rename dismiss to "Keep booking" (already) and action to "Yes, cancel".

---

## 3. Component consistency deltas

Concrete fixes to standardize:

### 3.1 Cards

Every "card" (Session, Tutor, Material, KPI, banner) uses slightly different padding + border. Standardize on:

- Base: `rounded-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-sm`
- Padding: `p-5` (or `p-4` mobile)
- Hover surface: `hover:shadow-md transition-shadow`

Extract into a `<Card>` shadcn wrapper if not already. (There is one in `components/ui/card.tsx` — not being used consistently.)

### 3.2 Tables

Admin uses raw `<table>` with hand-rolled classes. Extract into `<Table>` shadcn primitive (`components/ui/table.tsx` exists). Adds: consistent row hover, header type-scale, correct spacing, dark-mode.

### 3.3 Form inputs

Inputs styled inline in every form (booking notes, availability form, profile). Use shadcn `<Input>` / `<Textarea>` / `<Select>` (all exist). Standardize label ("eyebrow" style) + hint + error message pattern via `<FormItem>`.

### 3.4 Filter pills

Both TutorFilterBar and admin filter chips (sessions status, users role) reimplement pill styling separately. Extract into `<FilterPill active label onClick />`.

### 3.5 Status pills

STATUS_TONE mapping repeated 3 times (SessionCard, TutorSessionsSection, admin SessionsSection). Move to `src/lib/status-tone.ts`.

### 3.6 Dashboard sidebars

`AdminShell` and `TutorShell` are 90% duplicate. Extract `<DashboardShell nav sections user footer />`.

---

## 4. Copy + tone pass

Current copy is functional but blunt. Real product copy is warmer and clearer.

Examples to rewrite:

- "Book a session" → "Book your first lesson" for signed-out CTA.
- "Preparing your session…" → "Setting up your classroom…" (edtech vernacular).
- "Nothing pending — the queue is clear." → "All caught up! No tutors waiting for review."
- "It's not time yet" (room outside window) → "You're a bit early. The room opens 15 minutes before your session starts."
- Empty upcoming: "You haven't booked any upcoming sessions yet." → "No lessons booked yet. Ready to meet a tutor?"

Do a full sweep in Batch 5.

---

## 5. Polish execution batches

Five focused PR-sized batches, each 3-6 hours of work. Land one per session; verify + smoke test in between.

### Batch 1 — Foundation (this session)

Non-visual + cross-cutting:

1. Standardize Button usage in the 20 most-visible places (Header/Footer + all page CTAs).
2. Extract shared `focus-ring` class in globals.css.
3. Typography scale documented in globals.css comments + audit doc.
4. Kill KR translations from `translations.ts` (drop the KR block, ~400 lines).
5. Remove KR from Header language dropdown.
6. Rewrite Footer: drop golf-legacy legal block + payment icons + heavy brand bg; use slate-950 with 3-column layout.
7. Kill ChatWidget mount (gate behind `NEXT_PUBLIC_ENABLE_CHAT` env, default off).
8. RootLayout `<html lang={lang}>` dynamic.
9. Extract status-tone map into `lib/status-tone.ts`.
10. Sonner styling matches shadcn card tokens.

### Batch 2 — Home + Discovery

1. Home: replace category emojis with Lucide icons + tinted brand bubbles.
2. Home: rebalance featured-tutor price hierarchy.
3. Home: add icons to how-it-works cards.
4. TutorCard: single Link, no nested button; hide zero session count.
5. TutorFilterBar: sort as dropdown, specialty as pills.
6. Add branded EmptyState component + use in tutors list.
7. Add breadcrumbs on `/tutors/[id]`.

### Batch 3 — Booking + Sessions

1. Booking dialog: step indicator (1/4 chips), char count on notes, timezone label on time step.
2. Booking dialog: replace `hover:scale-110` on star with color-only.
3. SessionCard: collapse actions into popover on narrow widths.
4. SessionCard: countdown for sessions starting within 30 min.
5. SessionCard: tooltip on disabled Join.
6. Reschedule dialog — actually build (Phase 4 API supports it; UI missing).

### Batch 4 — Room + Dashboards

1. Room: persistent top bar (course + participant name + End button).
2. Room: on disconnect, if user=student + status=completed → open ReviewDialog inline.
3. AdminShell: kill inter-section motion animations; instant swap.
4. Admin sessions table: pagination + column sort using shadcn Table.
5. Admin overview KPI numbers → sans tabular-nums.
6. Admin: grey out `admin` option in role select when target === self.
7. Tutor availability: drag-select weekly grid (biggest single item).

### Batch 5 — Dialogs + Copy

1. Review dialog: aria-radiogroup, char count, high-rating micro-copy.
2. Materials dialog: better error copy for non-`no_access` codes.
3. Cancel AlertDialog action label → "Yes, cancel this session".
4. Full copy sweep per §4.
5. Extract `<Card>`, `<FilterPill>`, `<EmptyState>` and refactor call sites.
6. Verify color-contrast on `text-slate-400` → `text-slate-500` sweep for muted text.
7. Header wordmark lockup + slogan replacement.

---

## 6. What's explicitly OUT of scope

To keep momentum on polish:

- New features (payments, group class, courses CRUD, timezone respect, etc.) — reserved for P16+.
- Testing infrastructure — P12, separate track.
- Security / observability — P13/P14, separate tracks.
- Real content authoring (news, banners CMS UI) — post-launch.

---

## 7. Definition of done for the polish sprint

- Zero occurrences of `text-emerald-*` where the intent was brand primary (only status pills use emerald).
- Zero hand-rolled buttons using `bg-brand text-white text-xs …` where a `<Button>` variant fits.
- `translations.ts` under 700 lines, VN + EN only.
- Footer has no golf-legacy content.
- Every list-based page has a designed empty state.
- Every long-running server component wraps in Suspense with a skeleton.
- Manual walkthrough: Home → sign in as student → book → my-sessions → join → review → all feels like one product.

That last point is the real judge.
