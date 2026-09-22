/**
 * Giữ một store zustand lưu localStorage đồng bộ giữa các tab.
 *
 * Vì sao cần: mỗi tab nạp store vào bộ nhớ MỘT lần lúc mở, rồi mỗi lần ghi
 * là ghi đè CẢ khối xuống localStorage. Hai tab cùng mở (vd đọc sách ở tab
 * này, tra bài ở tab kia) thì tab ghi sau lặng lẽ xoá những gì tab kia vừa
 * ghi — trang đã đọc, bài làm, phút học biến mất mà không ai biết vì sao.
 *
 * Sự kiện `storage` chỉ bắn ở các tab KHÁC tab vừa ghi, nên nghe nó rồi nạp
 * lại là mỗi tab luôn ghi tiếp trên bản mới nhất.
 */
interface PersistedStore<T> {
  getInitialState: () => T;
  setState: (state: T, replace: true) => void;
  persist: {
    rehydrate: () => Promise<void> | void;
    getOptions: () => { name?: string };
  };
}

export function syncAcrossTabs<T>(store: PersistedStore<T>): void {
  if (typeof window === "undefined") return;
  const name = store.persist.getOptions().name;
  if (!name) return;
  window.addEventListener("storage", (e) => {
    // `key === null`: tab kia xoá sạch localStorage.
    if (e.key !== name && e.key !== null) return;
    // Dữ liệu bị XOÁ (không phải sửa): phải về trạng thái ban đầu trước khi
    // nạp. `rehydrate` mà không thấy gì trong kho sẽ ghi trả bản đang giữ
    // trong bộ nhớ xuống — tức là khôi phục đúng thứ người dùng vừa cố ý xoá.
    if (e.newValue === null) store.setState(store.getInitialState(), true);
    void store.persist.rehydrate();
  });
}
