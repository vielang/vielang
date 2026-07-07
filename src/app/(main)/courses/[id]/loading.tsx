import { DetailPageSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams while getPublishedCourseDetail() resolves.
export default function Loading() {
  return <DetailPageSkeleton />;
}
