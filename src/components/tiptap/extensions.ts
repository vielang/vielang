// ==============================
// Base Kit (TipTap Core)
// ==============================
import { Document } from '@tiptap/extension-document'
import { HardBreak } from '@tiptap/extension-hard-break'
import { ListItem } from '@tiptap/extension-list'
import { Paragraph } from '@tiptap/extension-paragraph'
import { Text } from '@tiptap/extension-text'
import { TextStyle } from '@tiptap/extension-text-style'
import { Dropcursor, Gapcursor, Placeholder, TrailingNode } from '@tiptap/extensions'

// ==============================
// Reactjs-Tiptap-Editor Extensions
// ==============================
import { Attachment } from 'reactjs-tiptap-editor/attachment'
import { Blockquote } from 'reactjs-tiptap-editor/blockquote'
import { Bold } from 'reactjs-tiptap-editor/bold'
import { BulletList } from 'reactjs-tiptap-editor/bulletlist'
import { Clear } from 'reactjs-tiptap-editor/clear'
import { Code } from 'reactjs-tiptap-editor/code'
import { CodeBlock } from 'reactjs-tiptap-editor/codeblock'
import { CodeView } from 'reactjs-tiptap-editor/codeview'
import { Color } from 'reactjs-tiptap-editor/color'
import { Column, ColumnNode, MultipleColumnNode } from 'reactjs-tiptap-editor/column'
import { Drawer } from 'reactjs-tiptap-editor/drawer'
import { Emoji } from 'reactjs-tiptap-editor/emoji'
import { Excalidraw } from 'reactjs-tiptap-editor/excalidraw'
import { ExportPdf } from 'reactjs-tiptap-editor/exportpdf'
import { ExportWord } from 'reactjs-tiptap-editor/exportword'
import { FontFamily } from 'reactjs-tiptap-editor/fontfamily'
import { FontSize } from 'reactjs-tiptap-editor/fontsize'
import { Heading } from 'reactjs-tiptap-editor/heading'
import { Highlight } from 'reactjs-tiptap-editor/highlight'
import { History } from 'reactjs-tiptap-editor/history'
import { HorizontalRule } from 'reactjs-tiptap-editor/horizontalrule'
import { Iframe } from 'reactjs-tiptap-editor/iframe'
import { Image } from 'reactjs-tiptap-editor/image'
import { ImageGif } from 'reactjs-tiptap-editor/imagegif'
import { ImportWord } from 'reactjs-tiptap-editor/importword'
import { Indent } from 'reactjs-tiptap-editor/indent'
import { Italic } from 'reactjs-tiptap-editor/italic'
import { Katex } from 'reactjs-tiptap-editor/katex'
import { LineHeight } from 'reactjs-tiptap-editor/lineheight'
import { Link } from 'reactjs-tiptap-editor/link'
import { Mermaid } from 'reactjs-tiptap-editor/mermaid'
import { MoreMark } from 'reactjs-tiptap-editor/moremark'
import { OrderedList } from 'reactjs-tiptap-editor/orderedlist'
import { SearchAndReplace } from 'reactjs-tiptap-editor/searchandreplace'
import { Strike } from 'reactjs-tiptap-editor/strike'
import { Table } from 'reactjs-tiptap-editor/table'
import { TaskList } from 'reactjs-tiptap-editor/tasklist'
import { TextAlign } from 'reactjs-tiptap-editor/textalign'
import { TextDirection } from 'reactjs-tiptap-editor/textdirection'
import { TextUnderline } from 'reactjs-tiptap-editor/textunderline'
import { Twitter } from 'reactjs-tiptap-editor/twitter'
import { Video } from 'reactjs-tiptap-editor/video'
import { SlashCommand } from 'reactjs-tiptap-editor/slashcommand'

// ==============================
// Custom Extensions (Registry-based)
// ==============================
import { getExtensions as getCustomExtensions, type ExtensionConfigs } from './custom-extensions/registry'

// ==============================
// Axios
// ==============================
import axios from 'axios'

const pbAxios = axios.create({
  baseURL: '/api/pb_proxy',
})

// ==============================
// Upload helper (PocketBase)
// ==============================
async function uploadToPocketBase(file: File): Promise<string> {
  const formData = new FormData()
  formData.append('image_file', file)

  try {
    const res = await pbAxios.post(
      '/collections/images_tbl/records',
      formData,
      { headers: { 'Content-Type': 'multipart/form-data' } }
    )

    const record = res.data

    return `https://pocketbase.vielang.com/api/files/${record.collectionName}/${record.id}/${record.image_file}`
  } catch (error) {
    console.error('PocketBase upload error:', error)
    throw new Error('Image upload failed. Please try again.')
  }
}

// ==============================
// Custom Document (Multi-column)
// ==============================
const DocumentColumn = Document.extend({
  content: '(block|columns)+',
})

// ==============================
// Base extensions
// ==============================
export const BaseKit = [
  DocumentColumn,
  Text,
  Dropcursor,
  Gapcursor,
  HardBreak,
  Paragraph,
  TrailingNode,
  ListItem,
  TextStyle,
  Placeholder.configure({
    placeholder: "Nhập nội dung hoặc gõ '/' để xem lệnh",
  }),
]

// ==============================
// All Extensions (PRODUCTION READY)
// ==============================
export function getExtensions(configs?: ExtensionConfigs) {
  return [
    ...BaseKit,

    History,
    SearchAndReplace,
    Clear,

    FontFamily,
    FontSize,
    Heading,

    Bold,
    Italic,
    TextUnderline,
    Strike,
    MoreMark,

    Emoji,
    Color,
    Highlight,

    BulletList,
    OrderedList,
    TaskList,

    TextAlign,
    Indent,
    LineHeight,

    Link,

    // ===== MEDIA (UPLOAD THẬT) =====
    Image.configure({
      upload: async (file: File) => uploadToPocketBase(file),
    }),

    Video.configure({
      upload: async (file: File) => uploadToPocketBase(file),
    }),

    // Custom extensions from registry
    ...getCustomExtensions(configs),

    Attachment.configure({
      upload: async (file: File) => uploadToPocketBase(file),
    }),

    Mermaid.configure({
      upload: async (file: File) => uploadToPocketBase(file),
    }),

    Drawer.configure({
      upload: async (file: File) => uploadToPocketBase(file),
    }),

    // ===== GIF =====
    ImageGif.configure({
      provider: 'giphy',
      API_KEY: (process.env.NEXT_PUBLIC_GIPHY_API_KEY || '') as string,
    }),

    // ===== BLOCKS =====
    Blockquote,
    HorizontalRule,
    Code,
    CodeBlock,

    Column,
    ColumnNode,
    MultipleColumnNode,

    Table,
    Iframe,

    ExportPdf,
    ImportWord,
    ExportWord,

    TextDirection,
    Katex,
    Excalidraw,
    Twitter,
    SlashCommand,
    CodeView,
  ]
}

// ==============================
// Default content
// ==============================
export const DEFAULT_CONTENT = '<p></p>'
