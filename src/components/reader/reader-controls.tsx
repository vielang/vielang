"use client";

import Link from "next/link";
import { useState } from "react";
import {
  Bookmark,
  BookOpen,
  ChevronLeft,
  ChevronRight,
  Columns2,
  NotebookText,
  Pen,
  RotateCcw,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { Book } from "@/lib/books";
import type { Chapter } from "@/lib/chapters";

export function ReaderControls({
  visible,
  book,
  page,
  pages,
  chapters,
  currentLesson,
  isBookmarked,
  jumpOpen,
  onJumpOpenChange,
  noteHasContent,
  noteLabel,
  onNoteToggle,
  drawActive,
  drawHasContent,
  onDrawToggle,
  onPrev,
  onNext,
  onJump,
  onToggleBookmark,
  onResetZoom,
  showLayoutToggle,
  pageLayout,
  onTogglePageLayout,
}: {
  visible: boolean;
  book: Book;
  page: number;
  /** Trang đang hiện — 1 hoặc 2 phần tử tuỳ chế độ xem, dùng để hiện nhãn "Trang N" / "Trang N–M". */
  pages: number[];
  /** Ranh giới bài học (xem lib/chapters.ts) — [] nếu sách không xác định được */
  chapters: Chapter[];
  /** Bài học chứa trang hiện tại — null nếu trang thuộc phần mở đầu (chưa vào bài nào) */
  currentLesson: number | null;
  isBookmarked: boolean;
  jumpOpen: boolean;
  onJumpOpenChange: (open: boolean) => void;
  /** Có bài giảng để đọc (gốc hoặc do người dùng viết) — chấm báo trên nút. */
  noteHasContent: boolean;
  /** Chữ trên tooltip — đổi theo trạng thái panel (đóng/mở/ở cửa sổ riêng). */
  noteLabel: string;
  onNoteToggle: () => void;
  /** Đang bật chế độ vẽ tay lên ảnh trang. */
  drawActive: boolean;
  /** Trang đang hiện đã có nét vẽ — chấm báo trên nút bút. */
  drawHasContent: boolean;
  onDrawToggle: () => void;
  onPrev: () => void;
  onNext: () => void;
  onJump: (page: number) => void;
  onToggleBookmark: () => void;
  onResetZoom: () => void;
  /** Chỉ hiện nút đổi 1/2 trang khi màn hình đủ rộng — màn hẹp thì 2 trang không đọc được. */
  showLayoutToggle: boolean;
  pageLayout: "single" | "double";
  onTogglePageLayout: () => void;
}) {
  const [pendingPage, setPendingPage] = useState(page);

  function openJumpDialog() {
    setPendingPage(page);
    onJumpOpenChange(true);
  }

  return (
    <>
      {/* Top bar */}
      <div
        className={cn(
          "fixed inset-x-0 top-0 z-20 flex items-center justify-between gap-2 bg-gradient-to-b from-black/70 to-transparent p-3 text-white transition-transform duration-200",
          visible ? "translate-y-0" : "pointer-events-none -translate-y-full"
        )}
      >
        <Button asChild variant="ghost" size="icon" className="text-white hover:bg-white/10 hover:text-white">
          <Link
            href={
              currentLesson
                ? `/books/${book.id}#bai-${currentLesson}`
                : `/books/${book.id}`
            }
            aria-label="Về danh sách trang"
          >
            <X className="size-5" aria-hidden />
          </Link>
        </Button>
        <div className="min-w-0 flex-1 truncate text-center text-sm font-medium">
          {book.titleVi}
        </div>
        <div className="flex items-center gap-1">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="relative text-white hover:bg-white/10 hover:text-white"
                onClick={onNoteToggle}
                aria-label={noteLabel}
              >
                <NotebookText className="size-5" aria-hidden />
                {/* Chấm báo trang đã có bài giảng — nút không bao giờ bị
                    disable nữa vì trang trống vẫn mở được để tự soạn. */}
                {noteHasContent && (
                  <span
                    className="absolute right-1.5 top-1.5 size-1.5 rounded-full bg-sky-400"
                    aria-hidden
                  />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>{noteLabel}</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className={cn(
                  "relative text-white hover:bg-white/10 hover:text-white",
                  // Đang vẽ thì nút sáng hẳn lên: thanh công cụ vẽ nằm mãi
                  // dưới đáy màn, cần một dấu hiệu ngay chỗ vừa bấm để biết
                  // chế độ đang bật.
                  drawActive && "bg-white text-neutral-900 hover:bg-white hover:text-neutral-900"
                )}
                onClick={onDrawToggle}
                aria-label={drawActive ? "Tắt chế độ vẽ" : "Vẽ lên trang này"}
                aria-pressed={drawActive}
              >
                <Pen className="size-5" aria-hidden />
                {drawHasContent && !drawActive && (
                  <span
                    className="absolute right-1.5 top-1.5 size-1.5 rounded-full bg-sky-400"
                    aria-hidden
                  />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {drawActive ? "Tắt chế độ vẽ" : "Vẽ lên trang này"}
            </TooltipContent>
          </Tooltip>

          <Button
            variant="ghost"
            size="icon"
            className="text-white hover:bg-white/10 hover:text-white"
            onClick={onToggleBookmark}
            aria-label={isBookmarked ? "Bỏ đánh dấu trang" : "Đánh dấu trang"}
            aria-pressed={isBookmarked}
          >
            <Bookmark
              className={cn("size-5", isBookmarked && "fill-current")}
              aria-hidden
            />
          </Button>

          {showLayoutToggle && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-white hover:bg-white/10 hover:text-white"
                  onClick={onTogglePageLayout}
                  aria-label={
                    pageLayout === "double" ? "Xem 1 trang" : "Xem 2 trang"
                  }
                  aria-pressed={pageLayout === "double"}
                >
                  {pageLayout === "double" ? (
                    <Columns2 className="size-5" aria-hidden />
                  ) : (
                    <BookOpen className="size-5" aria-hidden />
                  )}
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                {pageLayout === "double" ? "Xem 1 trang" : "Xem 2 trang"}
              </TooltipContent>
            </Tooltip>
          )}
        </div>
      </div>

      {/* Bottom bar */}
      <div
        className={cn(
          "fixed inset-x-0 bottom-0 z-20 flex items-center justify-center gap-1 bg-gradient-to-t from-black/70 to-transparent p-3 text-white transition-transform duration-200",
          visible ? "translate-y-0" : "pointer-events-none translate-y-full"
        )}
      >
        <Button
          variant="ghost"
          size="icon"
          className="text-white hover:bg-white/10 hover:text-white disabled:opacity-30"
          onClick={onPrev}
          disabled={page <= 1}
          aria-label="Trang trước"
        >
          <ChevronLeft className="size-5" aria-hidden />
        </Button>

        <Button
          variant="ghost"
          className="text-white hover:bg-white/10 hover:text-white tabular-nums"
          onClick={openJumpDialog}
        >
          {currentLesson && (
            <span className="text-white/70">Bài {currentLesson} ·</span>
          )}
          Trang {pages.length === 2 ? `${pages[0]}–${pages[1]}` : pages[0]}/
          {book.totalPages}
        </Button>

        <Button
          variant="ghost"
          size="icon"
          className="text-white hover:bg-white/10 hover:text-white disabled:opacity-30"
          onClick={onNext}
          disabled={page >= book.totalPages}
          aria-label="Trang sau"
        >
          <ChevronRight className="size-5" aria-hidden />
        </Button>

        <Button
          variant="ghost"
          size="icon"
          className="ml-2 text-white hover:bg-white/10 hover:text-white"
          onClick={onResetZoom}
          aria-label="Đặt lại độ phóng to"
        >
          <RotateCcw className="size-4" aria-hidden />
        </Button>
      </div>

      <Dialog open={jumpOpen} onOpenChange={onJumpOpenChange}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>Nhảy tới trang</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col gap-4 py-2">
            {chapters.length > 0 && (
              <div className="flex flex-col gap-1.5">
                <span className="text-xs text-muted-foreground">Theo bài học</span>
                <div className="flex flex-wrap gap-1.5">
                  {chapters.map((ch) => (
                    <Button
                      key={ch.lesson}
                      type="button"
                      size="sm"
                      variant={ch.lesson === currentLesson ? "default" : "outline"}
                      className="h-7 rounded-full px-2.5 text-xs"
                      onClick={() => {
                        onJump(ch.startPage);
                        onJumpOpenChange(false);
                      }}
                    >
                      Bài {ch.lesson}
                    </Button>
                  ))}
                </div>
                <span className="text-xs text-muted-foreground">Theo số trang</span>
              </div>
            )}
            <Slider
              min={1}
              max={book.totalPages}
              step={1}
              value={[pendingPage]}
              onValueChange={([v]) => setPendingPage(v)}
            />
            <div className="flex items-center gap-2">
              <Input
                type="number"
                min={1}
                max={book.totalPages}
                value={pendingPage}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  if (!Number.isNaN(v)) {
                    setPendingPage(Math.min(Math.max(v, 1), book.totalPages));
                  }
                }}
                className="w-24"
              />
              <span className="text-sm text-muted-foreground">
                / {book.totalPages} trang
              </span>
            </div>
          </div>
          <DialogFooter>
            <Button
              onClick={() => {
                onJump(pendingPage);
                onJumpOpenChange(false);
              }}
            >
              Đi tới trang {pendingPage}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
