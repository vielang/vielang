#!/usr/bin/env tsx
/**
 * Thu thập danh bạ cơ sở vận hành KIIP (사회통합프로그램 운영기관) từ Socinet.
 *
 * Nguồn: trang "운영기관현황" của 사회통합정보망 (socinet.go.kr) — CÔNG KHAI,
 * không cần đăng nhập. Lưu ý phạm vi: trang này chỉ có DANH BẠ cơ sở (tên,
 * người phụ trách, điện thoại, địa chỉ, văn phòng XNC quản lý). Lịch mở lớp
 * và số chỗ trống nằm sau màn đăng nhập (과정신청) nên KHÔNG lấy ở đây.
 *
 * Socinet là trang JSP render phía server, không có JSON API — phải POST form
 * rồi parse HTML. Bù lại `q_listSize` nhận giá trị tuỳ ý, nên xin 1 trang thật
 * lớn để chỉ gọi ĐÚNG 1 request thay vì 49 lần phân trang 10 bản ghi.
 */
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { JSDOM } from "jsdom";

const LIST_URL = "https://www.socinet.go.kr/soci/oper/mgmt/sy/SyOperOrgnList.jsp";
const OUT_PATH = path.resolve(process.cwd(), "content", "kiip-orgs.json");

/** Mặc định trang là 10 bản ghi/trang; tổng hiện tại 484. */
const PAGE_SIZE = 2000;

export interface KiipOrg {
  /** 구분 — phân loại cơ sở (일반, 거점 ...). */
  type: string;
  /** 상위기관명 — cơ quan chủ quản. */
  parentOrg: string;
  /** 운영기관명 — tên cơ sở trực tiếp dạy. */
  name: string;
  /** 담당자 */
  contact: string;
  /** 연락처 */
  phone: string;
  /** 주소 */
  address: string;
  /** 사무소 — văn phòng xuất nhập cảnh quản lý (đã bỏ khoảng trắng căn lề). */
  office: string;
}

/** Gộp mọi khoảng trắng (kể cả &nbsp;) thành 1 dấu cách rồi cắt 2 đầu. */
function clean(text: string): string {
  return text.replace(/ /g, " ").replace(/\s+/g, " ").trim();
}

async function fetchListHtml(): Promise<string> {
  const body = new URLSearchParams({
    q_global_menu_id: "S_SIP_SUB07",
    q_pageNo: "1",
    q_listSize: String(PAGE_SIZE),
    q_offi: "",
    q_oper_orgn_nm: "",
  });

  const res = await fetch(LIST_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
      // Trang từ chối phục vụ nếu thiếu UA trông giống trình duyệt.
      "User-Agent":
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 " +
        "(KHTML, like Gecko) Chrome/126.0 Safari/537.36",
    },
    body,
  });

  if (!res.ok) throw new Error(`Socinet trả về HTTP ${res.status}`);
  return res.text();
}

/**
 * Bảng kết quả nhận diện bằng ô `td.table_body` — không dựa vào thứ tự bảng
 * trong trang, vì trang còn vài bảng khác dùng cho layout/ô tìm kiếm.
 */
function parseOrgs(html: string): KiipOrg[] {
  const { document } = new JSDOM(html).window;
  const orgs: KiipOrg[] = [];

  for (const row of document.querySelectorAll("tr")) {
    const cells = row.querySelectorAll("td.table_body");
    // 8 cột: 구분 | 상위기관명 | 운영기관명 | 담당자 | 연락처 | 주소 |
    //        약도보기 (luôn rỗng, bỏ) | 사무소
    if (cells.length !== 8) continue;
    const text = (i: number) => clean(cells[i].textContent ?? "");

    orgs.push({
      type: text(0),
      parentOrg: text(1),
      name: text(2),
      contact: text(3),
      phone: text(4),
      address: text(5),
      office: text(7).replace(/\s/g, ""), // "서  울" -> "서울"
    });
  }

  return orgs;
}

/** Số tổng trang tự in ra ("[총 484 건]") — dùng để phát hiện parse thiếu. */
function reportedTotal(html: string): number | null {
  const m = html.match(/\[총\s*([\d,]+)\s*건\]/);
  return m ? Number(m[1].replace(/,/g, "")) : null;
}

async function main() {
  const html = await fetchListHtml();
  const orgs = parseOrgs(html);
  const total = reportedTotal(html);

  if (total !== null && total !== orgs.length) {
    throw new Error(
      `Trang báo ${total} cơ sở nhưng parse được ${orgs.length} — nhiều khả ` +
        `năng Socinet đổi cấu trúc bảng, xem lại parseOrgs().`
    );
  }

  await writeFile(OUT_PATH, JSON.stringify(orgs, null, 2) + "\n", "utf8");

  const byOffice = new Map<string, number>();
  for (const o of orgs) byOffice.set(o.office, (byOffice.get(o.office) ?? 0) + 1);

  console.log(`  ${orgs.length} cơ sở -> ${path.relative(process.cwd(), OUT_PATH)}`);
  for (const [office, count] of [...byOffice].sort((a, b) => b[1] - a[1])) {
    console.log(`    ${office.padEnd(12)} ${count}`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
