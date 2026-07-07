'use client';

import React from 'react';
import { BrandMark } from '@/components/shared/BrandMark';
import { motion } from 'motion/react';

// Left column of the LoginPage: brand mark, hero copy in the active language,
// and a small tag rail at the bottom. Hidden < lg; the mobile layout is the
// form column alone.

const COPY = {
  VN: {
    heading: (
      <>
        Học tiếng Anh
        <br />
        <span className="text-accent-warm italic">1-1</span> qua video call
        <br />
        với VieLang
      </>
    ),
    desc: 'Nền tảng học tiếng Anh 1-1 với giáo viên bản ngữ và giáo viên có chứng chỉ, lịch học linh hoạt.',
    tags: ['Tutor', 'Lớp học', 'Video call'],
  },
  EN: {
    heading: (
      <>
        Learn English <br />
        <span className="text-accent-warm italic">1-on-1</span> Live <br />
        with VieLang
      </>
    ),
    desc: 'Book flexible video sessions with certified English tutors. Improve speaking, IELTS, business English and more.',
    tags: ['Tutors', 'Classes', 'Video'],
  },
} as const;

export function BrandingSide({ lang }: { lang: 'VN' | 'EN' }) {
  const lc = COPY[lang];
  return (
    <div className="bg-brand relative hidden flex-col justify-between overflow-hidden p-10 text-white lg:flex">
      <div className="absolute top-0 right-0 h-64 w-64 translate-x-1/2 -translate-y-1/2 rounded-full bg-white/5 blur-3xl" />
      <div className="bg-accent-warm/10 absolute bottom-0 left-0 h-96 w-96 -translate-x-1/2 translate-y-1/2 rounded-full blur-3xl" />

      <div className="relative z-10 flex items-center gap-3">
        <BrandMark className="size-12 text-white" />
        <span className="font-serif text-2xl font-bold tracking-tight uppercase italic">
          Vie<span className="text-accent-warm">Lang</span>
        </span>
      </div>

      <motion.div
        key={lang}
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className="relative z-10 space-y-5"
      >
        <h1 className="font-serif text-4xl leading-tight font-bold xl:text-5xl">{lc.heading}</h1>
        <p className="max-w-md text-sm leading-relaxed text-slate-300">{lc.desc}</p>
      </motion.div>

      <div className="relative z-10 flex items-center gap-5 text-[11px] font-semibold tracking-wider text-slate-400 uppercase">
        {lc.tags.map((tag, i) => (
          <React.Fragment key={tag}>
            {i > 0 && <div className="bg-accent-warm size-1 rounded-full" />}
            <span>{tag}</span>
          </React.Fragment>
        ))}
      </div>
    </div>
  );
}
