'use client'

// Import Tiptap CSS directly from node_modules
import 'reactjs-tiptap-editor/lib/style.css'
import 'katex/dist/katex.min.css'
import 'prism-code-editor-lightweight/layout.css'
import 'prism-code-editor-lightweight/themes/github-dark.css'
import '@excalidraw/excalidraw/index.css'

import { useCallback, useEffect, useState } from 'react'
import { EditorContent, useEditor } from '@tiptap/react'
import type { JSONContent } from '@tiptap/core'

import { RichTextProvider } from 'reactjs-tiptap-editor'
import { localeActions } from 'reactjs-tiptap-editor/locale-bundle'
import { themeActions } from 'reactjs-tiptap-editor/theme'

// Toolbar components
import {
  RichTextUndo,
  RichTextRedo,
} from 'reactjs-tiptap-editor/history'
import { RichTextSearchAndReplace } from 'reactjs-tiptap-editor/searchandreplace'
import { RichTextClear } from 'reactjs-tiptap-editor/clear'
import { RichTextFontFamily } from 'reactjs-tiptap-editor/fontfamily'
import { RichTextHeading } from 'reactjs-tiptap-editor/heading'
import { RichTextFontSize } from 'reactjs-tiptap-editor/fontsize'
import { RichTextBold } from 'reactjs-tiptap-editor/bold'
import { RichTextItalic } from 'reactjs-tiptap-editor/italic'
import { RichTextUnderline } from 'reactjs-tiptap-editor/textunderline'
import { RichTextStrike } from 'reactjs-tiptap-editor/strike'
import { RichTextMoreMark } from 'reactjs-tiptap-editor/moremark'
import { RichTextEmoji } from 'reactjs-tiptap-editor/emoji'
import { RichTextColor } from 'reactjs-tiptap-editor/color'
import { RichTextHighlight } from 'reactjs-tiptap-editor/highlight'
import { RichTextBulletList } from 'reactjs-tiptap-editor/bulletlist'
import { RichTextOrderedList } from 'reactjs-tiptap-editor/orderedlist'
import { RichTextAlign } from 'reactjs-tiptap-editor/textalign'
import { RichTextIndent } from 'reactjs-tiptap-editor/indent'
import { RichTextLineHeight } from 'reactjs-tiptap-editor/lineheight'
import { RichTextTaskList } from 'reactjs-tiptap-editor/tasklist'
import { RichTextLink } from 'reactjs-tiptap-editor/link'
import { RichTextImage } from 'reactjs-tiptap-editor/image'
import { RichTextVideo } from 'reactjs-tiptap-editor/video'
import { RichTextImageGif } from 'reactjs-tiptap-editor/imagegif'
import { RichTextBlockquote } from 'reactjs-tiptap-editor/blockquote'
import { RichTextHorizontalRule } from 'reactjs-tiptap-editor/horizontalrule'
import { RichTextCode } from 'reactjs-tiptap-editor/code'
import { RichTextCodeBlock } from 'reactjs-tiptap-editor/codeblock'
import { RichTextColumn } from 'reactjs-tiptap-editor/column'
import { RichTextTable } from 'reactjs-tiptap-editor/table'
import { RichTextIframe } from 'reactjs-tiptap-editor/iframe'
import { RichTextExportPdf } from 'reactjs-tiptap-editor/exportpdf'
import { RichTextImportWord } from 'reactjs-tiptap-editor/importword'
import { RichTextExportWord } from 'reactjs-tiptap-editor/exportword'
import { RichTextTextDirection } from 'reactjs-tiptap-editor/textdirection'
import { RichTextAttachment } from 'reactjs-tiptap-editor/attachment'
import { RichTextKatex } from 'reactjs-tiptap-editor/katex'
import { RichTextExcalidraw } from 'reactjs-tiptap-editor/excalidraw'
import { RichTextMermaid } from 'reactjs-tiptap-editor/mermaid'
import { RichTextDrawer } from 'reactjs-tiptap-editor/drawer'
import { RichTextTwitter } from 'reactjs-tiptap-editor/twitter'
import { RichTextCodeView } from 'reactjs-tiptap-editor/codeview'

// Bubble menus
import {
  RichTextBubbleColumns,
  RichTextBubbleDrawer,
  RichTextBubbleExcalidraw,
  RichTextBubbleIframe,
  RichTextBubbleImage,
  RichTextBubbleImageGif,
  RichTextBubbleKatex,
  RichTextBubbleLink,
  RichTextBubbleMermaid,
  RichTextBubbleTable,
  RichTextBubbleText,
  RichTextBubbleTwitter,
  RichTextBubbleVideo,
} from 'reactjs-tiptap-editor/bubble'

// Slash command
import { SlashCommandList } from 'reactjs-tiptap-editor/slashcommand'

// Custom extensions (Registry-based)
import {
  getToolbarComponents,
  getBubbleMenuComponents,
  type ExtensionConfigs,
} from './custom-extensions/registry'

import { getExtensions as getBaseExtensions, DEFAULT_CONTENT } from './extensions'
import { cn } from '@/lib/utils'

/**
 * Debounce function to limit how often a function is called
 */
function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout
  return function (...args: Parameters<T>) {
    clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

export interface TiptapEditorProps {
  /** Initial content as HTML string or Tiptap JSON */
  initialContent?: string | JSONContent
  /** Callback when content changes */
  onChange?: (json: JSONContent, html: string) => void
  /** Whether the editor is editable */
  editable?: boolean
  /** Placeholder text */
  placeholder?: string
  /** Additional CSS class name */
  className?: string
  /** Show language selector */
  showLanguageSelector?: boolean
  /** Show theme switcher */
  showThemeSwitcher?: boolean
  /** Show toolbar (defaults to true, auto-hides when editable=false) */
  showToolbar?: boolean
  /** Extension configurations */
  extensionConfigs?: ExtensionConfigs
}

export function TiptapEditor({
  initialContent,
  onChange,
  editable = true,
  placeholder,
  className,
  showLanguageSelector = false,
  showThemeSwitcher = false,
  showToolbar,
  extensionConfigs,
}: TiptapEditorProps) {
  // Auto-hide toolbar when in read-only mode unless explicitly set
  const shouldShowToolbar = showToolbar !== undefined ? showToolbar : editable
  const [content, setContent] = useState(initialContent || DEFAULT_CONTENT)
  const [theme, setTheme] = useState('light')
  const [color, setColor] = useState('default')
  const [lang, setLang] = useState('vi')

  const onValueChange = useCallback(
    debounce((json: JSONContent, html: string) => {
      onChange?.(json, html)
    }, 300),
    [onChange]
  )

  const editor = useEditor({
    textDirection: 'auto',
    content,
    extensions: getBaseExtensions(extensionConfigs),
    editable,
    immediatelyRender: false, // Critical for Next.js
    onUpdate: ({ editor }) => {
      const json = editor.getJSON()
      const html = editor.getHTML()
      setContent(html)
      onValueChange(json, html)
    },
  })

  // Update editor content when initialContent changes
  useEffect(() => {
    if (editor && initialContent !== undefined) {
      const currentContent = editor.getHTML()
      const newContent =
        typeof initialContent === 'string' ? initialContent : null

      if (newContent && newContent !== currentContent) {
        setTimeout(() => {
          if (!editor.isDestroyed) {
            editor.commands.setContent(initialContent)
          }
        }, 0)
      }
    }
  }, [editor, initialContent])

  // Update editor editable state
  useEffect(() => {
    if (editor && editor.isEditable !== editable) {
      editor.setEditable(editable)
    }
  }, [editor, editable])

  if (!editor) return null
  return (
    <RichTextProvider editor={editor}>
      <div className={cn('flex flex-col gap-2', className)}>
        {/* Header with language and theme selectors (optional) */}
        {(showLanguageSelector || showThemeSwitcher) && (
          <div className="flex items-center justify-between gap-3 rounded-md border border-border bg-muted/30 p-2">
            {showLanguageSelector && (
              <div className="flex items-center gap-2">
                <label className="text-sm font-medium">Ngôn ngữ:</label>
                <select
                  value={lang}
                  onChange={(e) => {
                    setLang(e.target.value)
                    localeActions.setLang(e.target.value)
                  }}
                  className="rounded border border-border bg-background px-2 py-1 text-sm"
                >
                  <option value="vi">Tiếng Việt</option>
                  <option value="en">English</option>
                  <option value="zh_CN">中文</option>
                  <option value="pt_BR">Português</option>
                  <option value="hu_HU">Magyar</option>
                  <option value="fi">Suomi</option>
                </select>
              </div>
            )}

            {showThemeSwitcher && (
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => {
                    const newTheme = theme === 'dark' ? 'light' : 'dark'
                    setTheme(newTheme)
                    themeActions.setTheme(newTheme)
                  }}
                  className="rounded border border-border bg-background px-3 py-1 text-sm hover:bg-muted"
                >
                  {theme === 'dark' ? '☀️ Sáng' : '🌙 Tối'}
                </button>

                <select
                  value={color}
                  onChange={(e) => {
                    setColor(e.target.value)
                    themeActions.setColor(e.target.value as any)
                  }}
                  className="rounded border border-border bg-background px-2 py-1 text-sm"
                >
                  <option value="default">Mặc định</option>
                  <option value="red">Đỏ</option>
                  <option value="blue">Xanh dương</option>
                  <option value="green">Xanh lá</option>
                  <option value="orange">Cam</option>
                  <option value="rose">Hồng</option>
                  <option value="violet">Tím</option>
                  <option value="yellow">Vàng</option>
                </select>
              </div>
            )}
          </div>
        )}

        {/* Editor */}
        <div
          className={cn(
            "overflow-hidden rounded-[0.5rem]",
            editable ? "bg-background shadow outline outline-1 outline-border" : ""
          )}
          style={!editable ? {
            boxShadow: 'none',
            border: 'none',
            outline: 'none',
            background: 'transparent'
          } : undefined}
        >
          <div className="flex max-h-full w-full flex-col">
            {/* Toolbar - Hidden in read-only mode */}
            {shouldShowToolbar && (
            <div className="flex items-center !p-1 gap-2 flex-wrap !border-b !border-solid !border-border">
              <RichTextUndo />
              <RichTextRedo />
              <RichTextSearchAndReplace />
              <RichTextClear />
              <RichTextFontFamily />
              <RichTextHeading />
              <RichTextFontSize />
              <RichTextBold />
              <RichTextItalic />
              <RichTextUnderline />
              <RichTextStrike />
              <RichTextMoreMark />
              <RichTextEmoji />
              <RichTextColor />
              <RichTextHighlight />
              <RichTextBulletList />
              <RichTextOrderedList />
              <RichTextAlign />
              <RichTextIndent />
              <RichTextLineHeight />
              <RichTextTaskList />
              <RichTextLink />
              <RichTextImage />
              <RichTextVideo />

              {/* Custom extension toolbars - AUTO RENDER */}
              {getToolbarComponents().map(({ name, Component }) => (
                <Component key={name} editor={editor} />
              ))}

              <RichTextImageGif />
              <RichTextBlockquote />
              <RichTextHorizontalRule />
              <RichTextCode />
              <RichTextCodeBlock />
              <RichTextColumn />
              <RichTextTable />
              <RichTextIframe />
              <RichTextExportPdf />
              <RichTextImportWord />
              <RichTextExportWord />
              <RichTextTextDirection />
              <RichTextAttachment />
              <RichTextKatex />
              <RichTextExcalidraw />
              <RichTextMermaid />
              <RichTextDrawer />
              <RichTextTwitter />
              <RichTextCodeView />
            </div>
            )}

            {/* Editor Content */}
            <div
              className={!editable ? "tiptap-read-only-content" : ""}
              style={!editable ? { padding: 0 } : undefined}
            >
              <EditorContent editor={editor} />
            </div>

            {/* Bubble Menus - Only show when editable */}
            {editable && (
              <>
                <RichTextBubbleColumns />
                <RichTextBubbleDrawer />
                <RichTextBubbleExcalidraw />
                <RichTextBubbleIframe />
                <RichTextBubbleKatex />
                <RichTextBubbleLink />
                <RichTextBubbleImage />
                <RichTextBubbleVideo />
                <RichTextBubbleImageGif />
                <RichTextBubbleMermaid />
                <RichTextBubbleTable />
                <RichTextBubbleText />
                <RichTextBubbleTwitter />

                {/* Custom extension bubble menus - AUTO RENDER */}
                {getBubbleMenuComponents().map(({ name, Component }) => (
                  <Component key={name} editor={editor} />
                ))}

                {/* Slash Command List */}
                <SlashCommandList />
              </>
            )}
          </div>
        </div>
      </div>
    </RichTextProvider>
  )
}

export default TiptapEditor
