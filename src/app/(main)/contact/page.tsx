'use client';

import { Mail, Clock, HelpCircle } from 'lucide-react';
import { useLang, useSiteSettings } from '@/contexts';

export default function ContactPage() {
  const { lang } = useLang();
  const { settings } = useSiteSettings();

  const copy = {
    VN: {
      title: 'Liên hệ',
      intro:
        'Chúng tôi thường phản hồi trong vòng 1 ngày làm việc. Chọn kênh phù hợp với câu hỏi của bạn.',
      emailLabel: 'Email hỗ trợ',
      hoursLabel: 'Giờ hỗ trợ',
      hours: 'Thứ 2 – Thứ 6, 9:00 – 18:00 (GMT+7)',
      faqLabel: 'Câu hỏi thường gặp',
      faqBody: 'Trước khi nhắn cho chúng tôi, xem thử FAQ — nhiều câu trả lời có sẵn ở đó.',
      faqCta: 'Đọc FAQ',
    },
    EN: {
      title: 'Contact us',
      intro: 'We usually reply within one business day. Pick the channel that fits your question.',
      emailLabel: 'Support email',
      hoursLabel: 'Support hours',
      hours: 'Mon – Fri, 9:00 – 18:00 (GMT+7)',
      faqLabel: 'FAQ',
      faqBody: 'Before pinging us, check the FAQ — most answers live there.',
      faqCta: 'Read the FAQ',
    },
  }[lang];

  return (
    <main className="mx-auto max-w-3xl space-y-8 px-4 py-10 md:py-16">
      <header className="space-y-2">
        <h1 className="text-brand dark:text-accent-warm font-serif text-3xl font-bold md:text-4xl">
          {copy.title}
        </h1>
        <p className="max-w-2xl pt-2 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
          {copy.intro}
        </p>
      </header>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <Card
          icon={<Mail className="text-brand size-5" />}
          label={copy.emailLabel}
          body={
            settings.email ? (
              <a
                href={`mailto:${settings.email}`}
                className="text-brand focus-ring dark:text-accent-warm text-sm font-semibold hover:underline"
              >
                {settings.email}
              </a>
            ) : (
              <span className="text-sm text-slate-500 dark:text-slate-400">—</span>
            )
          }
        />
        <Card
          icon={<Clock className="text-brand size-5" />}
          label={copy.hoursLabel}
          body={<p className="text-sm text-slate-700 dark:text-slate-200">{copy.hours}</p>}
        />
      </div>

      <div className="rounded-2xl border border-slate-200 bg-slate-50/50 p-6 dark:border-slate-800 dark:bg-slate-900/40">
        <div className="flex items-start gap-3">
          <HelpCircle className="text-brand mt-0.5 size-5 shrink-0" />
          <div className="space-y-2">
            <p className="text-sm font-semibold text-slate-900 dark:text-slate-100">
              {copy.faqLabel}
            </p>
            <p className="text-sm text-slate-600 dark:text-slate-300">{copy.faqBody}</p>
            <a
              href="/faq"
              className="text-brand focus-ring dark:text-accent-warm inline-flex text-sm font-semibold hover:underline"
            >
              {copy.faqCta} →
            </a>
          </div>
        </div>
      </div>
    </main>
  );
}

interface CardProps {
  icon: React.ReactNode;
  label: string;
  body: React.ReactNode;
}

function Card({ icon, label, body }: CardProps) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="mb-2 flex items-center gap-2">
        {icon}
        <span className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
          {label}
        </span>
      </div>
      {body}
    </div>
  );
}
