import { ListPageSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams during getApprovedTutors() in page.tsx — six placeholder tiles at
// the same density as the real TutorCard grid.
export default function Loading() {
  return <ListPageSkeleton count={6} />;
}
