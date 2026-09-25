import { GUIDE_SECTIONS, type GuideSectionId } from "@/lib/guide";
import { SegmentedNav } from "@/components/layout/segmented-nav";

/**
 * Hàng tab chọn mục của cẩm nang (Visa, Trường đại học, Việc làm).
 *
 * Mỗi mục một trang riêng thay vì xếp chồng trên một trang: mục nào cũng sẽ
 * có hàng chục bài, xếp chồng thì muốn xem Việc làm phải cuộn qua hết Visa
 * và Trường. Nguồn lấy từ GUIDE_SECTIONS: thêm mục ở đó là có thêm tab.
 */
export function GuideTabs({ active }: { active: GuideSectionId }) {
  return (
    <SegmentedNav
      label="Mục cẩm nang"
      items={GUIDE_SECTIONS.map((section) => ({
        key: section.id,
        label: section.title,
        href: `/cam-nang/${section.id}`,
        active: section.id === active,
      }))}
    />
  );
}
