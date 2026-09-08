import Link from "next/link";
import { BookOpenText } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function NotFound() {
  return (
    <div className="flex min-h-dvh flex-col items-center justify-center gap-4 p-6 text-center">
      <BookOpenText className="size-10 text-muted-foreground" aria-hidden />
      <div>
        <h1 className="text-xl font-semibold">Không tìm thấy trang</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Trang hoặc sách bạn tìm không tồn tại.
        </p>
      </div>
      <Button asChild>
        <Link href="/">Về thư viện</Link>
      </Button>
    </div>
  );
}
