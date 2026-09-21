const fs = require("fs");
const must = (ok, w) => { if (!ok) throw new Error("không khớp: " + w); };

let a = fs.readFileSync("src/lib/offline-books.test.ts", "utf8");
const E1 = a.includes("\r\n") ? "\r\n" : "\n";
const old1 = [
  '  it("ước lượng dung lượng có tính audio", () => {',
  "    // Audio mới là phần nặng: bỏ nó ra là báo thiếu tới ba lần, người dùng",
  "    // bấm tải xong mới ngã ngửa vì hết chỗ máy.",
  '    const step1 = BOOKS.find((b) => b.id === "step1")!;',
  "    const imagesOnly = step1.totalPages * 180 * 1024;",
  "",
  "    expect(estimateBytes(step1)).toBeGreaterThan(imagesOnly * 2.5);",
  "  });",
].join(E1);
must(a.includes(old1), "test ước lượng");
a = a.replace(old1, [
  '  it("ước lượng khớp với thứ THẬT SỰ tải về", () => {',
  "    // Đây mới là bất biến, chứ không phải “có cộng audio hay không”. Báo",
  "    // một đằng tải một nẻo thì hoặc người dùng hết chỗ máy, hoặc tưởng đã",
  "    // mang sách theo mà hoá ra không — cả hai đều chỉ lộ ra lúc mất mạng.",
  '    const step1 = BOOKS.find((b) => b.id === "step1")!;',
  "    const images = step1.totalPages * 180 * 1024;",
  '    const audio = countAudioTracks("step1") * 1300 * 1024;",',
  "",
  "    expect(estimateBytes(step1)).toBe(",
  "      AUDIO_CAN_BE_CACHED ? images + audio : images",
  "    );",
  "  });",
].join(E1));
must(a.includes("AUDIO_CAN_BE_CACHED"), "đã chèn");
// sửa lỗi gõ dấu nháy thừa ở dòng audio
a = a.replace('const audio = countAudioTracks("step1") * 1300 * 1024;",', 'const audio = countAudioTracks("step1") * 1300 * 1024;');
// bổ sung import
const imp = a.match(/import \{[^}]*\} from "\.\/offline-books";/);
must(imp, "import offline-books");
a = a.replace(imp[0], imp[0].replace("import {", "import {" + E1 + "  AUDIO_CAN_BE_CACHED,"));
fs.writeFileSync("src/lib/offline-books.test.ts", a);

let b = fs.readFileSync("src/components/library/offline-download.test.tsx", "utf8");
const E2 = b.includes("\r\n") ? "\r\n" : "\n";
const old2 = [
  '  it("vẫn tra được con số chính xác khi cần", () => {',
  "    // “phần” chứ không phải “trang”: gói tải về nay gồm cả bài nghe, mà một",
  "    // bài nghe nặng gấp hơn hai chục lần một trang nên phải đếm riêng, nếu",
  "    // không thanh tiến độ sẽ đứng im từng quãng dài.",
  "    widget();",
  "    expect(",
  '      screen.getByTitle("Đang tải 57/228 phần (trang sách và bài nghe)")',
  "    ).toBeTruthy();",
  "  });",
].join(E2);
must(b.includes(old2), "test chữ tiến độ");
b = b.replace(old2, [
  '  it("nói đúng thứ gói tải về THẬT SỰ có", () => {',
  "    // Hứa cả bài nghe trong khi nó không được lưu là lời hứa suông, và",
  "    // người dùng chỉ phát hiện ra lúc mất mạng — đúng lúc không sửa được.",
  "    widget();",
  "    const label = AUDIO_CAN_BE_CACHED",
  '      ? "phần (trang sách và bài nghe)"',
  '      : "trang sách";',
  "",
  "    expect(screen.getByTitle(`Đang tải 57/228 ${label}`)).toBeTruthy();",
  "  });",
].join(E2));
b = b.replace(
  'import { useDownloadStore }',
  'import { AUDIO_CAN_BE_CACHED } from "@/lib/offline-books";' + E2 + 'import { useDownloadStore }'
);
fs.writeFileSync("src/components/library/offline-download.test.tsx", b);
console.log("đã sửa test");
