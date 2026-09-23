import { describe, expect, it } from "vitest";
import {
  courseLessons,
  courseMinutes,
  courseProgressId,
  getCourse,
  getLesson,
  lessonHref,
  lessonNeighbours,
  lessonNumber,
  listCourses,
} from "@/lib/courses";

const courses = listCourses();

describe("dữ liệu khoá học IT", () => {
  it("có ít nhất một khoá, mỗi bài đủ nội dung", () => {
    expect(courses.length).toBeGreaterThan(0);
    for (const course of courses) {
      const lessons = courseLessons(course);
      expect(lessons.length).toBeGreaterThan(0);
      for (const lesson of lessons) {
        expect(lesson.slug).toMatch(/^[a-z0-9-]+\/[a-z0-9-]+$/);
        expect(lesson.title).not.toBe("");
        expect(lesson.minutes).toBeGreaterThan(0);
        expect(lesson.html).toContain("<");
      }
    }
  });

  it("slug của bài không trùng nhau trong cùng khoá", () => {
    for (const course of courses) {
      const slugs = courseLessons(course).map((l) => l.slug);
      expect(new Set(slugs).size).toBe(slugs.length);
    }
  });

  it("mục lục trong bài trỏ tới id có thật trong HTML", () => {
    for (const course of courses) {
      for (const lesson of courseLessons(course)) {
        for (const h of lesson.headings) {
          expect(lesson.html).toContain(`<h${h.level} id="${h.id}">`);
        }
      }
    }
  });

  it("tra được khoá và bài theo slug", () => {
    const course = courses[0];
    expect(getCourse(course.id)).toBe(course);
    expect(getCourse("không-có")).toBeUndefined();
    const first = courseLessons(course)[0];
    expect(getLesson(course, first.slug)).toBe(first);
    expect(getLesson(course, "không/có")).toBeUndefined();
  });
});

describe("điều hướng trong khoá", () => {
  const course = courses[0];
  const lessons = courseLessons(course);

  it("bài đầu không có bài trước, bài cuối không có bài sau", () => {
    expect(lessonNeighbours(course, lessons[0].slug).prev).toBeNull();
    expect(lessonNeighbours(course, lessons[lessons.length - 1].slug).next).toBeNull();
  });

  it("đi xuyên chương theo đúng thứ tự học", () => {
    for (let i = 0; i < lessons.length - 1; i++) {
      expect(lessonNeighbours(course, lessons[i].slug).next).toBe(lessons[i + 1]);
      expect(lessonNeighbours(course, lessons[i + 1].slug).prev).toBe(lessons[i]);
    }
  });

  it("đánh số bài đếm từ 1", () => {
    expect(lessonNumber(course, lessons[0].slug)).toBe(1);
    expect(lessonNumber(course, "không/có")).toBe(0);
  });

  it("link bài học nằm dưới /learn", () => {
    expect(lessonHref("csharp-core", "nen-tang/bien")).toBe("/learn/csharp-core/nen-tang/bien");
  });

  it("tổng thời gian đọc là tổng các bài", () => {
    expect(courseMinutes(course)).toBe(lessons.reduce((n, l) => n + l.minutes, 0));
  });
});

describe("courseProgressId", () => {
  it("có tiền tố riêng để không đụng id sách", () => {
    expect(courseProgressId("csharp-core")).toBe("it:csharp-core");
  });
});
