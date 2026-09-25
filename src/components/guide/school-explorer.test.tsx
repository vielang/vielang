import { describe, expect, it } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { SchoolExplorer } from "./school-explorer";
import type { GuideArticle } from "@/lib/guide";

function school(slug: string, facts: Record<string, string>): GuideArticle {
  return {
    slug,
    title: `Đại học ${slug}`,
    summary: "",
    updated: "2026-09-25",
    facts,
    html: "",
    headings: [],
    checks: 0,
  };
}

const SCHOOLS = [
  school("A", { city: "Seoul", kind: "Tư thục", topik: "4", tuition: "5,8–7,8 triệu won/học kỳ" }),
  school("B", { city: "Suwon (Gyeonggi)", kind: "Tư thục", topik: "Cấp 3 (…)", tuition: "4,1–5,4 triệu" }),
  school("C", { city: "Daegu", kind: "Quốc lập", topik: "3", tuition: "1,9–2,7 triệu" }),
  school("D", { city: "Busan", kind: "Tư thục", topik: "Không bắt buộc (…)", tuition: "3,3–4,7 triệu" }),
];

function names(): string[] {
  return within(screen.getByRole("list"))
    .getAllByRole("link")
    .map((a) => a.textContent!.match(/Đại học (\w)/)![1]);
}

function pick(label: string, value: string) {
  fireEvent.change(screen.getByLabelText(label), { target: { value } });
}

describe("bộ lọc trường", () => {
  it("mặc định hiện tất cả theo thứ tự sẵn có", () => {
    render(<SchoolExplorer articles={SCHOOLS} />);
    expect(names()).toEqual(["A", "B", "C", "D"]);
  });

  it("lọc theo vùng", () => {
    render(<SchoolExplorer articles={SCHOOLS} />);
    pick("Vùng", "Tỉnh khác");
    expect(names()).toEqual(["C", "D"]);
    expect(screen.getByText("2/4 trường khớp bộ lọc")).toBeTruthy();
  });

  it("lọc theo TOPIK đang có: chỉ còn trường yêu cầu không cao hơn", () => {
    render(<SchoolExplorer articles={SCHOOLS} />);
    pick("TOPIK hiện có", "0");
    expect(names()).toEqual(["D"]);
    pick("TOPIK hiện có", "3");
    expect(names()).toEqual(["B", "C", "D"]);
  });

  it("lọc theo loại trường", () => {
    render(<SchoolExplorer articles={SCHOOLS} />);
    pick("Loại trường", "Quốc lập");
    expect(names()).toEqual(["C"]);
  });

  it("sắp theo học phí thấp trước", () => {
    render(<SchoolExplorer articles={SCHOOLS} />);
    pick("Sắp xếp", "tuition");
    expect(names()).toEqual(["C", "D", "B", "A"]);
  });

  it("không trường nào khớp thì nói rõ và cho bỏ lọc một chạm", () => {
    render(<SchoolExplorer articles={SCHOOLS} />);
    pick("Vùng", "Seoul");
    pick("Loại trường", "Quốc lập");
    expect(screen.getByText(/Không có trường nào khớp/)).toBeTruthy();
    fireEvent.click(screen.getAllByRole("button", { name: "Bỏ lọc" })[0]);
    expect(names()).toEqual(["A", "B", "C", "D"]);
  });
});
