'use client'

import { useState } from 'react'
import type { Editor } from '@tiptap/react'
import { BaseBubbleMenu } from './base/BaseBubbleMenu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'

interface AudioBubbleMenuProps {
  editor: Editor | null
}

export function AudioBubbleMenuV2({ editor }: AudioBubbleMenuProps) {
  const [editDialogOpen, setEditDialogOpen] = useState(false)
  const [audioUrl, setAudioUrl] = useState('')
  const [audioTitle, setAudioTitle] = useState('')

  const handleEdit = () => {
    if (!editor) return
    const attrs = editor.getAttributes('audio')
    setAudioUrl(attrs.src || '')
    setAudioTitle(attrs.title || '')
    setEditDialogOpen(true)
  }

  const handleUpdate = () => {
    if (!editor || !audioUrl.trim()) return
    editor
      .chain()
      .focus()
      .updateAttributes('audio', {
        src: audioUrl.trim(),
        title: audioTitle.trim() || 'Audio',
      })
      .run()
    setEditDialogOpen(false)
    setAudioUrl('')
    setAudioTitle('')
  }

  const handleDelete = () => {
    if (!editor) return
    editor.chain().focus().deleteSelection().run()
  }

  return (
    <>
      <BaseBubbleMenu
        editor={editor}
        nodeName="audio"
        onEdit={handleEdit}
        onDelete={handleDelete}
      />

      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Chỉnh sửa Audio</DialogTitle>
            <DialogDescription>
              Cập nhật URL và tiêu đề của audio
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="edit-audio-url">URL Audio *</Label>
              <Input
                id="edit-audio-url"
                placeholder="https://example.com/audio.mp3"
                value={audioUrl}
                onChange={(e) => setAudioUrl(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && audioUrl.trim()) {
                    handleUpdate()
                  }
                }}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="edit-audio-title">Tiêu đề (tùy chọn)</Label>
              <Input
                id="edit-audio-title"
                placeholder="Phát âm bài học 1"
                value={audioTitle}
                onChange={(e) => setAudioTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && audioUrl.trim()) {
                    handleUpdate()
                  }
                }}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setEditDialogOpen(false)
                setAudioUrl('')
                setAudioTitle('')
              }}
            >
              Hủy
            </Button>
            <Button
              type="button"
              onClick={handleUpdate}
              disabled={!audioUrl.trim()}
            >
              Cập nhật
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
