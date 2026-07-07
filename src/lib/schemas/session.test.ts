import { describe, it, expect } from 'vitest';
import {
  createPrivateSessionSchema,
  createGroupSessionSchema,
  createSessionSchema,
  patchSessionSchema,
} from './session';

// Zod v4 enforces UUID version + variant bits. Use RFC-4122 v4 UUIDs (version
// nibble = 4, variant nibble ∈ {8,9,a,b}).
const validUuid = '11111111-1111-4111-8111-111111111111';
const otherUuid = '22222222-2222-4222-8222-222222222222';
const validIso = '2026-08-15T14:30:00+07:00';

describe('createPrivateSessionSchema', () => {
  it('accepts a minimal valid private booking', () => {
    const result = createPrivateSessionSchema.safeParse({
      tutor_id: validUuid,
      course_id: otherUuid,
      scheduled_at: validIso,
    });
    expect(result.success).toBe(true);
  });

  it('accepts explicit type="private" and optional student_notes', () => {
    const result = createPrivateSessionSchema.safeParse({
      type: 'private',
      tutor_id: validUuid,
      course_id: otherUuid,
      scheduled_at: validIso,
      student_notes: '  I need help with pronunciation  ',
    });
    expect(result.success).toBe(true);
    if (result.success) {
      // z.string().trim() runs on parse — trailing spaces gone.
      expect(result.data.student_notes).toBe('I need help with pronunciation');
    }
  });

  it('rejects non-uuid tutor_id', () => {
    const result = createPrivateSessionSchema.safeParse({
      tutor_id: 'not-a-uuid',
      course_id: otherUuid,
      scheduled_at: validIso,
    });
    expect(result.success).toBe(false);
  });

  it('rejects scheduled_at without timezone offset', () => {
    // datetime({ offset: true }) requires an explicit offset like +07:00 or Z.
    const result = createPrivateSessionSchema.safeParse({
      tutor_id: validUuid,
      course_id: otherUuid,
      scheduled_at: '2026-08-15T14:30:00',
    });
    expect(result.success).toBe(false);
  });

  it('caps student_notes at 1000 chars', () => {
    const result = createPrivateSessionSchema.safeParse({
      tutor_id: validUuid,
      course_id: otherUuid,
      scheduled_at: validIso,
      student_notes: 'x'.repeat(1001),
    });
    expect(result.success).toBe(false);
  });
});

describe('createGroupSessionSchema', () => {
  it('accepts a minimal valid group session with defaults', () => {
    const result = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Free talk about travel',
      scheduled_at: validIso,
    });
    expect(result.success).toBe(true);
    if (result.success) {
      // Defaults applied: level, duration_min, capacity.
      expect(result.data.level).toBe('all');
      expect(result.data.duration_min).toBe(45);
      expect(result.data.capacity).toBe(20);
    }
  });

  it('rejects capacity below 2', () => {
    const result = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Free talk',
      scheduled_at: validIso,
      capacity: 1,
    });
    expect(result.success).toBe(false);
  });

  it('rejects capacity above 100', () => {
    const result = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Free talk',
      scheduled_at: validIso,
      capacity: 101,
    });
    expect(result.success).toBe(false);
  });

  it('rejects duration_min outside 15–180', () => {
    const tooShort = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Free talk',
      scheduled_at: validIso,
      duration_min: 10,
    });
    const tooLong = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Free talk',
      scheduled_at: validIso,
      duration_min: 240,
    });
    expect(tooShort.success).toBe(false);
    expect(tooLong.success).toBe(false);
  });

  it('rejects topic_en shorter than 3 chars', () => {
    const result = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'ab',
      scheduled_at: validIso,
    });
    expect(result.success).toBe(false);
  });

  it('accepts optional host_tutor_id as null', () => {
    const result = createGroupSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Admin-hosted room',
      scheduled_at: validIso,
      host_tutor_id: null,
    });
    expect(result.success).toBe(true);
  });
});

describe('createSessionSchema (discriminated union)', () => {
  it('parses a private body', () => {
    const result = createSessionSchema.safeParse({
      tutor_id: validUuid,
      course_id: otherUuid,
      scheduled_at: validIso,
    });
    expect(result.success).toBe(true);
  });

  it('parses a group body', () => {
    const result = createSessionSchema.safeParse({
      type: 'group',
      topic_en: 'Travel talk',
      scheduled_at: validIso,
    });
    expect(result.success).toBe(true);
  });

  it('rejects a body missing both tutor_id and topic_en', () => {
    const result = createSessionSchema.safeParse({
      scheduled_at: validIso,
    });
    expect(result.success).toBe(false);
  });
});

describe('patchSessionSchema', () => {
  it('accepts a cancel action with reason', () => {
    const result = patchSessionSchema.safeParse({
      action: 'cancel',
      cancelled_reason: 'sick',
    });
    expect(result.success).toBe(true);
  });

  it('accepts a confirm action', () => {
    const result = patchSessionSchema.safeParse({ action: 'confirm' });
    expect(result.success).toBe(true);
  });

  it('accepts a reschedule action WITH scheduled_at', () => {
    const result = patchSessionSchema.safeParse({
      action: 'reschedule',
      scheduled_at: validIso,
    });
    expect(result.success).toBe(true);
  });

  it('REJECTS a reschedule action WITHOUT scheduled_at (refine rule)', () => {
    const result = patchSessionSchema.safeParse({ action: 'reschedule' });
    expect(result.success).toBe(false);
    if (!result.success) {
      const issue = result.error.issues.find((i) => i.path.includes('scheduled_at'));
      expect(issue?.message).toBe('scheduled_at required for reschedule');
    }
  });

  it('rejects an unknown action', () => {
    const result = patchSessionSchema.safeParse({ action: 'delete' });
    expect(result.success).toBe(false);
  });
});
