import type { MetadataRoute } from "next";
import { BOOKS } from "@/lib/books";
import { courseLessons, lessonHref, listCourses } from "@/lib/courses";
import { EXAM_LEVELS } from "@/lib/exam-levels";
import { listExams } from "@/lib/exams";
import { GUIDE_SECTIONS, articleHref, sectionArticles } from "@/lib/guide";
import { LANGUAGES } from "@/lib/languages";
import { absoluteUrl } from "@/lib/seo";
import { SERIES } from "@/lib/series";

/**
 * Sitemap sinh từ chính các bảng dữ liệu mà route dùng để dựng trang —
 * thêm khoá học, bài cẩm nang, đề thi là tự có mặt ở đây.
 *
 * KHÔNG có trang đọc sách (`/read/…`): đó là ảnh scan giáo trình, gắn
 * noindex (xem app/read/[bookId]/[page]/page.tsx).
 *
 * Không khai `changeFrequency`/`priority`: Google bỏ qua cả hai. `lastModified`
 * chỉ ghi ở chỗ có ngày thật (bài cẩm nang) — ngày bịa ra làm Google mất tin
 * vào cả sitemap.
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const paths: { path: string; lastModified?: string }[] = [];
  const add = (path: string, lastModified?: string) => paths.push({ path, lastModified });

  // IT — trang chủ là danh sách khoá học
  add("/");
  for (const course of listCourses()) {
    add(`/it/${course.id}`);
    for (const l of courseLessons(course)) add(lessonHref(course.id, l.slug));
  }

  // Tiếng Anh
  for (const lang of LANGUAGES) add(`/${lang.slug}`);
  for (const series of SERIES) add(`/sach/${series.id}`);
  for (const book of BOOKS) add(`/books/${book.id}`);

  // Luyện thi
  for (const level of EXAM_LEVELS) add(`/exam/${level.slug}`);
  for (const exam of listExams()) add(`/exam/${exam.id}`);

  // Cẩm nang
  for (const section of GUIDE_SECTIONS) {
    add(`/cam-nang/${section.id}`);
    for (const a of sectionArticles(section.id)) add(articleHref(section.id, a.slug), a.updated);
  }

  add("/install");

  return paths.map(({ path, lastModified }) => ({
    url: absoluteUrl(path),
    ...(lastModified && { lastModified }),
  }));
}
