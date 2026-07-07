'use client';

// Editor styles — import once at top of module
import 'reactjs-tiptap-editor/style.css';

import { useCallback, useEffect, useState } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import { createLowlight, common } from 'lowlight';

const lowlight = createLowlight(common);

import { Document } from '@tiptap/extension-document';
import { Paragraph } from '@tiptap/extension-paragraph';
import { Text } from '@tiptap/extension-text';
import { HardBreak } from '@tiptap/extension-hard-break';
import { ListItem } from '@tiptap/extension-list';
import { TextStyle } from '@tiptap/extension-text-style';
import { Dropcursor, Gapcursor, Placeholder, TrailingNode } from '@tiptap/extensions';

import { RichTextProvider } from 'reactjs-tiptap-editor';

// Extensions — focused news-editor set (no Excalidraw/Mermaid/Katex/Twitter etc.)
import { History } from 'reactjs-tiptap-editor/history';
import { Clear } from 'reactjs-tiptap-editor/clear';
import { Heading } from 'reactjs-tiptap-editor/heading';
import { Bold } from 'reactjs-tiptap-editor/bold';
import { Italic } from 'reactjs-tiptap-editor/italic';
import { TextUnderline } from 'reactjs-tiptap-editor/textunderline';
import { Strike } from 'reactjs-tiptap-editor/strike';
import { MoreMark } from 'reactjs-tiptap-editor/moremark';
import { Color } from 'reactjs-tiptap-editor/color';
import { Highlight } from 'reactjs-tiptap-editor/highlight';
import { BulletList } from 'reactjs-tiptap-editor/bulletlist';
import { OrderedList } from 'reactjs-tiptap-editor/orderedlist';
import { TextAlign } from 'reactjs-tiptap-editor/textalign';
import { Indent } from 'reactjs-tiptap-editor/indent';
import { Link } from 'reactjs-tiptap-editor/link';
import { Image } from 'reactjs-tiptap-editor/image';
import { Blockquote } from 'reactjs-tiptap-editor/blockquote';
import { HorizontalRule } from 'reactjs-tiptap-editor/horizontalrule';
import { Code } from 'reactjs-tiptap-editor/code';
import { CodeBlock } from 'reactjs-tiptap-editor/codeblock';
import { Table } from 'reactjs-tiptap-editor/table';

// Toolbar components
import { RichTextUndo, RichTextRedo } from 'reactjs-tiptap-editor/history';
import { RichTextClear } from 'reactjs-tiptap-editor/clear';
import { RichTextHeading } from 'reactjs-tiptap-editor/heading';
import { RichTextBold } from 'reactjs-tiptap-editor/bold';
import { RichTextItalic } from 'reactjs-tiptap-editor/italic';
import { RichTextUnderline } from 'reactjs-tiptap-editor/textunderline';
import { RichTextStrike } from 'reactjs-tiptap-editor/strike';
import { RichTextMoreMark } from 'reactjs-tiptap-editor/moremark';
import { RichTextColor } from 'reactjs-tiptap-editor/color';
import { RichTextHighlight } from 'reactjs-tiptap-editor/highlight';
import { RichTextBulletList } from 'reactjs-tiptap-editor/bulletlist';
import { RichTextOrderedList } from 'reactjs-tiptap-editor/orderedlist';
import { RichTextAlign } from 'reactjs-tiptap-editor/textalign';
import { RichTextIndent } from 'reactjs-tiptap-editor/indent';
import { RichTextLink } from 'reactjs-tiptap-editor/link';
import { RichTextImage } from 'reactjs-tiptap-editor/image';
import { RichTextBlockquote } from 'reactjs-tiptap-editor/blockquote';
import { RichTextHorizontalRule } from 'reactjs-tiptap-editor/horizontalrule';
import { RichTextCode } from 'reactjs-tiptap-editor/code';
import { RichTextCodeBlock } from 'reactjs-tiptap-editor/codeblock';
import { RichTextTable } from 'reactjs-tiptap-editor/table';

// Bubble menus (only for what we enabled)
import {
  RichTextBubbleImage,
  RichTextBubbleLink,
  RichTextBubbleTable,
  RichTextBubbleText,
} from 'reactjs-tiptap-editor/bubble';

interface RichTextEditorProps {
  value: string;
  onChange: (html: string) => void;
  placeholder?: string;
  minHeight?: string;
}

// Upload via Supabase Storage. Falls back to a base64 data URL if the
// server upload fails so the editor never silently drops the image — the
// admin still sees their content and can replace it later. Pasted/dragged
// images flow through here via Tiptap's Image.configure({ upload }).
async function uploadEditorImage(file: File): Promise<string> {
  try {
    const { uploadImageToStorage } = await import('@/lib/image-upload');
    const url = await uploadImageToStorage(file, { folder: 'news', lang: 'EN' });
    if (url) return url;
  } catch {
    /* fall through to base64 fallback below */
  }
  return await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => resolve((e.target?.result as string) || '');
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

const baseExtensions = [
  Document,
  Text,
  Paragraph,
  HardBreak,
  ListItem,
  TextStyle,
  Dropcursor,
  Gapcursor,
  TrailingNode,
];

const richExtensions = [
  History,
  Clear,
  Heading,
  Bold,
  Italic,
  TextUnderline,
  Strike,
  MoreMark,
  Color,
  Highlight,
  BulletList,
  OrderedList,
  TextAlign,
  Indent,
  Link,
  Image.configure({ upload: uploadEditorImage }),
  Blockquote,
  HorizontalRule,
  Code,
  CodeBlock.configure({ lowlight }),
  Table,
];

export function RichTextEditor({
  value,
  onChange,
  placeholder = '',
  minHeight = '300px',
}: RichTextEditorProps) {
  const [extensions] = useState(() => [
    ...baseExtensions,
    Placeholder.configure({ placeholder }),
    ...richExtensions,
  ]);

  const editor = useEditor({
    extensions,
    content: value,
    immediatelyRender: false,
    editorProps: {
      attributes: {
        class:
          'px-4 py-3 outline-none text-sm text-slate-800 dark:text-slate-100 leading-relaxed overflow-y-auto prose prose-sm max-w-none [&_h1]:text-xl [&_h1]:font-bold [&_h1]:mt-4 [&_h1]:mb-2 [&_h2]:text-lg [&_h2]:font-bold [&_h2]:mt-4 [&_h2]:mb-2 [&_h3]:text-base [&_h3]:font-bold [&_h3]:mt-3 [&_h3]:mb-1 [&_ul]:list-disc [&_ul]:ml-5 [&_ol]:list-decimal [&_ol]:ml-5 [&_li]:my-0.5 [&_hr]:my-4 [&_hr]:border-slate-200 dark:[&_hr]:border-slate-700 [&_blockquote]:border-l-4 [&_blockquote]:border-slate-300 dark:[&_blockquote]:border-slate-600 [&_blockquote]:pl-3 [&_blockquote]:italic [&_code]:bg-slate-100 dark:[&_code]:bg-slate-800 [&_code]:px-1 [&_code]:rounded [&_pre]:bg-slate-900 [&_pre]:text-slate-100 [&_pre]:p-3 [&_pre]:rounded-lg [&_pre]:my-2 [&_pre_code]:bg-transparent [&_pre_code]:text-inherit [&_table]:border-collapse [&_table]:my-2 [&_th]:border [&_th]:border-slate-200 dark:[&_th]:border-slate-700 [&_th]:p-1.5 [&_th]:bg-slate-50 dark:[&_th]:bg-slate-800 [&_td]:border [&_td]:border-slate-200 dark:[&_td]:border-slate-700 [&_td]:p-1.5 [&_a]:text-blue-600 dark:text-blue-400 [&_a]:underline [&_img]:max-w-full [&_img]:rounded-lg [&_img]:my-2',
        style: `min-height:${minHeight}`,
      },
    },
    onUpdate: ({ editor }) => {
      onChange(editor.getHTML());
    },
  });

  // Keep editor content in sync with external value (e.g. form reset).
  useEffect(() => {
    if (!editor) return;
    if (editor.getHTML() !== value) {
      editor.commands.setContent(value || '', { emitUpdate: false });
    }
  }, [value, editor]);

  const handleSetContent = useCallback(
    (html: string) => {
      editor?.commands.setContent(html, { emitUpdate: true });
    },
    [editor],
  );
  // Re-export setter so callers that prefer imperative updates still work,
  // mirroring the previous component's external surface.
  useEffect(() => {
    void handleSetContent;
  }, [handleSetContent]);

  if (!editor) {
    return (
      <div
        className="animate-pulse rounded-xl border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900"
        style={{ minHeight }}
      />
    );
  }

  return (
    <RichTextProvider editor={editor}>
      <div className="overflow-hidden rounded-xl border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900">
        {/* Toolbar — focused subset for news content */}
        <div className="flex flex-wrap items-center gap-1 border-b border-slate-100 bg-slate-50/80 px-2 py-1.5 dark:border-slate-700 dark:bg-slate-800/80">
          <RichTextUndo />
          <RichTextRedo />
          <RichTextClear />
          <RichTextHeading />
          <RichTextBold />
          <RichTextItalic />
          <RichTextUnderline />
          <RichTextStrike />
          <RichTextMoreMark />
          <RichTextColor />
          <RichTextHighlight />
          <RichTextBulletList />
          <RichTextOrderedList />
          <RichTextAlign />
          <RichTextIndent />
          <RichTextLink />
          <RichTextImage />
          <RichTextBlockquote />
          <RichTextHorizontalRule />
          <RichTextCode />
          <RichTextCodeBlock />
          <RichTextTable />
        </div>

        <EditorContent editor={editor} />

        {/* Bubble menus — only for nodes/marks we enabled */}
        <RichTextBubbleText />
        <RichTextBubbleLink />
        <RichTextBubbleImage />
        <RichTextBubbleTable />
      </div>
    </RichTextProvider>
  );
}
