import type { Metadata } from "next";
import { InstallGuide } from "@/components/layout/install-guide";

export const metadata: Metadata = {
  title: "Cài app vào máy",
  description:
    "Hướng dẫn cài VieTopik vào màn hình chính trên iPhone, Android và máy tính để đọc sách tiếng Hàn cả khi không có mạng.",
};

export default function InstallPage() {
  return (
    <div className="flex max-w-xl flex-col gap-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Cài app vào máy</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Thêm VieTopik vào màn hình chính để mở nhanh như một ứng dụng thường,
          không còn thanh địa chỉ che mất chỗ đọc.
        </p>
      </div>
      <InstallGuide />
    </div>
  );
}
