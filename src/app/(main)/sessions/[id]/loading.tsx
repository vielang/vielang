import { DetailPageSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams while getGroupSessionDetail() + hasReservedGroupSession() resolve.
export default function Loading() {
  return <DetailPageSkeleton />;
}
