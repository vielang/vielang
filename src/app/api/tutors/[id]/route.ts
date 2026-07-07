import { type NextRequest, NextResponse } from 'next/server';
import { getTutorById } from '@/lib/supabase';
import { apiError } from '@/lib/api-errors';

export const dynamic = 'force-dynamic';

export async function GET(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const { id } = await params;
    const tutor = await getTutorById(id);
    if (!tutor || !tutor.profile?.is_approved) {
      return NextResponse.json({ error: 'Tutor not found' }, { status: 404 });
    }
    return NextResponse.json({ tutor });
  } catch (err) {
    return apiError(err);
  }
}
