import 'server-only';
import { fromZonedTime, format as formatInTz } from 'date-fns-tz';
import { supabase, getAvailabilityForTutor, getCourseById } from './supabase';

/**
 * Availability + booking helpers. Server-only — they use the service_role
 * client.
 *
 * Timezone model:
 *   • Every availability rule is authored by a tutor and stored as a bare
 *     weekday + HH:MM string. It's interpreted in the tutor's own timezone
 *     (users.timezone). Default is 'Asia/Ho_Chi_Minh' when the tutor row
 *     doesn't set one — matches the pre-M4 behaviour so existing rows keep
 *     resolving to Vietnam local.
 *   • sessions.scheduled_at is UTC. Anything user-facing (the `startLabel`
 *     field, calendar previews) is formatted per the caller's requested
 *     `displayTimezone` — usually the student's browser timezone. When
 *     unspecified we fall back to the tutor's timezone so the label matches
 *     the tutor's calendar and there's no drift.
 *   • DST-safe: date-fns-tz honours the IANA rules for the given zone, so a
 *     "10:00" rule stays at 10:00 wall-clock even across a DST boundary.
 */

export interface AvailabilitySlot {
  /** ISO UTC string — matches how sessions.scheduled_at is stored. */
  startAt: string;
  /** ISO UTC string. */
  endAt: string;
  /** ISO date (YYYY-MM-DD) in Vietnam local, for grouping in the UI. */
  date: string;
  /** 'HH:MM' local (Vietnam) — human-friendly label. */
  startLabel: string;
  /** Session length in minutes. */
  durationMin: number;
}

const DEFAULT_TIMEZONE = 'Asia/Ho_Chi_Minh';

const pad2 = (n: number) => n.toString().padStart(2, '0');

/** 'HH:MM:SS' or 'HH:MM' → minutes since midnight. */
function timeToMinutes(t: string): number {
  const [h = '0', m = '0'] = t.split(':');
  return Number(h) * 60 + Number(m);
}

/**
 * Compose a wall-clock time (YYYY-MM-DD + minutes) in the given timezone into
 * a UTC Date. Uses date-fns-tz so DST boundaries + non-standard offsets
 * (e.g. Asia/Kathmandu +05:45) resolve correctly without hand-rolled math.
 */
function localToUtc(dateISO: string, minutesSinceMidnight: number, timezone: string): Date {
  const h = Math.floor(minutesSinceMidnight / 60);
  const m = minutesSinceMidnight % 60;
  const local = `${dateISO}T${pad2(h)}:${pad2(m)}:00`;
  return fromZonedTime(local, timezone);
}

/** Format a UTC instant as an ISO date (YYYY-MM-DD) in the target timezone. */
function toLocalDateISO(utc: Date, timezone: string): string {
  return formatInTz(utc, 'yyyy-MM-dd', { timeZone: timezone });
}

/** Weekday (0-6, Sun-Sat) of a UTC instant in the target timezone. */
function weekdayInTz(utc: Date, timezone: string): number {
  return Number(formatInTz(utc, 'i', { timeZone: timezone })) % 7; // date-fns 'i' returns 1-7 (Mon-Sun)
}

/**
 * Compute bookable slots for a tutor across a date range.
 *
 *  1. Fetch recurring availability rows (weekly, per weekday).
 *  2. For each date in range, generate slots at `durationMin` intervals
 *     within each matching availability window.
 *  3. Fetch existing sessions in the range with status pending/confirmed/live.
 *  4. Drop slots that either (a) start in the past or (b) overlap an
 *     existing session.
 *
 * Returned slots are already sorted by startAt.
 */
export async function getAvailableSlots(opts: {
  tutorId: string;
  from: Date; // inclusive
  to: Date; // exclusive
  durationMin: number;
  now?: Date; // overridable for testing
  /** IANA zone the tutor's availability rules are authored in. */
  tutorTimezone?: string;
  /** IANA zone to format `date` + `startLabel` in — defaults to tutorTimezone. */
  displayTimezone?: string;
}): Promise<AvailabilitySlot[]> {
  const now = opts.now ?? new Date();
  const { tutorId, from, to, durationMin } = opts;
  const tutorTz = opts.tutorTimezone || DEFAULT_TIMEZONE;
  const displayTz = opts.displayTimezone || tutorTz;

  const [rules, existingSessions] = await Promise.all([
    getAvailabilityForTutor(tutorId),
    fetchSessionsInRange(tutorId, from, to),
  ]);
  if (rules.length === 0) return [];

  // Index rules by weekday for O(1) lookup per iteration day.
  const rulesByDay = new Map<number, typeof rules>();
  for (const r of rules) {
    const list = rulesByDay.get(r.weekday) ?? [];
    list.push(r);
    rulesByDay.set(r.weekday, list);
  }

  const slots: AvailabilitySlot[] = [];

  // Iterate calendar dates in the tutor's timezone. We compute the starting
  // and ending YYYY-MM-DD strings once and walk forward one day at a time
  // via a bare UTC Date (safe: dates without times don't hit DST edges).
  const startIso = toLocalDateISO(from, tutorTz);
  const endIso = toLocalDateISO(to, tutorTz);

  let localDateISO = startIso;
  while (localDateISO < endIso) {
    // Weekday derives from the YYYY-MM-DD string itself — TZ-independent.
    const [yy, mm, dd] = localDateISO.split('-').map(Number);
    const weekday = new Date(Date.UTC(yy, mm - 1, dd)).getUTCDay();
    const dayRules = rulesByDay.get(weekday);
    // Advance the cursor now so `continue` still moves forward.
    const nextCursor = new Date(Date.UTC(yy, mm - 1, dd));
    nextCursor.setUTCDate(nextCursor.getUTCDate() + 1);
    const nextLocalDateISO = nextCursor.toISOString().slice(0, 10);

    if (!dayRules) {
      localDateISO = nextLocalDateISO;
      continue;
    }

    for (const rule of dayRules) {
      const startMin = timeToMinutes(rule.start_time);
      const endMin = timeToMinutes(rule.end_time);

      for (let m = startMin; m + durationMin <= endMin; m += durationMin) {
        const startAt = localToUtc(localDateISO, m, tutorTz);
        const endAt = new Date(startAt.getTime() + durationMin * 60_000);

        // Past slots — drop.
        if (startAt.getTime() <= now.getTime()) continue;

        // Overlap with any existing session? Two intervals overlap iff
        // start1 < end2 AND end1 > start2.
        const clash = existingSessions.some((s) => {
          const sStart = new Date(s.scheduled_at).getTime();
          const sEnd = sStart + s.duration_min * 60_000;
          return startAt.getTime() < sEnd && endAt.getTime() > sStart;
        });
        if (clash) continue;

        // When the caller wants the display in a different zone (student's
        // browser), reformat startAt via date-fns-tz. Otherwise emit the raw
        // tutor-authored wall clock — cheaper and matches what the tutor sees
        // in their availability grid.
        const sameZone = displayTz === tutorTz;
        slots.push({
          startAt: startAt.toISOString(),
          endAt: endAt.toISOString(),
          date: sameZone ? localDateISO : toLocalDateISO(startAt, displayTz),
          startLabel: sameZone
            ? `${pad2(Math.floor(m / 60))}:${pad2(m % 60)}`
            : formatInTz(startAt, 'HH:mm', { timeZone: displayTz }),
          durationMin,
        });
      }
    }
    localDateISO = nextLocalDateISO;
  }

  return slots.sort((a, b) => a.startAt.localeCompare(b.startAt));
}

// Keep `weekdayInTz` exported-adjacent — future group-session logic may want
// it. Not currently used but kept close to the timezone helpers.
export const __weekdayInTz = weekdayInTz;

async function fetchSessionsInRange(tutorId: string, from: Date, to: Date) {
  const { data } = await supabase
    .from('sessions')
    .select('scheduled_at, duration_min, status')
    .eq('tutor_id', tutorId)
    .in('status', ['pending', 'confirmed', 'live'])
    .gte('scheduled_at', from.toISOString())
    .lt('scheduled_at', new Date(to.getTime() + 24 * 60 * 60_000).toISOString());
  return (data || []) as { scheduled_at: string; duration_min: number; status: string }[];
}

/**
 * Check whether a specific proposed session slot is still bookable. Called at
 * the top of POST /api/sessions right before the insert, in the same request
 * as the DB unique-index guard so a race that beats the check still fails at
 * the DB level.
 */
export async function isSlotStillOpen(input: {
  tutorId: string;
  scheduledAt: Date;
  durationMin: number;
}): Promise<boolean> {
  const { data } = await supabase
    .from('sessions')
    .select('scheduled_at, duration_min')
    .eq('tutor_id', input.tutorId)
    .in('status', ['pending', 'confirmed', 'live']);
  const proposedEnd = input.scheduledAt.getTime() + input.durationMin * 60_000;
  for (const s of data || []) {
    const sStart = new Date(s.scheduled_at).getTime();
    const sEnd = sStart + s.duration_min * 60_000;
    if (input.scheduledAt.getTime() < sEnd && proposedEnd > sStart) return false;
  }
  return true;
}

/**
 * Resolve the course duration + price + tutor pairing used by the booking
 * flow. Returns null if the course doesn't exist, isn't published, or belongs
 * to a different tutor.
 */
export async function resolveCourseForBooking(courseId: string, tutorId: string) {
  const course = await getCourseById(courseId);
  if (!course || !course.is_published || course.tutor_id !== tutorId) return null;
  return course;
}
