import { ListPageSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams during the getUpcomingGroupSessions() fetch in page.tsx.
export default function Loading() {
  return <ListPageSkeleton count={6} />;
}
