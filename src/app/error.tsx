"use client";

import { useEffect } from "react";
import { AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <div className="flex min-h-dvh flex-col items-center justify-center gap-4 p-6 text-center">
      <AlertTriangle className="size-10 text-destructive" aria-hidden />
      <div>
        <h1 className="text-xl font-semibold">Đã có lỗi xảy ra</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Vui lòng thử lại. Nếu vẫn lỗi, hãy tải lại trang.
        </p>
      </div>
      <Button onClick={() => reset()}>Thử lại</Button>
    </div>
  );
}
