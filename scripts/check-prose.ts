#!/usr/bin/env tsx
/**
 * Đo văn phong từng bài học IT theo các ngưỡng trong content/it/FORMAT.md.
 *
 * Có công cụ này vì "văn chưa hay" là nhận xét không sửa được. Đo ra số thì
 * biết chính xác bài nào còn nợ gì: câu dài lê thê, thiếu bảng tra cứu, tiêu
 * đề chỉ là cái nhãn, hay định danh còn viết tiếng Việt.
 *
 * Công cụ này KHÔNG biết code có chạy được không — đó là việc của check-code.
 *
 * Chạy: npm run check-prose          (toàn bộ)
 *       npm run check-prose -- linq  (lọc theo tên file)
 */
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const ROOT = path.resolve(process.cwd(), "content", "it");

/** Ngưỡng lấy từ FORMAT.md — sửa ở đó thì sửa cả ở đây. */
const NGUONG = {
  cauDaiToiDa: 3, // câu trên 22 từ
  tiLeCauNganMin: 30, // % câu dưới 10 từ
  tiLeCauNganMax: 55,
  tbMin: 11,
  tbMax: 14,
  tuMin: 700,
  tuMax: 1200,
};

/** Tiêu đề cố định của phần khung, không tính là "nhãn". */
const TIEU_DE_KHUNG = ["Dấu hiệu trong code của bạn", "Ghi nhớ", "Bước tiếp theo", "Lỗi hay gặp lần đầu"];

/** Âm tiết tiếng Việt hay bị dùng làm tên biến. */
const AM_TIET_VIET = new Set([
  "don", "hang", "khach", "tien", "gia", "ten", "tong", "luong", "so", "nguoi",
  "ngay", "thang", "luu", "xoa", "them", "sua", "tim", "lay", "dat", "gui",
  "nhan", "kiem", "tinh", "dem", "doi", "mo", "dong", "chay", "xuly", "cho",
  "moi", "cu", "dau", "cuoi", "truoc", "sau", "trong", "ngoai", "noi", "dung",
  "doc", "ghi", "bat", "tat", "phi", "thue", "diem",
]);

/**
 * Định danh tiếng Việt còn sót. Tách camelCase/PascalCase ra từng âm tiết rồi
 * mới so — nếu chỉ so cả từ thì `DocDong`, `noiDung`, `Tong` đều lọt, vì `\b`
 * không có ranh giới nào ở giữa một định danh ghép.
 */
function timDinhDanhViet(code: string): string[] {
  const ra = new Set<string>();
  for (const m of code.matchAll(/\b[A-Za-z_][A-Za-z0-9_]*\b/g)) {
    const ten = m[0];
    if (ten.length < 2) continue;
    const phan = ten.split(/(?=[A-Z])|_/).filter(Boolean).map((p) => p.toLowerCase());
    // Một âm tiết Việt đứng riêng thành một phần của tên: `DocDong` → doc, dong.
    if (phan.some((p) => AM_TIET_VIET.has(p))) ra.add(ten);
  }
  return [...ra];
}

interface KetQua {
  file: string;
  tu: number;
  cau: number;
  dai: number;
  nganPhanTram: number;
  tb: number;
  bang: number;
  nhan: string[];
  vietNam: string[];
  /** Vị trí đáp án đúng, 1–4. Dồn hết vào một vị trí là quiz đoán được. */
  dapAn: number[];
  /** Ô bảng có `|` chưa escape — dấu đó cắt ô, nội dung sau nó mất khi render. */
  bangVoPipe: string[];
  /** Mục h2 mở thẳng bằng code hoặc bảng, không câu bản lề. */
  mucTran: string[];
  /** Câu hỏi có code chép lại nguyên văn từ thân bài. */
  quizChep: number;
}

async function lietKe(dir: string): Promise<string[]> {
  const out: string[] = [];
  for (const ten of await readdir(dir)) {
    const p = path.join(dir, ten);
    if ((await stat(p)).isDirectory()) out.push(...(await lietKe(p)));
    else if (ten.endsWith(".md") && ten !== "FORMAT.md" && ten !== "_module.md" && ten !== "course.md")
      out.push(p);
  }
  return out;
}

function doMotBai(raw: string): Omit<KetQua, "file"> {
  const than = raw.replace(/^---[\s\S]*?---/, "").replace(/```quiz[\s\S]*?```/, "");
  const vanXuoi = than
    .replace(/```[\s\S]*?```/g, "")
    .replace(/^\|.*$/gm, "")
    .replace(/^[->#].*$/gm, "");

  const cau = vanXuoi
    .split(/(?<=[.!?])\s+/)
    .map((c) => c.replace(/\s+/g, " ").trim())
    .filter((c) => c.split(" ").length > 2);
  const doDai = cau.map((c) => c.split(" ").length);
  const ngan = doDai.filter((n) => n < 10).length;

  const nhan = [...than.matchAll(/^## (.+)$/gm)]
    .map((m) => m[1])
    .filter((t) => !TIEU_DE_KHUNG.includes(t))
    // Tiêu đề là "nhãn" khi quá ngắn và không chứa động từ/mệnh đề.
    .filter((t) => t.split(" ").length <= 4 && !t.includes(":"));

  const code = [...than.matchAll(/```csharp[^\n]*\n([\s\S]*?)```/g)]
    .map((m) => m[1].replace(/\/\/[^\n]*/g, "").replace(/"[^"]*"/g, '""'))
    .join("\n");

  // Trong bảng Markdown, `|` cắt ô kể cả khi nằm trong backtick. Phải viết \|
  // nếu không nửa sau của ô biến mất lúc render — lỗi chỉ thấy trên web.
  //
  // Tách theo backtick rồi chỉ soi các đoạn BÊN TRONG code (chỉ số lẻ). Một
  // regex kiểu /`[^`]*\|[^`]*`/ trông hợp lý nhưng khớp cả khoảng trống giữa
  // hai ô code — `| `3` | `int` |` cũng bị báo, tức 36 báo động giả.
  const bangVoPipe = than
    .split("\n")
    .filter((d) => d.startsWith("|"))
    .filter((d) => d.split("`").some((doan, i) => i % 2 === 1 && /(^|[^\\])\|/.test(doan)))
    .map((d) => d.trim());

  // Mục mở thẳng bằng code hay bảng thì văn xuôi bị đẩy xuống vai thuyết minh
  // lại thứ người đọc vừa thấy. FORMAT.md đòi một câu bản lề trước đã.
  const mucTran: string[] = [];
  for (const khuc of than.split(/^## /m).slice(1)) {
    const [tieuDe, ...conLai] = khuc.split("\n");
    const dongDau = conLai.find((d) => d.trim());
    if (dongDau && (dongDau.startsWith("```") || dongDau.startsWith("|"))) mucTran.push(tieuDe.trim());
  }

  // Quiz: vị trí đáp án, và câu hỏi chép lại code của thân bài.
  const dapAn: number[] = [];
  let quizChep = 0;
  const khoiQuiz = raw.match(/```quiz\n([\s\S]*?)```/);
  if (khoiQuiz) {
    try {
      const cauHoi = JSON.parse(khoiQuiz[1]) as { answer: number; code?: string }[];
      for (const c of cauHoi) {
        dapAn.push(c.answer);
        const dongDau = c.code?.split("\\n")[0]?.trim();
        if (dongDau && dongDau.length > 15 && than.includes(dongDau)) quizChep++;
      }
    } catch {
      // Quiz sai JSON đã có check-code báo, ở đây bỏ qua.
    }
  }

  return {
    dapAn,
    bangVoPipe,
    mucTran,
    quizChep,
    tu: than.replace(/```[\s\S]*?```/g, "").split(/\s+/).filter(Boolean).length,
    cau: doDai.length,
    dai: doDai.filter((n) => n > 22).length,
    nganPhanTram: doDai.length ? Math.round((100 * ngan) / doDai.length) : 0,
    tb: doDai.length ? +(doDai.reduce((a, b) => a + b, 0) / doDai.length).toFixed(1) : 0,
    bang: (than.match(/^\|/gm) ?? []).length,
    nhan,
    vietNam: timDinhDanhViet(code),
  };
}

/** Đáp án dồn về một vị trí thì người học đoán được mà không cần đọc đề. */
function quizDoanDuoc(dapAn: number[]): boolean {
  if (dapAn.length < 3) return false;
  const dem = new Map<number, number>();
  for (const a of dapAn) dem.set(a, (dem.get(a) ?? 0) + 1);
  return Math.max(...dem.values()) > dapAn.length - 2;
}

function dat(k: KetQua): boolean {
  return (
    k.dai <= NGUONG.cauDaiToiDa &&
    k.nganPhanTram >= NGUONG.tiLeCauNganMin &&
    k.nganPhanTram <= NGUONG.tiLeCauNganMax &&
    k.tb >= NGUONG.tbMin &&
    k.tb <= NGUONG.tbMax &&
    k.bang > 0 &&
    k.nhan.length === 0 &&
    k.vietNam.length === 0 &&
    k.bangVoPipe.length === 0 &&
    k.mucTran.length === 0 &&
    k.quizChep === 0 &&
    !quizDoanDuoc(k.dapAn)
  );
}

/**
 * Thuật ngữ cốt lõi: khái niệm mà cả khoá dựa lên, nên phải có định nghĩa
 * chính thức trước khi dùng nhiều.
 *
 * Vì sao cần danh sách này: FORMAT.md đòi "định nghĩa một câu, chính xác, không
 * ẩn dụ", nhưng các phép đo khác chỉ thấy nhịp câu và bảng. Nên cả khoá từng
 * dùng "kế thừa" 39 lần, "composition" 25 lần mà không định nghĩa lần nào —
 * người học phải tự đoán, và không công cụ nào báo.
 */
const THUAT_NGU = [
  "đóng gói",
  "trừu tượng",
  "kế thừa",
  "composition",
  "đa hình",
  "interface",
  "abstract class",
  "value type",
  "reference type",
  "stack trace",
  "middleware",
  "pattern matching",
];

/** Dùng từ ngần này lần trong một bài thì bài đó đang dựa vào khái niệm ấy. */
const LAN_COI_LA_DUA_VAO = 3;

const thoat = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/**
 * Định nghĩa chính thức: thuật ngữ in đậm, ngay sau là "là" hoặc dấu hai chấm.
 * Cố ý không nhận ẩn dụ kiểu "Interface là một lời hứa" — FORMAT cấm ẩn dụ, nên
 * mẫu này đòi thuật ngữ được in đậm để phân biệt định nghĩa với cách nói ví von.
 */
function coDinhNghia(than: string, tu: string): boolean {
  // Phần in đậm phải gần như CHÍNH thuật ngữ, cho phép thêm tối đa một cụm ngắn
  // như "(encapsulation)". Nếu nới ra thì "**Interface Segregation Principle**:"
  // cũng bị tính là định nghĩa của "interface" — một lần báo đạt sai.
  // Dùng `là\s` chứ không `là\b`: \b của JavaScript dựa trên [A-Za-z0-9_], nên
  // chữ "à" bị coi là không phải ký tự từ và `là\b` không bao giờ khớp.
  for (const m of than.matchAll(/\*\*([^*]+)\*\*\s*(?:là\s|:)/gi)) {
    const dam = m[1].trim();
    if (!new RegExp(`\\b${thoat(tu)}\\b`, "i").test(dam)) continue;
    if (dam.length - tu.length <= 18) return true;
  }
  return false;
}

/** Trả về số thuật ngữ còn thiếu định nghĩa đúng chỗ. */
async function soatDinhNghia(files: string[]): Promise<number> {
  const than = new Map<string, string>();
  for (const f of files) {
    const raw = (await readFile(f, "utf8")).split("\r\n").join("\n");
    than.set(f, raw.replace(/```quiz[\s\S]*?```/, ""));
  }

  const loi: string[] = [];
  for (const tu of THUAT_NGU) {
    const re = new RegExp(`\\b${thoat(tu)}\\b`, "gi");
    let duaVao: string | null = null;
    let dinhNghia: string | null = null;

    for (const f of files) {
      const t = than.get(f)!;
      if (!dinhNghia && coDinhNghia(t, tu)) dinhNghia = f;
      if (!duaVao && (t.match(re) ?? []).length >= LAN_COI_LA_DUA_VAO) duaVao = f;
      if (duaVao) break; // chỉ cần biết bài đầu tiên dựa vào nó
    }

    if (!duaVao) continue;
    if (dinhNghia) continue; // định nghĩa nằm ở bài đó hoặc trước đó

    loi.push(`${tu} — bài ${path.basename(duaVao)} dùng ${(than.get(duaVao)!.match(re) ?? []).length} lần`);
  }

  if (loi.length) {
    console.log("\nThuật ngữ dùng nhiều mà chưa định nghĩa chính thức trước đó:");
    for (const d of loi) console.log(`  ${d}`);
  }
  return loi.length;
}

async function main() {
  const loc = process.argv[2];
  const files = (await lietKe(ROOT)).filter((f) => !loc || f.includes(loc));
  const ketQua: KetQua[] = [];

  for (const f of files) {
    ketQua.push({ file: path.relative(ROOT, f), ...doMotBai(await readFile(f, "utf8")) });
  }

  console.log(
    "bài".padEnd(46) + "từ".padStart(6) + "dài".padStart(5) + "ngắn".padStart(6) + "TB".padStart(6) + "bảng".padStart(6)
  );
  for (const k of ketQua) {
    const dau = dat(k) ? "✓" : "·";
    console.log(
      `${dau} ${k.file.slice(-44).padEnd(44)}` +
        String(k.tu).padStart(6) +
        String(k.dai).padStart(5) +
        `${k.nganPhanTram}%`.padStart(6) +
        String(k.tb).padStart(6) +
        String(k.bang).padStart(6)
    );
    const loi: string[] = [];
    if (k.dai > NGUONG.cauDaiToiDa) loi.push(`${k.dai} câu dài quá 22 từ`);
    if (k.nganPhanTram < NGUONG.tiLeCauNganMin) loi.push("thiếu câu ngắn");
    if (k.nganPhanTram > NGUONG.tiLeCauNganMax) loi.push("cụt quá, gộp bớt");
    if (k.tb < NGUONG.tbMin || k.tb > NGUONG.tbMax) loi.push(`TB ${k.tb} từ ngoài khoảng ${NGUONG.tbMin}–${NGUONG.tbMax}`);
    if (k.tu < NGUONG.tuMin) loi.push(`mới ${k.tu} từ, còn mỏng`);
    if (k.tu > NGUONG.tuMax) loi.push(`${k.tu} từ, vượt ngân sách`);
    if (k.bang === 0) loi.push("chưa có bảng tra cứu");
    if (k.nhan.length) loi.push(`tiêu đề còn là nhãn: ${k.nhan.join(" / ")}`);
    if (k.vietNam.length) loi.push(`định danh tiếng Việt: ${k.vietNam.slice(0, 6).join(", ")}`);
    if (k.bangVoPipe.length) loi.push(`${k.bangVoPipe.length} ô bảng có | chưa escape (mất chữ khi render)`);
    if (k.mucTran.length) loi.push(`${k.mucTran.length} mục mở trần: ${k.mucTran.slice(0, 2).join(" / ")}`);
    if (k.quizChep) loi.push(`${k.quizChep} câu hỏi chép code thân bài`);
    if (quizDoanDuoc(k.dapAn)) loi.push(`đáp án dồn một vị trí: ${k.dapAn.join("")}`);
    if (loi.length) console.log("    " + loi.join(" · "));
  }

  const thieu = await soatDinhNghia(files);

  const xong = ketQua.filter(dat).length;
  console.log(`\n${xong}/${ketQua.length} bài đạt chuẩn.`);
  if (thieu > 0) console.log(`${thieu} thuật ngữ cốt lõi chưa có định nghĩa đúng chỗ.`);
}

main();
