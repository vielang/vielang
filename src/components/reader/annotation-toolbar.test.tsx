import { beforeEach, describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { AnnotationToolbar } from "./annotation-toolbar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { PEN_COLORS, useAnnotationStore } from "@/lib/annotation-store";

/**
 * Test cho thao tác KÉO và THU NHỎ thanh công cụ vẽ.
 *
 * Viết vì đúng hai việc này từng hỏng mà không ai thấy: `dragHandlers` chỉ
 * được gắn vào cái tay nắm rộng ~20px chứ không gắn vào thân thanh, nên túm
 * giữa thanh mà kéo thì không có gì xảy ra. Lỗi kiểu đó không có test nào
 * bắt được — kiểu dữ liệu vẫn đúng, lint vẫn sạch, build vẫn chạy.
 */
const BOOK = "step1";

function reset() {
  useAnnotationStore.setState({
    strokes: {},
    hasHydrated: true,
    quotaExceeded: false,
    active: true,
    peeking: false,
    tool: "pen",
    penColor: PEN_COLORS[0],
    size: 1,
    lastPage: null,
    toolbarPos: { x: 100, y: 400 },
    toolbarCollapsed: false,
  });
}

/** `TooltipProvider` thật nằm ở app/layout.tsx — dựng lại ở đây cho khớp. */
function toolbar() {
  render(
    <TooltipProvider>
      <AnnotationToolbar bookId={BOOK} pages={[10]} />
    </TooltipProvider>
  );
}

/** Thân thanh — vùng kéo chính, nhận diện qua tay nắm nằm trong nó. */
function bar(): HTMLElement {
  return screen.getByTestId("annotation-toolbar-bar");
}

/**
 * Bấm xuống trên `el` rồi kéo. Các sự kiện sau đó bắn trên `window` vì thanh
 * nghe ở đó — nhờ vậy kéo ra khỏi thanh vẫn theo được (xem `onPointerDown`).
 */
function drag(el: HTMLElement, dx: number, dy: number) {
  fireEvent.pointerDown(el, { pointerId: 1, clientX: 200, clientY: 500 });
  fireEvent.pointerMove(window, { pointerId: 1, clientX: 200 + dx, clientY: 500 + dy });
  fireEvent.pointerUp(window, { pointerId: 1, clientX: 200 + dx, clientY: 500 + dy });
}

const pos = () => useAnnotationStore.getState().toolbarPos!;

/** Nhường một nhịp cho hiệu ứng/timer của Radix chạy xong. */
const tick = () => new Promise((resolve) => setTimeout(resolve, 10));

beforeEach(reset);

describe("chỗ đứng mặc định", () => {
  it("dựng dọc sát mép phải, ngay trên nút audio", () => {
    useAnnotationStore.setState({ toolbarPos: null });
    toolbar();

    // jsdom: màn 1024×768; thanh đo được 300×44 (xem test-setup).
    // Nút audio: cao 48, cách đáy 84 -> mép trên của nó ở 768-48-84 = 636.
    expect(pos()).toEqual({
      x: 1024 - 300 - 8, // sát mép phải, chừa lề 8
      y: 636 - 10 - 44, // cách nút audio 10, rồi lùi lên đúng chiều cao thanh
    });
  });

  it("không đè lên nút audio", () => {
    useAnnotationStore.setState({ toolbarPos: null });
    toolbar();

    const audioTop = 768 - 48 - 84;
    expect(pos().y + 44).toBeLessThanOrEqual(audioTop);
  });
});

describe("kéo thanh công cụ", () => {
  it("kéo THÂN thanh thì thanh đi theo", () => {
    toolbar();
    drag(bar(), -60, -120);

    expect(pos()).toEqual({ x: 40, y: 280 });
  });

  it("kéo từ ngay TRÊN một cái nút cũng dời được thanh", () => {
    // Cột dọc gần như toàn là nút — bắt phải nhắm trúng tay nắm 16px mới kéo
    // được thì coi như không kéo được.
    toolbar();
    drag(screen.getByLabelText(/Hoàn tác/), -60, -120);

    expect(pos()).toEqual({ x: 40, y: 280 });
  });

  it("chạm nhẹ lên nút thì nút vẫn ăn, thanh đứng yên", () => {
    useAnnotationStore.setState({
      strokes: { "step1:10": [{ id: "a", tool: "pen", color: "#000", width: 0.004, points: [0, 0] }] },
    });
    toolbar();
    const before = pos();
    const undo = screen.getByLabelText(/Hoàn tác/);

    // Nhúc nhích 2px — dưới ngưỡng, vẫn tính là bấm.
    fireEvent.pointerDown(undo, { pointerId: 1, clientX: 200, clientY: 500 });
    fireEvent.pointerMove(window, { pointerId: 1, clientX: 202, clientY: 500 });
    fireEvent.pointerUp(window, { pointerId: 1, clientX: 202, clientY: 500 });
    fireEvent.click(undo);

    expect(pos()).toEqual(before);
    expect(useAnnotationStore.getState().strokes["step1:10"]).toBeUndefined();
  });

  it("trên điện thoại: kéo xong thì lần chạm NGAY SAU phải ăn luôn", async () => {
    // Chuột kéo xong vẫn sinh ra một cú `click`, ngón tay thì không — nên cờ
    // "nuốt click" từng nằm lại và ăn mất lần chạm kế tiếp. Dựng đúng cảnh
    // cảm ứng: kéo mà KHÔNG có `click` ở cuối.
    toolbar();
    fireEvent.pointerDown(bar(), { pointerId: 1, pointerType: "touch", clientX: 200, clientY: 500 });
    fireEvent.pointerMove(window, { pointerId: 1, pointerType: "touch", clientX: 260, clientY: 560 });
    fireEvent.pointerUp(window, { pointerId: 1, pointerType: "touch", clientX: 260, clientY: 560 });
    await tick();

    const exit = screen.getByLabelText(/Thoát chế độ vẽ/);
    fireEvent.pointerDown(exit, { pointerId: 2, pointerType: "touch", clientX: 260, clientY: 560 });
    fireEvent.pointerUp(window, { pointerId: 2, pointerType: "touch", clientX: 260, clientY: 560 });
    fireEvent.click(exit);

    expect(useAnnotationStore.getState().active).toBe(false);
  });

  it("kéo rồi nhả trên một cái nút thì KHÔNG kích hoạt nút đó", () => {
    toolbar();
    const exit = screen.getByLabelText(/Thoát chế độ vẽ/);

    drag(exit, 0, -100);
    fireEvent.click(exit);

    // Kéo xong mà vô tình thoát luôn chế độ vẽ thì rất khó chịu.
    expect(useAnnotationStore.getState().active).toBe(true);
  });

  it("không cho kéo thanh ra ngoài mép màn hình", () => {
    toolbar();
    drag(bar(), -9999, -9999);

    // MARGIN = 8 ở cả hai trục.
    expect(pos()).toEqual({ x: 8, y: 8 });
  });

  it("bỏ qua ngón khác đang chạm dở", () => {
    toolbar();
    const before = pos();

    fireEvent.pointerDown(bar(), { pointerId: 1, clientX: 200, clientY: 500 });
    fireEvent.pointerMove(window, { pointerId: 2, clientX: 400, clientY: 500 });

    expect(pos()).toEqual(before);
  });

  it("kéo ra khỏi thanh vẫn theo được tay", () => {
    toolbar();
    fireEvent.pointerDown(bar(), { pointerId: 1, clientX: 200, clientY: 500 });
    // Con trỏ đã rời hẳn thanh — sự kiện chỉ còn tới `window`.
    fireEvent.pointerMove(window, { pointerId: 1, clientX: 600, clientY: 300 });

    expect(pos()).toEqual({ x: 500, y: 200 });
  });
});

describe("thu nhỏ", () => {
  it("bấm nút thu nhỏ thì thanh gọn lại thành một nút", () => {
    toolbar();
    fireEvent.click(screen.getByLabelText("Thu nhỏ thanh công cụ"));

    expect(useAnnotationStore.getState().toolbarCollapsed).toBe(true);
    expect(screen.getByLabelText("Mở lại thanh công cụ vẽ")).toBeTruthy();
  });

  it("nhấn nút đã thu nhỏ thì bung ra lại", () => {
    useAnnotationStore.setState({ toolbarCollapsed: true });
    toolbar();
    const pill = screen.getByLabelText("Mở lại thanh công cụ vẽ");

    fireEvent.pointerDown(pill, { pointerId: 1, clientX: 200, clientY: 500 });
    fireEvent.pointerUp(pill, { pointerId: 1, clientX: 200, clientY: 500 });

    expect(useAnnotationStore.getState().toolbarCollapsed).toBe(false);
  });

  it("KÉO nút đã thu nhỏ thì chỉ đổi chỗ, không bung ra", () => {
    useAnnotationStore.setState({ toolbarCollapsed: true });
    toolbar();
    drag(screen.getByLabelText("Mở lại thanh công cụ vẽ"), 50, 40);

    expect(useAnnotationStore.getState().toolbarCollapsed).toBe(true);
    expect(pos()).toEqual({ x: 150, y: 440 });
  });
});

describe("bảng chọn", () => {
  it("mở bảng công cụ rồi chọn xong thì tự đóng", () => {
    toolbar();
    fireEvent.click(screen.getByLabelText(/^Công cụ:/));
    fireEvent.click(screen.getByLabelText("Tẩy"));

    expect(useAnnotationStore.getState().tool).toBe("eraser");
    expect(screen.queryByLabelText("Tẩy")).toBeNull();
  });

  it("chọn màu thì để bảng mở, còn chỉnh tiếp cỡ nét", () => {
    toolbar();
    fireEvent.click(screen.getByLabelText(/^Công cụ:/));
    fireEvent.click(screen.getByLabelText(`Màu ${PEN_COLORS[2]}`));

    expect(useAnnotationStore.getState().penColor).toBe(PEN_COLORS[2]);
    expect(screen.getByLabelText("Cỡ 3")).toBeTruthy();
  });

  it("bấm ra ngoài thì đóng bảng", async () => {
    toolbar();
    fireEvent.click(screen.getByLabelText(/^Công cụ:/));
    // Radix chỉ gắn listener "bấm ra ngoài" ở tick sau khi mở, để chính cú
    // nhấn vừa mở popover không đóng luôn nó. Và nó chốt ở `click` chứ không
    // ở `pointerdown`, để không đóng nhầm khi người dùng đang bôi chọn chữ.
    await tick();
    fireEvent.pointerDown(document.body);
    fireEvent.click(document.body);
    await tick();

    expect(screen.queryByLabelText("Tẩy")).toBeNull();
  });

  it("xem ảnh gốc nằm trong bảng thêm", () => {
    toolbar();
    fireEvent.click(screen.getByLabelText("Ảnh gốc và khôi phục"));
    fireEvent.click(screen.getByText("Xem ảnh gốc"));

    expect(useAnnotationStore.getState().peeking).toBe(true);
  });
});
