// @vitest-environment node
// booking.ts imports 'server-only', which throws in a browser-like (jsdom)
// env. Force node here — the module is server-only anyway.
import { describe, it, expect, vi, beforeEach } from 'vitest';

type Rule = { weekday: number; start_time: string; end_time: string };
type Session = { scheduled_at: string; duration_min: number; status: string };

// Hoisted mocks so vi.mock (which is hoisted to the top of the file) can
// close over the setters below.
const rulesRef: { current: Rule[] } = { current: [] };
const sessionsRef: { current: Session[] } = { current: [] };

vi.mock('./supabase', () => {
  // Fluent Supabase builder — every chain method returns `this` and finally
  // resolves to `{ data, error: null }`. Only the last `await` matters.
  const makeQuery = (rows: unknown[]) => {
    const chain: Record<string, unknown> = {};
    const methods = ['select', 'eq', 'in', 'gte', 'lt', 'order', 'limit'];
    for (const m of methods) chain[m] = () => chain;
    chain.then = (resolve: (v: unknown) => unknown) => resolve({ data: rows, error: null });
    return chain;
  };
  return {
    supabase: {
      from: (table: string) => {
        if (table === 'sessions') return makeQuery(sessionsRef.current);
        return makeQuery([]);
      },
    },
    getAvailabilityForTutor: async (_tutorId: string) => rulesRef.current,
    getCourseById: async () => null,
  };
});

// Import after vi.mock — hoisting ensures the mock is in place first, but the
// linter can't tell.
const { getAvailableSlots, isSlotStillOpen } = await import('./booking');

const TUTOR = '11111111-1111-4111-8111-111111111111';
const now = new Date('2026-08-10T05:00:00Z'); // Monday 12:00 Vietnam local

beforeEach(() => {
  rulesRef.current = [];
  sessionsRef.current = [];
});

describe('getAvailableSlots', () => {
  it('returns an empty list when the tutor has no availability rules', async () => {
    rulesRef.current = [];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'),
      to: new Date('2026-08-11T00:00:00Z'),
      durationMin: 60,
      now,
    });
    expect(slots).toEqual([]);
  });

  it('generates 60-min slots inside a single availability window', async () => {
    // Monday 14:00–17:00 VN → produces 3 slots at 14:00, 15:00, 16:00 VN.
    rulesRef.current = [{ weekday: 1, start_time: '14:00', end_time: '17:00' }];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'), // Monday VN
      to: new Date('2026-08-11T00:00:00Z'),
      durationMin: 60,
      now,
    });
    expect(slots).toHaveLength(3);
    expect(slots.map((s) => s.startLabel)).toEqual(['14:00', '15:00', '16:00']);
    expect(slots[0].date).toBe('2026-08-10');
    expect(slots[0].durationMin).toBe(60);
  });

  it('drops slots that end past the availability window', async () => {
    // 14:00–15:30 gives one full 60-min slot (14:00), NOT two.
    rulesRef.current = [{ weekday: 1, start_time: '14:00', end_time: '15:30' }];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'),
      to: new Date('2026-08-11T00:00:00Z'),
      durationMin: 60,
      now,
    });
    expect(slots).toHaveLength(1);
    expect(slots[0].startLabel).toBe('14:00');
  });

  it('drops slots in the past relative to `now`', async () => {
    // now = Monday 12:00 VN. 9:00 and 10:00 slots must not appear.
    rulesRef.current = [{ weekday: 1, start_time: '09:00', end_time: '14:00' }];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'),
      to: new Date('2026-08-11T00:00:00Z'),
      durationMin: 60,
      now,
    });
    // Kept: 13:00 only (12:00 is not strictly > now — the guard uses <=).
    expect(slots.map((s) => s.startLabel)).toEqual(['13:00']);
  });

  it('drops slots that overlap an existing session', async () => {
    rulesRef.current = [{ weekday: 1, start_time: '14:00', end_time: '17:00' }];
    // A confirmed session at 15:00 VN (= 08:00 UTC) blocks the 15:00 slot.
    sessionsRef.current = [
      { scheduled_at: '2026-08-10T08:00:00.000Z', duration_min: 60, status: 'confirmed' },
    ];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'),
      to: new Date('2026-08-11T00:00:00Z'),
      durationMin: 60,
      now,
    });
    expect(slots.map((s) => s.startLabel)).toEqual(['14:00', '16:00']);
  });

  it('spans multiple days honoring per-weekday rules', async () => {
    rulesRef.current = [
      { weekday: 1, start_time: '10:00', end_time: '12:00' }, // Monday
      { weekday: 2, start_time: '10:00', end_time: '11:00' }, // Tuesday
    ];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'), // Monday VN
      to: new Date('2026-08-12T00:00:00Z'), // Wednesday exclusive
      durationMin: 60,
      now,
    });
    // Monday: 10:00, 11:00 (still future given now=12:00 VN? Actually now is
    // 12:00 VN so 10:00 and 11:00 are past → dropped). Tuesday: 10:00 kept.
    // So: only Tuesday 10:00 remains.
    expect(slots.map((s) => `${s.date} ${s.startLabel}`)).toEqual(['2026-08-11 10:00']);
  });

  it('returned slots are sorted ascending by startAt', async () => {
    rulesRef.current = [
      { weekday: 2, start_time: '15:00', end_time: '17:00' }, // Tuesday
      { weekday: 1, start_time: '14:00', end_time: '16:00' }, // Monday (defined 2nd)
    ];
    const slots = await getAvailableSlots({
      tutorId: TUTOR,
      from: new Date('2026-08-10T00:00:00Z'),
      to: new Date('2026-08-12T00:00:00Z'),
      durationMin: 60,
      now,
    });
    const starts = slots.map((s) => s.startAt);
    const sortedCopy = [...starts].sort();
    expect(starts).toEqual(sortedCopy);
  });
});

describe('isSlotStillOpen', () => {
  it('returns true when no sessions overlap', async () => {
    sessionsRef.current = [];
    const open = await isSlotStillOpen({
      tutorId: TUTOR,
      scheduledAt: new Date('2026-08-15T07:00:00Z'),
      durationMin: 60,
    });
    expect(open).toBe(true);
  });

  it('returns false when the proposed slot overlaps an existing session', async () => {
    // Existing session 07:00–08:00 UTC. Proposed slot 07:30–08:30 overlaps.
    sessionsRef.current = [
      { scheduled_at: '2026-08-15T07:00:00.000Z', duration_min: 60, status: 'confirmed' },
    ];
    const open = await isSlotStillOpen({
      tutorId: TUTOR,
      scheduledAt: new Date('2026-08-15T07:30:00Z'),
      durationMin: 60,
    });
    expect(open).toBe(false);
  });

  it('returns true for a slot that starts exactly when an existing one ends (no overlap)', async () => {
    sessionsRef.current = [
      { scheduled_at: '2026-08-15T07:00:00.000Z', duration_min: 60, status: 'confirmed' },
    ];
    const open = await isSlotStillOpen({
      tutorId: TUTOR,
      scheduledAt: new Date('2026-08-15T08:00:00Z'),
      durationMin: 60,
    });
    expect(open).toBe(true);
  });
});
