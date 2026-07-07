# VieLang — Local Setup

Follow these steps to run the app locally against a fresh Supabase project.

---

## 1. Prerequisites

- Node.js 20+
- npm (bundled with Node)
- Docker Desktop — required to run the LiveKit + Redis stack that powers
  video calls
- A Supabase account (free tier works)

---

## 2. Clone + install

```bash
git clone https://github.com/vielang/vielang-v2.git
cd vielang-v2
npm install
```

---

## 3. Create the Supabase project

1. Go to https://supabase.com/dashboard → **New project**
2. Region: **Southeast Asia (Singapore)** — lowest latency for VN/KR users
3. Set a strong DB password (save it; you'll rarely need it after this)
4. Wait for provisioning (~2 minutes)

Once provisioned, grab the following from **Project Settings → API**:

| Value                                      | Env var                                    |
| ------------------------------------------ | ------------------------------------------ |
| Project URL                                | `NEXT_PUBLIC_SUPABASE_URL`, `SUPABASE_URL` |
| Project API keys → `anon` `public`         | `NEXT_PUBLIC_SUPABASE_ANON_KEY`            |
| Project API keys → `service_role` (secret) | `SUPABASE_SERVICE_ROLE_KEY`                |

---

## 4. Configure env

```bash
cp .env.example .env.local
```

Fill in the Supabase values from the previous step. LiveKit vars can stay empty
for now — they're only needed from Phase 5.

---

## 5. Apply the schema

Open the Supabase Dashboard → **SQL Editor** → **New query** and run the file:

```
supabase/migrations/20260702000001_vielang_init.sql
```

You should see 9 tables created: `users`, `tutor_profiles`, `courses`,
`availability`, `sessions`, `reviews`, `materials`, `banners`, `news`.

_(Alternative if you use the Supabase CLI: `supabase db push` from a linked
project.)_

---

## 6. Seed demo data

Start the dev server:

```bash
npm run dev
```

Then in another terminal, hit the seed endpoint. The first invocation runs
without auth as long as the `users` table is empty (bootstrap):

```bash
curl -X POST http://localhost:3000/api/seed
```

Expected response:

```json
{
  "ok": true,
  "counts": {
    "users": 6,
    "tutor_profiles": 3,
    "availability": 8,
    "courses": 5,
    "banners": 3,
    "news": 2
  }
}
```

Re-running the seed later is safe — every row is upserted by primary key.

---

## 7. Verify

Visit **http://localhost:3000** — you should see the VieLang landing page.

Sanity-check DB access:

```bash
curl http://localhost:3000/api/health
# → { "ok": true, ... }
```

Or in Supabase SQL Editor:

```sql
SELECT count(*) FROM users;    -- 6
SELECT count(*) FROM courses;  -- 5
```

---

## 8. Sign in

- **Email**: register at `/login` with any address. The user row is auto-
  provisioned on first `/api/users/me` call (role defaults to `user`).
- **Google OAuth**: configure the Google provider in Supabase Auth →
  Providers, then add `http://localhost:3000/auth/callback` to the allowed
  redirect URLs.
- **Demo quick-login** (dev only): the login page shows three colored
  buttons for Student / Tutor / Admin that skip Supabase entirely by
  setting the `vielang_demo_user` cookie. Handy for walking the app
  without provisioning real auth.

To promote yourself to admin (dev-only), run in SQL Editor:

```sql
UPDATE users SET role='admin' WHERE email='your@email.com';
```

---

## 9. Start the LiveKit stack (for video calls)

The `/session/[id]/room` route needs a running LiveKit server. Everything
is wired up in `docker/` — bring it up in one command:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

See `docker/README.md` for teardown, verify commands, and prod notes.

That launches:

| Service   | Port(s)                                | Purpose                            |
| --------- | -------------------------------------- | ---------------------------------- |
| `livekit` | 7880 (WS), 7881 (TCP), 50000-50100/UDP | SFU + TURN, participants join here |
| `redis`   | 6379                                   | LiveKit signaling backend          |

Dev credentials in `.env.local` already match `livekit.yaml`:

```
LIVEKIT_API_KEY=devkey
LIVEKIT_API_SECRET=devsecret_at_least_32_chars_long_xxxxxxxx
NEXT_PUBLIC_LIVEKIT_WS_URL=ws://localhost:7880
```

To smoke-test the room:

1. Sign in as **Student** and **Tutor** in two different browser profiles.
2. Book a session on `/tutors/<alice-id>` starting within the next 15 min.
3. On `/my-sessions`, once you're inside the join window, click **Join
   class** — you'll land on `/session/<id>/room` with video + audio.

If Docker isn't available yet, the app still boots; the room page will
render `Video not configured` because the token endpoint returns 503.

---

## Troubleshooting

| Symptom                                      | Fix                                                                                                                                |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Home returns 500                             | Check the terminal — usually missing `SUPABASE_URL`. Restart `npm run dev` after editing `.env.local`.                             |
| `POST /api/seed` returns 401                 | The `users` table already has rows. Sign in as admin (see step 8) and retry — auth will pass.                                      |
| Google callback loops back to /login         | The redirect URL isn't allowed in Supabase Auth settings. Add `http://localhost:3000/auth/callback`.                               |
| Room page stuck on "Preparing your session…" | LiveKit container not running or firewall blocked. Check `docker compose ps` and `docker logs livekit`.                            |
| Room shows "It's not time yet"               | You're outside the 15-min pre-start window. Book a session <=15 min out or edit `scheduled_at` in SQL for testing.                 |
| Types don't match the DB                     | You may have edited the schema without regenerating `src/lib/types.ts`. Keep the two in sync manually until we add a codegen step. |

---

## What's next

See `docs/VIELANG_ROADMAP.md` for the phased plan. After completing this
setup the codebase is at **Phase 5** — LiveKit self-hosted, booking and
video call flows both live. Phases 6-10 build the admin/tutor dashboards
and production deploy.
