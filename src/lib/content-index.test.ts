import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Nội dung soạn tay (ghi chú, bài tập, ngữ pháp, bản dịch, đáp án) đi vào app
 * qua MỘT file gộp `content/<loại>/index.json` (xem `buildIndex` trong
 * build-content). Trước đây mỗi file lib tự liệt kê import từng sách: thêm
 * nội dung cho sách khác thì build vẫn qua mà app không hiện gì.
 */
const KINDS = ["notes", "quiz", "translate", "grammar", "answers"];

describe("file nội dung gộp", () => {
  for (const kind of KINDS) {
    it(`${kind}: sách nào có nội dung soạn tay cũng có mặt trong index.json`, () => {
      const root = path.resolve(process.cwd(), "content", kind);
      const index = JSON.parse(readFileSync(path.join(root, "index.json"), "utf8")) as Record<
        string,
        unknown
      >;
      const withContent = readdirSync(root, { withFileTypes: true })
        .filter((d) => d.isDirectory() && readdirSync(path.join(root, d.name)).length > 0)
        .map((d) => d.name);
      for (const bookId of withContent) {
        expect(Object.keys(index), `${kind}/${bookId}`).toContain(bookId);
      }
    });
  }

  it("lib chỉ import file gộp, không import từng sách", () => {
    const lib = path.resolve(process.cwd(), "src", "lib");
    for (const file of ["notes.ts", "quiz.ts", "page-grammar.ts", "page-translation.ts", "page-answers.ts"]) {
      expect(existsSync(path.join(lib, file))).toBe(true);
      const src = readFileSync(path.join(lib, file), "utf8");
      expect(src, file).not.toMatch(/content\/\w+\/(?!index)[\w-]+\.json/);
    }
  });
});
