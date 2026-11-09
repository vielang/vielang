import type { MetadataRoute } from "next";
import { courseLessons, lessonHref, listCourses } from "@/lib/courses";
import { GUIDE_SECTIONS, articleHref, sectionArticles } from "@/lib/guide";
import { absoluteUrl } from "@/lib/seo";

/**
 * Sitemap sinh từ chính các bảng dữ liệu mà route dùng để dựng trang —
 * thêm khoá học, bài cẩm nang là tự có mặt ở đây.
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
