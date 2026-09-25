import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { GuideArticleView } from "./guide-article-view";
import { GUIDE_SECTIONS, type GuideArticle } from "@/lib/guide";
import { useGuideChecklistStore } from "@/lib/guide-checklist-store";

const visa = GUIDE_SECTIONS.find((s) => s.id === "visa")!;
const truong = GUIDE_SECTIONS.find((s) => s.id === "truong")!;

/** Đúng dạng HTML mà bước build sinh ra cho checklist (xem interactiveChecklist). */
const CHECKLIST_HTML =
  '<h2 id="ho-so">Hồ sơ cần chuẩn bị</h2><ul>' +
  '<li class="check-item"><label><input type="checkbox" data-check="aaa"><span>Hộ chiếu</span></label></li>' +
  '<li class="check-item"><label><input type="checkbox" data-check="bbb"><span>Ảnh 3.5x4.5</span></label></li>' +
  "</ul>";

function article(partial: Partial<GuideArticle> = {}): GuideArticle {
  return {
    slug: "d-2-du-hoc",
    title: "Visa D-2",
    summary: "Du học đại học",
    updated: "2026-09-25",
    facts: { code: "D-2" },
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
  it("ghi ngày cập nhật và nhắc kiểm tra lại ngay đầu bài", () => {
    render(<GuideArticleView section={visa} article={article()} />);
    expect(screen.getByText("Cập nhật 25/09/2026")).toBeTruthy();
    expect(screen.getByText(/Kiểm tra lại tại HiKorea/)).toBeTruthy();
  });

  it("tick giấy tờ thì nhớ lại và đếm tiến độ", () => {
    render(<GuideArticleView section={visa} article={article()} />);
    expect(screen.getByText(/đã chuẩn bị/).textContent).toContain("0/2");

    fireEvent.click(box("Hộ chiếu"));
    expect(useGuideChecklistStore.getState().checked["visa/d-2-du-hoc"]).toEqual(["aaa"]);
    expect(screen.getByText(/đã chuẩn bị/).textContent).toContain("1/2");
  });

  it("mở lại bài thì các ô đã tick vẫn còn tick", () => {
    useGuideChecklistStore.setState({ checked: { "visa/d-2-du-hoc": ["bbb"] } });
    render(<GuideArticleView section={visa} article={article()} />);
    expect(box("Ảnh 3.5x4.5").checked).toBe(true);
    expect(box("Hộ chiếu").checked).toBe(false);
  });

  it("bỏ tick toàn bộ bằng một nút", () => {
    useGuideChecklistStore.setState({ checked: { "visa/d-2-du-hoc": ["aaa", "bbb"] } });
    render(<GuideArticleView section={visa} article={article()} />);
    fireEvent.click(screen.getByRole("button", { name: "Bỏ tick toàn bộ checklist" }));
    expect(box("Hộ chiếu").checked).toBe(false);
    expect(screen.getByText(/đã chuẩn bị/).textContent).toContain("0/2");
  });

  it("bài không có checklist thì không hiện thanh tiến độ hồ sơ", () => {
    render(<GuideArticleView section={visa} article={article({ html: "<p>x</p>", checks: 0 })} />);
    expect(screen.queryByText(/đã chuẩn bị/)).toBeNull();
  });

  it("bài trường hiện bảng thông số, TOPIK 0 nói bằng chữ", () => {
    render(
      <GuideArticleView
        section={truong}
        article={article({
          facts: { nameKo: "고려대학교", city: "Seoul", topik: "0", website: "https://www.korea.ac.kr/" },
          html: "<p>x</p>",
          checks: 0,
        })}
      />
    );
    expect(screen.getByText("Thành phố").nextElementSibling?.textContent).toBe("Seoul");
    expect(screen.getByText("TOPIK tối thiểu").nextElementSibling?.textContent).toBe("Không yêu cầu");
    expect(screen.getByText("www.korea.ac.kr").getAttribute("target")).toBe("_blank");
  });
});
