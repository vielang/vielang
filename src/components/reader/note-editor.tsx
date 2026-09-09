"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import {
  EditorContent,
  useEditor,
  useEditorState,
  type Editor,
} from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { Placeholder } from "@tiptap/extensions";
import { Highlight } from "@tiptap/extension-highlight";
import { TextAlign } from "@tiptap/extension-text-align";
import { TableKit } from "@tiptap/extension-table";
import {
  Bold,
  Check,
  Code,
  Highlighter,
  Italic,
  Link2,
  List,
  ListOrdered,
  Loader2,
  Minus,
  Quote,
  Redo2,
  Strikethrough,
  Table as TableIcon,
  Underline,
  Undo2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import { Separator } from "@/components/ui/separator";
import { NOTE_PROSE_CLASS, useNoteStore } from "@/lib/note-store";

/**
 * Ghi xuống localStorage sau khi ngừng gõ — không có nút "Lưu": bản sửa là
 * của riêng người dùng trên máy họ, mất chữ vì quên bấm lưu là lỗi tệ nhất
 * ở đây. Muốn bỏ bản sửa thì dùng "Khôi phục bản gốc" (xem note-sheet).
 */
const AUTOSAVE_DELAY_MS = 600;

const extensions = [
  StarterKit.configure({
    heading: { levels: [2, 3, 4] },
    link: {
      openOnClick: false,
      HTMLAttributes: { rel: "noopener noreferrer", target: "_blank" },
    },
  }),
  Placeholder.configure({
    placeholder: "Viết bài giảng cho trang này…",
  }),
  Highlight,
  TextAlign.configure({ types: ["heading", "paragraph"] }),
  TableKit.configure({ table: { resizable: true } }),
];

export function NoteEditor({
  bookId,
  page,
  initialHtml,
}: {
  bookId: string;
  page: number;
  /** Nội dung mở editor lên: bản sửa cục bộ nếu có, không thì bản gốc. */
  initialHtml: string;
}) {
  const saveNote = useNoteStore((s) => s.saveNote);
  const [status, setStatus] = useState<"idle" | "saving" | "saved">("idle");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingRef = useRef<string | null>(null);

  const flush = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    if (pendingRef.current === null) return;
    saveNote(bookId, page, pendingRef.current);
    pendingRef.current = null;
    setStatus("saved");
  }, [bookId, page, saveNote]);

  const editor = useEditor({
    extensions,
    content: initialHtml,
    // Editor chỉ chạy ở client (note-sheet import động với ssr: false) nhưng
    // vẫn phải tắt render đồng bộ, nếu không Tiptap cảnh báo hydration.
    immediatelyRender: false,
    editorProps: {
      attributes: {
        class: `${NOTE_PROSE_CLASS} min-h-[40vh] px-4 py-3 outline-none`,
      },
    },
    onUpdate: ({ editor }) => {
      pendingRef.current = editor.getHTML();
      setStatus("saving");
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(flush, AUTOSAVE_DELAY_MS);
    },
  });

  // Đóng sheet / chuyển trang khi vừa gõ xong: ghi nốt phần còn treo trong
  // debounce thay vì để mất.
  useEffect(() => flush, [flush]);

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border border-border bg-card">
      {/* `immediatelyRender: false` -> editor còn null ở lần render đầu.
          NoteToolbar chỉ mount khi đã có editor thật: `useEditorState` bên
          trong nó subscribe ngay lúc mount, không bắt kịp editor null -> có. */}
      {editor ? (
        <NoteToolbar editor={editor} />
      ) : (
        <div className="h-11 border-b border-border" aria-hidden />
      )}
      <div className="min-h-0 flex-1 overflow-y-auto">
        {editor ? (
          <EditorContent editor={editor} />
        ) : (
          <div className="min-h-[40vh] animate-pulse bg-muted/40" />
        )}
      </div>
      <div
        className="flex items-center gap-1.5 border-t border-border px-3 py-1.5 text-xs text-muted-foreground"
        aria-live="polite"
      >
        {status === "saving" && (
          <>
            <Loader2 className="size-3.5 animate-spin" aria-hidden />
            Đang lưu…
          </>
        )}
        {status === "saved" && (
          <>
            <Check className="size-3.5" aria-hidden />
            Đã lưu trên thiết bị này
          </>
        )}
        {status === "idle" && "Thay đổi tự lưu trên thiết bị này."}
      </div>
    </div>
  );
}

/** Trạng thái nút toolbar — `useEditorState` để chỉ render lại khi đổi. */
function NoteToolbar({ editor }: { editor: Editor }) {
  const state = useEditorState({
    editor,
    selector: ({ editor }) => ({
      h2: editor.isActive("heading", { level: 2 }),
      h3: editor.isActive("heading", { level: 3 }),
      bold: editor.isActive("bold"),
      italic: editor.isActive("italic"),
      underline: editor.isActive("underline"),
      strike: editor.isActive("strike"),
      highlight: editor.isActive("highlight"),
      bulletList: editor.isActive("bulletList"),
      orderedList: editor.isActive("orderedList"),
      blockquote: editor.isActive("blockquote"),
      code: editor.isActive("code"),
      link: editor.isActive("link"),
      canUndo: editor.can().undo(),
      canRedo: editor.can().redo(),
    }),
  });

  const toggleLink = useCallback(() => {
    if (editor.isActive("link")) {
      editor.chain().focus().unsetLink().run();
      return;
    }
    // `prompt` là dialog trình duyệt duy nhất trong app — chấp nhận được cho
    // thao tác hiếm này, đỡ phải dựng thêm 1 popover chèn link.
    const href = window.prompt("Địa chỉ liên kết (URL):");
    if (!href) return;
    editor.chain().focus().setLink({ href }).run();
  }, [editor]);

  return (
    <div className="flex items-center gap-0.5 overflow-x-auto border-b border-border bg-muted/40 px-2 py-1.5">
      <Button
        variant="ghost"
        size="icon"
        className="size-8"
        onClick={() => editor.chain().focus().undo().run()}
        disabled={!state.canUndo}
        aria-label="Hoàn tác"
      >
        <Undo2 className="size-4" aria-hidden />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        className="size-8"
        onClick={() => editor.chain().focus().redo().run()}
        disabled={!state.canRedo}
        aria-label="Làm lại"
      >
        <Redo2 className="size-4" aria-hidden />
      </Button>

      <Separator orientation="vertical" className="mx-1 h-6" />

      <Toggle
        size="sm"
        pressed={state.h2}
        onPressedChange={() =>
          editor.chain().focus().toggleHeading({ level: 2 }).run()
        }
        aria-label="Tiêu đề lớn"
      >
        H2
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.h3}
        onPressedChange={() =>
          editor.chain().focus().toggleHeading({ level: 3 }).run()
        }
        aria-label="Tiêu đề nhỏ"
      >
        H3
      </Toggle>

      <Separator orientation="vertical" className="mx-1 h-6" />

      <Toggle
        size="sm"
        pressed={state.bold}
        onPressedChange={() => editor.chain().focus().toggleBold().run()}
        aria-label="In đậm"
      >
        <Bold aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.italic}
        onPressedChange={() => editor.chain().focus().toggleItalic().run()}
        aria-label="In nghiêng"
      >
        <Italic aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.underline}
        onPressedChange={() => editor.chain().focus().toggleUnderline().run()}
        aria-label="Gạch chân"
      >
        <Underline aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.strike}
        onPressedChange={() => editor.chain().focus().toggleStrike().run()}
        aria-label="Gạch ngang"
      >
        <Strikethrough aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.highlight}
        onPressedChange={() => editor.chain().focus().toggleHighlight().run()}
        aria-label="Tô sáng"
      >
        <Highlighter aria-hidden />
      </Toggle>

      <Separator orientation="vertical" className="mx-1 h-6" />

      <Toggle
        size="sm"
        pressed={state.bulletList}
        onPressedChange={() => editor.chain().focus().toggleBulletList().run()}
        aria-label="Danh sách gạch đầu dòng"
      >
        <List aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.orderedList}
        onPressedChange={() => editor.chain().focus().toggleOrderedList().run()}
        aria-label="Danh sách đánh số"
      >
        <ListOrdered aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.blockquote}
        onPressedChange={() => editor.chain().focus().toggleBlockquote().run()}
        aria-label="Trích dẫn"
      >
        <Quote aria-hidden />
      </Toggle>
      <Toggle
        size="sm"
        pressed={state.code}
        onPressedChange={() => editor.chain().focus().toggleCode().run()}
        aria-label="Mã"
      >
        <Code aria-hidden />
      </Toggle>

      <Separator orientation="vertical" className="mx-1 h-6" />

      <Toggle
        size="sm"
        pressed={state.link}
        onPressedChange={toggleLink}
        aria-label="Chèn liên kết"
      >
        <Link2 aria-hidden />
      </Toggle>
      <Button
        variant="ghost"
        size="icon"
        className="size-8"
        onClick={() =>
          editor
            .chain()
            .focus()
            .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
            .run()
        }
        aria-label="Chèn bảng"
      >
        <TableIcon className="size-4" aria-hidden />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        className="size-8"
        onClick={() => editor.chain().focus().setHorizontalRule().run()}
        aria-label="Chèn đường kẻ ngang"
      >
        <Minus className="size-4" aria-hidden />
      </Button>
    </div>
  );
}
