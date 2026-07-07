import { SessionListSkeleton } from '@/components/skeletons/ListPageSkeleton';

// Streams during the SSR fetch (getSessionListForUser + getReviewedSessionIdsForStudent
// in page.tsx). Matches the my-sessions layout — header, three-tab strip, four
// placeholder rows — so the swap to real content doesn't shift the viewport.
export default function Loading() {
  return <SessionListSkeleton count={4} />;
}
