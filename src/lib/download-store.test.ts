import { beforeEach, describe, expect, it, vi } from "vitest";
import { MAX_OFFLINE_BOOKS, hasFreeSlot, useDownloadStore } from "./download-store";
import { BOOKS } from "@/lib/books";

/**
 * Luật "mỗi lần một cuốn" là phần dễ hỏng nhất của tính năng offline: hỏng
 * theo kiểu người dùng tải đầy máy rồi trình duyệt âm thầm xoá bớt.
 */
const downloadBook = vi.hoisted(() => vi.fn());
const deleteOfflineBook = vi.hoisted(() => vi.fn());
const listOfflineBooks = vi.hoisted(() => vi.fn());

vi.mock("./offline-books", () => ({
  downloadBook,
  deleteOfflineBook,
  listOfflineBooks,
}));

const [bookA, bookB] = BOOKS;
const state = () => useDownloadStore.getState();

beforeEach(() => {
  vi.clearAllMocks();
  listOfflineBooks.mockResolvedValue([]);
  downloadBook.mockResolvedValue({ saved: bookA.totalPages, total: bookA.totalPages });
  useDownloadStore.setState({
    offlineBooks: [],
    hasLoaded: true,
    active: null,
    error: null,
  });
});

describe("giới hạn số sách", () => {
  it("chỉ cho giữ một cuốn", () => {
    expect(MAX_OFFLINE_BOOKS).toBe(1);
    expect(hasFreeSlot([])).toBe(true);
    expect(hasFreeSlot(["step1"])).toBe(false);
  });

  it("từ chối tải cuốn thứ hai khi đã đầy", async () => {
    // Chặn ở store chứ không chỉ ở giao diện: hai tab cùng mở thì nút bên
    // tab kia vẫn đang ở trạng thái cũ.
    useDownloadStore.setState({ offlineBooks: [bookA.id] });
    await state().start(bookB);

    expect(downloadBook).not.toHaveBeenCalled();
  });

  it("xoá cuốn cũ xong thì tải được cuốn mới", async () => {
    useDownloadStore.setState({ offlineBooks: [bookA.id] });
    listOfflineBooks.mockResolvedValue([]);
    await state().remove(bookA.id);

    expect(deleteOfflineBook).toHaveBeenCalledWith(bookA.id);

    await state().start(bookB);
    expect(downloadBook).toHaveBeenCalledOnce();
  });

  it("không tải lại cuốn đã có trên máy", async () => {
    useDownloadStore.setState({ offlineBooks: [bookA.id] });
    await state().start(bookA);

    expect(downloadBook).not.toHaveBeenCalled();
  });

  it("không tải hai cuốn cùng lúc", async () => {
    useDownloadStore.setState({
      active: { bookId: bookA.id, progress: { done: 1, total: 10 } },
    });
    await state().start(bookB);

    expect(downloadBook).not.toHaveBeenCalled();
  });
});

describe("kết quả tải", () => {
  it("tải đủ thì không báo lỗi gì", async () => {
    await state().start(bookA);
    expect(state().error).toBeNull();
  });

  it("thiếu trang thì nói rõ thiếu bao nhiêu", async () => {
    downloadBook.mockResolvedValue({ saved: 200, total: 228 });
    await state().start(bookA);

    expect(state().error).toContain("200/228");
  });

  it("hỏng hẳn thì báo lỗi chứ không im lặng", async () => {
    downloadBook.mockRejectedValue(new Error("mạng rớt"));
    await state().start(bookA);

    expect(state().error).toBeTruthy();
    expect(state().active).toBeNull();
  });

  it("tải xong thì luôn dọn trạng thái đang tải", async () => {
    await state().start(bookA);
    expect(state().active).toBeNull();
  });
});

describe("huỷ giữa chừng", () => {
  it("dọn sạch phần đã tải dở", async () => {
    // Cuốn dở dang vẫn chiếm chỗ mà đọc offline lại hụt trang — tệ hơn là
    // không có gì.
    downloadBook.mockImplementation(async (_book, { signal }) => {
      signal.addEventListener("abort", () => {});
      state().cancel();
      return { saved: 5, total: 228 };
    });
    await state().start(bookA);

    expect(deleteOfflineBook).toHaveBeenCalledWith(bookA.id);
  });

  it("huỷ thì không báo lỗi thiếu trang", async () => {
    downloadBook.mockImplementation(async () => {
      state().cancel();
      return { saved: 5, total: 228 };
    });
    await state().start(bookA);

    expect(state().error).toBeNull();
  });
});
