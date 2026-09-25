/**
 * Các mục của Cẩm nang — tách khỏi lib/guide.ts vì file đó `import` cả nội
 * dung bài (guide.json, HTML của hàng chục bài). Header và thanh điều hướng
 * cần danh sách mục ở MỌI trang, kéo theo cả nội dung bài vào gói JS chung là
 * phí. lib/guide.ts dùng lại từ đây.
 */
export type GuideSectionId = "visa" | "truong" | "viec-lam";

export interface GuideSection {
  id: GuideSectionId;
  title: string;
  /** Một câu dưới tiêu đề mục. */
  description: string;
}

/** Thứ tự hiển thị các mục. Thêm mục = thêm thư mục + một dòng ở đây và ở build-content. */
export const GUIDE_SECTIONS: readonly GuideSection[] = [
  {
    id: "visa",
    title: "Visa",
    description: "Điều kiện, hồ sơ và các bước cho từng loại visa.",
  },
  {
    id: "truong",
    title: "Trường đại học",
    description: "Tuyển sinh, học phí, học bổng và trường tiếng của từng trường.",
  },
  {
    id: "viec-lam",
    title: "Việc làm",
    description: "Đi làm hợp pháp, quyền lợi người lao động và tìm việc an toàn.",
  },
];
