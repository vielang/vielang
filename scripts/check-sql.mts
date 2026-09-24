#!/usr/bin/env tsx
/**
 * Chạy THẬT mọi khối ```sql trong khoá SQL trên Oracle, và so kết quả phần
 * "Thử ngay" với bảng Markdown trong <details>.
 *
 * Vì sao cần: đáp án câu đoán trong bài SQL là một bảng dữ liệu. Viết tay thì
 * rất dễ lệch một dòng, một con số, mà người học tin bảng đó hơn tin mình.
 *
 * Cần một Oracle đang chạy (Docker):
 *   docker run -d --name kiip-oracle -p 1521:1521 \
 *     -e ORACLE_PASSWORD=kiip_sys_pw -e APP_USER=shop \
 *     -e APP_USER_PASSWORD=shop_pw gvenzl/oracle-free:slim-faststart
 *
 * Chạy: npm run check-sql            (toàn bộ)
 *       npm run check-sql -- join    (lọc theo tên file)
 *
 * Quy ước trong bài (xem content/it/FORMAT.md):
 * - ```sql setup  : script tạo dữ liệu mẫu, chỉ có ở bài đầu của khoá.
 * - ```sql reset  : xoá mọi bảng của user shop để làm lại; không chạy.
 * - ```sql        : chạy được trên dữ liệu mẫu, không được lỗi.
 * - "-- SAI" ở dòng đầu: phản ví dụ, được phép lỗi; nếu comment nói "lỗi"
 *   thì BẮT BUỘC lỗi.
 * - Khối trong "## Thử ngay" có bảng Markdown trong <details> ngay sau: câu
 *   SELECT cuối cùng của khối phải trả về đúng bảng đó. NULL viết "(null)".
 */
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import oracledb from "oracledb";

const ROOT = path.resolve(process.cwd(), "content", "it", "sql");
const CONNECT = {
  user: process.env.SQL_USER ?? "shop",
  password: process.env.SQL_PASSWORD ?? "shop_pw",
  connectString: process.env.SQL_CONNECT ?? "localhost:1521/FREEPDB1",
};

async function lietKe(dir: string): Promise<string[]> {
  const out: string[] = [];
  for (const ten of (await readdir(dir)).sort()) {
    const p = path.join(dir, ten);
    if ((await stat(p)).isDirectory()) out.push(...(await lietKe(p)));
    else if (ten.endsWith(".md") && ten !== "_module.md" && ten !== "course.md") out.push(p);
  }
  return out;
}

/** Tách câu lệnh theo dấu ; cuối dòng, bỏ qua ; nằm trong chuỗi '...'. */
function tachCauLenh(sql: string): string[] {
  const out: string[] = [];
  let cur = "";
  let trongChuoi = false;
  for (let i = 0; i < sql.length; i++) {
    const c = sql[i];
    if (c === "'") trongChuoi = !trongChuoi;
    if (c === "-" && sql[i + 1] === "-" && !trongChuoi) {
      // bỏ comment tới hết dòng
      while (i < sql.length && sql[i] !== "\n") i++;
      cur += "\n";
      continue;
    }
    if (c === ";" && !trongChuoi) {
      if (cur.trim()) out.push(cur.trim());
      cur = "";
      continue;
    }
    cur += c;
  }
  if (cur.trim()) out.push(cur.trim());
  return out;
}

/** Bảng Markdown đầu tiên trong đoạn text: [header, ...rows]. */
function docBang(text: string): string[][] | null {
  const dong = text.split("\n").map((d) => d.trim()).filter((d) => d.startsWith("|"));
  if (dong.length < 2) return null;
  const cat = (d: string) => d.slice(1, -1).split("|").map((o) => o.trim().replace(/\\\|/g, "|"));
  return [cat(dong[0]), ...dong.slice(2).map(cat)];
}

function oToChu(v: unknown): string {
  if (v === null || v === undefined) return "(null)";
  if (v instanceof Date) return v.toISOString().slice(0, 10);
  return String(v);
}

async function xoaHetBang(conn: oracledb.Connection) {
  await conn.execute(`
    BEGIN
      FOR t IN (SELECT table_name FROM user_tables) LOOP
        EXECUTE IMMEDIATE 'DROP TABLE "' || t.table_name || '" CASCADE CONSTRAINTS PURGE';
      END LOOP;
      FOR s IN (SELECT sequence_name FROM user_sequences
                WHERE sequence_name NOT LIKE 'ISEQ$$%') LOOP
        EXECUTE IMMEDIATE 'DROP SEQUENCE "' || s.sequence_name || '"';
      END LOOP;
      FOR i IN (SELECT index_name FROM user_indexes
                WHERE index_name NOT LIKE 'SYS_%') LOOP
        BEGIN EXECUTE IMMEDIATE 'DROP INDEX "' || i.index_name || '"';
        EXCEPTION WHEN OTHERS THEN NULL; END;
      END LOOP;
    END;`);
}

async function chayKhoi(conn: oracledb.Connection, sql: string) {
  let cuoi: { cot: string[]; dong: string[][] } | null = null;
  for (const cau of tachCauLenh(sql)) {
    const kq = await conn.execute(cau, [], { outFormat: oracledb.OUT_FORMAT_ARRAY });
    if (kq.metaData && kq.rows) {
      cuoi = {
        cot: kq.metaData.map((m) => m.name),
        dong: (kq.rows as unknown[][]).map((r) => r.map(oToChu)),
      };
    }
  }
  return cuoi;
}

async function main() {
  const loc = process.argv[2];
  const files = await lietKe(ROOT);
  const conn = await oracledb.getConnection(CONNECT);

  // Script dữ liệu mẫu: khối ```sql setup duy nhất trong khoá.
  let setup = "";
  for (const f of files) {
    const m = (await readFile(f, "utf8")).match(/```sql setup\r?\n([\s\S]*?)```/);
    if (m) setup = m[1];
  }
  if (!setup) {
    console.error("Không thấy khối ```sql setup nào trong khoá SQL.");
    process.exit(2);
  }
  const lamMoi = async () => {
    await xoaHetBang(conn);
    await chayKhoi(conn, setup);
    await conn.commit();
  };

  let khoi = 0;
  let loi = 0;
  for (const f of files.filter((f) => !loc || f.includes(loc))) {
    const text = (await readFile(f, "utf8")).split("\r\n").join("\n");
    const ten = path.relative(ROOT, f);
    for (const m of text.matchAll(/```sql( setup)?\n([\s\S]*?)```/g)) {
      if (m[1]) continue; // khối setup đã chạy trong lamMoi
      khoi++;
      const code = m[2];
      const dongSo = text.slice(0, m.index).split("\n").length;
      const laSai = code.trimStart().startsWith("-- SAI");
      const huaLoi = laSai && /--[^\n]*lỗi/.test(code.split("\n")[0]);

      await lamMoi();
      let kq: Awaited<ReturnType<typeof chayKhoi>> = null;
      let err: Error | null = null;
      try {
        kq = await chayKhoi(conn, code);
      } catch (e) {
        err = e as Error;
      }
      await conn.rollback();

      if (err && !laSai) {
        loi++;
        console.log(`\n${ten} (khối ở dòng ${dongSo}): ${err.message.split("\n")[0]}`);
        continue;
      }
      if (!err && huaLoi) {
        loi++;
        console.log(`\n${ten} (khối ở dòng ${dongSo}): comment hứa lỗi nhưng câu lệnh chạy được`);
        continue;
      }
      if (err || laSai) continue;

      // So kết quả với bảng trong <details> ngay sau khối (chỉ mục Thử ngay).
      const truoc = text.slice(0, m.index);
      const muc = [...truoc.matchAll(/^## (.+)$/gm)].pop()?.[1] ?? "";
      if (!muc.startsWith("Thử ngay")) continue;
      const sau = text.slice(m.index! + m[0].length);
      const details = sau.match(/<details>([\s\S]*?)<\/details>/);
      const bang = details ? docBang(details[1]) : null;
      if (!bang || !kq) continue;

      const [cotMong, ...dongMong] = bang;
      const lech: string[] = [];
      if (cotMong.join("|").toUpperCase() !== kq.cot.join("|"))
        lech.push(`cột: bài ghi ${cotMong.join(", ")} — thật là ${kq.cot.join(", ")}`);
      const a = dongMong.map((r) => r.join(" | "));
      const b = kq.dong.map((r) => r.join(" | "));
      if (a.join("\n") !== b.join("\n")) {
        lech.push("dòng khác nhau:");
        lech.push(...a.map((d) => `    bài : ${d}`));
        lech.push(...b.map((d) => `    thật: ${d}`));
      }
      if (lech.length) {
        loi++;
        console.log(`\n${ten} (khối ở dòng ${dongSo}): kết quả không khớp`);
        for (const l of lech) console.log(`  ${l}`);
      }
    }
  }

  await xoaHetBang(conn);
  await conn.close();
  console.log(`\n${khoi} khối SQL, ${loi} khối có vấn đề.`);
  process.exitCode = loi ? 1 : 0;
}

main().catch((e) => {
  console.error(e instanceof Error ? e.message : e);
  process.exit(2);
});
