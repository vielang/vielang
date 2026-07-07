'use client';

import { useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { useLang } from '@/contexts';

type QA = { q: { vn: string; en: string }; a: { vn: string; en: string } };

const FAQ: QA[] = [
  {
    q: {
      vn: 'Tôi bắt đầu học tại VieLang thế nào?',
      en: 'How do I get started with VieLang?',
    },
    a: {
      vn: 'Tạo tài khoản, duyệt qua danh sách giáo viên, chọn một khoá học và khung giờ phù hợp. Buổi học đầu tiên diễn ra trong trình duyệt qua video call — không cần cài thêm phần mềm.',
      en: 'Create an account, browse the tutor list, pick a course and time slot. Your first session runs in the browser over a video call — no extra software required.',
    },
  },
  {
    q: {
      vn: 'Giáo viên VieLang có được kiểm duyệt không?',
      en: 'Are VieLang tutors vetted?',
    },
    a: {
      vn: 'Có. Mỗi giáo viên phải cung cấp chứng chỉ và trải qua quá trình duyệt bởi đội ngũ admin trước khi hiển thị công khai. Chỉ giáo viên đã được duyệt (is_approved=true) mới xuất hiện trên trang /tutors.',
      en: 'Yes. Every tutor submits credentials and goes through an admin approval step before appearing publicly. Only approved tutors (is_approved=true) show up on /tutors.',
    },
  },
  {
    q: {
      vn: 'Buổi học 1-on-1 và phòng free-talk khác nhau thế nào?',
      en: 'What is the difference between 1-on-1 sessions and group free-talk rooms?',
    },
    a: {
      vn: '1-on-1 là buổi học riêng với giáo viên, gắn với một khoá học cụ thể, có tài liệu, và có thu phí. Group free-talk là phòng chung do đội ngũ VieLang tổ chức để luyện nói với người học khác — hiện miễn phí.',
      en: '1-on-1 sessions are private lessons with a tutor, tied to a specific course, with materials, and paid. Group free-talk rooms are open community rooms hosted by the VieLang team for speaking practice with other learners — currently free.',
    },
  },
  {
    q: {
      vn: 'Tôi cần chuẩn bị gì trước buổi học?',
      en: 'What do I need before a session?',
    },
    a: {
      vn: 'Trình duyệt hiện đại (Chrome/Safari/Edge), microphone và camera hoạt động, kết nối internet ổn định. Phòng học mở trước giờ bắt đầu 15 phút để bạn kiểm tra thiết bị.',
      en: 'A modern browser (Chrome/Safari/Edge), a working microphone and camera, and a stable internet connection. The room opens 15 minutes before start time so you can test your setup.',
    },
  },
  {
    q: {
      vn: 'Huỷ buổi học được không?',
      en: 'Can I cancel a session?',
    },
    a: {
      vn: 'Có — huỷ trước ≥ 24 giờ được hoàn tiền đầy đủ. Chi tiết ở Chính sách hoàn tiền.',
      en: 'Yes — cancel at least 24 hours in advance for a full refund. Details on the Refund Policy page.',
    },
  },
  {
    q: {
      vn: 'Tôi quên mật khẩu, phải làm sao?',
      en: 'I forgot my password — what now?',
    },
    a: {
      vn: 'Trên trang đăng nhập, bấm "Quên mật khẩu". Chúng tôi gửi email reset — mở link trong email để đặt mật khẩu mới.',
      en: 'On the login page, click "Forgot password". We send a reset email — open the link to set a new password.',
    },
  },
  {
    q: {
      vn: 'Video call có được ghi lại không?',
      en: 'Are video calls recorded?',
    },
    a: {
      vn: 'Không, VieLang KHÔNG ghi hình hoặc ghi âm buổi học. Chúng tôi chỉ lưu tin nhắn text trong lớp và thời gian tham gia.',
      en: 'No, VieLang does NOT record video or audio. We only store in-class text messages and attendance timestamps.',
    },
  },
  {
    q: {
      vn: 'Tôi muốn trở thành giáo viên, đăng ký thế nào?',
      en: 'How can I become a tutor?',
    },
    a: {
      vn: 'Tính năng đăng ký giáo viên đang mở beta. Gửi email liên hệ để trao đổi với đội ngũ — chúng tôi sẽ hướng dẫn quy trình cung cấp chứng chỉ và duyệt hồ sơ.',
      en: 'Tutor sign-up is in beta. Reach out via the Contact page — we will walk you through the credential and approval steps.',
    },
  },
];

export default function FAQPage() {
  const { lang } = useLang();
  const [openIx, setOpenIx] = useState<number | null>(0);

  const copy = {
    VN: {
      title: 'Câu hỏi thường gặp',
      intro: 'Câu trả lời nhanh cho những thắc mắc phổ biến nhất.',
    },
    EN: {
      title: 'Frequently asked questions',
      intro: 'Fast answers to the questions we hear most.',
    },
  }[lang];

  return (
    <main className="mx-auto max-w-3xl space-y-6 px-4 py-10 md:py-16">
      <header className="space-y-2">
        <h1 className="text-brand dark:text-accent-warm font-serif text-3xl font-bold md:text-4xl">
          {copy.title}
        </h1>
        <p className="max-w-2xl pt-2 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
          {copy.intro}
        </p>
      </header>

      <ul className="divide-y divide-slate-200 rounded-2xl border border-slate-200 bg-white shadow-sm dark:divide-slate-800 dark:border-slate-800 dark:bg-slate-900">
        {FAQ.map((item, i) => {
          const open = openIx === i;
          return (
            <li key={i}>
              <button
                type="button"
                onClick={() => setOpenIx(open ? null : i)}
                aria-expanded={open}
                className="focus-ring flex w-full items-center justify-between gap-4 rounded-2xl px-5 py-4 text-left"
              >
                <span className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                  {lang === 'VN' ? item.q.vn : item.q.en}
                </span>
                <ChevronDown
                  className={`size-4 shrink-0 text-slate-400 transition-transform ${
                    open ? 'rotate-180' : ''
                  }`}
                  aria-hidden
                />
              </button>
              {open && (
                <div className="px-5 pb-4 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
                  {lang === 'VN' ? item.a.vn : item.a.en}
                </div>
              )}
            </li>
          );
        })}
      </ul>
    </main>
  );
}
