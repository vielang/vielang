'use client';

import Image from 'next/image';
import Link from 'next/link';
import { motion } from 'motion/react';
import { ArrowLeft, ArrowRight, BookOpen, Clock, GraduationCap, Star, Tag } from 'lucide-react';
import { AvatarBubble } from '@/components/shared/AvatarBubble';
import { useLang } from '@/contexts';
import { formatVnd } from '@/lib/format';
import type { CourseDetail } from '@/lib/supabase';

interface Props {
  course: CourseDetail;
}

const COPY = {
  VN: {
    back: 'Quay lại danh sách',
    tutor: 'Giáo viên',
    noTutor: 'Chưa có giáo viên phụ trách',
    duration: (m: number) => `${m} phút / buổi`,
    level: 'Cấp độ',
    category: 'Chủ đề',
    price: 'Học phí',
    perSession: '/ buổi',
    about: 'Về khoá học này',
    tutorRate: '/ giờ',
    bookCta: 'Đặt buổi học với giáo viên',
    noTutorHint: 'Khoá học này chưa gán giáo viên — vui lòng liên hệ đội ngũ VieLang.',
  },
  EN: {
    back: 'Back to courses',
    tutor: 'Tutor',
    noTutor: 'No assigned tutor yet',
    duration: (m: number) => `${m} min / session`,
    level: 'Level',
    category: 'Category',
    price: 'Price',
    perSession: '/ session',
    about: 'About this course',
    tutorRate: '/ hour',
    bookCta: 'Book a session with the tutor',
    noTutorHint: "This course doesn't have a tutor assigned yet — reach out to the VieLang team.",
  },
} as const;

export function CourseDetailClient({ course }: Props) {
  const { lang } = useLang();
  const lc = COPY[lang];

  const title =
    (lang === 'VN' ? course.title_vn || course.title_en : course.title_en || course.title_vn) ||
    'Course';
  const description =
    lang === 'VN'
      ? course.description_vn || course.description_en
      : course.description_en || course.description_vn;

  return (
    <div className="space-y-6">
      <Link
        href="/tutors"
        className="focus-ring inline-flex items-center gap-1.5 rounded-md text-xs font-semibold text-slate-500 transition-colors hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
      >
        <ArrowLeft className="size-3.5" />
        {lc.back}
      </Link>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
        <motion.article
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.35 }}
          className="space-y-6 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm md:p-8 dark:border-slate-800 dark:bg-slate-900"
        >
          {course.image && (
            <div className="relative aspect-[16/9] w-full overflow-hidden rounded-xl bg-slate-100 dark:bg-slate-800">
              <Image
                src={course.image}
                alt=""
                fill
                className="object-cover"
                sizes="(max-width: 1024px) 100vw, 800px"
              />
            </div>
          )}

          <header className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              {course.level && (
                <span className="inline-flex h-5 items-center gap-1 rounded-full bg-slate-100 px-2 text-[10px] font-semibold tracking-wider uppercase dark:bg-slate-800">
                  <GraduationCap className="size-3" />
                  {course.level}
                </span>
              )}
              {course.category && (
                <span className="inline-flex h-5 items-center gap-1 rounded-full bg-slate-100 px-2 text-[10px] font-semibold tracking-wider uppercase dark:bg-slate-800">
                  <Tag className="size-3" />
                  {course.category}
                </span>
              )}
            </div>
            <h1 className="text-brand dark:text-accent-warm font-serif text-2xl leading-tight font-bold md:text-3xl">
              {title}
            </h1>
            <div className="flex flex-wrap items-center gap-x-6 gap-y-2 border-y border-slate-100 py-3 text-sm text-slate-600 dark:border-slate-800 dark:text-slate-300">
              <span className="flex items-center gap-2">
                <Clock className="text-brand size-4" />
                {lc.duration(course.duration_min)}
              </span>
              <span className="flex items-center gap-2">
                <BookOpen className="text-brand size-4" />
                {lc.category}: {course.category || '—'}
              </span>
            </div>
          </header>

          {description && (
            <section className="space-y-2">
              <h2 className="text-xs font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                {lc.about}
              </h2>
              <p className="text-sm leading-relaxed whitespace-pre-wrap text-slate-700 dark:text-slate-200">
                {description}
              </p>
            </section>
          )}

          <section className="space-y-3">
            <h2 className="text-xs font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
              {lc.tutor}
            </h2>
            {course.tutor_id ? (
              <Link
                href={`/tutors/${course.tutor_id}`}
                className="focus-ring hover:border-brand/40 flex items-center gap-4 rounded-xl border border-slate-200 p-4 transition-colors dark:border-slate-700"
              >
                <AvatarBubble src={course.tutor_avatar} name={course.tutor_name} size={48} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                    {course.tutor_name || lc.noTutor}
                  </p>
                  {course.tutor_rating != null && course.tutor_rating > 0 && (
                    <p className="mt-0.5 flex items-center gap-1 text-xs text-slate-500 dark:text-slate-400">
                      <Star className="size-3 fill-amber-400 text-amber-400" />
                      {course.tutor_rating.toFixed(1)}
                    </p>
                  )}
                  {course.tutor_bio && (
                    <p className="mt-1 line-clamp-2 text-xs text-slate-500 dark:text-slate-400">
                      {course.tutor_bio}
                    </p>
                  )}
                </div>
                <span className="hidden text-xs font-semibold text-slate-500 sm:inline dark:text-slate-400">
                  →
                </span>
              </Link>
            ) : (
              <p className="text-sm text-slate-500 dark:text-slate-400">{lc.noTutorHint}</p>
            )}
          </section>
        </motion.article>

        <aside className="lg:sticky lg:top-24 lg:self-start">
          <div className="space-y-4 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
            <div>
              <p className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                {lc.price}
              </p>
              <p className="mt-1 text-3xl font-bold text-slate-900 tabular-nums dark:text-slate-100">
                {formatVnd(course.price_vnd)}
                <span className="ml-1.5 text-sm font-medium text-slate-500 dark:text-slate-400">
                  ₫ {lc.perSession}
                </span>
              </p>
            </div>
            {course.tutor_id ? (
              <Link
                href={`/tutors/${course.tutor_id}?course=${course.id}`}
                className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-11 w-full items-center justify-center gap-1.5 rounded-lg text-sm font-semibold text-white transition-colors"
              >
                {lc.bookCta}
                <ArrowRight className="size-4" />
              </Link>
            ) : (
              <p className="text-center text-xs text-slate-500 dark:text-slate-400">
                {lc.noTutorHint}
              </p>
            )}
          </div>
        </aside>
      </div>
    </div>
  );
}
