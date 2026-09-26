/**
 * Các mục của Cẩm nang — tách khỏi lib/guide.ts vì file đó `import` cả nội
 * dung bài (guide.json, HTML của hàng chục bài). Header và thanh điều hướng
 * cần danh sách mục ở MỌI trang, kéo theo cả nội dung bài vào gói JS chung là
 * phí. lib/guide.ts dùng lại từ đây.
 */
export type GuideSectionId = string;

export interface GuideSection {
  id: GuideSectionId;
  title: string;
  /** Một câu dưới tiêu đề mục. */
  description: string;
}

/** Thứ tự hiển thị các mục. Thêm mục = thêm thư mục content/cam-nang/<id> + một dòng ở đây. */
// Rỗng thì tab Cẩm nang tự ẩn (xem lib/nav.ts). Mục đầu là đích của tab và của `/cam-nang`.
export const GUIDE_SECTIONS: readonly GuideSection[] = [
  { id: "lo-trinh", title: "Lộ trình nghề", description: "Lộ trình học và phát triển nghề lập trình." },
  { id: "phong-van", title: "Phỏng vấn", description: "Chuẩn bị và câu hỏi phỏng vấn kèm đáp án." },
  { id: "cong-cu", title: "Công cụ", description: "Công cụ làm việc hằng ngày của developer." },
  { id: "viec-lam", title: "Việc làm IT", description: "CV, portfolio, tìm việc, lương, remote." },
];
