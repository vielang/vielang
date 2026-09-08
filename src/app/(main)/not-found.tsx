import Link from "next/link";
import { BookOpenText } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function MainNotFound() {
  return (
    <div className="flex flex-col items-center justify-center gap-4 py-24 text-center">
      <BookOpenText className="size-10 text-muted-foreground" aria-hidden />
      <div>
        <h1 className="text-xl font-semibold">Không tìm thấy sách</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Sách bạn tìm không tồn tại trong thư viện.
        </p>
      </div>
      <Button asChild>
        <Link href="/">Về thư viện</Link>
      </Button>
    </div>
  );
}
