import { describe, it, expect, vi, beforeEach } from 'vitest';

type Promo = {
  code: string;
  active: boolean;
  discount: number;
  type: string;
  maxUses: number | null;
  expiryDate?: string | null;
};

// Controllable state for the Supabase mock.
const state: {
  promo: Promo | null;
  existingUsage: { id: string } | null;
  usageCount: number;
} = { promo: null, existingUsage: null, usageCount: 0 };

vi.mock('./supabase', () => {
  const promosBuilder = () => {
    const chain: Record<string, unknown> = {};
    chain.select = () => chain;
    chain.ilike = () => chain;
    chain.order = () => chain;
    chain.limit = () => chain;
    chain.then = (resolve: (v: unknown) => unknown) =>
      resolve({ data: state.promo ? [state.promo] : [], error: null });
    return chain;
  };
  const usageBuilder = (opts?: { count?: boolean }) => {
    const chain: Record<string, unknown> = {};
    chain.select = () => chain;
    chain.eq = () => chain;
    chain.maybeSingle = async () => ({ data: state.existingUsage, error: null });
    if (opts?.count) {
      // Second usage call — a count/head query.
      chain.select = () => chain;
      chain.eq = async () => ({ count: state.usageCount, error: null });
    }
    return chain;
  };
  return {
    supabase: {
      from: (table: string) => {
        if (table === 'promo_codes') return promosBuilder();
        if (table === 'promo_usage') {
          // Return a builder whose `select(_, { count, head })` returns a
          // count-head chain, and plain `select(...)` returns the dedupe chain.
          return {
            select: (_: string, opts?: { count?: string; head?: boolean }) => {
              if (opts?.count === 'exact' && opts.head) {
                const c: Record<string, unknown> = {};
                c.eq = async () => ({ count: state.usageCount, error: null });
                return c;
              }
              const d: Record<string, unknown> = {};
              d.eq = () => d;
              d.maybeSingle = async () => ({ data: state.existingUsage, error: null });
              return d;
            },
          };
        }
        return usageBuilder();
      },
    },
  };
});

import { applyPromoToTotal, isPromoCheckFail, lookupPromoForUser } from './promo';

beforeEach(() => {
  state.promo = null;
  state.existingUsage = null;
  state.usageCount = 0;
});

describe('applyPromoToTotal', () => {
  it('applies a percent discount', () => {
    const result = applyPromoToTotal(1_000_000, { discount: 20, type: 'percent' });
    expect(result.discountVND).toBe(200_000);
    expect(result.totalVND).toBe(800_000);
  });

  it('accepts the legacy "percentage" alias', () => {
    const result = applyPromoToTotal(500_000, { discount: 10, type: 'percentage' });
    expect(result.discountVND).toBe(50_000);
    expect(result.totalVND).toBe(450_000);
  });

  it('applies a fixed discount', () => {
    const result = applyPromoToTotal(1_000_000, { discount: 150_000, type: 'fixed' });
    expect(result.discountVND).toBe(150_000);
    expect(result.totalVND).toBe(850_000);
  });

  it('accepts the "amount" alias for fixed', () => {
    const result = applyPromoToTotal(500_000, { discount: 100_000, type: 'amount' });
    expect(result.discountVND).toBe(100_000);
    expect(result.totalVND).toBe(400_000);
  });

  it('caps a fixed discount at the order total (never goes negative)', () => {
    const result = applyPromoToTotal(50_000, { discount: 200_000, type: 'fixed' });
    expect(result.discountVND).toBe(50_000);
    expect(result.totalVND).toBe(0);
  });

  it('clamps a >100% percentage to 100%', () => {
    const result = applyPromoToTotal(100_000, { discount: 150, type: 'percent' });
    expect(result.discountVND).toBe(100_000);
    expect(result.totalVND).toBe(0);
  });

  it('clamps a negative discount to 0', () => {
    const result = applyPromoToTotal(100_000, { discount: -50, type: 'percent' });
    expect(result.discountVND).toBe(0);
    expect(result.totalVND).toBe(100_000);
  });

  it('unknown promo type falls through to zero discount (safe fallback)', () => {
    const result = applyPromoToTotal(100_000, { discount: 50, type: 'mystery' });
    expect(result.discountVND).toBe(0);
    expect(result.totalVND).toBe(100_000);
  });

  it('is case-insensitive on type', () => {
    const percent = applyPromoToTotal(100_000, { discount: 10, type: 'PERCENT' });
    const fixed = applyPromoToTotal(100_000, { discount: 10_000, type: 'FIXED' });
    expect(percent.discountVND).toBe(10_000);
    expect(fixed.discountVND).toBe(10_000);
  });

  it('rounds fractional percent discounts to an integer VND', () => {
    // 12.5% of 123_456 = 15_432 (rounded).
    const result = applyPromoToTotal(123_456, { discount: 12.5, type: 'percent' });
    expect(result.discountVND).toBe(15_432);
    expect(result.totalVND).toBe(108_024);
  });
});

describe('lookupPromoForUser', () => {
  it('returns PROMO_CODE_REQUIRED when the code is empty or whitespace', async () => {
    const empty = await lookupPromoForUser('', 'user-1');
    const spaces = await lookupPromoForUser('   ', 'user-1');
    expect(empty.ok).toBe(false);
    expect(spaces.ok).toBe(false);
    if (!empty.ok) expect(empty.errorCode).toBe('PROMO_CODE_REQUIRED');
    if (!spaces.ok) expect(spaces.errorCode).toBe('PROMO_CODE_REQUIRED');
  });

  it('returns PROMO_NOT_FOUND when no matching row exists', async () => {
    state.promo = null;
    const result = await lookupPromoForUser('MISSING', 'user-1');
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errorCode).toBe('PROMO_NOT_FOUND');
  });

  it('returns PROMO_INACTIVE when the promo has active=false', async () => {
    state.promo = { code: 'X', active: false, discount: 10, type: 'percent', maxUses: null };
    const result = await lookupPromoForUser('X', 'user-1');
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errorCode).toBe('PROMO_INACTIVE');
  });

  it('returns PROMO_EXPIRED when the expiryDate is in the past', async () => {
    state.promo = {
      code: 'X',
      active: true,
      discount: 10,
      type: 'percent',
      maxUses: null,
      expiryDate: '2020-01-01',
    };
    const result = await lookupPromoForUser('X', 'user-1');
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errorCode).toBe('PROMO_EXPIRED');
  });

  it('returns PROMO_ALREADY_USED when the user has already redeemed it', async () => {
    state.promo = { code: 'X', active: true, discount: 10, type: 'percent', maxUses: null };
    state.existingUsage = { id: 'usage-1' };
    const result = await lookupPromoForUser('X', 'user-1');
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errorCode).toBe('PROMO_ALREADY_USED');
  });

  it('returns PROMO_MAX_USES when the global usage count reached maxUses', async () => {
    state.promo = { code: 'X', active: true, discount: 10, type: 'percent', maxUses: 5 };
    state.usageCount = 5;
    const result = await lookupPromoForUser('X', 'user-1');
    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.errorCode).toBe('PROMO_MAX_USES');
  });

  it('returns ok with the normalized promo when everything passes', async () => {
    state.promo = { code: 'HELLO', active: true, discount: 25, type: 'percent', maxUses: null };
    const result = await lookupPromoForUser('hello', 'user-1');
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.promo.code).toBe('HELLO');
      expect(result.promo.discount).toBe(25);
      expect(result.promo.type).toBe('percent');
    }
  });
});

describe('isPromoCheckFail', () => {
  it('narrows a failing PromoCheckResult', () => {
    const fail = {
      ok: false as const,
      status: 404,
      error: 'not found',
      errorCode: 'PROMO_NOT_FOUND' as const,
    };
    expect(isPromoCheckFail(fail)).toBe(true);
  });

  it('rejects a successful PromoCheckResult', () => {
    const ok = {
      ok: true as const,
      promo: { code: 'HELLO', discount: 10, type: 'percent', maxUses: null },
    };
    expect(isPromoCheckFail(ok)).toBe(false);
  });
});
