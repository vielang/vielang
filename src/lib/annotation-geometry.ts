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
