#!/usr/bin/env tsx
/**
 * Kiểm từng bài học IT theo content/it/FORMAT.md: đúng khuôn mục, đủ ngắn,
 * có định nghĩa một dòng, quiz không đoán mò được.
 *
 * Công cụ này KHÔNG biết code có chạy được không — đó là việc của check-code.
 *
 * Chạy: npm run check-prose              (toàn bộ)
 *       npm run check-prose -- ke-thua   (lọc theo tên file)
 */
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const ROOT = path.resolve(process.cwd(), "content", "it");

/** Ngưỡng lấy từ FORMAT.md — sửa ở đó thì sửa cả ở đây. */
const NGUONG = {
  tuMin: 200,
  tuMax: 700,
  /** Câu dài quá ngần này âm tiết thì nên tách. */
  cauDai: 40,
  cauDaiToiDa: 2,
  mucPhuToiDa: 2,
};

/** Các mục cố định, theo đúng thứ tự. */
const MUC_BAT_BUOC = ["Khái niệm", "Ví dụ", "Thử ngay", "Lỗi hay gặp", "Tóm tắt"];

/** Câu nối rỗng và lối viết lan man mà FORMAT.md cấm. */
const CAU_SAO = [
  "đủ lý thuyết",
  "chúng ta hãy cùng",
  "hãy cùng tìm hiểu",
  "đoán sai một lần",
  "như chúng ta đã biết",
  "trong bài viết này",
];

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
 * mới so — `DocDong`, `noiDung` đều là một từ với `\b`.
 */
function timDinhDanhViet(code: string): string[] {
  const ra = new Set<string>();
  for (const m of code.matchAll(/\b[A-Za-z_][A-Za-z0-9_]*\b/g)) {
    const ten = m[0];
    if (ten.length < 2) continue;
    const phan = ten.split(/(?=[A-Z])|_/).filter(Boolean).map((p) => p.toLowerCase());
    if (phan.some((p) => AM_TIET_VIET.has(p))) ra.add(ten);
  }
  return [...ra];
}

/** Dòng định nghĩa: `🧬 **Thuật ngữ (english)**: một câu.` */
const DONG_DINH_NGHIA = /^\S+\s+\*\*[^*]+\*\*:\s+\S/;

interface KetQua {
  file: string;
  tu: number;
  loi: string[];
}

async function lietKe(dir: string): Promise<string[]> {
  const out: string[] = [];
  for (const ten of (await readdir(dir)).sort()) {
    const p = path.join(dir, ten);
    if ((await stat(p)).isDirectory()) out.push(...(await lietKe(p)));
    else if (ten.endsWith(".md") && ten !== "FORMAT.md" && ten !== "_module.md" && ten !== "course.md")
      out.push(p);
  }
  return out;
}

function doMotBai(raw: string): { tu: number; loi: string[] } {
  const loi: string[] = [];
  const than = raw.replace(/^---[\s\S]*?---/, "").replace(/```quiz[\s\S]*?```/, "");
  const khongCode = than.replace(/```[\s\S]*?```/g, "");
  const vanXuoi = khongCode.replace(/^\|.*$/gm, "").replace(/^#.*$/gm, "").replace(/<[^>]+>/g, "");

  // Độ dài
  const tu = vanXuoi.split(/\s+/).filter(Boolean).length;
  if (tu < NGUONG.tuMin) loi.push(`mới ${tu} chữ, còn mỏng`);
  if (tu > NGUONG.tuMax) loi.push(`${tu} chữ, vượt ${NGUONG.tuMax}`);

  // Câu dài. Tiếng Việt tách theo dấu cách là ra âm tiết, không phải từ.
  const cauDai = vanXuoi
    .split(/(?<=[.!?])\s+|\n\s*\n|\n-\s/)
    .map((c) => c.replace(/\s+/g, " ").trim())
    .filter((c) => c.split(" ").length > NGUONG.cauDai);
  if (cauDai.length > NGUONG.cauDaiToiDa)
    loi.push(`${cauDai.length} câu quá ${NGUONG.cauDai} âm tiết: "${cauDai[0].slice(0, 50)}…"`);

  // Khuôn mục
  const muc = [...than.matchAll(/^## (.+)$/gm)].map((m) => m[1].trim());
  const thieu = MUC_BAT_BUOC.filter((m) => !muc.includes(m));
  if (thieu.length) loi.push(`thiếu mục: ${thieu.join(", ")}`);
  const thuTu = muc.filter((m) => MUC_BAT_BUOC.includes(m));
  if (!thieu.length && thuTu.join("|") !== MUC_BAT_BUOC.join("|")) loi.push(`sai thứ tự mục: ${thuTu.join(" → ")}`);
  const mucPhu = muc.filter((m) => !MUC_BAT_BUOC.includes(m));
  if (mucPhu.length > NGUONG.mucPhuToiDa) loi.push(`${mucPhu.length} mục phụ, tối đa ${NGUONG.mucPhuToiDa}`);

  // Định nghĩa một dòng trong mục Khái niệm
  const khaiNiem = than.split(/^## /m).find((k) => k.startsWith("Khái niệm"));
  const dinhNghia = (khaiNiem ?? "").split("\n").filter((d) => DONG_DINH_NGHIA.test(d));
  if (khaiNiem && dinhNghia.length === 0) loi.push("mục Khái niệm chưa có dòng định nghĩa `🧬 **X**: …`");

  // Câu sáo
  const thuong = vanXuoi.toLowerCase();
  const sao = CAU_SAO.filter((c) => thuong.includes(c));
  if (sao.length) loi.push(`câu sáo: ${sao.join(", ")}`);

  // Định danh tiếng Việt trong code
  const code = [...than.matchAll(/```csharp[^\n]*\n([\s\S]*?)```/g)]
    .map((m) => m[1].replace(/\/\/[^\n]*/g, "").replace(/"[^"]*"/g, '""'))
    .join("\n");
  const viet = timDinhDanhViet(code);
  if (viet.length) loi.push(`định danh tiếng Việt: ${viet.slice(0, 6).join(", ")}`);

  // `|` trong backtick của bảng vẫn cắt ô: phải viết \|. Chỉ soi đoạn BÊN
  // TRONG backtick (chỉ số lẻ), nếu không khoảng giữa hai ô code cũng bị báo.
  const pipe = than
    .split("\n")
    .filter((d) => d.startsWith("|"))
    .filter((d) => d.split("`").some((doan, i) => i % 2 === 1 && /(^|[^\\])\|/.test(doan)));
  if (pipe.length) loi.push(`${pipe.length} ô bảng có | chưa escape`);

  // Quiz
  const khoiQuiz = raw.match(/```quiz\n([\s\S]*?)```/);
  if (!khoiQuiz) loi.push("thiếu khối quiz");
  else {
    try {
      const cauHoi = JSON.parse(khoiQuiz[1]) as { answer: number; code?: string }[];
      if (cauHoi.length < 3 || cauHoi.length > 5) loi.push(`quiz có ${cauHoi.length} câu, cần 3`);
      const dapAn = cauHoi.map((c) => c.answer);
      const dem = new Map<number, number>();
      for (const a of dapAn) dem.set(a, (dem.get(a) ?? 0) + 1);
      if (dapAn.length >= 3 && Math.max(...dem.values()) > dapAn.length - 2)
        loi.push(`đáp án dồn một vị trí: ${dapAn.join("")}`);
      const chep = cauHoi.filter((c) => {
        const dau = c.code?.split("\n")[0]?.trim();
        return dau && dau.length > 15 && than.includes(dau);
      }).length;
      if (chep) loi.push(`${chep} câu hỏi chép code thân bài`);
    } catch {
      loi.push("quiz sai JSON");
    }
  }

  return { tu, loi };
}

/**
 * Thuật ngữ cốt lõi: phải có dòng định nghĩa trước (hoặc ngay trong) bài đầu
 * tiên dùng nó nhiều lần. Thêm vào đây khi khoá có khái niệm nền mới.
 */
const THUAT_NGU = [
  "value type",
  "reference type",
  "exception",
  "đóng gói",
  "kế thừa",
  "đa hình",
  "abstract class",
  "interface",
  "composition",
  "dependency injection",
];

const LAN_COI_LA_DUA_VAO = 3;

const thoat = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** Có dòng định nghĩa mà phần in đậm gần như chính là thuật ngữ đó. */
function coDinhNghia(than: string, tu: string): boolean {
  for (const m of than.matchAll(/\*\*([^*]+)\*\*:/g)) {
    const dam = m[1].trim();
    if (!new RegExp(`(^|[^\\p{L}])${thoat(tu)}($|[^\\p{L}])`, "iu").test(dam)) continue;
    if (dam.length - tu.length <= 20) return true;
  }
  return false;
}

async function soatDinhNghia(files: string[]): Promise<string[]> {
  const than = new Map<string, string>();
  for (const f of files) {
    const raw = (await readFile(f, "utf8")).split("\r\n").join("\n");
    than.set(f, raw.replace(/```quiz[\s\S]*?```/, ""));
  }

  const loi: string[] = [];
  for (const tu of THUAT_NGU) {
    const re = new RegExp(`(^|[^\\p{L}])${thoat(tu)}(?=$|[^\\p{L}])`, "giu");
    for (const f of files) {
      const t = than.get(f)!;
      if (coDinhNghia(t, tu)) break; // định nghĩa đến trước
      const lan = (t.match(re) ?? []).length;
      if (lan >= LAN_COI_LA_DUA_VAO) {
        loi.push(`${tu} — ${path.relative(ROOT, f)} dùng ${lan} lần`);
        break;
      }
    }
  }
  return loi;
}

async function main() {
  const loc = process.argv[2];
  const tatCa = await lietKe(ROOT);
  const files = tatCa.filter((f) => !loc || f.includes(loc));
  const ketQua: KetQua[] = [];

  for (const f of files) {
    const raw = (await readFile(f, "utf8")).split("\r\n").join("\n");
    ketQua.push({ file: path.relative(ROOT, f), ...doMotBai(raw) });
  }

  for (const k of ketQua) {
    console.log(`${k.loi.length ? "·" : "✓"} ${k.file.padEnd(60)} ${String(k.tu).padStart(5)} chữ`);
    for (const l of k.loi) console.log(`    ${l}`);
  }

  // Soát định nghĩa luôn chạy trên TOÀN BỘ khoá, vì định nghĩa có thể nằm ở
  // bài trước bài đang lọc.
  const thieu = (await soatDinhNghia(tatCa)).filter((l) => !loc || l.includes(loc));
  if (thieu.length) {
    console.log("\nThuật ngữ dùng nhiều mà chưa có dòng định nghĩa trước đó:");
    for (const d of thieu) console.log(`  ${d}`);
  }

  const xong = ketQua.filter((k) => !k.loi.length).length;
  console.log(`\n${xong}/${ketQua.length} bài đạt chuẩn.`);
  if (xong < ketQua.length || thieu.length) process.exitCode = 1;
}

main();
