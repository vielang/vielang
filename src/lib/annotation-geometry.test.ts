import { describe, expect, it } from "vitest";
import type { Stroke } from "./annotation-store";
import { farEnough, quantize, strokeHit, strokePath } from "./annotation-geometry";

/** Trang sách trong app cao hơn rộng — dùng tỉ lệ này để bắt lỗi nhầm trục. */
const AR = 0.7;

function stroke(points: number[], width = 0.004): Stroke {
  return { id: "s", tool: "pen", color: "#000", width, points };
}

describe("strokePath", () => {
  it("không vẽ gì khi chưa có điểm nào", () => {
    expect(strokePath([], 1000, 1400)).toBe("");
  });

  it("chạm rồi nhả tại chỗ vẫn ra một chấm", () => {
    // Chỉ mỗi lệnh M thì <path> không vẽ gì — phải có thêm đoạn dài 0.
    expect(strokePath([0.5, 0.25], 1000, 1400)).toBe("M 500 350 l 0 0");
  });

  it("nối 2 điểm bằng một đoạn thẳng", () => {
    expect(strokePath([0, 0, 1, 1], 1000, 1400)).toBe("M 0 0 L 1000 1400");
  });

  it("làm mượt qua trung điểm khi có từ 3 điểm trở lên", () => {
    // Điểm giữa thành điểm điều khiển, đường đi qua trung điểm của nó và điểm sau.
    expect(strokePath([0, 0, 0.5, 0, 1, 0], 1000, 1000)).toBe(
      "M 0 0 Q 500 0 750 0 L 1000 0"
    );
  });

  it("nhân x theo bề rộng và y theo bề cao của viewBox", () => {
    expect(strokePath([0.5, 0.5], 1000, 1400)).toBe("M 500 700 l 0 0");
  });
});

describe("quantize", () => {
  it("giữ 4 chữ số thập phân", () => {
    expect(quantize([0.123456, 0.999999])).toEqual([0.1235, 1]);
  });
});

describe("farEnough", () => {
  it("luôn nhận điểm đầu tiên", () => {
    expect(farEnough([], 0.5, 0.5, 0.01)).toBe(true);
  });

  it("bỏ điểm nằm sát điểm vừa ghi", () => {
    expect(farEnough([0.5, 0.5], 0.501, 0.5, 0.01)).toBe(false);
  });

  it("nhận điểm đã đi đủ xa", () => {
    expect(farEnough([0.5, 0.5], 0.52, 0.5, 0.01)).toBe(true);
  });
});

describe("strokeHit", () => {
  it("chạm trúng giữa nét", () => {
    expect(strokeHit(stroke([0.2, 0.5, 0.8, 0.5]), 0.5, 0.5, AR, 0.01)).toBe(true);
  });

  it("không chạm khi đầu tẩy còn xa", () => {
    expect(strokeHit(stroke([0.2, 0.5, 0.8, 0.5]), 0.5, 0.9, AR, 0.01)).toBe(false);
  });

  it("chỉ tính phần trong đoạn, không kéo dài vô hạn", () => {
    // Nét nằm từ x=0.2 tới x=0.8; điểm ở x=0.95 thẳng hàng nhưng đã ra ngoài.
    expect(strokeHit(stroke([0.2, 0.5, 0.8, 0.5]), 0.95, 0.5, AR, 0.01)).toBe(false);
  });

  it("đo khoảng cách theo đơn vị bề rộng trang, không lẫn trục", () => {
    // Nét dọc. Lệch 0.02 theo trục y trên trang cao hơn rộng chỉ bằng
    // 0.02/0.7 ≈ 0.029 đơn vị bề rộng... nhưng lệch theo y thì trượt DỌC nét
    // nên vẫn trúng, còn lệch theo x mới là ra khỏi nét.
    const vertical = stroke([0.5, 0.2, 0.5, 0.8]);
    expect(strokeHit(vertical, 0.5, 0.7, AR, 0.005)).toBe(true);
    expect(strokeHit(vertical, 0.53, 0.5, AR, 0.005)).toBe(false);
  });

  it("nét dày thì dễ chạm hơn nét mảnh", () => {
    const thin = stroke([0.2, 0.5, 0.8, 0.5], 0.002);
    const thick = stroke([0.2, 0.5, 0.8, 0.5], 0.06);
    // Cách tim nét 0.02 đơn vị bề rộng: quá xa với nét mảnh, vẫn trong nét dày.
    const y = 0.5 + 0.02 * AR;
    expect(strokeHit(thin, 0.5, y, AR, 0.005)).toBe(false);
    expect(strokeHit(thick, 0.5, y, AR, 0.005)).toBe(true);
  });

  it("chạm được cả nét một chấm", () => {
    expect(strokeHit(stroke([0.5, 0.5]), 0.5, 0.5, AR, 0.01)).toBe(true);
    expect(strokeHit(stroke([0.5, 0.5]), 0.9, 0.5, AR, 0.01)).toBe(false);
  });

  it("nét rỗng thì không bao giờ chạm", () => {
    expect(strokeHit(stroke([]), 0.5, 0.5, AR, 0.1)).toBe(false);
  });
});
