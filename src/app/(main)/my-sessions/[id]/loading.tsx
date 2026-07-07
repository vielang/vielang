import { DetailPageSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams while the private session detail page (chat + attendance) fetches.
export default function Loading() {
  return <DetailPageSkeleton />;
}
