"use client";

/**
 * Trỏ Excalidraw vào bản font tự phục vụ ở `public/excalidraw/` thay vì CDN
 * esm.sh mặc định — xem `scripts/copy-excalidraw-fonts.ts` để biết vì sao.
 *
 * Module này chỉ có tác dụng phụ, và phải được import TRƯỚC gói Excalidraw
 * (xem `components/reader/note-draw.tsx`): danh sách URL của mỗi @font-face
 * được dựng ở lần đầu chạm vào bảng font, nên đặt sau là muộn.
 *
 * Địa chỉ CDN vẫn nằm lại trong `src` của @font-face như lựa chọn dự phòng —
 * thư mục này thiếu file thì font chậm chứ không mất.
 */
declare global {
  interface Window {
    EXCALIDRAW_ASSET_PATH?: string | string[];
  }
}

if (typeof window !== "undefined") {
  window.EXCALIDRAW_ASSET_PATH = "/excalidraw/";
}

export {};
