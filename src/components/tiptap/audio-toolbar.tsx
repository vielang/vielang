'use client'

import { useState } from 'react'
import type { Editor } from '@tiptap/react'
import { Volume2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
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

interface RichTextAudioProps {
  editor: Editor | null
}

export function RichTextAudio({ editor }: RichTextAudioProps) {
  const [open, setOpen] = useState(false)
  const [audioUrl, setAudioUrl] = useState('')
  const [audioTitle, setAudioTitle] = useState('')
  const [savedSelection, setSavedSelection] = useState<{ from: number; to: number } | null>(null)

  if (!editor) return null

  const handleOpenDialog = () => {
    // Save current cursor position before opening dialog
    const { from, to } = editor.state.selection
    setSavedSelection({ from, to })
    setOpen(true)
  }

  const handleInsert = () => {
    if (audioUrl.trim() && savedSelection) {
      // Restore cursor position and insert audio at that position
      editor
        .chain()
        .focus()
        .setTextSelection(savedSelection.from)
        .setAudio({
          src: audioUrl.trim(),
          title: audioTitle.trim() || 'Audio',
        })
        .run()

      // Reset form
      setAudioUrl('')
      setAudioTitle('')
      setSavedSelection(null)
      setOpen(false)
    }
  }

  return (
    <>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className="h-8 w-8"
        onClick={handleOpenDialog}
        title="Thêm audio"
      >
        <Volume2 className="h-4 w-4" />
      </Button>

      <Dialog open={open} onOpenChange={(isOpen) => {
        setOpen(isOpen)
        if (!isOpen) {
          // Clear saved selection when dialog closes
          setSavedSelection(null)
          setAudioUrl('')
          setAudioTitle('')
        }
      }}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Thêm Audio</DialogTitle>
            <DialogDescription>
              Nhập URL của file audio và tiêu đề (tùy chọn)
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="audio-url">URL Audio *</Label>
              <Input
                id="audio-url"
                placeholder="https://example.com/audio.mp3"
                value={audioUrl}
                onChange={(e) => setAudioUrl(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && audioUrl.trim()) {
                    handleInsert()
                  }
                }}
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="audio-title">Tiêu đề (tùy chọn)</Label>
              <Input
                id="audio-title"
                placeholder="Phát âm bài học 1"
                value={audioTitle}
                onChange={(e) => setAudioTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && audioUrl.trim()) {
                    handleInsert()
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
                setOpen(false)
                setAudioUrl('')
                setAudioTitle('')
                setSavedSelection(null)
              }}
            >
              Hủy
            </Button>
            <Button
              type="button"
              onClick={handleInsert}
              disabled={!audioUrl.trim()}
            >
              Thêm Audio
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
