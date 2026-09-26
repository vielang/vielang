import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { GuideArticleView } from "./guide-article-view";
import type { GuideArticle, GuideSection } from "@/lib/guide";
import { useGuideChecklistStore } from "@/lib/guide-checklist-store";

/** Mục mẫu — cơ chế checklist không phụ thuộc mục nào có thật. */
const section: GuideSection = {
  id: "phong-van",
  title: "Phỏng vấn",
  description: "Chuẩn bị đi phỏng vấn IT",
};

/** Đúng dạng HTML mà bước build sinh ra cho checklist (xem interactiveChecklist). */
const CHECKLIST_HTML =
  '<h2 id="chuan-bi">Cần chuẩn bị</h2><ul>' +
  '<li class="check-item"><label><input type="checkbox" data-check="aaa"><span>CV bản mới nhất</span></label></li>' +
  '<li class="check-item"><label><input type="checkbox" data-check="bbb"><span>Portfolio</span></label></li>' +
  "</ul>";

function article(partial: Partial<GuideArticle> = {}): GuideArticle {
  return {
    slug: "vong-ky-thuat",
    title: "Vòng phỏng vấn kỹ thuật",
    summary: "Những gì cần mang theo",
    updated: "2026-09-25",
    facts: {},
    html: CHECKLIST_HTML,
    headings: [],
    checks: 2,
    ...partial,
  };
}

function box(label: string): HTMLInputElement {
  return screen.getByText(label).closest("label")!.querySelector("input")!;
}

beforeEach(() => {
  useGuideChecklistStore.setState({ checked: {}, hasHydrated: true });
});

describe("bài cẩm nang", () => {
  it("ghi ngày cập nhật ngay đầu bài", () => {
    render(<GuideArticleView section={section} article={article()} />);
    expect(screen.getByText("Cập nhật 25/09/2026.")).toBeTruthy();
  });

  it("tick checklist thì nhớ lại và đếm tiến độ", () => {
    render(<GuideArticleView section={section} article={article()} />);
    expect(screen.getByText(/đã xong/).textContent).toContain("0/2");

    fireEvent.click(box("CV bản mới nhất"));
    expect(useGuideChecklistStore.getState().checked["phong-van/vong-ky-thuat"]).toEqual(["aaa"]);
    expect(screen.getByText(/đã xong/).textContent).toContain("1/2");
  });

  it("mở lại bài thì các ô đã tick vẫn còn tick", () => {
    useGuideChecklistStore.setState({ checked: { "phong-van/vong-ky-thuat": ["bbb"] } });
    render(<GuideArticleView section={section} article={article()} />);
    expect(box("Portfolio").checked).toBe(true);
    expect(box("CV bản mới nhất").checked).toBe(false);
  });

  it("bỏ tick toàn bộ bằng một nút", () => {
    useGuideChecklistStore.setState({ checked: { "phong-van/vong-ky-thuat": ["aaa", "bbb"] } });
    render(<GuideArticleView section={section} article={article()} />);
    fireEvent.click(screen.getByRole("button", { name: "Bỏ tick toàn bộ checklist" }));
    expect(box("CV bản mới nhất").checked).toBe(false);
    expect(screen.getByText(/đã xong/).textContent).toContain("0/2");
  });

  it("bài không có checklist thì không hiện thanh tiến độ", () => {
    render(<GuideArticleView section={section} article={article({ html: "<p>x</p>", checks: 0 })} />);
    expect(screen.queryByText(/đã xong/)).toBeNull();
  });
});
