import type { Metadata } from 'next';
import Image from 'next/image';
import { notFound } from 'next/navigation';
import { Star, GraduationCap, Award, Languages } from 'lucide-react';
import { getTutorById, getPublishedCourses, getReviewsWithStudentForTutor } from '@/lib/supabase';
import { AvailabilityPreview } from '@/components/tutor/AvailabilityPreview';
import { BookCta } from '@/components/tutor/BookCta';
import { StickyBookBar } from '@/components/tutor/StickyBookBar';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import Link from 'next/link';
import { Home } from 'lucide-react';
import { formatVnd, initials } from '@/lib/format';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

export const dynamic = 'force-dynamic';

const formatDate = (iso: string) => {
  const d = new Date(iso);
  return Number.isNaN(d.getTime())
    ? iso
    : d.toLocaleDateString('en-US', { day: '2-digit', month: 'short', year: 'numeric' });
};

export async function generateMetadata({
  params,
}: {
  params: Promise<{ id: string }>;
}): Promise<Metadata> {
  const { id } = await params;
  const tutor = await getTutorById(id).catch(() => null);
  if (!tutor) return { title: 'Tutor · VieLang' };
  return {
    title: `${tutor.name} — English tutor · VieLang`,
    description:
      tutor.bio?.slice(0, 160) ||
      `${tutor.name} teaches English on VieLang. Book a 1-on-1 video lesson.`,
    openGraph: {
      title: tutor.name,
      description: tutor.bio || '',
      images: tutor.avatar ? [{ url: tutor.avatar }] : undefined,
    },
  };
}

export default async function TutorDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;

  const [tutor, courses, reviews] = await Promise.all([
    getTutorById(id).catch(() => null),
    getPublishedCourses({ tutorId: id }).catch(() => []),
    getReviewsWithStudentForTutor(id, 10).catch(() => []),
  ]);

  if (!tutor || !tutor.profile?.is_approved) notFound();

  const { profile } = tutor;

  return (
    // Extra bottom padding on mobile keeps the last section (reviews) clear of
    // the fixed StickyBookBar. Desktop resets since the bar is md:hidden.
    <div className="space-y-10 pb-24 md:pb-0">
      <Breadcrumb className="mb-4">
        <BreadcrumbList className="text-xs font-medium">
          <BreadcrumbItem>
            <BreadcrumbLink
              render={
                <Link
                  href="/"
                  className="focus-ring hover:text-brand inline-flex items-center gap-1.5 rounded"
                />
              }
            >
              <Home className="size-3 shrink-0" aria-hidden />
              Home
            </BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbLink
              render={<Link href="/tutors" className="focus-ring hover:text-brand rounded" />}
            >
              Tutors
            </BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem>
            <BreadcrumbPage className="max-w-[200px] truncate sm:max-w-none">
              {tutor.name}
            </BreadcrumbPage>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>

      {/* Hero */}
      <section className="grid grid-cols-1 gap-8 lg:grid-cols-[minmax(0,1fr)_320px] lg:gap-12">
        <div className="space-y-8">
          <div className="flex flex-col gap-5 sm:flex-row sm:items-start sm:gap-6">
            {tutor.avatar ? (
              <Image
                src={tutor.avatar}
                alt={tutor.name}
                width={128}
                height={128}
                priority
                className="size-24 shrink-0 rounded-2xl object-cover ring-2 ring-indigo-100 md:size-32 dark:ring-indigo-900/40"
              />
            ) : (
              <div className="bg-brand flex size-24 shrink-0 items-center justify-center rounded-2xl font-serif text-3xl font-bold text-white md:size-32">
                {initials(tutor.name)}
              </div>
            )}
            <div className="min-w-0 flex-1 space-y-4">
              <div className="space-y-2">
                <h1 className="type-page text-slate-900 dark:text-slate-100">{tutor.name}</h1>
                <div className="flex items-center gap-3 text-sm text-slate-500 dark:text-slate-400">
                  <div className="flex items-center gap-1 text-amber-500">
                    <Star className="size-4 fill-amber-500" />
                    <span className="font-semibold text-slate-800 tabular-nums dark:text-slate-100">
                      {profile.rating_avg > 0 ? profile.rating_avg.toFixed(1) : 'New'}
                    </span>
                    <span className="ml-0.5 text-xs text-slate-500 dark:text-slate-400">
                      · {reviews.length} {reviews.length === 1 ? 'review' : 'reviews'}
                    </span>
                  </div>
                  <span className="text-slate-300 dark:text-slate-600" aria-hidden>
                    ·
                  </span>
                  <span>{profile.years_experience}+ yrs teaching</span>
                  {profile.session_count > 0 && (
                    <>
                      <span className="text-slate-300 dark:text-slate-600" aria-hidden>
                        ·
                      </span>
                      <span className="tabular-nums">{profile.session_count} sessions</span>
                    </>
                  )}
                </div>
              </div>
              {profile.specialties.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                  {profile.specialties.map((s) => (
                    <Badge
                      key={s}
                      className={cn(
                        'rounded-full border-0 px-2.5 py-1 text-[11px] font-semibold tracking-wide uppercase',
                        'bg-brand/10 text-brand dark:bg-brand/15 dark:text-indigo-300',
                      )}
                    >
                      {s}
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          </div>

          {tutor.bio && (
            <p className="max-w-3xl text-sm leading-relaxed text-slate-700 dark:text-slate-300">
              {tutor.bio}
            </p>
          )}

          <dl className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            {profile.certifications.length > 0 && (
              <div className="rounded-xl border border-slate-100 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-800/40">
                <dt className="type-eyebrow flex items-center gap-2">
                  <Award className="size-3.5" /> Certifications
                </dt>
                <dd className="mt-1 text-sm font-medium text-slate-800 dark:text-slate-200">
                  {profile.certifications.join(' · ')}
                </dd>
              </div>
            )}
            {profile.languages_spoken.length > 0 && (
              <div className="rounded-xl border border-slate-100 bg-slate-50 p-4 dark:border-slate-800 dark:bg-slate-800/40">
                <dt className="type-eyebrow flex items-center gap-2">
                  <Languages className="size-3.5" /> Speaks
                </dt>
                <dd className="mt-1 text-sm font-medium text-slate-800 uppercase dark:text-slate-200">
                  {profile.languages_spoken.join(' · ')}
                </dd>
              </div>
            )}
          </dl>
        </div>

        <aside className="h-fit space-y-5 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm lg:sticky lg:top-24 dark:border-slate-800 dark:bg-slate-900">
          <div>
            <p className="type-eyebrow">Starting from</p>
            <p className="mt-1 text-3xl font-bold text-slate-900 tabular-nums dark:text-slate-100">
              {formatVnd(profile.hourly_rate_vnd)}
              <span className="ml-1.5 text-sm font-medium text-slate-500 dark:text-slate-400">
                ₫ / hour
              </span>
            </p>
          </div>
          <BookCta tutorId={tutor.id} tutorName={tutor.name} courses={courses} />
          <p className="text-center text-[11px] text-slate-500 dark:text-slate-400">
            Free trial for your first lesson · Cancel up to 24 h before
          </p>
        </aside>
      </section>

      {/* Courses */}
      <section className="space-y-4">
        <h2 className="type-section text-slate-900 dark:text-slate-100">
          Courses ({courses.length})
        </h2>
        {courses.length === 0 ? (
          <p className="text-sm text-slate-500 dark:text-slate-400">
            No published courses yet. Check back soon.
          </p>
        ) : (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            {courses.map((c) => (
              <article
                key={c.id}
                className="flex flex-col gap-3 rounded-xl border border-slate-200 bg-white p-4 transition-shadow hover:shadow-sm dark:border-slate-800 dark:bg-slate-900"
              >
                <div className="flex gap-4">
                  {c.image && (
                    <div className="relative size-20 shrink-0 overflow-hidden rounded-lg bg-slate-100">
                      <Image src={c.image} alt="" fill className="object-cover" sizes="80px" />
                    </div>
                  )}
                  <div className="min-w-0 flex-1 space-y-1">
                    <div className="flex items-start justify-between gap-2">
                      <h3 className="line-clamp-2 text-sm font-semibold text-slate-900 dark:text-slate-100">
                        {c.title_en}
                      </h3>
                      {c.level && (
                        <Badge className="h-5 shrink-0 rounded-full border-0 bg-slate-100 px-2 text-[10px] font-bold text-slate-700 uppercase dark:bg-slate-800 dark:text-slate-300">
                          {c.level}
                        </Badge>
                      )}
                    </div>
                    <p className="line-clamp-2 text-xs text-slate-500 dark:text-slate-400">
                      {c.description_en}
                    </p>
                  </div>
                </div>
                <div className="mt-auto flex items-center justify-between border-t border-slate-100 pt-2 text-xs dark:border-slate-800">
                  <span className="flex items-center gap-1 text-slate-500 dark:text-slate-400">
                    <GraduationCap className="size-3" />
                    {c.duration_min}m · {c.category}
                  </span>
                  <span className="font-semibold text-slate-900 tabular-nums dark:text-slate-100">
                    {formatVnd(c.price_vnd)} ₫
                  </span>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      {/* Availability + Reviews */}
      <section className="grid grid-cols-1 gap-8 lg:grid-cols-2">
        <div className="space-y-4">
          <div className="flex flex-wrap items-baseline justify-between gap-3">
            <h2 className="type-section text-slate-900 dark:text-slate-100">Weekly availability</h2>
            <p className="text-[11px] text-slate-500 dark:text-slate-400">
              Times shown in Vietnam local (GMT+7)
            </p>
          </div>
          <AvailabilityPreview tutorId={tutor.id} />
        </div>

        <div className="space-y-4">
          <h2 className="type-section text-slate-900 dark:text-slate-100">
            Reviews ({reviews.length})
          </h2>
          {reviews.length === 0 ? (
            <p className="text-sm text-slate-500 dark:text-slate-400">
              No reviews yet. Be the first to book a session and leave feedback.
            </p>
          ) : (
            <ul className="space-y-3">
              {reviews.map((r) => (
                <li
                  key={r.id}
                  className="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900"
                >
                  <div className="mb-1 flex items-center justify-between gap-2">
                    <div className="flex items-center gap-1 text-amber-500">
                      {Array.from({ length: 5 }).map((_, i) => (
                        <Star
                          key={i}
                          className={`size-3.5 ${i < r.rating ? 'fill-amber-500' : 'text-slate-200'}`}
                        />
                      ))}
                    </div>
                    <span className="text-[11px] font-medium text-slate-500 dark:text-slate-400">
                      {formatDate(r.created_at)}
                    </span>
                  </div>
                  {r.comment && (
                    <p className="text-sm leading-relaxed text-slate-700 dark:text-slate-200">
                      {r.comment}
                    </p>
                  )}
                  {r.student_name && (
                    <p className="mt-2 text-[11px] font-semibold text-slate-500 dark:text-slate-400">
                      — {r.student_name}
                    </p>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      </section>

      <StickyBookBar
        tutorId={tutor.id}
        tutorName={tutor.name}
        hourlyRateVnd={profile.hourly_rate_vnd}
        courses={courses}
      />
    </div>
  );
}
