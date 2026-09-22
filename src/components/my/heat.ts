/**
 * Thang đậm nhạt dùng chung cho lịch học và ô tiến độ từng bài: MỘT màu
 * (`primary`), nhạt → đậm. Đại lượng ở đây là "nhiều hay ít", không phải
 * "loại nào", nên không dùng nhiều màu — nhiều màu thì mắt đi tìm ý nghĩa
 * cho từng màu.
 *
 * `primary` của theme là màu trung tính (gần đen ở nền sáng, gần trắng ở nền
 * tối), nên cùng một bậc độ mờ đọc đúng ở cả hai chế độ.
 */
export const HEAT_CLASSES = [
  "bg-muted",
  "bg-primary/20",
  "bg-primary/40",
  "bg-primary/65",
  "bg-primary",
] as const;
