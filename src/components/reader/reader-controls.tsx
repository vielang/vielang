"use client";

import Link from "next/link";
import { useState } from "react";
import {
  Bookmark,
  BookOpen,
  ChevronLeft,
  ChevronRight,
  CircleQuestionMark,
  Columns2,
  Mic,
  MoreVertical,
  NotebookText,
  Pen,
  RotateCcw,
  X,
  type LucideIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Slider } from "@/components/ui/slider";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useZoomStore } from "@/lib/zoom-store";
import type { Book } from "@/lib/books";
import type { Chapter } from "@/lib/chapters";

/** Một công cụ của trang đọc — hiện thành icon (máy tính) hoặc dòng có chữ (menu). */
interface Tool {
  icon: LucideIcon;
  label: string;
  onSelect: () => void;
  /** Đang bật (vẽ, ghi âm, đã đánh dấu) — nút sáng lên. */
  active?: boolean;
  /** Trang này đã có nội dung (bài giảng, nét vẽ, bản ghi) — chấm báo. */
  dot?: boolean;
}

const BAR = "bg-black/55 text-white backdrop-blur";
const GHOST = "text-white hover:bg-white/10 hover:text-white";

function ToolButton({ tool }: { tool: Tool }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className={cn(
            "relative",
            GHOST,
            tool.active && "bg-white text-neutral-900 hover:bg-white hover:text-neutral-900"
          )}
          onClick={tool.onSelect}
          aria-label={tool.label}
          aria-pressed={tool.active}
        >
          <tool.icon className="size-5" aria-hidden />
          {tool.dot && !tool.active && (
            <span className="absolute top-1.5 right-1.5 size-1.5 rounded-full bg-sky-400" aria-hidden />
          )}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{tool.label}</TooltipContent>
    </Tooltip>
  );
}

/**
 * Thanh điều khiển trang đọc — chỉ hai thanh, mỗi thanh một việc:
 *
 * - TRÊN: thoát, tên sách, và công cụ. Máy tính hiện thẳng mấy công cụ hay
 *   dùng thành icon; điện thoại gom hết vào nút "⋮" (menu có CHỮ, khỏi đoán
 *   icon) — màn hẹp mà bày 6 icon thì vừa chật vừa dễ bấm nhầm.
 * - DƯỚI: chỉ điều hướng trang. Nút đặt lại phóng to chỉ hiện khi đang
 *   phóng to; "Cách dùng" nằm trong menu.
 *
 * Nền thanh là một dải mờ đặc (không phải gradient trong suốt): chữ của
 * trang sách hay lọt vào sau icon, nhìn như icon đè lên bài.
 */
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
  recordOpen,
  recordHasContent,
  onRecordToggle,
  onPrev,
  onNext,
  onJump,
  onToggleBookmark,
  onResetZoom,
  onOpenHelp,
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
  /** Bảng ghi âm đang mở. */
  recordOpen: boolean;
  /** Trang đang hiện đã có bản ghi — chấm báo trên nút micro. */
  recordHasContent: boolean;
  onRecordToggle: () => void;
  onPrev: () => void;
  onNext: () => void;
  onJump: (page: number) => void;
  onToggleBookmark: () => void;
  onResetZoom: () => void;
  onOpenHelp: () => void;
  /** Chỉ hiện nút đổi 1/2 trang khi màn hình đủ rộng — màn hẹp thì 2 trang không đọc được. */
  showLayoutToggle: boolean;
  pageLayout: "single" | "double";
  onTogglePageLayout: () => void;
}) {
  const [pendingPage, setPendingPage] = useState(page);
  const [menuOpen, setMenuOpen] = useState(false);
  const zoomed = useZoomStore((s) => Math.abs(s.scale - 1) > 0.01);

  function openJumpDialog() {
    setPendingPage(page);
    onJumpOpenChange(true);
  }

  const tools: Tool[] = [
    { icon: NotebookText, label: noteLabel, onSelect: onNoteToggle, dot: noteHasContent },
    {
      icon: Pen,
      label: drawActive ? "Tắt chế độ vẽ" : "Vẽ lên trang này",
      onSelect: onDrawToggle,
      active: drawActive,
      dot: drawHasContent,
    },
    {
      icon: Mic,
      label: recordOpen ? "Đóng bảng ghi âm" : "Ghi âm trang này",
      onSelect: onRecordToggle,
      active: recordOpen,
      dot: recordHasContent,
    },
    {
      icon: Bookmark,
      label: isBookmarked ? "Bỏ đánh dấu trang" : "Đánh dấu trang",
      onSelect: onToggleBookmark,
      active: isBookmarked,
    },
  ];
  const layoutTool: Tool | null = showLayoutToggle
    ? {
        icon: pageLayout === "double" ? Columns2 : BookOpen,
        label: pageLayout === "double" ? "Xem 1 trang" : "Xem 2 trang",
        onSelect: onTogglePageLayout,
      }
    : null;

  return (
    <>
      {/* Thanh trên */}
      <div
        className={cn(
          "fixed inset-x-0 top-0 z-20 flex items-center justify-between gap-2 p-2 pt-[max(0.5rem,env(safe-area-inset-top))] transition-transform duration-200",
          BAR,
          visible ? "translate-y-0" : "pointer-events-none -translate-y-full"
        )}
      >
        <Button asChild variant="ghost" size="icon" className={GHOST}>
          <Link
            href={currentLesson ? `/books/${book.id}#bai-${currentLesson}` : `/books/${book.id}`}
            aria-label="Về danh sách trang"
          >
            <X className="size-5" aria-hidden />
          </Link>
        </Button>

        <div className="min-w-0 flex-1 truncate text-center text-sm font-medium">{book.titleVi}</div>

        <div className="flex items-center gap-1">
          {/* Máy tính: công cụ hay dùng bày sẵn. Điện thoại: nằm trong menu. */}
          <span className="hidden items-center gap-1 sm:flex">
            {tools.map((tool) => (
              <ToolButton key={tool.label} tool={tool} />
            ))}
            {layoutTool && <ToolButton tool={layoutTool} />}
          </span>

          <Popover open={menuOpen} onOpenChange={setMenuOpen}>
            <PopoverTrigger asChild>
              <Button variant="ghost" size="icon" className={GHOST} aria-label="Công cụ trang đọc">
                <MoreVertical className="size-5" aria-hidden />
              </Button>
            </PopoverTrigger>
            <PopoverContent align="end" className="w-56 p-1">
              <span className="flex flex-col sm:hidden">
                {[...tools, ...(layoutTool ? [layoutTool] : [])].map((tool) => (
                  <Button
                    key={tool.label}
                    variant="ghost"
                    size="sm"
                    className="w-full justify-start"
                    onClick={() => {
                      setMenuOpen(false);
                      tool.onSelect();
                    }}
                  >
                    <tool.icon className="size-4" aria-hidden />
                    {tool.label}
                    {tool.dot && !tool.active && (
                      <span className="ml-auto size-1.5 rounded-full bg-sky-500" aria-hidden />
                    )}
                  </Button>
                ))}
                <span className="my-1 h-px bg-border" aria-hidden />
              </span>
              <Button
                variant="ghost"
                size="sm"
                className="w-full justify-start"
                onClick={() => {
                  setMenuOpen(false);
                  onOpenHelp();
                }}
              >
                <CircleQuestionMark className="size-4" aria-hidden />
                Cách dùng trang đọc
              </Button>
            </PopoverContent>
          </Popover>
        </div>
      </div>

      {/* Thanh dưới: chỉ điều hướng trang. */}
      <div
        className={cn(
          "fixed inset-x-0 bottom-0 z-20 flex items-center justify-center gap-1 p-2 pb-[max(0.5rem,env(safe-area-inset-bottom))] transition-transform duration-200",
          BAR,
          visible ? "translate-y-0" : "pointer-events-none translate-y-full"
        )}
      >
        <Button
          variant="ghost"
          size="icon"
          className={cn(GHOST, "disabled:opacity-30")}
          onClick={onPrev}
          disabled={page <= 1}
          aria-label="Trang trước"
        >
          <ChevronLeft className="size-5" aria-hidden />
        </Button>

        <Button variant="ghost" className={cn(GHOST, "tabular-nums")} onClick={openJumpDialog}>
          {currentLesson && <span className="text-white/70">Bài {currentLesson} ·</span>}
          Trang {pages.length === 2 ? `${pages[0]}–${pages[1]}` : pages[0]}/{book.totalPages}
        </Button>

        <Button
          variant="ghost"
          size="icon"
          className={cn(GHOST, "disabled:opacity-30")}
          onClick={onNext}
          disabled={page >= book.totalPages}
          aria-label="Trang sau"
        >
          <ChevronRight className="size-5" aria-hidden />
        </Button>

        {/* Chỉ hiện khi đang phóng to — lúc xem cỡ thường thì nút này vô nghĩa. */}
        {zoomed && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className={cn("ml-2", GHOST)}
                onClick={onResetZoom}
                aria-label="Đặt lại độ phóng to"
              >
                <RotateCcw className="size-4" aria-hidden />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Đặt lại độ phóng to</TooltipContent>
          </Tooltip>
        )}
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
              <span className="text-sm text-muted-foreground">/ {book.totalPages} trang</span>
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
