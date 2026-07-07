// VieLang seed data — populated by POST /api/seed for local dev demos.
// Keep counts small; the point is a coherent walkthrough, not scale testing.
//
// Deterministic UUIDs so re-running the seed is idempotent (upserts key off id).
// Grouped by entity so you can eyeball the shape of a well-formed row.

export const ADMIN_USER = {
  id: '00000000-0000-4000-8000-000000000001',
  email: 'admin@vielang.com',
  name: 'VieLang Admin',
  role: 'admin' as const,
  avatar: null,
  bio: 'Platform administrator',
};

export const TUTORS = [
  {
    id: '10000000-0000-4000-8000-000000000001',
    email: 'alice@vielang.com',
    name: 'Alice Nguyen',
    bio: 'IELTS speaking specialist. TESOL-certified, 8 years teaching Vietnamese learners.',
    avatar: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=400',
    profile: {
      hourly_rate_vnd: 350_000,
      intro_video_url: null,
      intro_video_thumbnail: null,
      specialties: ['IELTS', 'Speaking', 'Pronunciation'],
      years_experience: 8,
      certifications: ['TESOL', 'IELTS 8.5'],
      languages_spoken: ['en', 'vi'],
      is_approved: true,
    },
  },
  {
    id: '10000000-0000-4000-8000-000000000002',
    email: 'bob@vielang.com',
    name: 'Bob Chen',
    bio: 'Business English coach with 5 years of corporate training experience.',
    avatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=400',
    profile: {
      hourly_rate_vnd: 420_000,
      intro_video_url: null,
      intro_video_thumbnail: null,
      specialties: ['Business', 'Presentation', 'Interview'],
      years_experience: 5,
      certifications: ['CELTA', 'MBA'],
      languages_spoken: ['en', 'zh'],
      is_approved: true,
    },
  },
  {
    id: '10000000-0000-4000-8000-000000000003',
    email: 'chi@vielang.com',
    name: 'Chi Tran',
    bio: 'Fun and patient with kids. 6 years teaching young learners aged 6-12.',
    avatar: 'https://images.unsplash.com/photo-1580489944761-15a19d654956?w=400',
    profile: {
      hourly_rate_vnd: 280_000,
      intro_video_url: null,
      intro_video_thumbnail: null,
      specialties: ['Kids', 'Phonics', 'Storytelling'],
      years_experience: 6,
      certifications: ['TEFL Young Learners'],
      languages_spoken: ['en', 'vi'],
      is_approved: true,
    },
  },
];

export const STUDENTS = [
  {
    id: '20000000-0000-4000-8000-000000000001',
    email: 'student1@demo.vielang.com',
    name: 'Minh Le',
  },
  {
    id: '20000000-0000-4000-8000-000000000002',
    email: 'student2@demo.vielang.com',
    name: 'Huong Pham',
  },
];

// Recurring weekly availability. weekday 0=Sunday to match JS Date.getDay().
// Times are TIME (no timezone) — tutor's local slot, rendered client-side per
// user.timezone.
export const AVAILABILITY = [
  { tutor_id: TUTORS[0].id, weekday: 1, start_time: '08:00', end_time: '11:00' },
  { tutor_id: TUTORS[0].id, weekday: 3, start_time: '08:00', end_time: '11:00' },
  { tutor_id: TUTORS[0].id, weekday: 6, start_time: '14:00', end_time: '18:00' },
  { tutor_id: TUTORS[1].id, weekday: 2, start_time: '19:00', end_time: '22:00' },
  { tutor_id: TUTORS[1].id, weekday: 4, start_time: '19:00', end_time: '22:00' },
  { tutor_id: TUTORS[2].id, weekday: 1, start_time: '15:00', end_time: '18:00' },
  { tutor_id: TUTORS[2].id, weekday: 3, start_time: '15:00', end_time: '18:00' },
  { tutor_id: TUTORS[2].id, weekday: 5, start_time: '15:00', end_time: '18:00' },
];

export const COURSES = [
  {
    id: '30000000-0000-4000-8000-000000000001',
    title_vn: 'IELTS Speaking 6.5+ với Alice',
    title_en: 'IELTS Speaking 6.5+ with Alice',
    description_vn: 'Luyện phát âm + trả lời Part 1-3 sát band 6.5-7.5.',
    description_en: 'Pronunciation + Part 1-3 practice targeting bands 6.5-7.5.',
    level: 'B2',
    category: 'IELTS',
    tutor_id: TUTORS[0].id,
    price_vnd: 350_000,
    duration_min: 60,
    image: 'https://images.unsplash.com/photo-1546410531-bb4caa6b424d?w=800',
    is_published: true,
  },
  {
    id: '30000000-0000-4000-8000-000000000002',
    title_vn: 'General English Speaking B1',
    title_en: 'General English Speaking B1',
    description_vn: 'Giao tiếp hàng ngày, mở rộng vốn từ, sửa lỗi ngữ pháp.',
    description_en: 'Daily conversation, vocabulary building, grammar correction.',
    level: 'B1',
    category: 'General',
    tutor_id: TUTORS[0].id,
    price_vnd: 300_000,
    duration_min: 60,
    image: 'https://images.unsplash.com/photo-1523240795612-9a054b0db644?w=800',
    is_published: true,
  },
  {
    id: '30000000-0000-4000-8000-000000000003',
    title_vn: 'Business English cho người đi làm',
    title_en: 'Business English for Professionals',
    description_vn: 'Email, meeting, presentation — phong cách chuyên nghiệp.',
    description_en: 'Emails, meetings, presentations — polished professional tone.',
    level: 'B2',
    category: 'Business',
    tutor_id: TUTORS[1].id,
    price_vnd: 420_000,
    duration_min: 60,
    image: 'https://images.unsplash.com/photo-1552664730-d307ca884978?w=800',
    is_published: true,
  },
  {
    id: '30000000-0000-4000-8000-000000000004',
    title_vn: 'Job Interview English',
    title_en: 'Job Interview English',
    description_vn: 'Chuẩn bị phỏng vấn tiếng Anh: câu hỏi, cách trả lời, ngôn ngữ cơ thể.',
    description_en: 'Master interview questions, STAR method, and body language.',
    level: 'B2',
    category: 'Business',
    tutor_id: TUTORS[1].id,
    price_vnd: 450_000,
    duration_min: 45,
    image: 'https://images.unsplash.com/photo-1521737604893-d14cc237f11d?w=800',
    is_published: true,
  },
  {
    id: '30000000-0000-4000-8000-000000000005',
    title_vn: 'Tiếng Anh cho trẻ em (6-10 tuổi)',
    title_en: 'English for Kids (age 6-10)',
    description_vn: 'Storytelling, songs, phonics — bài học vui, tương tác.',
    description_en: 'Storytelling, songs, phonics — fun and interactive lessons.',
    level: 'A1',
    category: 'Kids',
    tutor_id: TUTORS[2].id,
    price_vnd: 280_000,
    duration_min: 45,
    image: 'https://images.unsplash.com/photo-1503676260728-1c00da094a0b?w=800',
    is_published: true,
  },
];

export const BANNERS = [
  {
    id: '40000000-0000-4000-8000-000000000001',
    title_vn: 'Học thử miễn phí buổi đầu',
    title_en: 'First lesson free',
    subtitle_vn: 'Đặt lịch ngay với tutor bạn chọn.',
    subtitle_en: 'Book a trial with any of our approved tutors.',
    image: 'https://images.unsplash.com/photo-1497633762265-9d179a990aa6?w=1600',
    link: '/tutors',
    order_index: 0,
    is_active: true,
  },
  {
    id: '40000000-0000-4000-8000-000000000002',
    title_vn: 'IELTS Speaking chuyên sâu',
    title_en: 'Deep-dive IELTS Speaking',
    subtitle_vn: 'Đạt band 6.5+ với coaching 1-1.',
    subtitle_en: 'Hit band 6.5+ with focused 1-1 coaching.',
    image: 'https://images.unsplash.com/photo-1434030216411-0b793f4b4173?w=1600',
    link: '/courses',
    order_index: 1,
    is_active: true,
  },
  {
    id: '40000000-0000-4000-8000-000000000003',
    title_vn: 'Business English cho doanh nghiệp',
    title_en: 'Business English for teams',
    subtitle_vn: 'Đào tạo linh hoạt cho team của bạn.',
    subtitle_en: 'Flexible training for your team.',
    image: 'https://images.unsplash.com/photo-1521737604893-d14cc237f11d?w=1600',
    link: '/contact',
    order_index: 2,
    is_active: true,
  },
];

// Free-talk group sessions — public-facing rooms shown on / and /sessions.
// Scheduled relative to "now" so the demo always has upcoming rows regardless
// of when the seed is run (times are computed at request time by
// buildGroupSessionRows() in the seed route). Deterministic UUIDs so re-seeding
// is idempotent.
export const GROUP_SESSIONS: {
  id: string;
  topic_en: string;
  topic_vn: string;
  description: string;
  level: 'all' | 'a1_a2' | 'b1_b2' | 'c1_c2';
  capacity: number;
  duration_min: number;
  cover_emoji: string;
  // Hours from now to schedule the session at. Small positives = "soon";
  // negatives cover live/just-started rooms.
  offset_hours: number;
  tutor_index: number | null; // index into TUTORS or null for admin-hosted
}[] = [
  {
    id: '60000000-0000-4000-8000-000000000001',
    topic_en: 'Weekend Free Talk · Travel Stories',
    topic_vn: 'Free Talk cuối tuần · Chuyện du lịch',
    description:
      'Share a travel story from the past year — a place, a mishap, a favorite meal. Easy warm-up prompts, no pressure.',
    level: 'a1_a2',
    capacity: 8,
    duration_min: 45,
    cover_emoji: '✈️',
    offset_hours: 3,
    tutor_index: 2,
  },
  {
    id: '60000000-0000-4000-8000-000000000002',
    topic_en: 'Business English · Small Talk at Meetings',
    topic_vn: 'Business English · Small talk trong họp',
    description:
      'The 90 seconds before a meeting starts — how to open, hold, and exit small talk without going quiet.',
    level: 'b1_b2',
    capacity: 6,
    duration_min: 60,
    cover_emoji: '💼',
    offset_hours: 6,
    tutor_index: 1,
  },
  {
    id: '60000000-0000-4000-8000-000000000003',
    topic_en: 'IELTS Speaking · Part 2 Cue Card Practice',
    topic_vn: 'IELTS Speaking · Luyện Part 2 Cue Card',
    description:
      'Rapid-fire Part 2 rounds — one minute prep, two minutes speaking, then peer + host feedback.',
    level: 'b1_b2',
    capacity: 6,
    duration_min: 60,
    cover_emoji: '🎯',
    offset_hours: 22,
    tutor_index: 0,
  },
  {
    id: '60000000-0000-4000-8000-000000000004',
    topic_en: 'Advanced Discussion · News & Opinions',
    topic_vn: 'Thảo luận nâng cao · Tin tức & quan điểm',
    description:
      'One curated headline, structured debate. Bring an opinion; leave with two new phrases.',
    level: 'c1_c2',
    capacity: 5,
    duration_min: 60,
    cover_emoji: '🗞️',
    offset_hours: 26,
    tutor_index: 1,
  },
  {
    id: '60000000-0000-4000-8000-000000000005',
    topic_en: 'Kids Storytime · Read Aloud Together',
    topic_vn: 'Kids Storytime · Đọc truyện cùng nhau',
    description:
      'A short story with picture prompts, songs and vocabulary games. Ages 6–10, parents welcome to sit in.',
    level: 'a1_a2',
    capacity: 8,
    duration_min: 30,
    cover_emoji: '📚',
    offset_hours: 30,
    tutor_index: 2,
  },
  {
    id: '60000000-0000-4000-8000-000000000006',
    topic_en: 'Pronunciation Clinic · Th, R and V sounds',
    topic_vn: 'Pronunciation Clinic · Âm Th, R, V',
    description:
      'Targeted drills for the three sounds Vietnamese learners find hardest, with real-time feedback.',
    level: 'b1_b2',
    capacity: 6,
    duration_min: 45,
    cover_emoji: '🎤',
    offset_hours: 48,
    tutor_index: 0,
  },
  {
    id: '60000000-0000-4000-8000-000000000007',
    topic_en: 'Coffee Chat · Everyday English',
    topic_vn: 'Coffee Chat · Tiếng Anh mỗi ngày',
    description:
      'Casual free-talk on ordinary topics — weekend plans, favorite food, weather. Perfect for first-timers.',
    level: 'all',
    capacity: 10,
    duration_min: 45,
    cover_emoji: '☕',
    offset_hours: 52,
    tutor_index: null,
  },
  {
    id: '60000000-0000-4000-8000-000000000008',
    topic_en: 'Job Interview Roleplay',
    topic_vn: 'Đóng vai phỏng vấn xin việc',
    description:
      'Mock interview roleplay — behavioural + STAR-method questions. Bring your CV highlights.',
    level: 'b1_b2',
    capacity: 5,
    duration_min: 60,
    cover_emoji: '🧑‍💻',
    offset_hours: 72,
    tutor_index: 1,
  },
  {
    id: '60000000-0000-4000-8000-000000000009',
    topic_en: 'Debate Club · Would You Rather',
    topic_vn: 'CLB Tranh biện · Would You Rather',
    description:
      'Playful debate prompts, structured turn-taking. Good for building fluency under mild pressure.',
    level: 'c1_c2',
    capacity: 6,
    duration_min: 60,
    cover_emoji: '⚖️',
    offset_hours: 96,
    tutor_index: 0,
  },
];

export const NEWS = [
  {
    id: '50000000-0000-4000-8000-000000000001',
    title_vn: 'Ra mắt VieLang — nền tảng học tiếng Anh mới',
    title_en: 'Introducing VieLang — live 1-on-1 English tutoring',
    content_vn:
      '<p>VieLang chính thức ra mắt với sứ mệnh giúp học viên Việt Nam nói tiếng Anh tự tin hơn qua các buổi học 1-1 trực tiếp với giáo viên chất lượng.</p>',
    content_en:
      '<p>VieLang launches today with a mission to help Vietnamese learners speak English with confidence through live 1-on-1 lessons with certified tutors.</p>',
    category: 'Announcement',
    image: 'https://images.unsplash.com/photo-1523050854058-8df90110c9f1?w=1600',
    is_published: true,
  },
  {
    id: '50000000-0000-4000-8000-000000000002',
    title_vn: '5 mẹo luyện IELTS Speaking hiệu quả tại nhà',
    title_en: '5 tips for effective IELTS Speaking practice at home',
    content_vn:
      '<p>Luyện Speaking một mình khó, nhưng vẫn khả thi. Sau đây là 5 kỹ thuật giáo viên IELTS thường khuyên dùng...</p>',
    content_en:
      '<p>Practicing Speaking alone is hard but doable. Here are five techniques IELTS teachers regularly recommend...</p>',
    category: 'Study Tips',
    image: 'https://images.unsplash.com/photo-1434030216411-0b793f4b4173?w=1600',
    is_published: true,
  },
];
