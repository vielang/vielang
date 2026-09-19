import { getStroke } from "perfect-freehand";
import type { Stroke } from "@/lib/annotation-store";

/**
 * Hình học cho nét vẽ trên trang sách. Tách khỏi component để test được —
 * đây là phần dễ sai nhất: toạ độ lưu theo tỉ lệ nhưng trang sách không
 * vuông, nên đo khoảng cách phải quy về cùng một đơn vị.
 *
 * "Đơn vị bề rộng trang": chiều rộng trang = 1, chiều cao = 1/aspectRatio
 * (aspectRatio = rộng/cao). Một điểm lưu dạng (x, y) ∈ [0,1]² nằm ở
 * (x, y / aspectRatio) trong hệ này.
 */

/** Giữ 4 chữ số thập phân — đủ mịn hơn 1 pixel trên màn 4K, mà JSON ngắn đi quá nửa. */
export function quantize(points: number[]): number[] {
  return points.map((v) => Math.round(v * 1e4) / 1e4);
}

/** Lực bút chỉ cần 2 chữ số — 0.01 lực không ai nhìn ra, mà lưu thì tốn gấp đôi. */
export function quantizePressure(pressures: number[]): number[] {
  return pressures.map((v) => Math.round(v * 100) / 100);
}

/**
 * Tham số dựng nét bút mực. `thinning` là mức lực bút ăn vào bề dày — 0.55
 * cho nét đầu mảnh cuối đậm rõ ràng mà chưa tới mức đứt đoạn khi viết nhanh.
 * `streamline` làm mượt rung tay, quan trọng nhất lúc vẽ bằng ngón.
 */
const PEN_STROKE_OPTIONS = {
  thinning: 0.55,
  smoothing: 0.5,
  streamline: 0.4,
  last: true,
} as const;

/**
 * Viền ngoài của nét bút mực, dạng đa giác khép kín để TÔ chứ không phải kẻ.
 *
 * Vì sao không dùng `strokePath` như bút dạ quang: nét kẻ có bề dày cố định,
 * còn bút thật thì đầu nét mảnh, giữa nét đậm, nhấn mạnh thì phình ra. Muốn
 * bề dày đổi dọc theo nét thì phải tự dựng viền — đó đúng là việc của
 * `perfect-freehand` (cũng là thư viện Excalidraw dùng bên trong).
 *
 * `pressures` chỉ có khi người dùng vẽ bằng bút cảm ứng thật. Vẽ bằng chuột
 * hay ngón tay thì không có lực, để thư viện suy ra bề dày theo tốc độ —
 * viết nhanh thì mảnh, chậm thì đậm, vẫn ra dáng chữ viết tay.
 */
export function penOutlinePath(
  points: number[],
  pressures: number[] | undefined,
  sx: number,
  sy: number,
  size: number
): string {
  const n = Math.floor(points.length / 2);
  if (n === 0) return "";

  const input: number[][] = [];
  for (let i = 0; i < n; i++) {
    input.push([points[i * 2] * sx, points[i * 2 + 1] * sy, pressures?.[i] ?? 0.5]);
  }

  return outlineToPath(
    getStroke(input, {
      ...PEN_STROKE_OPTIONS,
      size,
      simulatePressure: pressures === undefined || pressures.length !== n,
    })
  );
}

/**
 * Đa giác viền -> lệnh `d`, nối các đỉnh bằng đường bậc hai đi qua trung
 * điểm để viền không thành hình đa giác gãy góc.
 */
export function outlineToPath(outline: number[][]): string {
  if (outline.length === 0) return "";
  const parts: (string | number)[] = ["M", outline[0][0], outline[0][1], "Q"];
  for (let i = 0; i < outline.length; i++) {
    const [x0, y0] = outline[i];
    const [x1, y1] = outline[(i + 1) % outline.length];
    parts.push(x0, y0, (x0 + x1) / 2, (y0 + y1) / 2);
  }
  parts.push("Z");
  return parts.join(" ");
}

/**
 * Chuỗi lệnh `d` của <path>, làm mượt bằng đường bậc hai đi qua trung điểm:
 * mỗi điểm thô thành điểm điều khiển, nên nét không gãy khúc ở tần số lấy
 * mẫu của trình duyệt.
 *
 * `sx`/`sy` đưa toạ độ tỉ lệ về hệ toạ độ của viewBox.
 */
export function strokePath(points: number[], sx: number, sy: number): string {
  const n = Math.floor(points.length / 2);
  if (n === 0) return "";

  const px = (i: number) => points[i * 2] * sx;
  const py = (i: number) => points[i * 2 + 1] * sy;

  // Chạm rồi nhả tại chỗ: `l 0 0` + linecap tròn cho ra một chấm tròn, chứ
  // <path> chỉ có mỗi lệnh M thì không vẽ gì cả.
  if (n === 1) return `M ${px(0)} ${py(0)} l 0 0`;

  let d = `M ${px(0)} ${py(0)}`;
  for (let i = 1; i < n - 1; i++) {
    d += ` Q ${px(i)} ${py(i)} ${(px(i) + px(i + 1)) / 2} ${(py(i) + py(i + 1)) / 2}`;
  }
  return `${d} L ${px(n - 1)} ${py(n - 1)}`;
}

/** Bình phương khoảng cách từ P tới đoạn AB. Bình phương để khỏi gọi sqrt. */
function distanceToSegmentSq(
  px: number,
  py: number,
  ax: number,
  ay: number,
  bx: number,
  by: number
): number {
  const dx = bx - ax;
  const dy = by - ay;
  const lenSq = dx * dx + dy * dy;
  // A trùng B (nét một chấm) — đo thẳng tới điểm.
  const t = lenSq === 0 ? 0 : Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / lenSq));
  const cx = ax + t * dx;
  const cy = ay + t * dy;
  return (px - cx) ** 2 + (py - cy) ** 2;
}

/**
 * Đầu tẩy (tâm `x`,`y`, bán kính `radius`, đều theo đơn vị bề rộng trang) có
 * chạm vào nét này không. Chạm là xoá cả nét — tẩy gặm từng mẩu nghe thì
 * hay nhưng trên ảnh scan hầu như chỉ để xoá hẳn một vòng khoanh vẽ hỏng.
 */
export function strokeHit(
  stroke: Stroke,
  x: number,
  y: number,
  aspectRatio: number,
  radius: number
): boolean {
  const pts = stroke.points;
  const n = Math.floor(pts.length / 2);
  if (n === 0) return false;

  const reach = radius + stroke.width / 2;
  const reachSq = reach * reach;
  const ux = (i: number) => pts[i * 2];
  const uy = (i: number) => pts[i * 2 + 1] / aspectRatio;
  const py = y / aspectRatio;

  if (n === 1) return (x - ux(0)) ** 2 + (py - uy(0)) ** 2 <= reachSq;

  for (let i = 0; i < n - 1; i++) {
    if (distanceToSegmentSq(x, py, ux(i), uy(i), ux(i + 1), uy(i + 1)) <= reachSq) return true;
  }
  return false;
}

/**
 * Điểm mới có đủ xa điểm cuối để ghi thêm không. Trình duyệt bắn pointermove
 * dày đặc; không lọc thì một nét ngắn cũng thành hàng trăm toạ độ nằm chồng
 * lên nhau trong localStorage.
 */
export function farEnough(points: number[], x: number, y: number, minDistance: number): boolean {
  const n = points.length;
  if (n < 2) return true;
  const dx = x - points[n - 2];
  const dy = y - points[n - 1];
  return dx * dx + dy * dy >= minDistance * minDistance;
}
