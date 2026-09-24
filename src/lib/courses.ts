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
import type { QuizSection } from "@/lib/quiz";

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
  /**
   * Câu tự kiểm tra cuối bài, viết bằng khối ```quiz trong file .md. Dùng
   * đúng kiểu dữ liệu của bài tập trong sách nên phần chấm, phần lưu bài làm
   * và thống kê ở Góc học tập chạy chung một đường.
   */
  quiz?: QuizSection[];
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

/**
 * Bản RÚT GỌN của khoá/bài để đưa sang component phía client.
 *
 * `Lesson.html` là cả bài viết — trang mục lục không cần một chữ nào trong đó,
 * mà props của client component thì bị đóng gói vào payload gửi về trình
 * duyệt. Truyền nguyên `Course` là tải hết mọi bài chỉ để vẽ một danh sách
 * tiêu đề.
 */
export interface LessonOutline {
  slug: string;
  title: string;
  minutes: number;
  /** Số thứ tự trong khoá, đếm từ 1 — cũng là "số trang" của tiến độ. */
  no: number;
}

export interface ModuleOutline {
  slug: string;
  title: string;
  summary?: string;
  lessons: LessonOutline[];
}

export interface CourseOutline {
  id: string;
  title: string;
  summary: string;
  level: string;
  /** Tổng số bài và tổng thời gian đọc của cả khoá. */
  total: number;
  minutes: number;
  modules: ModuleOutline[];
}

export function courseOutline(course: Course): CourseOutline {
  let no = 0;
  return {
    id: course.id,
    title: course.title,
    summary: course.summary,
    level: course.level,
    total: courseLessons(course).length,
    minutes: courseMinutes(course),
    modules: course.modules.map((m) => ({
      slug: m.slug,
      title: m.title,
      ...(m.summary ? { summary: m.summary } : {}),
      lessons: m.lessons.map((l) => ({ slug: l.slug, title: l.title, minutes: l.minutes, no: ++no })),
    })),
  };
}

/** Một dòng trong lộ trình ở trang `/it` — không kèm mục lục lẫn nội dung. */
export interface CourseCard {
  id: string;
  title: string;
  summary: string;
  level: string;
  total: number;
  minutes: number;
  /** Tên và đường dẫn từng bài theo thứ tự học, để thẻ "Học tiếp" biết bài đang dở. */
  lessons: { title: string; slug: string }[];
}

export function courseCard(course: Course): CourseCard {
  const lessons = courseLessons(course);
  return {
    id: course.id,
    title: course.title,
    summary: course.summary,
    level: course.level,
    total: lessons.length,
    minutes: courseMinutes(course),
    lessons: lessons.map((l) => ({ title: l.title, slug: l.slug })),
  };
}

/**
 * Bài trước / bài sau theo thứ tự học (null ở hai đầu khoá). Trả bản rút gọn
 * vì nút điều hướng chỉ cần tiêu đề và đường dẫn, không cần cả bài viết.
 */
export function lessonNeighbours(
  course: Course,
  slug: string
): { prev: LessonOutline | null; next: LessonOutline | null } {
  const all = courseOutline(course).modules.flatMap((m) => m.lessons);
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

/**
 * Khoá lưu bài làm phần "Tự kiểm tra" của một bài học.
 *
 * Kho bài làm (`quiz-store`) dùng chung với bài tập trong sách, mà khoá của
 * sách là `"<bookId>:<số trang>"`. Tiền tố `it:` cộng với dấu `/` trong slug
 * khiến khoá của bài học không thể trùng — và các màn hình của sách lọc theo
 * BOOKS nên chúng bỏ qua mục này.
 */
export function lessonQuizId(courseId: string, slug: string): string {
  return `it:${courseId}/${slug}`;
}

/** Số thứ tự của bài trong khoá, đếm từ 1 (0 = không tìm thấy). */
export function lessonNumber(course: Course, slug: string): number {
  return courseLessons(course).findIndex((l) => l.slug === slug) + 1;
}
