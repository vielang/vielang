"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";

export function NoteSheet({
  open,
  onOpenChange,
  content,
  page,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  content: string | null;
  page: number;
}) {
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="bottom" className="flex h-[85vh] flex-col">
        <SheetHeader>
          <SheetTitle>Bài giảng — Trang {page}</SheetTitle>
          <SheetDescription>
            Giải thích nội dung trang này bằng tiếng Việt.
          </SheetDescription>
        </SheetHeader>
        <div className="flex-1 overflow-y-auto px-4 pb-8">
          {content ? (
            <div className="prose prose-sm dark:prose-invert max-w-none prose-headings:font-heading prose-table:text-sm">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {content}
              </ReactMarkdown>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              Trang này chưa có bài giảng.
            </p>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
