/**
 * Dữ liệu có cấu trúc (schema.org) cho máy tìm kiếm — cách Next khuyên dùng
 * là một thẻ `<script>` ngay trong trang.
 *
 * Thay `<` bằng `<`: nội dung lấy từ file bài viết, một chuỗi có
 * `</script>` sẽ đóng thẻ sớm và phần sau thành HTML chạy được.
 */
export function JsonLd({ data }: { data: object | object[] }) {
  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(data).replace(/</g, "\\u003c") }}
    />
  );
}
