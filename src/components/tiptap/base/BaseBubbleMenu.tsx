'use client'

import { useEffect, useState } from 'react'
import type { Editor } from '@tiptap/react'
import { Edit2, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface BaseBubbleMenuProps {
  editor: Editor | null
  nodeName: string
  onEdit: () => void
  onDelete: () => void
  children?: React.ReactNode
}

export function BaseBubbleMenu({
  editor,
  nodeName,
  onEdit,
  onDelete,
  children
}: BaseBubbleMenuProps) {
  const [showMenu, setShowMenu] = useState(false)
  const [position, setPosition] = useState({ top: 0, left: 0 })

  useEffect(() => {
    if (!editor) return

    const updateMenu = () => {
      const { selection, doc } = editor.state
      const { from } = selection

      let targetNode = null
      let nodePos = -1

      doc.nodesBetween(from - 1, from + 1, (node, pos) => {
        if (node.type.name === nodeName) {
          targetNode = node
          nodePos = pos
          return false
        }
      })

      if (targetNode && nodePos >= 0) {
        setShowMenu(true)
        setTimeout(() => {
          const domNode = editor.view.nodeDOM(nodePos)
          if (domNode && domNode instanceof HTMLElement) {
            const rect = domNode.getBoundingClientRect()
            setPosition({
              top: rect.top - 50,
              left: rect.left + rect.width / 2,
            })
          }
        }, 0)
      } else {
        setShowMenu(false)
      }
    }

    editor.on('selectionUpdate', updateMenu)
    editor.on('transaction', updateMenu)

    return () => {
      editor.off('selectionUpdate', updateMenu)
      editor.off('transaction', updateMenu)
    }
  }, [editor, nodeName])

  if (!showMenu || !editor) return null

  return (
    <div
      className="fixed z-50 flex gap-1 rounded-lg border border-border bg-background p-1 shadow-lg"
      style={{
        top: `${position.top}px`,
        left: `${position.left}px`,
        transform: 'translateX(-50%)',
      }}
    >
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-8 w-8"
        onClick={onEdit}
        title="Chỉnh sửa"
      >
        <Edit2 className="h-4 w-4" />
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-8 w-8 text-destructive hover:text-destructive"
        onClick={onDelete}
        title="Xóa"
      >
        <Trash2 className="h-4 w-4" />
      </Button>
      {children}
    </div>
  )
}
