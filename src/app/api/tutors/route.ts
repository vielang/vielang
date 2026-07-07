import { type NextRequest, NextResponse } from 'next/server';
import { getApprovedTutors } from '@/lib/supabase';
import { apiError } from '@/lib/api-errors';

export const dynamic = 'force-dynamic';

// Public list of approved tutors — powers the /tutors discovery page and the
// homepage "featured" row. All params optional; unrecognized sort values fall
// through to the default in getApprovedTutors.
export async function GET(req: NextRequest) {
  try {
    const params = req.nextUrl.searchParams;
    const specialty = params.get('specialty') || undefined;
    const sort = (params.get('sort') as any) || undefined;
    const minRatingRaw = params.get('minRating');
    const maxPriceRaw = params.get('maxPrice');
    const limitRaw = params.get('limit');

    const tutors = await getApprovedTutors({
      specialty,
      sort,
      minRating: minRatingRaw ? Number(minRatingRaw) : undefined,
      maxPriceVnd: maxPriceRaw ? Number(maxPriceRaw) : undefined,
    });

    const limit = limitRaw ? Math.min(Math.max(Number(limitRaw), 1), 50) : undefined;
    return NextResponse.json({ tutors: limit ? tutors.slice(0, limit) : tutors });
  } catch (err) {
    return apiError(err);
  }
}
