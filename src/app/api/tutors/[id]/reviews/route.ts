import { type NextRequest, NextResponse } from 'next/server';
import { getReviewsForTutor } from '@/lib/supabase';
import { apiError } from '@/lib/api-errors';

export const dynamic = 'force-dynamic';

export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const { id } = await params;
    const limitParam = req.nextUrl.searchParams.get('limit');
    const limit = limitParam ? Math.min(Math.max(Number(limitParam), 1), 50) : 20;
    const reviews = await getReviewsForTutor(id, limit);
    return NextResponse.json({ reviews });
  } catch (err) {
    return apiError(err);
  }
}
