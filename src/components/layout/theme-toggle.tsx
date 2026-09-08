"use client";

import { useTheme } from "next-themes";
import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label="Đổi giao diện sáng/tối"
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
    >
      {/* next-themes gắn class .dark lên <html> trước khi paint (script
          chặn render), nên CSS chọn đúng icon ngay từ lần render đầu —
          không cần state "đã mount" + effect để tránh mismatch hydration. */}
      <Moon className="hidden size-5 dark:block" />
      <Sun className="size-5 dark:hidden" />
    </Button>
  );
}
