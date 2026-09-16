import { Languages } from "lucide-react";

/**
 * Khung sườn thư viện tiếng Anh — chưa có giáo trình nào để hiển thị.
 * Khi có file giáo trình tiếng Anh: thêm entry vào BOOKS (lib/books.ts) với
 * `lang: "en"`, tải ảnh/audio như quy trình SB_step hiện tại, rồi thay phần
 * empty-state dưới đây bằng nhóm theo cấp độ giống trang chủ
 * (`(main)/page.tsx`, xem groupByLevel).
 */
export default function EnglishLibraryPage() {
  return (
    <div className="flex flex-col gap-8">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">
          Thư viện tiếng Anh
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Giáo trình học tiếng Anh, tổ chức theo cấp độ tương tự chương trình
          KIIP tiếng Hàn — giáo trình chính đi kèm sách bài tập ở mỗi cấp.
        </p>
      </div>

      <div className="flex flex-col items-center justify-center gap-4 rounded-xl border border-dashed border-border py-24 text-center">
        <Languages className="size-10 text-muted-foreground" aria-hidden />
        <div>
          <h2 className="text-lg font-semibold">Sắp có nội dung</h2>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            Chưa có giáo trình tiếng Anh nào được thêm vào. Phần này sẽ hiển
            thị sách theo từng cấp độ, cùng định dạng với thư viện tiếng Hàn.
          </p>
        </div>
      </div>
    </div>
  );
}
