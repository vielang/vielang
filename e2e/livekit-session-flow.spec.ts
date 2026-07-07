import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test';
import { createClient, type SupabaseClient } from '@supabase/supabase-js';
import { randomUUID } from 'node:crypto';
import { config as loadDotenv } from 'dotenv';

// Playwright's global env doesn't include our SUPABASE_SERVICE_ROLE_KEY (the
// webServer child inherits it, but the test runner itself does not). Load
// .env.local explicitly so the setup helpers can talk to Supabase directly.
loadDotenv({ path: '.env.local', quiet: true });

/**
 * End-to-end verification of the multi-participant LiveKit flow that the API
 * smoke test can't cover: real WebRTC + LiveKit permission events + client
 * chat merge + webhook attendance capture.
 *
 * Only runs on chromium — the fake-media Chrome flags are a Chromium-only
 * feature, and running WebRTC on Firefox/WebKit needs different orchestration
 * that isn't worth the maintenance for these flows.
 *
 * Each test creates + tears down its own session so state (session_admissions
 * rows, deny-locks) never leaks between tests. Cleanups are best-effort so a
 * failing assertion doesn't leave orphan sessions in the DB.
 */

const TUTOR = {
  id: '10000000-0000-4000-8000-000000000003',
  role: 'tutor',
  email: 'chi@vielang.com',
  name: 'Chi Tran',
};
const STUDENT = {
  id: '20000000-0000-4000-8000-000000000001',
  role: 'user',
  email: 'student1@demo.com',
  name: 'Minh Le',
};
const STUDENT_2 = {
  id: '20000000-0000-4000-8000-000000000002',
  role: 'user',
  email: 'student2@demo.com',
  name: 'Huong Pham',
};

// Chromium fake devices need to be enabled at browser launch. The default
// chromium project uses `chrome-headless-shell`, which strips down WebRTC
// support enough that fake-media negotiation times out — running headed
// (with the full Chromium binary) avoids the timeout and still works
// unattended on a dev machine.
test.use({
  headless: false,
  channel: 'chromium',
  launchOptions: {
    args: [
      '--use-fake-ui-for-media-stream',
      '--use-fake-device-for-media-stream',
      '--autoplay-policy=no-user-gesture-required',
    ],
  },
  permissions: ['camera', 'microphone'],
});

function admin(): SupabaseClient {
  const url = process.env.SUPABASE_URL || process.env.NEXT_PUBLIC_SUPABASE_URL;
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY;
  if (!url || !key) throw new Error('Missing Supabase env — check .env.local');
  return createClient(url, key);
}

/**
 * Provision a fresh group session for one test to exercise. Returns the
 * session id + a cleanup callback. `require_admission` is exposed because
 * some paths (moderation without waiting) prefer straight admission.
 */
async function createTestSession(opts: { requireAdmission: boolean; topic?: string }) {
  const sb = admin();
  const scheduledAt = new Date(Date.now() + 3 * 60_000).toISOString();
  const roomName = `group-e2e-${randomUUID()}`;
  const { data, error } = await sb
    .from('sessions')
    .insert({
      type: 'group',
      tutor_id: TUTOR.id,
      student_id: null,
      course_id: null,
      topic_en: opts.topic ?? 'E2E test session',
      level: 'all',
      capacity: 5,
      scheduled_at: scheduledAt,
      duration_min: 30,
      status: 'confirmed',
      livekit_room_name: roomName,
      price_vnd: 0,
      require_admission: opts.requireAdmission,
    })
    .select('id')
    .single();
  if (error) throw new Error(`session insert failed: ${error.message}`);
  const sessionId = data.id;
  const { error: partErr } = await sb
    .from('session_participants')
    .upsert({ session_id: sessionId, user_id: STUDENT.id });
  if (partErr) throw new Error(`participant insert failed: ${partErr.message}`);
  return {
    sessionId,
    async cleanup() {
      // ON DELETE CASCADE takes care of messages / attendance / admissions.
      await sb
        .from('sessions')
        .delete()
        .eq('id', sessionId)
        .then(
          () => {},
          () => {},
        );
    },
  };
}

/**
 * Wire up demo auth for a Playwright context. Two moving parts:
 *   • Cookie `vielang_demo_user` → readDemoCookieUser() in the SSR helper.
 *   • localStorage `vielang_demo_user` → getAuthHeaders() on the client.
 *
 * Also preseeds `lk-user-choices` to publish-off so LiveKit's PreJoin
 * dispatches videoEnabled/audioEnabled=false when Join classroom is clicked.
 * Publishing real media over Docker's NAT'd UDP times out — the flows we're
 * validating only need the LiveKit signal channel + metadata + data channel.
 */
async function loginAs(
  ctx: BrowserContext,
  user: typeof TUTOR | typeof STUDENT | typeof STUDENT_2,
) {
  const url = new URL(process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:3000');
  await ctx.addCookies([
    {
      name: 'vielang_demo_user',
      value: encodeURIComponent(JSON.stringify(user)),
      domain: url.hostname,
      path: '/',
      httpOnly: false,
      secure: false,
      sameSite: 'Lax',
    },
  ]);
  await ctx.addInitScript((u) => {
    window.localStorage.setItem('vielang_demo_user', JSON.stringify(u));
    window.localStorage.setItem(
      'lk-user-choices',
      JSON.stringify({
        videoEnabled: false,
        audioEnabled: false,
        videoDeviceId: '',
        audioDeviceId: '',
        username: '',
      }),
    );
  }, user);
}

/**
 * Build two isolated contexts + pages, one per actor. Kept as a helper
 * because every test does the same setup and it hides the newContext ceremony
 * (permissions must be passed explicitly — they don't inherit from `test.use`).
 */
async function twoActors(browser: Browser) {
  const ctxOpts = { permissions: ['camera' as const, 'microphone' as const] };
  const tutorCtx = await browser.newContext(ctxOpts);
  const studentCtx = await browser.newContext(ctxOpts);
  await loginAs(tutorCtx, TUTOR);
  await loginAs(studentCtx, STUDENT);
  return {
    tutorCtx,
    studentCtx,
    tutorPage: await tutorCtx.newPage(),
    studentPage: await studentCtx.newPage(),
    async closeAll() {
      await tutorCtx.close();
      await studentCtx.close();
    },
  };
}

/**
 * Advance a page through the PreJoin lobby: wait for the Join button, click.
 * The button label matches PreJoinPanel's joinLabel when the gate is open.
 */
async function completePrejoin(page: Page) {
  await page.getByRole('button', { name: 'Join classroom' }).waitFor({ timeout: 20_000 });
  await page.getByRole('button', { name: 'Join classroom' }).click();
}

// Serial so the four WebRTC-heavy contexts don't fight over the shared
// docker LiveKit instance + fake media devices in the host Chrome.
test.describe.configure({ mode: 'serial' });

test.describe('LiveKit session — waiting room, admit, chat, attendance', () => {
  test('full flow: prejoin → waiting → admit → chat → DB persistence', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: true,
      topic: 'E2E waiting-room + chat + attendance',
    });
    const { tutorPage, studentPage, closeAll } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);

      // Student lands in the waiting room.
      await expect(studentPage.getByRole('heading', { name: /waiting room/i })).toBeVisible({
        timeout: 20_000,
      });

      // Tutor opens Host controls, sees Waiting section, admits.
      const hostPill = tutorPage.getByRole('button', { name: 'Open host controls' });
      await hostPill.waitFor({ timeout: 20_000 });
      await hostPill.click();
      await expect(tutorPage.getByRole('heading', { name: /waiting to join/i })).toBeVisible();
      await tutorPage.getByRole('button', { name: 'Admit' }).click();

      // Student's overlay clears once permissions flip to canPublish=true.
      await expect(studentPage.getByRole('heading', { name: /waiting room/i })).toBeHidden({
        timeout: 20_000,
      });

      // Student sends a chat message via SessionChatPanel.
      await studentPage.getByRole('button', { name: /open chat/i }).click();
      const chatInput = studentPage.getByPlaceholder('Type a message…');
      await chatInput.waitFor({ timeout: 5_000 });
      const marker = `E2E ping ${randomUUID().slice(0, 8)}`;
      await chatInput.fill(marker);
      await studentPage.getByRole('button', { name: 'Send message' }).click();

      // Close the host controls Sheet first — its backdrop would intercept
      // the next click. Escape is the standard Sheet close shortcut.
      await tutorPage.keyboard.press('Escape');
      // Tutor opens their chat drawer and sees the message. Scope the search
      // to our SessionChatPanel dialog — LiveKit's prefab chat is mounted
      // hidden and its lk-message-body element also carries the marker.
      await tutorPage.getByRole('button', { name: /open chat/i }).click();
      const tutorChatPanel = tutorPage.getByRole('dialog', { name: /session chat/i });
      await expect(tutorChatPanel.getByText(marker)).toBeVisible({ timeout: 15_000 });

      // DB persistence — sender POSTs after useChat.send resolves.
      await studentPage.waitForTimeout(1500);
      const sb = admin();
      const { data: msgs } = await sb
        .from('session_messages')
        .select('body, sender_id, client_id')
        .eq('session_id', sessionId);
      const bodies = (msgs ?? []).map((m) => m.body);
      expect(bodies, 'expected the sent message to be persisted').toContain(marker);

      // Attendance webhook fires from LiveKit as participants connect.
      await studentPage.waitForTimeout(3_000);
      const { data: attendance } = await sb
        .from('session_attendance')
        .select('user_id, joined_at, left_at')
        .eq('session_id', sessionId);
      const uids = new Set((attendance ?? []).map((a) => a.user_id));
      expect(uids.has(TUTOR.id) || uids.has(STUDENT.id)).toBeTruthy();
    } finally {
      await closeAll();
      await cleanup();
    }
  });

  test('deny flow: tutor denies waiting student → student sees denied screen', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: true,
      topic: 'E2E deny path',
    });
    const { tutorPage, studentPage, closeAll } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);

      // Student is waiting.
      await expect(studentPage.getByRole('heading', { name: /waiting room/i })).toBeVisible({
        timeout: 20_000,
      });

      // Tutor opens the drawer and clicks Deny (destructive, kicks the
      // student from the LiveKit room + writes denied_at in DB).
      await tutorPage.getByRole('button', { name: 'Open host controls' }).click();
      await expect(tutorPage.getByRole('heading', { name: /waiting to join/i })).toBeVisible();
      // Deny button carries an aria-label; matching by name picks it up.
      await tutorPage.getByRole('button', { name: /^Deny /i }).click();

      // Verify server side accepted the deny before we wait on the client
      // side transition — makes a failure of the assertion below diagnose
      // as a client bug rather than an ambiguous server/client race.
      await tutorPage.waitForTimeout(1500);
      const denyRows = await admin()
        .from('session_admissions')
        .select('denied_at')
        .eq('session_id', sessionId)
        .eq('user_id', STUDENT.id);
      expect(denyRows.data?.[0]?.denied_at, 'denied_at should be set server-side').not.toBeNull();

      // Student's LiveKit connection drops → RoomClient's onDisconnected
      // fires → phase transitions to 'denied' since everAdmitted was false.
      // Heading source: RoomClient.tsx `You weren&apos;t admitted` (curly
      // apostrophe rendered as `’`). Match with a permissive regex.
      await expect(
        studentPage.getByRole('heading', { name: /weren.{1,3}t admitted/i }),
      ).toBeVisible({ timeout: 20_000 });

      // DB should reflect the denial — denied_at set on session_admissions.
      const sb = admin();
      const { data: rows } = await sb
        .from('session_admissions')
        .select('user_id, denied_at, denied_by, admitted_at')
        .eq('session_id', sessionId)
        .eq('user_id', STUDENT.id);
      expect(rows).toHaveLength(1);
      expect(rows![0].denied_at).not.toBeNull();
      expect(rows![0].denied_by).toBe(TUTOR.id);
      expect(rows![0].admitted_at).toBeNull();

      // Refresh attempt from the student is now permanently rejected — the
      // token endpoint returns 403 admission_denied so the RoomClient shows
      // the error phase, not the waiting overlay.
      await studentPage.goto(`/session/${sessionId}/room`);
      await completePrejoin(studentPage);
      await expect(
        studentPage.getByRole('heading', { name: /host declined your request/i }),
      ).toBeVisible({ timeout: 20_000 });
    } finally {
      await closeAll();
      await cleanup();
    }
  });

  test('attendance close: leaving the room populates left_at + duration_sec', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E attendance close',
    });
    const { tutorPage, studentPage, tutorCtx, studentCtx } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);

      // With require_admission=false, student joins straight in and there
      // is no waiting overlay — VideoConference should be up quickly.
      // Wait for the LiveKit control bar to render on the student side as
      // proof the room is connected.
      await expect(studentPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });

      // Give the participant_joined webhook a moment to write the row.
      await studentPage.waitForTimeout(2_500);
      const sb = admin();
      const { data: openRow } = await sb
        .from('session_attendance')
        .select('user_id, joined_at, left_at, duration_sec')
        .eq('session_id', sessionId)
        .eq('user_id', STUDENT.id)
        .maybeSingle();
      expect(openRow, 'joined row missing').not.toBeNull();
      expect(openRow!.left_at).toBeNull();
      expect(openRow!.duration_sec).toBeNull();

      // Simulate leave by closing the student's browser context — LiveKit
      // detects the disconnect and fires participant_left.
      await studentCtx.close();

      // Poll for the row to close. participant_left arrives asynchronously
      // through the webhook so we allow a few seconds of settle time.
      await tutorPage.waitForTimeout(8_000);
      const { data: closedRow } = await sb
        .from('session_attendance')
        .select('left_at, duration_sec')
        .eq('session_id', sessionId)
        .eq('user_id', STUDENT.id)
        .maybeSingle();
      expect(closedRow?.left_at, 'left_at should populate after leave').not.toBeNull();
      expect(
        closedRow?.duration_sec,
        'duration_sec should be a non-negative int',
      ).toBeGreaterThanOrEqual(0);
    } finally {
      await tutorCtx.close();
      // studentCtx already closed inside the test body.
      await cleanup();
    }
  });

  test('chat: tutor→student direction round-trips through LiveKit + persists', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E chat tutor→student',
    });
    const { tutorPage, studentPage, closeAll } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);

      // Room UI up on both sides — control bar renders once LiveKit
      // finishes its signal handshake.
      await expect(tutorPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });
      await expect(studentPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });

      // Tutor sends.
      await tutorPage.getByRole('button', { name: /open chat/i }).click();
      const tutorInput = tutorPage.getByPlaceholder('Type a message…');
      await tutorInput.waitFor({ timeout: 5_000 });
      const marker = `From tutor ${randomUUID().slice(0, 8)}`;
      await tutorInput.fill(marker);
      await tutorPage.getByRole('button', { name: 'Send message' }).click();

      // Student receives via useChat data-channel path; we still scope to
      // our SessionChatPanel dialog to skip the hidden prefab chat.
      await studentPage.getByRole('button', { name: /open chat/i }).click();
      const studentChatPanel = studentPage.getByRole('dialog', { name: /session chat/i });
      await expect(studentChatPanel.getByText(marker)).toBeVisible({ timeout: 15_000 });

      // Persistence — sender is the tutor so sender_id must be TUTOR.id.
      await tutorPage.waitForTimeout(1500);
      const sb = admin();
      const { data: msgs } = await sb
        .from('session_messages')
        .select('body, sender_id')
        .eq('session_id', sessionId);
      const persisted = (msgs ?? []).find((m) => m.body === marker);
      expect(persisted, 'expected tutor message to be persisted').toBeTruthy();
      expect(persisted!.sender_id).toBe(TUTOR.id);
    } finally {
      await closeAll();
      await cleanup();
    }
  });

  test('kicked flow: tutor removes admitted student → student sees kicked screen', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E kicked path',
    });
    const { tutorPage, studentPage, closeAll } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);

      // Both connected — control bar rendering is proof of the LiveKit
      // signal handshake completing on both sides.
      await expect(tutorPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });
      await expect(studentPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });

      // Tutor opens host controls, finds the admitted student, removes.
      await tutorPage.getByRole('button', { name: 'Open host controls' }).click();
      const removeBtn = tutorPage.getByRole('button', { name: /Remove .* from session/i });
      await removeBtn.waitFor({ timeout: 10_000 });
      await removeBtn.click();
      // Two-step confirm in the panel; click Confirm.
      await tutorPage.getByRole('button', { name: 'Confirm' }).click();

      // Student's client fires Disconnected with reason PARTICIPANT_REMOVED,
      // and since they were admitted first (everAdmitted=true), the phase
      // transitions to 'kicked' — never seen the waiting overlay in this
      // test, so we distinguish by the exact heading text.
      await expect(
        studentPage.getByRole('heading', { name: /removed from the session/i }),
      ).toBeVisible({ timeout: 20_000 });
    } finally {
      await closeAll();
      await cleanup();
    }
  });

  test('queue position: two waiting students see their rank in the waiting room', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(120_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: true,
      topic: 'E2E queue position',
    });
    // Add the second student to session_participants so both are valid
    // guests. createTestSession already added STUDENT.
    await admin()
      .from('session_participants')
      .upsert({ session_id: sessionId, user_id: STUDENT_2.id });

    const ctxOpts = { permissions: ['camera' as const, 'microphone' as const] };
    const student1Ctx = await browser.newContext(ctxOpts);
    const student2Ctx = await browser.newContext(ctxOpts);
    await loginAs(student1Ctx, STUDENT);
    await loginAs(student2Ctx, STUDENT_2);
    const student1Page = await student1Ctx.newPage();
    const student2Page = await student2Ctx.newPage();

    try {
      // Sequential so the queue ordering is deterministic — the token
      // endpoint orders by requested_at ASC.
      await student1Page.goto(`/session/${sessionId}/room`);
      await completePrejoin(student1Page);
      await expect(student1Page.getByRole('heading', { name: /waiting room/i })).toBeVisible({
        timeout: 20_000,
      });

      await student2Page.goto(`/session/${sessionId}/room`);
      await completePrejoin(student2Page);
      await expect(student2Page.getByRole('heading', { name: /waiting room/i })).toBeVisible({
        timeout: 20_000,
      });

      // Student 1 got their token first → rank 1 of 2. But their token was
      // minted BEFORE student 2 joined, so their overlay might still say
      // "1 of 1" until they refresh. Student 2 is the definitive assertion:
      // their token response snapshot includes both waiters.
      await expect(student2Page.getByText(/#2 of 2/i)).toBeVisible({ timeout: 5_000 });
    } finally {
      await student1Ctx.close();
      await student2Ctx.close();
      await cleanup();
    }
  });

  test('capacity gate: token endpoint rejects join when the room is full', async ({
    browserName,
    request,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(30_000);

    // Provision a 1-seat group session with STUDENT already in the seat.
    // STUDENT_2 asks the token endpoint for a seat → server sees the room
    // is at capacity and returns 409 session_full. The rate-limit prefix
    // that would normally block repeated tokens is fine here — we only
    // hit the endpoint twice with different identities.
    const sb = admin();
    const scheduledAt = new Date(Date.now() + 3 * 60_000).toISOString();
    const roomName = `group-e2e-cap-${randomUUID()}`;
    const { data: session, error } = await sb
      .from('sessions')
      .insert({
        type: 'group',
        tutor_id: TUTOR.id,
        student_id: null,
        course_id: null,
        topic_en: 'E2E capacity gate',
        level: 'all',
        capacity: 2,
        scheduled_at: scheduledAt,
        duration_min: 30,
        status: 'confirmed',
        livekit_room_name: roomName,
        price_vnd: 0,
        require_admission: false,
      })
      .select('id')
      .single();
    if (error) throw new Error(`session insert failed: ${error.message}`);
    const sessionId = session.id;
    // Membership rows so no one gets rejected as "not a participant" first.
    await sb.from('session_participants').upsert([
      { session_id: sessionId, user_id: STUDENT.id },
      { session_id: sessionId, user_id: STUDENT_2.id },
    ]);
    // Simulate two occupied seats via session_attendance (the actual seat
    // presence table used by capacity check). Two distinct users with open
    // rows saturate capacity=2.
    const now = new Date().toISOString();
    await sb.from('session_attendance').insert([
      {
        session_id: sessionId,
        user_id: STUDENT.id,
        joined_at: now,
        event_id: `seat-${randomUUID()}`,
      },
      {
        session_id: sessionId,
        user_id: TUTOR.id,
        joined_at: now,
        event_id: `seat-${randomUUID()}`,
      },
    ]);

    try {
      const res = await request.post('http://localhost:3000/api/livekit/token', {
        headers: { 'X-Demo-User': JSON.stringify(STUDENT_2) },
        data: { session_id: sessionId },
      });
      const body = await res.json();
      expect(res.status(), `expected 409, got ${res.status()}: ${JSON.stringify(body)}`).toBe(409);
      expect(body.error).toBe('session_full');
    } finally {
      await sb.from('sessions').delete().eq('id', sessionId);
    }
  });

  test('end for all: tutor ends session → student sees session_ended + status=completed', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E end-for-all',
    });
    const { tutorPage, studentPage, closeAll } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);
      await expect(tutorPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });
      await expect(studentPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });

      // Tutor: open host controls, click End for everyone, confirm dialog.
      await tutorPage.getByRole('button', { name: 'Open host controls' }).click();
      await tutorPage.getByRole('button', { name: 'End for everyone' }).click();
      // AlertDialog exposes the confirm action by its label ("End session").
      await tutorPage.getByRole('button', { name: 'End session' }).last().click();

      // Student's client sees DisconnectReason.ROOM_DELETED → phase transitions
      // to 'session_ended'. Heading source: RoomClient.tsx sessionEnded.title.
      await expect(
        studentPage.getByRole('heading', { name: /the session has ended/i }),
      ).toBeVisible({ timeout: 20_000 });

      // Webhook fires room_finished → sessions.status becomes 'completed'.
      // Give the webhook a moment.
      await tutorPage.waitForTimeout(2500);
      const sb = admin();
      const { data: row } = await sb
        .from('sessions')
        .select('status')
        .eq('id', sessionId)
        .maybeSingle();
      expect(row?.status, 'status should flip to completed after end_room').toBe('completed');

      // Audit trail — recordAudit is fire-and-forget so give it a beat.
      const { data: audit } = await sb
        .from('audit_log')
        .select('action')
        .eq('entity_id', sessionId)
        .eq('action', 'session.end_room');
      expect(audit ?? []).toHaveLength(1);
    } finally {
      await closeAll();
      await cleanup();
    }
  });

  test('mute all: endpoint mutes tracks + writes audit row', async ({ browserName, request }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(30_000);

    // Direct-API test because our E2E clients join with audioEnabled=false —
    // there is no live mic track to mute in the browser flow. What we
    // validate is: (1) tutor→200 with muted counter, (2) student→403 forbid,
    // (3) an audit_log row lands.
    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E mute-all API',
    });

    try {
      const forbid = await request.post('http://localhost:3000/api/livekit/room-action', {
        headers: { 'X-Demo-User': JSON.stringify(STUDENT) },
        data: { session_id: sessionId, action: 'mute_all' },
      });
      expect(forbid.status(), 'student must not be able to mute-all').toBe(403);

      const ok = await request.post('http://localhost:3000/api/livekit/room-action', {
        headers: { 'X-Demo-User': JSON.stringify(TUTOR) },
        data: { session_id: sessionId, action: 'mute_all' },
      });
      const body = await ok.json();
      expect(ok.status(), `expected 200, got ${ok.status()}: ${JSON.stringify(body)}`).toBe(200);
      expect(body.ok).toBe(true);
      // No live participants → nothing to mute. The counter comes back 0 but
      // the endpoint still succeeded — that's the idempotent behaviour we want.
      expect(typeof body.muted).toBe('number');

      // Audit trail — one row for the successful call.
      await new Promise((r) => setTimeout(r, 800));
      const sb = admin();
      const { data: audit } = await sb
        .from('audit_log')
        .select('action, actor_id')
        .eq('entity_id', sessionId)
        .eq('action', 'session.mute_all');
      expect(audit ?? []).toHaveLength(1);
      expect(audit![0].actor_id).toBe(TUTOR.id);
    } finally {
      await cleanup();
    }
  });

  test('raise hand: student raises → host sees indicator → host lowers → student reverts', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(90_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E raise-hand',
    });
    const { tutorPage, studentPage, closeAll } = await twoActors(browser);

    try {
      await Promise.all([
        tutorPage.goto(`/session/${sessionId}/room`),
        studentPage.goto(`/session/${sessionId}/room`),
      ]);
      await Promise.all([completePrejoin(tutorPage), completePrejoin(studentPage)]);
      await expect(tutorPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });
      await expect(studentPage.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });
      // Let both LiveKit sessions settle their data-channel subscriptions —
      // the raise packet gets dropped if it arrives before the tutor has
      // subscribed to DataReceived.
      await tutorPage.waitForTimeout(2000);

      // Student raises hand. Button label matches RaiseHandButton EN copy.
      const raiseBtn = studentPage.getByRole('button', { name: /raise your hand/i });
      await raiseBtn.waitFor({ timeout: 10_000 });
      await raiseBtn.click();
      await studentPage.waitForTimeout(1500);

      // The pill flips to "Lower hand" aria after publish.
      await expect(studentPage.getByRole('button', { name: /lower your hand/i })).toBeVisible({
        timeout: 5_000,
      });

      // Tutor opens host controls; the admitted student row now carries the
      // "Hand raised" badge from HostControlsPanel.
      await tutorPage.getByRole('button', { name: 'Open host controls' }).click();
      const drawer = tutorPage.getByRole('dialog', { name: /host controls/i });
      await expect(drawer.getByText(/hand raised/i)).toBeVisible({ timeout: 10_000 });

      // Tutor clicks the Lower Hand icon button for the raised student. The
      // aria-label mirrors lc.aria.lowerHandOf(displayName), so we match by
      // "Lower" + the student name prefix.
      const lowerBtn = drawer.getByRole('button', { name: /lower .*'s hand/i }).first();
      await lowerBtn.click();

      // Student's own pill drops back to "Raise your hand" — the host's
      // { lower: identity } data message flipped local state.
      await expect(studentPage.getByRole('button', { name: /raise your hand/i })).toBeVisible({
        timeout: 10_000,
      });
    } finally {
      await closeAll();
      await cleanup();
    }
  });
});

// ---------------------------------------------------------------------------
// Camera / video track tests — separate describe so they don't block the
// serial admission/chat suite above when they time out on UDP issues.
// ---------------------------------------------------------------------------
test.describe('Camera / video track', () => {
  test('camera: PreJoin preview renders + toggle works after joining', async ({
    browser,
    browserName,
  }) => {
    test.skip(browserName !== 'chromium', 'WebRTC E2E is chromium-only');
    test.setTimeout(60_000);

    const { sessionId, cleanup } = await createTestSession({
      requireAdmission: false,
      topic: 'E2E camera toggle',
    });

    const ctxOpts = { permissions: ['camera' as const, 'microphone' as const] };
    const ctx = await browser.newContext(ctxOpts);

    // Login with video ON — fake device produces synthetic frames so
    // getUserMedia succeeds even without a real webcam.
    const url = new URL(process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:3000');
    await ctx.addCookies([
      {
        name: 'vielang_demo_user',
        value: encodeURIComponent(JSON.stringify(TUTOR)),
        domain: url.hostname,
        path: '/',
        httpOnly: false,
        secure: false,
        sameSite: 'Lax',
      },
    ]);
    await ctx.addInitScript((u) => {
      window.localStorage.setItem('vielang_demo_user', JSON.stringify(u));
      window.localStorage.setItem(
        'lk-user-choices',
        JSON.stringify({
          videoEnabled: true,
          audioEnabled: false,
          videoDeviceId: '',
          audioDeviceId: '',
          username: '',
        }),
      );
    }, TUTOR);

    const page = await ctx.newPage();
    try {
      await page.goto(`/session/${sessionId}/room`);

      // 1. PreJoin — LiveKit's prefab renders a <video> element for the local
      //    camera preview once getUserMedia resolves with the fake device.
      const previewVideo = page.locator('.vielang-prejoin video');
      await expect(previewVideo).toBeVisible({ timeout: 15_000 });

      // 2. Join the room.
      await page.getByRole('button', { name: 'Join classroom' }).waitFor({ timeout: 10_000 });
      await page.getByRole('button', { name: 'Join classroom' }).click();

      // 3. Control bar must appear — signals LiveKit room is connected.
      await expect(page.locator('.lk-control-bar')).toBeVisible({ timeout: 20_000 });

      // 4. Camera button is visible in the control bar (LiveKit renders it
      //    with visible text "Camera").
      const camBtn = page.getByRole('button', { name: /camera/i }).first();
      await expect(camBtn).toBeVisible({ timeout: 5_000 });

      // 5. Keyboard shortcut C fires a toast — either "Camera on" or "Camera
      //    off" depending on current track state. We don't assert direction
      //    because Docker's UDP blocks track publication, which can cause
      //    LiveKit to auto-revert the camera state after the toggle fires.
      await page.keyboard.press('c');
      await expect(page.getByText(/camera (on|off)/i).first()).toBeVisible({ timeout: 3_000 });
    } finally {
      await ctx.close();
      await cleanup();
    }
  });
});
