import { type NextRequest, NextResponse } from 'next/server';
import { getPublishedCourses } from '@/lib/supabase';
import { apiError } from '@/lib/api-errors';

export const dynamic = 'force-dynamic';

export async function GET(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const { id } = await params;
    const courses = await getPublishedCourses({ tutorId: id });
    return NextResponse.json({ courses });
  } catch (err) {
    return apiError(err);
  }
}
