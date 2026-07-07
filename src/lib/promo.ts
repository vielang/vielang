import { supabase } from './supabase';

/**
 * Shared promo-code validation used by:
 *   - POST /api/promo/validate  → preview discount, no DB write
 *   - POST /api/promo/use       → legacy "claim and use" path
 *   - POST /api/payments/init   → applies discount + records usage atomically
 *
 * Reads are SELECTs only; insertion of `promo_usage` belongs to the caller so
 * we don't burn the customer's once-per-user slot just for a preview.
 *
 * Lookup is `.ilike(code).order(created_at asc).limit(1)` so a historical
 * duplicate (pre-`20260529130000_promo_codes_unique_code` row) resolves
 * deterministically to the earliest version instead of throwing
 * `.maybeSingle()`'s multi-row error.
 */

export type PromoErrorCode =
  | 'PROMO_CODE_REQUIRED'
  | 'PROMO_NOT_FOUND'
  | 'PROMO_INACTIVE'
  | 'PROMO_EXPIRED'
  | 'PROMO_ALREADY_USED'
  | 'PROMO_MAX_USES';

export interface PromoCheckOk {
  ok: true;
  promo: {
    code: string;
    discount: number;
    type: string;
    maxUses: number | null;
  };
}

export interface PromoCheckFail {
  ok: false;
  status: number;
  error: string;
  /** Stable machine-readable error tag the client localises against. */
  errorCode: PromoErrorCode;
}

export type PromoCheckResult = PromoCheckOk | PromoCheckFail;

/** Discriminator function — using a type predicate works around projects
 *  where `strict: false` weakens `if (!r.ok)` narrowing on the union. */
export function isPromoCheckFail(r: PromoCheckResult): r is PromoCheckFail {
  return r.ok === false;
}

export async function lookupPromoForUser(
  rawCode: string | undefined,
  userId: string,
): Promise<PromoCheckResult> {
  const code = typeof rawCode === 'string' ? rawCode.trim() : '';
  if (!code) {
    return { ok: false, status: 400, error: 'code is required', errorCode: 'PROMO_CODE_REQUIRED' };
  }

  const { data: promos, error: lookupErr } = await supabase
    .from('promo_codes')
    .select('*')
    .ilike('code', code)
    .order('created_at', { ascending: true })
    .limit(1);
  if (lookupErr) throw lookupErr;
  const promo = (promos || [])[0];
  if (!promo) {
    return { ok: false, status: 404, error: 'Promo code not found', errorCode: 'PROMO_NOT_FOUND' };
  }
  if (promo.active === false) {
    return { ok: false, status: 400, error: 'Promo code is inactive', errorCode: 'PROMO_INACTIVE' };
  }
  if (promo.expiryDate) {
    const today = new Date().toISOString().slice(0, 10);
    if (String(promo.expiryDate) < today) {
      return {
        ok: false,
        status: 400,
        error: 'Promo code has expired',
        errorCode: 'PROMO_EXPIRED',
      };
    }
  }

  // Per-user dedupe + maxUses. Both pre-checks; race correction happens at
  // the caller's insert site (see addPromoUsage in supabase.ts).
  const { data: existing } = await supabase
    .from('promo_usage')
    .select('id')
    .eq('code', promo.code)
    .eq('userId', userId)
    .maybeSingle();
  if (existing) {
    return {
      ok: false,
      status: 409,
      error: 'You have already used this promo code',
      errorCode: 'PROMO_ALREADY_USED',
    };
  }
  if (typeof promo.maxUses === 'number' && promo.maxUses > 0) {
    const { count } = await supabase
      .from('promo_usage')
      .select('*', { count: 'exact', head: true })
      .eq('code', promo.code);
    if ((count || 0) >= promo.maxUses) {
      return {
        ok: false,
        status: 409,
        error: 'Promo code usage limit reached',
        errorCode: 'PROMO_MAX_USES',
      };
    }
  }

  return {
    ok: true,
    promo: {
      code: promo.code,
      discount: Number(promo.discount) || 0,
      type: String(promo.type || 'percent'),
      maxUses: typeof promo.maxUses === 'number' ? promo.maxUses : null,
    },
  };
}

/**
 * Apply a validated promo to a VND total. Percentage and fixed-amount types
 * are both supported; unknown types fall through to a zero discount so an
 * accidentally-typo'd `type` value never makes the total negative.
 */
export function applyPromoToTotal(
  totalVND: number,
  promo: { discount: number; type: string },
): { discountVND: number; totalVND: number } {
  let discountVND = 0;
  const t = promo.type.toLowerCase();
  if (t === 'percent' || t === 'percentage') {
    const pct = Math.max(0, Math.min(100, promo.discount));
    discountVND = Math.round((totalVND * pct) / 100);
  } else if (t === 'fixed' || t === 'amount') {
    discountVND = Math.min(totalVND, Math.max(0, Math.round(promo.discount)));
  }
  return {
    discountVND,
    totalVND: Math.max(0, totalVND - discountVND),
  };
}
