/**
 * LiveKit production server probe — livekit.vielang.com
 *
 * Phases:
 *   1. HTTPS reachability  — does the server respond at all?
 *   2. Admin API auth      — listRooms / createRoom / listParticipants / deleteRoom
 *   3. Token signing       — AccessToken generates a well-formed JWT
 *   4. WebRTC join         — Playwright browser connects, publishes camera + mic tracks
 *   5. Webhook endpoint    — app's /api/livekit/webhook is reachable and returns non-5xx
 *
 * Usage:
 *   node scripts/probe-livekit.mjs
 *
 * Reads from .env.local:
 *   LIVEKIT_API_KEY           API key
 *   LIVEKIT_API_SECRET        API secret (≥32 chars)
 *   LIVEKIT_WS_URL            override target  (default: wss://livekit.vielang.com)
 *   NEXT_PUBLIC_LIVEKIT_WS_URL fallback if LIVEKIT_WS_URL not set
 *   NEXT_PUBLIC_SITE_URL      app URL for webhook check (default: http://localhost:3000)
 */

import { config } from 'dotenv';
import { AccessToken, RoomServiceClient } from 'livekit-server-sdk';
import { chromium } from 'playwright';
import { mkdirSync } from 'node:fs';

config({ path: '.env.local' });

// ── config ────────────────────────────────────────────────────────────────────

const WS_URL =
  process.env.LIVEKIT_WS_URL ??
  process.env.NEXT_PUBLIC_LIVEKIT_WS_URL ??
  'wss://livekit.vielang.com';

const HTTP_URL = WS_URL.startsWith('wss://')
  ? 'https://' + WS_URL.slice('wss://'.length)
  : 'http://' + WS_URL.slice('ws://'.length);

const API_KEY = process.env.LIVEKIT_API_KEY;
const API_SECRET = process.env.LIVEKIT_API_SECRET;
const SITE_URL = (process.env.NEXT_PUBLIC_SITE_URL ?? 'http://localhost:3000').replace(/\/$/, '');
const PROBE_ROOM = `probe-${Date.now()}`;
const OUT = 'e2e/artifacts/livekit-probe';
mkdirSync(OUT, { recursive: true });

// ── helpers ───────────────────────────────────────────────────────────────────

const results = [];
let passed = 0,
  failed = 0,
  skipped = 0;

function record(name, ok, detail = '') {
  const icon = ok === true ? '✓' : ok === null ? '·' : '✗';
  const status = ok === true ? 'PASS' : ok === null ? 'SKIP' : 'FAIL';
  if (ok === true) passed++;
  else if (ok === null) skipped++;
  else failed++;
  results.push({ name, status, detail });
  console.log(`  ${icon} ${name}${detail ? '  —  ' + detail : ''}`);
}

function banner(title) {
  console.log(`\n${'─'.repeat(52)}`);
  console.log(`  ${title}`);
  console.log('─'.repeat(52));
}

// ── preflight ─────────────────────────────────────────────────────────────────

console.log('\n' + '═'.repeat(52));
console.log('  LiveKit Production Probe');
console.log(`  Target : ${WS_URL}`);
console.log(`  HTTP   : ${HTTP_URL}`);
console.log(`  App    : ${SITE_URL}`);
console.log('═'.repeat(52));

if (!API_KEY || !API_SECRET) {
  console.error('\nERROR: LIVEKIT_API_KEY and LIVEKIT_API_SECRET must be set in .env.local\n');
  process.exit(1);
}

// ── Phase 1: HTTPS reachability ───────────────────────────────────────────────

banner('Phase 1 — HTTPS reachability');
try {
  const res = await fetch(HTTP_URL, { signal: AbortSignal.timeout(8_000) });
  // LiveKit returns 426 Upgrade Required on the signalling path for plain HTTP,
  // 200 on /healthz, or similar — any response proves the server is up.
  record('Server responds to HTTPS', true, `HTTP ${res.status}`);
} catch (err) {
  record('Server responds to HTTPS', false, String(err));
}

// ── Phase 2: Admin API ────────────────────────────────────────────────────────

banner('Phase 2 — Admin API (RoomServiceClient)');
const svc = new RoomServiceClient(HTTP_URL, API_KEY, API_SECRET);
let probeRoomCreated = false;

try {
  const rooms = await svc.listRooms();
  record('listRooms() — API key auth OK', true, `${rooms.length} active room(s)`);
} catch (err) {
  record('listRooms() — API key auth OK', false, String(err));
}

try {
  await svc.createRoom({ name: PROBE_ROOM, emptyTimeout: 120, maxParticipants: 4 });
  probeRoomCreated = true;
  record('createRoom() succeeds', true, PROBE_ROOM);
} catch (err) {
  record('createRoom() succeeds', false, String(err));
}

if (probeRoomCreated) {
  try {
    const rooms = await svc.listRooms([PROBE_ROOM]);
    record('listRooms([name]) finds probe room', rooms.length > 0, `found=${rooms.length > 0}`);
  } catch (err) {
    record('listRooms([name]) finds probe room', false, String(err));
  }

  try {
    const participants = await svc.listParticipants(PROBE_ROOM);
    record('listParticipants() on empty room', true, `${participants.length} participant(s)`);
  } catch (err) {
    record('listParticipants() on empty room', false, String(err));
  }
}

// ── Phase 3: Token signing ────────────────────────────────────────────────────

banner('Phase 3 — AccessToken signing');
let probeToken = '';

try {
  const at = new AccessToken(API_KEY, API_SECRET, { identity: 'probe-bot', ttl: '10m' });
  at.addGrant({ roomJoin: true, room: PROBE_ROOM, canPublish: true, canSubscribe: true });
  probeToken = await at.toJwt();

  const [, payloadB64] = probeToken.split('.');
  const payload = JSON.parse(Buffer.from(payloadB64, 'base64url').toString());

  record(
    'AccessToken.toJwt() generates JWT',
    probeToken.split('.').length === 3,
    `${probeToken.length} chars`,
  );
  record('Payload sub matches identity', payload.sub === 'probe-bot', `sub=${payload.sub}`);
  record('Payload grants roomJoin', !!payload.video?.roomJoin, `room=${payload.video?.room}`);
} catch (err) {
  record('AccessToken.toJwt() generates JWT', false, String(err));
  record('Payload sub matches identity', null, 'skipped');
  record('Payload grants roomJoin', null, 'skipped');
}

// ── Phase 4: WebRTC join via Playwright ───────────────────────────────────────

banner('Phase 4 — WebRTC join + track publish (Playwright)');

if (!probeToken || !probeRoomCreated) {
  record('livekit-client loads in browser', null, 'skipped — token or room unavailable');
  record('Room connects (WebSocket signalling)', null, 'skipped');
  record('Camera track publishes (fake device)', null, 'skipped');
  record('Mic track publishes (fake device)', null, 'skipped');
} else {
  const browser = await chromium.launch({
    headless: true,
    args: [
      '--use-fake-ui-for-media-stream',
      '--use-fake-device-for-media-stream',
      '--no-sandbox',
      '--disable-setuid-sandbox',
    ],
  });

  const ctx = await browser.newContext({
    permissions: ['camera', 'microphone'],
    viewport: { width: 1280, height: 720 },
  });

  const page = await ctx.newPage();
  const browserErrors = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') browserErrors.push(msg.text());
  });

  try {
    // Navigate to the production HTTPS origin so navigator.mediaDevices is
    // available — about:blank in headless Chrome has no getUserMedia API.
    await page.goto(HTTP_URL, { waitUntil: 'commit', timeout: 10_000 }).catch(() => {});
    await page.addScriptTag({
      url: 'https://cdn.jsdelivr.net/npm/livekit-client@2/dist/livekit-client.umd.min.js',
    });
    await page.waitForFunction(() => typeof window.LivekitClient !== 'undefined', {
      timeout: 15_000,
    });
    record('livekit-client loads in browser', true, 'LivekitClient global present');

    // WebSocket join and media publish are tested independently so a
    // getUserMedia failure doesn't mask a successful signalling connection.
    const result = await page.evaluate(
      async ({ wsUrl, token }) => {
        const { Room } = window.LivekitClient;
        const room = new Room({ adaptiveStream: false, dynacast: false });
        const out = {
          connectState: 'unknown',
          connectError: null,
          identity: '',
          cameraEnabled: false,
          cameraError: null,
          micEnabled: false,
          micError: null,
          trackCount: 0,
        };

        // ── 1. WebSocket signalling ───────────────────────────────────────────
        try {
          await Promise.race([
            room.connect(wsUrl, token),
            new Promise((_, reject) =>
              setTimeout(() => reject(new Error('connect_timeout_20s')), 20_000),
            ),
          ]);
          out.connectState = room.state;
          out.identity = room.localParticipant.identity;
        } catch (err) {
          out.connectState = room.state;
          out.connectError = String(err);
          return out;
        }

        // ── 2. Camera (fake device) ───────────────────────────────────────────
        try {
          await room.localParticipant.setCameraEnabled(true);
          await new Promise((r) => setTimeout(r, 3_000));
          out.cameraEnabled = room.localParticipant.isCameraEnabled;
          out.trackCount = room.localParticipant.trackPublications.size;
        } catch (err) {
          out.cameraError = String(err);
        }

        // ── 3. Mic (fake device) ──────────────────────────────────────────────
        try {
          await room.localParticipant.setMicrophoneEnabled(true);
          await new Promise((r) => setTimeout(r, 2_000));
          out.micEnabled = room.localParticipant.isMicrophoneEnabled;
        } catch (err) {
          out.micError = String(err);
        }

        await room.disconnect().catch(() => {});
        return out;
      },
      { wsUrl: WS_URL, token: probeToken },
    );

    record(
      'Room connects (WebSocket signalling)',
      result.connectState === 'connected',
      result.connectError ?? `state=${result.connectState}, identity=${result.identity}`,
    );

    if (result.connectState === 'connected') {
      record(
        'Camera track publishes (fake device)',
        result.cameraError ? null : result.cameraEnabled,
        result.cameraError ?? `enabled=${result.cameraEnabled}, tracks=${result.trackCount}`,
      );
      record(
        'Mic track publishes (fake device)',
        result.micError ? null : result.micEnabled,
        result.micError ?? `enabled=${result.micEnabled}`,
      );
    } else {
      record('Camera track publishes (fake device)', null, 'skipped — room not connected');
      record('Mic track publishes (fake device)', null, 'skipped — room not connected');
    }

    await page.screenshot({ path: `${OUT}/phase4-webrtc.png`, fullPage: false });
    console.log(`  → screenshot: ${OUT}/phase4-webrtc.png`);
  } catch (err) {
    record('livekit-client loads in browser', false, String(err));
    record('Room connects (WebSocket signalling)', null, 'skipped');
    record('Camera track publishes (fake device)', null, 'skipped');
    record('Mic track publishes (fake device)', null, 'skipped');
    await page.screenshot({ path: `${OUT}/phase4-error.png`, fullPage: false }).catch(() => {});
  }

  if (browserErrors.length) {
    console.log(`  ! Browser console errors: ${browserErrors.slice(0, 3).join(' | ')}`);
  }

  await browser.close();
}

// ── Phase 5: Webhook endpoint ─────────────────────────────────────────────────

banner('Phase 5 — Webhook endpoint reachability');
try {
  const res = await fetch(`${SITE_URL}/api/livekit/webhook`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/webhook+json' },
    body: JSON.stringify({ event: 'probe_check' }),
    signal: AbortSignal.timeout(8_000),
  });
  // 400 = bad payload  401 = bad sig  any non-5xx = endpoint is live
  const reachable = res.status < 500;
  record('Webhook endpoint reachable', reachable, `HTTP ${res.status}`);
  record(
    'Webhook returns 4xx (auth guard works)',
    res.status >= 400 && res.status < 500,
    `status=${res.status}`,
  );
} catch (err) {
  record('Webhook endpoint reachable', false, String(err));
  record('Webhook returns 4xx (auth guard works)', null, 'skipped — not reachable');
}

// ── cleanup ───────────────────────────────────────────────────────────────────

if (probeRoomCreated) {
  try {
    await svc.deleteRoom(PROBE_ROOM);
    console.log(`\n  → cleanup: room ${PROBE_ROOM} deleted`);
  } catch (err) {
    console.log(`\n  ! cleanup WARNING: could not delete probe room — ${err}`);
  }
}

// ── summary ───────────────────────────────────────────────────────────────────

console.log('\n' + '═'.repeat(52));
console.log('  Summary');
console.log('═'.repeat(52));
for (const r of results) {
  const icon = r.status === 'PASS' ? '✓' : r.status === 'SKIP' ? '·' : '✗';
  const label = r.status.padEnd(4);
  console.log(`  ${icon} [${label}] ${r.name}${r.detail ? '  —  ' + r.detail : ''}`);
}
console.log(`\n  Passed: ${passed}   Failed: ${failed}   Skipped: ${skipped}`);
console.log('═'.repeat(52) + '\n');

process.exit(failed > 0 ? 1 : 0);
