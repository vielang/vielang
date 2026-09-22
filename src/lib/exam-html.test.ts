import { describe, expect, it } from "vitest";
import { sanitizeExamHtml, tagsIn, webpName } from "./exam-html";

const BASE = "/img/exams/102";

describe("làm sạch HTML của đề", () => {
  it("giữ đúng những thẻ đề thật dùng", () => {
    expect(sanitizeExamHtml("<b>가</b><br/><u style='x'>나</u>", BASE)).toBe("<b>가</b><br><u>나</u>");
  });

  it("'<보기>' là CHỮ Hàn trong ngoặc nhọn, không phải thẻ", () => {
    const out = sanitizeExamHtml("※ 다음을 듣고 <보기>와 같이 고르십시오.", BASE);
    expect(out).toBe("※ 다음을 듣고 &lt;보기&gt;와 같이 고르십시오.");
    expect(tagsIn(out)).toEqual([]);
  });

  it("khung viền đoạn văn đổi sang class của app, bỏ mọi thuộc tính", () => {
    expect(sanitizeExamHtml("<div class='question_outline' onclick='x()'>글</div>", BASE)).toBe(
      '<div class="exam-box">글</div>'
    );
  });

  it("ảnh trong đoạn văn trỏ về tài nguyên của app, đuôi WebP", () => {
    const out = sanitizeExamHtml("<img src='/asset/exam/exam102/images/102_1_63.png' width='800' alt='광고'>", BASE);
    expect(out).toBe('<img src="/img/exams/102/images/102_1_63.webp" alt="광고" loading="lazy">');
  });

  it("chặn thẻ lạ, chỉ giữ chữ", () => {
    expect(sanitizeExamHtml("<script>alert(1)</script><a href='x'>링크</a>", BASE)).toBe("alert(1)링크");
  });

  it("&nbsp thiếu dấu chấm phẩy của trang gốc vẫn thành khoảng trắng", () => {
    expect(sanitizeExamHtml("&nbsp&nbsp우리 가족", BASE)).toBe("우리 가족");
    expect(sanitizeExamHtml("가&nbsp;나", BASE)).toBe("가 나");
  });

  it("đổi đuôi ảnh sang WebP", () => {
    expect(webpName("102_1_15_1.png")).toBe("102_1_15_1.webp");
  });
});
