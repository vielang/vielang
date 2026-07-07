import type { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import { getGroupSessionDetail, hasReservedGroupSession } from '@/lib/supabase';
import { SessionDetailClient } from './SessionDetailClient';

export const dynamic = 'force-dynamic';

interface Props {
  params: Promise<{ id: string }>;
}

// Metadata is dynamic so shared /sessions/<id> links preview with the room's
// actual topic instead of a generic title. Falls back to a bland default if
// the row is missing — Next will still 404 the page itself below.
export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { id } = await params;
  const session = await getGroupSessionDetail(id).catch(() => null);
  if (!session) return { title: 'Session · VieLang' };
  const topic = session.topic_en || session.topic_vn || 'Free-talk session';
  return {
    title: `${topic} · VieLang`,
    description:
      session.description ||
      `Join a live English free-talk room hosted by ${session.tutor_name || 'VieLang'}.`,
  };
}

export default async function GroupSessionDetailPage({ params }: Props) {
  const { id } = await params;
  const session = await getGroupSessionDetail(id).catch(() => null);
  if (!session) notFound();

  const user = await getCurrentUser();
  const viewerId = user?.id ?? null;
  const viewerRole = user?.role ?? null;

  // Only the actual seat-holder view needs this lookup — hosts and admins are
  // implied members and never carry a session_participants row.
  const isHostOrAdmin = !!viewerId && (viewerId === session.tutor_id || viewerRole === 'admin');
  const reserved =
    !!viewerId && !isHostOrAdmin ? await hasReservedGroupSession(id, viewerId) : false;

  return (
    <SessionDetailClient
      session={session}
      viewer={{
        id: viewerId,
        role: viewerRole,
        isHostOrAdmin,
        reserved,
      }}
    />
  );
}
