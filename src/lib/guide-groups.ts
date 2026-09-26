/**
 * Nhóm bài trong từng mục cẩm nang, theo thứ tự hiển thị. Bài khai báo nhóm
 * bằng dòng `group:` ở đầu file .md (xem content/cam-nang/FORMAT.md).
 *
 * Tách riêng khỏi lib/guide.ts vì bước build (scripts/build-content.ts) cần
 * danh sách này để kiểm tên nhóm, mà guide.ts lại `import` guide.json — thứ
 * chính bước build sinh ra.
 *
 * Mục không có ở đây (Trường đại học) thì không chia nhóm: trường lọc theo
 * vùng, loại trường, TOPIK thay vì theo nhóm.
 */
export const GUIDE_GROUPS: Partial<Record<string, readonly string[]>> = {};
