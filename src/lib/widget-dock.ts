"use client";

import { DRAG_MARGIN, type DragPos } from "@/lib/use-draggable";

/**
 * Chỗ đứng mặc định của mọi widget nổi trong trang đọc.
 *
 * Trang đọc có tới bốn thứ nổi cùng lúc — nút audio, thanh công cụ vẽ, bảng
 * ghi âm, panel bài giảng — và trước đây mỗi cái tự tính chỗ của mình bằng
 * hằng số riêng. Kết quả là chúng chồng lên nhau ngay từ lúc mở: bảng ghi âm
 * canh giữa đáy đè lên nút audio ở góc phải, panel bài giảng cao tới mức
 * chạm cả hai.
 *
 * Gom hết vào một chỗ để nhìn ra được bố cục tổng thể, và để widget thứ năm
 * sau này biết chỗ nào còn trống. Người dùng kéo đi đâu là quyền của họ —
 * đây chỉ là chỗ ĐẦU TIÊN chúng xuất hiện.
 *
 * Bố cục: cột PHẢI dành cho công cụ nhỏ (nút audio dưới cùng, thanh vẽ ngay
 * trên, thanh phóng to ở phần TRÊN của cột — chỉ màn rộng), bên TRÁI dành cho
 * hai khung làm việc lớn (panel bài giảng ở trên, bảng ghi âm ở dưới).
 *
 * Bảo đảm được: mấy nút nhỏ bên phải không bao giờ bị che, vì đó chính là
 * thứ cần bấm trong lúc một panel đang mở.
 *
 * KHÔNG bảo đảm: panel bài giảng và bảng ghi âm có thể chồng nhau. Màn điện
 * thoại không đủ chỗ cho hai khung lớn đứng cạnh nhau, mà ép cho vừa thì cả
 * hai cùng chật. Hiếm khi mở cùng lúc, và cái nào cũng kéo đi hoặc đóng lại
 * được.
 */

/** Khoảng hở giữa hai widget xếp cạnh nhau. */
const GAP = 10;

/** Đường kính nút audio lúc thu nhỏ. */
export const AUDIO_WIDGET_SIZE = 48;

/**
 * Nút audio nằm cao hơn đáy chừng này — đủ để không đè lên thanh điều hướng
 * trang (~64px) của trang đọc.
 */
const AUDIO_BOTTOM_OFFSET = 84;

/** Thanh công cụ trên cùng của trang đọc. */
const TOP_BAR = 64;

/**
 * Thanh phóng to: sát mép phải, ngay dưới thanh công cụ trên. Không kéo được.
 *
 * Ở phần TRÊN của cột phải chứ không canh giữa: nửa dưới cột đã có thanh vẽ
 * (dựng dọc ngay trên nút audio), canh giữa thì bật chế độ vẽ là hai thanh
 * đè lên nhau.
 */
export const ZOOM_BAR_TOP = TOP_BAR + GAP;

/** Mép TRÊN của nút audio khi nó còn ở chỗ mặc định. */
export function audioTop(): number {
  return window.innerHeight - AUDIO_WIDGET_SIZE - AUDIO_BOTTOM_OFFSET;
}

/** Nút audio: góc dưới phải. Nhỏ nhất nên nhận chỗ đắt nhất. */
export function audioAnchor(): DragPos {
  return {
    x: window.innerWidth - AUDIO_WIDGET_SIZE - DRAG_MARGIN,
    y: audioTop(),
  };
}

/**
 * Thanh công cụ vẽ: dựng dọc sát mép phải, ngay trên nút audio.
 *
 * Nằm dọc ở mép chứ không nằm ngang dưới đáy vì trang sách cao hơn rộng —
 * một cột dọc bên hông chỉ ăn mất chừng 45px bề ngang, còn thanh ngang dưới
 * đáy thì cắt mất một khoảng chiều cao, đúng chiều đang thiếu.
 */
export function drawToolbarAnchor(width: number, height: number): DragPos {
  return {
    x: window.innerWidth - width - DRAG_MARGIN,
    y: audioTop() - GAP - height,
  };
}

/**
 * Bảng ghi âm: góc dưới TRÁI.
 *
 * Trước đây canh giữa đáy, mà bảng rộng 288px thì trên màn điện thoại nó
 * với sang tận góc phải và đè lên nút audio. Góc trái đang trống.
 */
export function recorderAnchor(width: number, height: number): DragPos {
  return {
    x: DRAG_MARGIN,
    y: Math.max(DRAG_MARGIN, window.innerHeight - height - AUDIO_BOTTOM_OFFSET),
  };
}

/**
 * Thanh nghe tự động: giữa đáy. Thanh lật trang đang hiện thì đứng ngay trên
 * nó; ẩn thì tụt xuống sát đáy — lọt dưới mép bảng ghi âm (góc dưới trái),
 * đỡ đè lên nhau trên màn điện thoại. Chỉ là chỗ mặc định: người dùng kéo đi
 * rồi thì thôi không bám theo nữa.
 */
export function autoplayBarAnchor(
  width: number,
  height: number,
  toolbarVisible: boolean
): DragPos {
  const bottom = toolbarVisible ? TOOLBAR_BOTTOM + GAP : AUTOPLAY_BOTTOM_HIDDEN;
  return {
    x: Math.round((window.innerWidth - width) / 2),
    y: window.innerHeight - height - bottom,
  };
}

/** Thanh lật trang dưới đáy trang đọc. */
const TOOLBAR_BOTTOM = 64;

/** Cách đáy khi thanh lật trang ẩn — chừa vạch Home của iPhone. */
const AUTOPLAY_BOTTOM_HIDDEN = 20;

/**
 * Bề ngang cột công cụ bên phải phải chừa ra — thanh vẽ rộng chừng 45px.
 * Panel nào mở ra cũng không được lấn vào đây.
 */
const TOOL_COLUMN = 55;

/** Panel bài giảng: mép TRÁI, ngay dưới thanh công cụ trên. */
export function notePanelAnchor(): DragPos {
  return { x: DRAG_MARGIN, y: TOP_BAR };
}

/**
 * Cỡ mặc định của panel bài giảng.
 *
 * Neo bên trái và chừa cột công cụ bên phải, thay vì bám mép phải như trước.
 * Trên màn điện thoại panel rộng gần bằng cả màn, bám phải là nó trùm luôn
 * nút audio lẫn thanh vẽ — mở bài giảng ra là mất đường bấm hai thứ kia.
 *
 * Cao vừa đủ để DỪNG TRƯỚC nút audio; trước đây chừa 140px dưới đáy, tức
 * vẫn thò quá mép trên nút audio chừng 56px.
 */
export function notePanelSize(): { width: number; height: number } {
  const width = Math.min(400, window.innerWidth - DRAG_MARGIN * 2 - TOOL_COLUMN);
  const available = audioTop() - GAP - TOP_BAR;
  return {
    width: Math.max(260, width),
    height: Math.max(220, Math.min(560, available)),
  };
}
