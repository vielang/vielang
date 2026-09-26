import { describe, expect, it } from "vitest";
import { BOOKS } from "@/lib/books";
import { getAnswerPages, getPageAnswers } from "@/lib/page-answers";
import { grammarHeading, skillOfSection } from "./skills";

describe("kỹ năng theo tên mục", () => {
  it.each([
    ["어휘 3", "어휘"],
    ["문법 · 동-고 싶다 2", "문법"],
    ["말하기와 듣기 1", "듣기"],
    ["듣기", "듣기"],
    ["읽기와 쓰기 2", "읽기"],
    ["읽기 (글 2)", "읽기"],
    ["쓰기 1", "쓰기"],
    ["복습 1 — 문법 [2~4]", "문법"],
  ])("%s → %s", (section, skill) => {
    expect(skillOfSection(section)).toBe(skill);
  });

  it("mục luyện lẫn lộn thì không xếp bừa", () => {
    // "어휘와 문법" luyện cả hai — xếp vào một bên là nói sai về người học.
    expect(skillOfSection("어휘와 문법 1 — 수①")).toBeNull();
    expect(skillOfSection("발음")).toBeNull();
    expect(skillOfSection("문화와 정보 — 한국의 화폐")).toBeNull();
  });

  it("mọi mục đáp án của mọi cuốn đều xếp được vào một kỹ năng", () => {
    // Chốt cho dữ liệu soạn sau này: đặt tên mục lạ thì mục đó lặng lẽ biến
    // khỏi thống kê năng lực.
    const unmapped: string[] = [];
    for (const book of BOOKS) {
      for (const page of getAnswerPages(book.id)) {
        for (const key of getPageAnswers(book.id, page)) {
          if (!skillOfSection(key.section)) unmapped.push(`${book.id} p${page}: ${key.section}`);
        }
      }
    }
    expect(unmapped).toEqual([]);
  });
});

describe("điểm ngữ pháp của sách bài tập", () => {
  it("lấy tên điểm ngữ pháp, bỏ số bài và chú thích cột", () => {
    expect(grammarHeading("문법 · 동-어서 1 (기본형 → -아서/어서)")).toBe("동-어서");
    expect(grammarHeading("문법 · 사동 ① 2 (이어서)")).toBe("사동 ①");
    expect(grammarHeading("어휘 1")).toBeNull();
  });

});
