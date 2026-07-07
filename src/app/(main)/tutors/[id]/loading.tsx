import { DetailPageSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams while getTutorById() + getPublishedCourses() +
// getReviewsWithStudentForTutor() resolve in parallel.
export default function Loading() {
  return <DetailPageSkeleton />;
}
