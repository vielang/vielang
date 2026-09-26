import type { MetadataRoute } from "next";
import { BOOKS } from "@/lib/books";
import { bookLessonHref, bookLessons } from "@/lib/book-lessons";
import { courseLessons, lessonHref, listCourses } from "@/lib/courses";
import { EXAM_LEVELS } from "@/lib/exam-levels";
import { listExams } from "@/lib/exams";
import { GUIDE_SECTIONS, articleHref, sectionArticles } from "@/lib/guide";
import { LANGUAGES } from "@/lib/languages";
import { getNotePages } from "@/lib/notes";
import { getAllGrammar, getGrammarPages, grammarHref } from "@/lib/page-grammar";
import { getTranslatedPages } from "@/lib/page-translation";
import { absoluteUrl } from "@/lib/seo";
import { SERIES } from "@/lib/series";
import { getVideoLessons } from "@/lib/videos";

/**
 * Sitemap sinh từ chính các bảng dữ liệu mà route dùng để dựng trang —
 * thêm bài, thêm sách là tự có mặt ở đây.
 *
 * Trang đọc sách: CHỈ trang có bản dịch / ngữ pháp / bài giảng của sách
 * tiếng Hàn. Hơn 2.000 trang còn lại chỉ là ảnh scan, đưa vào chỉ làm Google
 * tốn công bò mà đánh giá site là mỏng nội dung.
 *
 * Không khai `changeFrequency`/`priority`: Google bỏ qua cả hai. `lastModified`
 * chỉ ghi ở chỗ có ngày thật (bài cẩm nang) — ngày bịa ra làm Google mất tin
 * vào cả sitemap.
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const paths: { path: string; lastModified?: string }[] = [];
  const add = (path: string, lastModified?: string) => paths.push({ path, lastModified });

  // Thư viện
  for (const lang of LANGUAGES) add(lang.slug ? `/${lang.slug}` : "/");
  for (const series of SERIES) add(`/sach/${series.id}`);
  for (const book of BOOKS) add(`/books/${book.id}`);

  // Ngữ pháp
  add("/ngu-phap");
  for (const g of getAllGrammar()) add(grammarHref(g.slug));

  // Bài học theo sách + trang đọc có nội dung chữ
  for (const book of BOOKS) {
    for (const lesson of bookLessons(book.id)) add(bookLessonHref(book.id, lesson.chapter.lesson));
    if (book.lang !== "ko") continue;
    const pages = new Set([...getTranslatedPages(book.id), ...getGrammarPages(book.id), ...getNotePages(book.id)]);
    for (const page of [...pages].sort((a, b) => a - b)) add(`/read/${book.id}/${page}`);
  }

  // Video
  add("/video");
  for (const v of getVideoLessons()) add(`/video/${v.id}`);

  // Luyện thi
  for (const level of EXAM_LEVELS) add(`/exam/${level.slug}`);
  for (const exam of listExams()) add(`/exam/${exam.id}`);

  // Cẩm nang
  for (const section of GUIDE_SECTIONS) {
    add(`/cam-nang/${section.id}`);
    for (const a of sectionArticles(section.id)) add(articleHref(section.id, a.slug), a.updated);
  }

  // IT
  add("/it");
  for (const course of listCourses()) {
    add(`/it/${course.id}`);
    for (const l of courseLessons(course)) add(lessonHref(course.id, l.slug));
  }

  add("/install");

  return paths.map(({ path, lastModified }) => ({
    url: absoluteUrl(path),
    ...(lastModified && { lastModified }),
  }));
}
