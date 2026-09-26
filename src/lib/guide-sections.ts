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
// Chủ đề IT sẽ thêm ở đây. Rỗng thì tab Cẩm nang tự ẩn (xem lib/nav.ts).
export const GUIDE_SECTIONS: readonly GuideSection[] = [];
