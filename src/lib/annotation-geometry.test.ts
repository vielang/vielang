import { describe, expect, it } from "vitest";
import type { Stroke } from "./annotation-store";
import {
  farEnough,
  hitPolyline,
  outlineToPath,
  penOutlinePath,
  quantize,
  quantizePressure,
  shapePath,
  strokeHit,
  strokePath,
  textBounds,
} from "./annotation-geometry";

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

  it("lực bút chỉ giữ 2 chữ số", () => {
    expect(quantizePressure([0.123456, 0.5, 0.987])).toEqual([0.12, 0.5, 0.99]);
  });
});

describe("outlineToPath", () => {
  it("không vẽ gì khi viền rỗng", () => {
    expect(outlineToPath([])).toBe("");
  });

  it("khép kín đường viền", () => {
    const d = outlineToPath([
      [0, 0],
      [10, 0],
      [10, 10],
    ]);
    expect(d.startsWith("M 0 0 Q")).toBe(true);
    expect(d.endsWith("Z")).toBe(true);
  });
});

describe("penOutlinePath", () => {
  it("không vẽ gì khi chưa có điểm nào", () => {
    expect(penOutlinePath([], undefined, 1000, 1400, 4)).toBe("");
  });

  it("chạm rồi nhả tại chỗ vẫn ra một chấm khép kín", () => {
    const d = penOutlinePath([0.5, 0.25], undefined, 1000, 1400, 8);
    expect(d).toMatch(/^M .+ Z$/);
    expect(d.length).toBeGreaterThan(50);
  });

  it("lực bút mạnh cho nét dày hơn lực bút nhẹ", () => {
    const pts = [0.2, 0.5, 0.4, 0.5, 0.6, 0.5];
    const light = penOutlinePath(pts, [0.05, 0.05, 0.05], 1000, 1400, 20);
    const heavy = penOutlinePath(pts, [1, 1, 1], 1000, 1400, 20);
    expect(spanY(light)).toBeLessThan(spanY(heavy));
  });

  it("nét dài ra thì viền dài theo", () => {
    const short = penOutlinePath([0.2, 0.5, 0.3, 0.5], undefined, 1000, 1400, 8);
    const long = penOutlinePath([0.2, 0.5, 0.8, 0.5], undefined, 1000, 1400, 8);
    expect(spanX(long)).toBeGreaterThan(spanX(short));
  });
});

describe("shapePath", () => {
  it("chưa đủ 2 điểm thì chưa thành hình", () => {
    expect(shapePath("line", [0.2, 0.5], 1000, 1400, 4)).toBeNull();
  });

  it("đường thẳng nối đúng 2 điểm", () => {
    expect(shapePath("line", [0, 0, 1, 1], 1000, 1400, 4)).toBe("M 0 0 L 1000 1400");
  });

  it("khung khép kín 4 góc chứ không phải đường chéo", () => {
    expect(shapePath("rect", [0, 0, 1, 1], 1000, 1400, 4)).toBe(
      "M 0 0 L 1000 0 L 1000 1400 L 0 1400 Z"
    );
  });

  it("mũi tên có thêm đầu nhọn ở điểm cuối", () => {
    const line = shapePath("line", [0.2, 0.5, 0.8, 0.5], 1000, 1400, 4)!;
    const arrow = shapePath("arrow", [0.2, 0.5, 0.8, 0.5], 1000, 1400, 4)!;
    expect(arrow.startsWith(line)).toBe(true);
    expect(arrow.length).toBeGreaterThan(line.length);
  });

  it("nét dày thì đầu mũi tên to theo, không phụ thuộc chiều dài", () => {
    const thin = shapePath("arrow", [0.2, 0.5, 0.8, 0.5], 1000, 1400, 2)!;
    const thick = shapePath("arrow", [0.2, 0.5, 0.8, 0.5], 1000, 1400, 20)!;
    expect(spanY(thick)).toBeGreaterThan(spanY(thin));
  });

  it("mũi tên dài 0 thì không dựng đầu nhọn, chỉ còn đoạn rỗng", () => {
    expect(shapePath("arrow", [0.5, 0.5, 0.5, 0.5], 1000, 1400, 4)).toBe(
      "M 500 700 L 500 700"
    );
  });
});

describe("textBounds", () => {
  const text = (s: string): Stroke => ({
    id: "t",
    tool: "text",
    color: "#000",
    width: 0.02,
    points: [0.1, 0.2],
    text: s,
  });

  it("không có chữ thì không có khung", () => {
    expect(textBounds({ ...text(""), text: undefined }, AR)).toBeNull();
  });

  it("khung bắt đầu đúng tại điểm neo", () => {
    const [x0, y0] = textBounds(text("abc"), AR)!;
    expect(x0).toBe(0.1);
    expect(y0).toBe(0.2);
  });

  it("chữ dài hơn thì khung rộng hơn", () => {
    const short = textBounds(text("a"), AR)!;
    const long = textBounds(text("aaaaaaaaaa"), AR)!;
    expect(long[2]).toBeGreaterThan(short[2]);
  });

  it("thêm dòng thì khung cao hơn", () => {
    const one = textBounds(text("a"), AR)!;
    const three = textBounds(text("a\nb\nc"), AR)!;
    expect(three[3]).toBeGreaterThan(one[3]);
  });
});

describe("hitPolyline", () => {
  it("khung được quy về 4 cạnh khép kín", () => {
    const rect: Stroke = {
      id: "r",
      tool: "rect",
      color: "#000",
      width: 0.004,
      points: [0.2, 0.3, 0.8, 0.7],
    };
    expect(hitPolyline(rect, AR)).toEqual([0.2, 0.3, 0.8, 0.3, 0.8, 0.7, 0.2, 0.7, 0.2, 0.3]);
  });

  it("nét tay giữ nguyên đường đi của nó", () => {
    expect(hitPolyline(stroke([0.1, 0.2, 0.3, 0.4]), AR)).toEqual([0.1, 0.2, 0.3, 0.4]);
  });
});

describe("strokeHit với hình và chữ", () => {
  const rect: Stroke = {
    id: "r",
    tool: "rect",
    color: "#000",
    width: 0.004,
    points: [0.2, 0.3, 0.8, 0.7],
  };

  it("tẩy bắt được cạnh dưới của khung, không chỉ đường chéo", () => {
    // Giữa cạnh dưới: nằm trên khung nhưng cách xa đường chéo hai góc.
    expect(strokeHit(rect, 0.5, 0.7, AR, 0.01)).toBe(true);
  });

  it("không bắt phần rỗng giữa khung", () => {
    expect(strokeHit(rect, 0.5, 0.5, AR, 0.01)).toBe(false);
  });

  it("tẩy bắt được chữ ở giữa dòng, không chỉ ở điểm neo", () => {
    const note: Stroke = {
      id: "t",
      tool: "text",
      color: "#000",
      width: 0.03,
      points: [0.1, 0.2],
      text: "chú thích dài",
    };
    expect(strokeHit(note, 0.2, 0.2, AR, 0.005)).toBe(true);
    expect(strokeHit(note, 0.9, 0.2, AR, 0.005)).toBe(false);
  });
});

/** Bề rộng/bề cao thật của một chuỗi `d`, đọc ngược từ các số trong đó. */
function bounds(d: string): { x: number[]; y: number[] } {
  const nums = d.match(/-?\d+(\.\d+)?/g)?.map(Number) ?? [];
  const x: number[] = [];
  const y: number[] = [];
  nums.forEach((n, i) => (i % 2 === 0 ? x : y).push(n));
  return { x, y };
}
const spanX = (d: string) => Math.max(...bounds(d).x) - Math.min(...bounds(d).x);
const spanY = (d: string) => Math.max(...bounds(d).y) - Math.min(...bounds(d).y);

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
