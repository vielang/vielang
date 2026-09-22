import { beforeEach, describe, expect, it } from "vitest";
import { BackupError, backupFileName, buildBackup, restoreBackup } from "./learning-backup";

beforeEach(() => localStorage.clear());

describe("sao lưu dữ liệu học", () => {
  it("xuất rồi nhập lại thì ra đúng dữ liệu cũ", () => {
    localStorage.setItem("kiip-progress-v1", JSON.stringify({ state: { books: { step1: 1 } } }));
    localStorage.setItem("kiip-activity-v1", JSON.stringify({ state: { days: {} } }));
    localStorage.setItem("unrelated", "x");
    const text = JSON.stringify(buildBackup(localStorage));

    localStorage.clear();
    expect(restoreBackup(localStorage, text)).toBe(2);
    expect(localStorage.getItem("kiip-progress-v1")).toContain("step1");
    // Chỉ sao lưu dữ liệu học của app, không vơ cả localStorage.
    expect(localStorage.getItem("unrelated")).toBeNull();
  });

  it("từ chối file không phải của app", () => {
    expect(() => restoreBackup(localStorage, "{}")).toThrow(BackupError);
    expect(() => restoreBackup(localStorage, "not json")).toThrow(BackupError);
  });

  it("từ chối file của phiên bản app mới hơn", () => {
    const text = JSON.stringify({ app: "vietopik-learning-backup", version: 99, data: {} });
    expect(() => restoreBackup(localStorage, text)).toThrow(/phiên bản app mới hơn/);
  });

  it("file hỏng một phần thì KHÔNG ghi phần nào", () => {
    // Ghi được nửa chừng thì tiến độ là bản mới, bài làm là bản cũ — lệch nhau
    // mà không ai biết.
    localStorage.setItem("kiip-progress-v1", "old");
    const text = JSON.stringify({
      app: "vietopik-learning-backup",
      version: 1,
      data: { "kiip-progress-v1": '{"new":true}', "kiip-quiz-v1": "{broken" },
    });

    expect(() => restoreBackup(localStorage, text)).toThrow(BackupError);
    expect(localStorage.getItem("kiip-progress-v1")).toBe("old");
  });

  it("khoá không có trong file thì giữ nguyên dữ liệu đang có", () => {
    localStorage.setItem("kiip-notes-v1", '{"keep":true}');
    const text = JSON.stringify({
      app: "vietopik-learning-backup",
      version: 1,
      data: { "kiip-progress-v1": "{}" },
    });
    restoreBackup(localStorage, text);

    expect(localStorage.getItem("kiip-notes-v1")).toBe('{"keep":true}');
  });

  it("tên file có ngày để cất nhiều bản không đè nhau", () => {
    expect(backupFileName(new Date(2025, 8, 3))).toBe("vietopik-du-lieu-hoc-2025-09-03.json");
  });
});
