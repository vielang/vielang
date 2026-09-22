import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render } from "@testing-library/react";
import { createRef } from "react";
import { PageViewer, type PageViewerHandle } from "./page-viewer";
import { BOOKS } from "@/lib/books";
import { TEST_ELEMENT_WIDTH } from "@/test-setup";
import { useReaderPrefsStore } from "@/lib/reader-prefs-store";

const book = BOOKS[0];

/**
 * Tự dựng sự kiện con trỏ thay vì dùng `fireEvent`, để đặt được toạ độ
 * và loại sự kiện đúng như trình duyệt bắn ra.
 */
function pointer(
  el: Element,
  type: "pointerdown" | "pointermove" | "pointerup" | "pointercancel",
  x: number,
  y: number
) {
  const ev = new window.PointerEvent(type, {
    bubbles: true,
    clientX: x,
    clientY: y,
    pointerId: 1,
  });
  act(() => {
    el.dispatchEvent(ev);
  });
}

const track = () => document.querySelector("[data-page-track]") as HTMLElement;
const surface = () => track().parentElement as HTMLElement;
/** Số px trong `translate3d(Xpx, 0, 0)`. */
const shift = () => Number(/translate3d\((-?[\d.]+)px/.exec(track().style.transform)?.[1]);

function view(over: Partial<React.ComponentProps<typeof PageViewer>> = {}) {
  const onTap = vi.fn();
  const onSwipePrev = vi.fn();
  const onSwipeNext = vi.fn();
  const ref = createRef<PageViewerHandle>();
  render(
    <PageViewer
      ref={ref}
      book={book}
      pages={[10]}
      prevPages={[9]}
      nextPages={[11]}
      onTap={onTap}
      onSwipePrev={onSwipePrev}
      onSwipeNext={onSwipeNext}
      {...over}
    />
  );
  return { onTap, onSwipePrev, onSwipeNext, ref };
}

/**
 * Kéo ngang thành NHIỀU nhịp rồi thả, giống ngón tay thật.
 *
 * Phải nhiều nhịp: nhịp đầu chỉ để quyết hướng và đặt mốc, quãng kéo tính
 * từ mốc đó trở đi. Gửi đúng một nhịp thì quãng luôn bằng 0.
 *
 * Nhích đồng hồ giả giữa các nhịp vì tốc độ vẩy đo bằng `performance.now()`.
 */
const STEPS = 6;
function drag(dx: number, { ms = 400, dy = 0, release = true } = {}) {
  const el = surface();
  pointer(el, "pointerdown", 200, 200);
  for (let i = 1; i <= STEPS; i++) {
    act(() => void vi.advanceTimersByTime(ms / STEPS));
    pointer(el, "pointermove", 200 + (dx * i) / STEPS, 200 + (dy * i) / STEPS);
  }
  if (release) pointer(el, "pointerup", 200 + dx, 200 + dy);
}

/** Quãng trang thật sự trôi được: tính từ mốc, tức là trừ đi nhịp đầu. */
const travelled = (dx: number) => dx - dx / STEPS;

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

/** Khung rộng 300px (xem test-setup), nên ngưỡng lật là 0.22 * 300 = 66px. */
describe("vuốt để lật trang", () => {
  it("kéo quá ngưỡng sang trái thì sang trang sau", () => {
    const { onSwipeNext } = view();
    drag(-120);

    // Route chỉ đổi SAU khi trang trượt xong, không phải ngay lúc thả tay.
    expect(onSwipeNext).not.toHaveBeenCalled();
    act(() => void vi.advanceTimersByTime(300));
    expect(onSwipeNext).toHaveBeenCalledTimes(1);
  });

  it("kéo quá ngưỡng sang phải thì về trang trước", () => {
    const { onSwipePrev } = view();
    drag(120);
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipePrev).toHaveBeenCalledTimes(1);
  });

  it("kéo chưa đủ thì trang trôi về chỗ cũ", () => {
    const { onSwipeNext, onSwipePrev } = view();
    drag(-40);
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).not.toHaveBeenCalled();
    expect(onSwipePrev).not.toHaveBeenCalled();
    expect(shift()).toBe(0);
  });

  it("vẩy nhanh một cái ngắn vẫn lật", () => {
    // Quãng chỉ 40px, chưa tới ngưỡng 66px, nhưng đi hết trong 40ms.
    const { onSwipeNext } = view();
    drag(-40, { ms: 40 });
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).toHaveBeenCalledTimes(1);
  });

  it("kéo đi rồi trôi ngược về thì KHÔNG lật", () => {
    // Đổi ý giữa chừng: kéo sang trái rồi đưa tay về gần chỗ cũ mới thả.
    // Lúc thả, trang vẫn đang ở bên trái mốc nhưng ngón tay đang đi sang
    // phải. Thiếu phép so dấu thì cú đi ngược đó vẫn tính là vẩy, và lật
    // sang đúng cái trang người ta vừa quyết định không sang.
    const { onSwipeNext, onSwipePrev } = view();
    const el = surface();
    pointer(el, "pointerdown", 200, 200);
    act(() => void vi.advanceTimersByTime(40));
    pointer(el, "pointermove", 160, 200);
    act(() => void vi.advanceTimersByTime(40));
    pointer(el, "pointermove", 60, 200);
    act(() => void vi.advanceTimersByTime(40));
    pointer(el, "pointermove", 130, 200);
    pointer(el, "pointerup", 132, 200);
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).not.toHaveBeenCalled();
    expect(onSwipePrev).not.toHaveBeenCalled();
  });

  it("kéo dọc thì không lật và cũng không tính là chạm", () => {
    const { onSwipeNext, onTap } = view();
    drag(-20, { dy: 120 });
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).not.toHaveBeenCalled();
    expect(onTap).not.toHaveBeenCalled();
  });

  it("một cú vuốt chỉ lật MỘT lần, dù vuốt tiếp", () => {
    // Route đổi xong thì component dựng lại; trước lúc đó mà vuốt thêm được
    // thì bắn ra hai lượt chuyển trang mà chỉ một lượt kịp có hiệu ứng.
    const { onSwipeNext } = view();
    drag(-120);
    drag(-120);
    act(() => void vi.advanceTimersByTime(600));

    expect(onSwipeNext).toHaveBeenCalledTimes(1);
  });
});

describe("hai đầu sách", () => {
  it("hết trang thì không lật, nhưng vẫn nhúc nhích cho biết là có nghe", () => {
    view({ nextPages: null });
    drag(-120, { release: false });

    // Kéo được 100px mà trang chỉ trôi 32px: đủ thấy là chạm đáy sách chứ
    // không phải ứng dụng treo.
    expect(shift()).toBeCloseTo(travelled(-120) * 0.32, 0);
  });

  it("hết trang thì thả tay ra cũng không chuyển", () => {
    const { onSwipeNext } = view({ nextPages: null });
    drag(-200);
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).not.toHaveBeenCalled();
  });
});

describe("chạm", () => {
  it("chạm nhẹ thì bật/tắt thanh công cụ (sau một nhịp chờ)", () => {
    const { onTap } = view();
    drag(3);
    // Chờ xem có phải bấm đúp không đã rồi mới bật/tắt.
    expect(onTap).not.toHaveBeenCalled();
    act(() => void vi.advanceTimersByTime(250));

    expect(onTap).toHaveBeenCalledTimes(1);
  });

  it("bấm đúp (để phóng to) thì KHÔNG bật/tắt thanh công cụ", () => {
    // Trước đây mỗi lần nhấc tay là một lần bật/tắt: bấm đúp làm thanh công
    // cụ nháy, có khi dừng ở trạng thái ẩn — kéo theo thanh phóng to biến
    // mất đúng lúc vừa phóng to.
    const { onTap } = view();
    drag(2, { ms: 40 });
    drag(2, { ms: 40 });
    act(() => void vi.advanceTimersByTime(500));

    expect(onTap).not.toHaveBeenCalled();
  });
});

describe("lật bằng nút và phím", () => {
  it("turn() lật y như vuốt", () => {
    const { ref, onSwipeNext } = view();
    act(() => {
      expect(ref.current?.turn(1)).toBe(true);
    });
    expect(onSwipeNext).not.toHaveBeenCalled();
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).toHaveBeenCalledTimes(1);
  });

  it("bấm nút hai cái liên tiếp cũng chỉ lật MỘT trang", () => {
    // Vuốt hai lần đã có chốt riêng ở nhánh thả tay; đây là đường KHÁC —
    // nút mũi tên và phím mũi tên đi thẳng vào `turn`, không qua nhánh đó.
    // Thiếu chốt ở đây thì bấm nhanh hai cái sẽ nhảy hai trang mà chỉ một
    // trang kịp có hiệu ứng.
    const { ref, onSwipeNext } = view();
    act(() => {
      expect(ref.current?.turn(1)).toBe(true);
      expect(ref.current?.turn(1)).toBe(false);
    });
    act(() => void vi.advanceTimersByTime(600));

    expect(onSwipeNext).toHaveBeenCalledTimes(1);
  });

  it("turn() báo false khi hết sách, để bên gọi tự xoay xở", () => {
    const { ref, onSwipeNext } = view({ nextPages: null });
    act(() => {
      expect(ref.current?.turn(1)).toBe(false);
    });
    act(() => void vi.advanceTimersByTime(300));

    expect(onSwipeNext).not.toHaveBeenCalled();
  });
});

describe("tôn trọng cài đặt bớt hiệu ứng", () => {
  it("bật giảm chuyển động thì chuyển trang ngay, không chờ trượt", () => {
    vi.stubGlobal("matchMedia", (q: string) => ({
      matches: q.includes("prefers-reduced-motion"),
      media: q,
      addEventListener() {},
      removeEventListener() {},
    }));
    const { ref, onSwipeNext } = view();
    act(() => void ref.current?.turn(1));

    expect(onSwipeNext).toHaveBeenCalledTimes(1);
    vi.unstubAllGlobals();
  });
});

describe("khung đo được", () => {
  it("dùng bề ngang thật của khung làm ngưỡng", () => {
    // Chốt lại giả định của cả nhóm test này: ngưỡng tính theo khung chứ
    // không phải một con số px cố định.
    expect(TEST_ELEMENT_WIDTH).toBe(300);
  });
});

describe("chạm trúng chấm trên trang", () => {
  it("chạm chấm ngữ pháp thì KHÔNG bật/tắt thanh công cụ", () => {
    // Chấm tự mở bong bóng của nó. Tính thêm là tap nữa thì một cái chạm vừa
    // mở bong bóng vừa ẩn thanh công cụ — người dùng bấm một lần mà hai thứ
    // nhảy. Bản trước chỉ né chấm dịch, bỏ sót chấm ngữ pháp.
    const { onTap } = view({ pages: [15], prevPages: [14], nextPages: [16] });
    const dot = document.querySelector("[data-grammar-point]");
    expect(dot).not.toBeNull();

    pointer(dot!, "pointerdown", 200, 200);
    pointer(dot!, "pointerup", 201, 201);

    expect(onTap).not.toHaveBeenCalled();
  });

  it("chạm vào chỗ trống trên trang thì vẫn bật/tắt như thường", () => {
    const { onTap } = view({ pages: [15], prevPages: [14], nextPages: [16] });
    drag(2);
    act(() => void vi.advanceTimersByTime(250));

    expect(onTap).toHaveBeenCalledTimes(1);
  });
});

describe("chấm đáp án theo công tắc của từng sách", () => {
  afterEach(() => useReaderPrefsStore.setState({ hiddenAnswerBooks: [] }));

  // step1 trang 18 có chấm đáp án (mục 듣기).
  const page18 = { pages: [18], prevPages: [17], nextPages: [19] };

  it("mặc định hiện chấm đáp án", () => {
    view(page18);

    expect(document.querySelector("[data-answer-key]")).not.toBeNull();
  });

  it("tắt đáp án của cuốn này thì trang không còn chấm nào", () => {
    useReaderPrefsStore.setState({ hiddenAnswerBooks: [book.id] });
    view(page18);

    expect(document.querySelector("[data-answer-key]")).toBeNull();
  });

  it("tắt cuốn khác thì cuốn này vẫn hiện", () => {
    useReaderPrefsStore.setState({ hiddenAnswerBooks: ["wb-step1"] });
    view(page18);

    expect(document.querySelector("[data-answer-key]")).not.toBeNull();
  });
});
