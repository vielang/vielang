/**
 * Khoá học dạng CHỮ của mảng IT (`/it`) — khác hẳn kệ sách ảnh: nội dung là
 * Markdown do mình viết, gồm giải thích tiếng Việt và thuật ngữ giữ nguyên
 * tiếng Anh, kèm code mẫu.
 *
 * Nguồn: `content/it/<khoá>/<chương>/<bài>.md` (có phần khai báo ở đầu file),
 * gộp thành `content/it/courses.json` qua `npm run build-content` — cùng cách
 * và cùng lý do với note/quiz: `import` TĨNH file JSON, vì Output File
 * Tracing của Next không lần được file đọc qua path dựng động lúc chạy.
 */
import data from "../../content/it/courses.json";

export interface LessonHeading {
  /** id của thẻ h2/h3 trong HTML — dùng cho mục lục bên trong bài. */
  id: string;
  text: string;
  level: 2 | 3;
}

export interface Lesson {
  /** Định danh trong khoá: "<chương>/<bài>", cũng là phần đuôi URL. */
  slug: string;
  title: string;
  /** Ước lượng thời gian đọc, phút. */
  minutes: number;
  /** Nội dung đã dựng sẵn sang HTML (code đã tô màu lúc build). */
  html: string;
  headings: LessonHeading[];
}

export interface CourseModule {
  slug: string;
  title: string;
  /** Mô tả ngắn của chương, hiện ở mục lục khoá. */
  summary?: string;
  lessons: Lesson[];
}

export interface Course {
  /** Dùng trong URL: `/it/<id>` */
  id: string;
  title: string;
  summary: string;
  /** Nhãn trình độ ("Nền tảng", "Trung cấp"…) — nhóm khoá ở trang /it. */
  level: string;
  /** Thứ tự hiển thị trong lộ trình. */
  order: number;
  modules: CourseModule[];
}

const COURSES = data as unknown as Course[];

export function listCourses(): Course[] {
  return [...COURSES].sort((a, b) => a.order - b.order || a.title.localeCompare(b.title));
}

export function getCourse(id: string): Course | undefined {
  return COURSES.find((c) => c.id === id);
}

/** Mọi bài của khoá, phẳng theo đúng thứ tự học. */
export function courseLessons(course: Course): Lesson[] {
  return course.modules.flatMap((m) => m.lessons);
}

export function getLesson(course: Course, slug: string): Lesson | undefined {
  return courseLessons(course).find((l) => l.slug === slug);
}

/** Bài trước / bài sau theo thứ tự học (null ở hai đầu khoá). */
export function lessonNeighbours(course: Course, slug: string): { prev: Lesson | null; next: Lesson | null } {
  const all = courseLessons(course);
  const i = all.findIndex((l) => l.slug === slug);
  return { prev: i > 0 ? all[i - 1] : null, next: i >= 0 && i < all.length - 1 ? all[i + 1] : null };
}

export function lessonHref(courseId: string, slug: string): string {
  return `/learn/${courseId}/${slug}`;
}

export function courseMinutes(course: Course): number {
  return courseLessons(course).reduce((n, l) => n + l.minutes, 0);
}

/**
 * Tiến độ học lưu chung kho với sách (`progress-store`): mỗi khoá là một
 * "cuốn", mỗi bài là một "trang" theo đúng thứ tự học. Dùng lại kho cũ thì
 * đồng bộ giữa các tab và sao lưu/khôi phục (learning-backup) có sẵn, khỏi
 * dựng kho thứ hai.
 *
 * Tiền tố `it:` để id không đụng id sách; các màn hình của sách lọc theo
 * BOOKS nên chúng bỏ qua mục của khoá học.
 */
export function courseProgressId(courseId: string): string {
  return `it:${courseId}`;
}

/** Số thứ tự của bài trong khoá, đếm từ 1 (0 = không tìm thấy). */
export function lessonNumber(course: Course, slug: string): number {
  return courseLessons(course).findIndex((l) => l.slug === slug) + 1;
}
