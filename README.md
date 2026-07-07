# VieLang

English learning platform with 1-on-1 video call sessions.

**Domain:** vielang.com (planned)
**Stack:** Next.js 16 (App Router), React 19, Supabase, LiveKit (self-hosted via Docker)
**Deploy:** Vercel (frontend) + DigitalOcean droplet (backend + LiveKit)

## Run Locally

```bash
npm install
cp .env.example .env.local  # fill in Supabase + LiveKit keys
npm run dev                 # http://localhost:3000
```

## Docs

- `CLAUDE.md` — project SRS and architecture reference
- `docs/VIELANG_ROADMAP.md` — phase-by-phase implementation plan
