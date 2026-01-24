'use client'

import { useEffect, useState, useRef } from 'react'
import { Edit2, Trash2, Plus, Image as ImageIcon, X } from 'lucide-react'
import axios from 'axios'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { ScrollArea } from '@/components/ui/scroll-area'
import type { Editor } from '@tiptap/react'
import { TiptapEditorClient } from './TiptapEditorClient'

// Helper to check if HTML content is empty
function isHTMLEmpty(html: string): boolean {
  const text = html.replace(/<[^>]*>/g, '').trim()
  return text.length === 0
}

// Helper to strip HTML and truncate text
function getPlainTextPreview(html: string, maxLength: number = 60): string {
  const text = html.replace(/<[^>]*>/g, '').trim()
  if (text.length <= maxLength) return text
  return text.substring(0, maxLength) + '...'
}

interface QuizBubbleMenuProps {
  editor: Editor | null
}

interface QuizItem {
  id: string
  question: string
  questionImage?: string
  optionA: string
  optionAImage?: string
  optionB: string
  optionBImage?: string
  optionC: string
  optionCImage?: string
  optionD: string
  optionDImage?: string
  correctAnswer: 'A' | 'B' | 'C' | 'D'
  layout?: 'vertical' | 'grid'
}

export function QuizBubbleMenu({ editor }: QuizBubbleMenuProps) {
  const [showMenu, setShowMenu] = useState(false)
  const [position, setPosition] = useState({ top: 0, left: 0 })
  const [editDialogOpen, setEditDialogOpen] = useState(false)
  const [quizItems, setQuizItems] = useState<QuizItem[]>([])
  const [quizGroupPos, setQuizGroupPos] = useState<number>(-1)
  const [editingQuizId, setEditingQuizId] = useState<string | null>(null)

  // Current quiz form state
  const [question, setQuestion] = useState('')
  const [questionImage, setQuestionImage] = useState('')
  const [optionA, setOptionA] = useState('')
  const [optionAImage, setOptionAImage] = useState('')
  const [optionB, setOptionB] = useState('')
  const [optionBImage, setOptionBImage] = useState('')
  const [optionC, setOptionC] = useState('')
  const [optionCImage, setOptionCImage] = useState('')
  const [optionD, setOptionD] = useState('')
  const [optionDImage, setOptionDImage] = useState('')
  const [correctAnswer, setCorrectAnswer] = useState<'A' | 'B' | 'C' | 'D'>('A')
  const [layout, setLayout] = useState<'vertical' | 'grid'>('vertical')
  const [uploading, setUploading] = useState<string | null>(null)

  // File input refs
  const questionImageRef = useRef<HTMLInputElement>(null)
  const optionAImageRef = useRef<HTMLInputElement>(null)
  const optionBImageRef = useRef<HTMLInputElement>(null)
  const optionCImageRef = useRef<HTMLInputElement>(null)
  const optionDImageRef = useRef<HTMLInputElement>(null)

  // PocketBase upload helper
  const pbAxios = axios.create({
    baseURL: '/api/pb_proxy',
  })

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

  const handleImageUpload = async (
    file: File | null,
    setter: (url: string) => void,
    uploadKey: string
  ) => {
    if (!file) return

    setUploading(uploadKey)
    try {
      const url = await uploadToPocketBase(file)
      setter(url)
    } catch (error) {
      console.error('Upload failed:', error)
      alert('Upload thất bại. Vui lòng thử lại.')
    } finally {
      setUploading(null)
    }
  }

  useEffect(() => {
    if (!editor) return

    const updateMenu = () => {
      const { selection, doc } = editor.state
      const { from } = selection

      // Check if there's a quizGroup node at or near the selection
      let quizGroupNode = null
      let nodePos = -1

      doc.nodesBetween(from - 1, from + 1, (node, pos) => {
        if (node.type.name === 'quizGroup') {
          quizGroupNode = node
          nodePos = pos
          return false
        }
      })

      if (quizGroupNode && nodePos >= 0) {
        setShowMenu(true)
        setQuizGroupPos(nodePos)

        // Get node DOM position
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

    // Listen to selection changes
    editor.on('selectionUpdate', updateMenu)
    editor.on('transaction', updateMenu)

    return () => {
      editor.off('selectionUpdate', updateMenu)
      editor.off('transaction', updateMenu)
    }
  }, [editor])

  const resetForm = () => {
    setQuestion('')
    setQuestionImage('')
    setOptionA('')
    setOptionAImage('')
    setOptionB('')
    setOptionBImage('')
    setOptionC('')
    setOptionCImage('')
    setOptionD('')
    setOptionDImage('')
    setCorrectAnswer('A')
    setLayout('vertical')
    setEditingQuizId(null)
  }

  const handleEdit = () => {
    if (!editor || quizGroupPos < 0) return

    const { doc } = editor.state
    const quizGroupNode = doc.nodeAt(quizGroupPos)

    if (!quizGroupNode || quizGroupNode.type.name !== 'quizGroup') return

    // Extract quiz items from quizGroup
    const items: QuizItem[] = []
    quizGroupNode.content.forEach((child) => {
      if (child.type.name === 'quizItem') {
        items.push({
          id: Date.now().toString() + Math.random(),
          question: child.attrs.question || '',
          questionImage: child.attrs.questionImage || undefined,
          optionA: child.attrs.optionA || '',
          optionAImage: child.attrs.optionAImage || undefined,
          optionB: child.attrs.optionB || '',
          optionBImage: child.attrs.optionBImage || undefined,
          optionC: child.attrs.optionC || '',
          optionCImage: child.attrs.optionCImage || undefined,
          optionD: child.attrs.optionD || '',
          optionDImage: child.attrs.optionDImage || undefined,
          correctAnswer: child.attrs.correctAnswer || 'A',
          layout: child.attrs.layout || 'vertical',
        })
      }
    })

    setQuizItems(items)
    setEditDialogOpen(true)
  }

  const handleAddQuiz = () => {
    if (isHTMLEmpty(question) || !optionA.trim() || !optionB.trim()) {
      return
    }

    if (editingQuizId) {
      // Update existing quiz
      setQuizItems(
        quizItems.map((item) =>
          item.id === editingQuizId
            ? {
                ...item,
                question: question, // Keep HTML as is
                questionImage: questionImage || undefined,
                optionA: optionA.trim(),
                optionAImage: optionAImage || undefined,
                optionB: optionB.trim(),
                optionBImage: optionBImage || undefined,
                optionC: optionC.trim() || 'Đáp án C',
                optionCImage: optionCImage || undefined,
                optionD: optionD.trim() || 'Đáp án D',
                optionDImage: optionDImage || undefined,
                correctAnswer,
                layout,
              }
            : item
        )
      )
    } else {
      // Add new quiz
      const newQuiz: QuizItem = {
        id: Date.now().toString(),
        question: question, // Keep HTML as is
        questionImage: questionImage || undefined,
        optionA: optionA.trim(),
        optionAImage: optionAImage || undefined,
        optionB: optionB.trim(),
        optionBImage: optionBImage || undefined,
        optionC: optionC.trim() || 'Đáp án C',
        optionCImage: optionCImage || undefined,
        optionD: optionD.trim() || 'Đáp án D',
        optionDImage: optionDImage || undefined,
        correctAnswer,
        layout,
      }
      setQuizItems([...quizItems, newQuiz])
    }

    resetForm()
  }

  const handleRemoveQuiz = (id: string) => {
    setQuizItems(quizItems.filter((item) => item.id !== id))
    // If deleting the quiz being edited, reset the form
    if (editingQuizId === id) {
      resetForm()
    }
  }

  const handleEditQuiz = (quiz: QuizItem) => {
    setQuestion(quiz.question)
    setQuestionImage(quiz.questionImage || '')
    setOptionA(quiz.optionA)
    setOptionAImage(quiz.optionAImage || '')
    setOptionB(quiz.optionB)
    setOptionBImage(quiz.optionBImage || '')
    setOptionC(quiz.optionC)
    setOptionCImage(quiz.optionCImage || '')
    setOptionD(quiz.optionD)
    setOptionDImage(quiz.optionDImage || '')
    setCorrectAnswer(quiz.correctAnswer)
    setLayout(quiz.layout || 'vertical')
    setEditingQuizId(quiz.id)
  }

  const handleUpdate = () => {
    if (!editor || quizGroupPos < 0 || quizItems.length === 0) return

    // Delete old quizGroup and insert new one at the same position
    const from = quizGroupPos
    const to = quizGroupPos + editor.state.doc.nodeAt(quizGroupPos)!.nodeSize

    editor
      .chain()
      .focus()
      .deleteRange({ from, to })
      .setTextSelection(from)
      .setQuizGroup(
        quizItems.map(({ question, questionImage, optionA, optionAImage, optionB, optionBImage, optionC, optionCImage, optionD, optionDImage, correctAnswer, layout }) => ({
          question,
          questionImage,
          optionA,
          optionAImage,
          optionB,
          optionBImage,
          optionC,
          optionCImage,
          optionD,
          optionDImage,
          correctAnswer,
          layout,
        }))
      )
      .run()

    setQuizItems([])
    resetForm()
    setEditDialogOpen(false)
  }

  const handleDelete = () => {
    if (!editor || quizGroupPos < 0) return

    const from = quizGroupPos
    const to = quizGroupPos + editor.state.doc.nodeAt(quizGroupPos)!.nodeSize

    editor.chain().focus().deleteRange({ from, to }).run()
  }

  if (!showMenu || !editor) return null

  return (
    <>
      {/* Bubble Menu */}
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
          onClick={handleEdit}
          title="Chỉnh sửa bài tập"
        >
          <Edit2 className="h-4 w-4" />
        </Button>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-destructive hover:text-destructive"
          onClick={handleDelete}
          title="Xóa bài tập"
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>

      {/* Edit Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="sm:max-w-[700px] max-h-[90vh]">
          <DialogHeader>
            <DialogTitle>Chỉnh sửa bài tập trắc nghiệm</DialogTitle>
            <DialogDescription>
              Cập nhật danh sách câu hỏi trắc nghiệm
            </DialogDescription>
          </DialogHeader>

          <ScrollArea className="max-h-[60vh] pr-4">
            <div className="space-y-6">
              {/* Quiz Form */}
              <div className="rounded-lg border border-border p-4 space-y-4 bg-muted/30">
                <div className="flex items-center justify-between">
                  <h4 className="font-semibold text-sm">Thêm câu hỏi mới</h4>
                </div>

                <div className="grid gap-4">
                  {/* Question */}
                  <div className="grid gap-2">
                    <Label htmlFor="edit-question">Câu hỏi *</Label>
                    <TiptapEditorClient
                      initialContent={question}
                      onChange={(json, html) => setQuestion(html)}
                      placeholder="Nhập câu hỏi..."
                      showLanguageSelector={false}
                      showThemeSwitcher={false}
                    />
                    <div className="flex items-center gap-2">
                      <input
                        type="file"
                        ref={questionImageRef}
                        accept="image/*"
                        className="hidden"
                        onChange={(e) => handleImageUpload(e.target.files?.[0] || null, setQuestionImage, 'question')}
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => questionImageRef.current?.click()}
                        disabled={uploading === 'question'}
                        className="gap-2"
                      >
                        <ImageIcon className="h-4 w-4" />
                        {uploading === 'question' ? 'Đang tải...' : 'Thêm hình ảnh'}
                      </Button>
                      {questionImage && (
                        <div className="relative">
                          <img src={questionImage} alt="Question" className="h-16 w-16 object-cover rounded border" />
                          <button
                            type="button"
                            onClick={() => setQuestionImage('')}
                            className="absolute -top-2 -right-2 bg-destructive text-destructive-foreground rounded-full p-1 hover:bg-destructive/90"
                          >
                            <X className="h-3 w-3" />
                          </button>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Options */}
                  <div className="grid gap-3">
                    <div className="grid gap-2">
                      <Label htmlFor="edit-optionA" className="text-sm">
                        A. *
                      </Label>
                      <Input
                        id="edit-optionA"
                        placeholder="Đáp án A"
                        value={optionA}
                        onChange={(e) => setOptionA(e.target.value)}
                      />
                      <div className="flex items-center gap-2">
                        <input
                          type="file"
                          ref={optionAImageRef}
                          accept="image/*"
                          className="hidden"
                          onChange={(e) => handleImageUpload(e.target.files?.[0] || null, setOptionAImage, 'optionA')}
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => optionAImageRef.current?.click()}
                          disabled={uploading === 'optionA'}
                          className="gap-2"
                        >
                          <ImageIcon className="h-3 w-3" />
                          {uploading === 'optionA' ? 'Đang tải...' : 'Hình'}
                        </Button>
                        {optionAImage && (
                          <div className="relative">
                            <img src={optionAImage} alt="Option A" className="h-12 w-12 object-cover rounded border" />
                            <button
                              type="button"
                              onClick={() => setOptionAImage('')}
                              className="absolute -top-1 -right-1 bg-destructive text-destructive-foreground rounded-full p-0.5 hover:bg-destructive/90"
                            >
                              <X className="h-2.5 w-2.5" />
                            </button>
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="grid gap-2">
                      <Label htmlFor="edit-optionB" className="text-sm">
                        B. *
                      </Label>
                      <Input
                        id="edit-optionB"
                        placeholder="Đáp án B"
                        value={optionB}
                        onChange={(e) => setOptionB(e.target.value)}
                      />
                      <div className="flex items-center gap-2">
                        <input
                          type="file"
                          ref={optionBImageRef}
                          accept="image/*"
                          className="hidden"
                          onChange={(e) => handleImageUpload(e.target.files?.[0] || null, setOptionBImage, 'optionB')}
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => optionBImageRef.current?.click()}
                          disabled={uploading === 'optionB'}
                          className="gap-2"
                        >
                          <ImageIcon className="h-3 w-3" />
                          {uploading === 'optionB' ? 'Đang tải...' : 'Hình'}
                        </Button>
                        {optionBImage && (
                          <div className="relative">
                            <img src={optionBImage} alt="Option B" className="h-12 w-12 object-cover rounded border" />
                            <button
                              type="button"
                              onClick={() => setOptionBImage('')}
                              className="absolute -top-1 -right-1 bg-destructive text-destructive-foreground rounded-full p-0.5 hover:bg-destructive/90"
                            >
                              <X className="h-2.5 w-2.5" />
                            </button>
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="grid gap-2">
                      <Label htmlFor="edit-optionC" className="text-sm">
                        C.
                      </Label>
                      <Input
                        id="edit-optionC"
                        placeholder="Đáp án C"
                        value={optionC}
                        onChange={(e) => setOptionC(e.target.value)}
                      />
                      <div className="flex items-center gap-2">
                        <input
                          type="file"
                          ref={optionCImageRef}
                          accept="image/*"
                          className="hidden"
                          onChange={(e) => handleImageUpload(e.target.files?.[0] || null, setOptionCImage, 'optionC')}
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => optionCImageRef.current?.click()}
                          disabled={uploading === 'optionC'}
                          className="gap-2"
                        >
                          <ImageIcon className="h-3 w-3" />
                          {uploading === 'optionC' ? 'Đang tải...' : 'Hình'}
                        </Button>
                        {optionCImage && (
                          <div className="relative">
                            <img src={optionCImage} alt="Option C" className="h-12 w-12 object-cover rounded border" />
                            <button
                              type="button"
                              onClick={() => setOptionCImage('')}
                              className="absolute -top-1 -right-1 bg-destructive text-destructive-foreground rounded-full p-0.5 hover:bg-destructive/90"
                            >
                              <X className="h-2.5 w-2.5" />
                            </button>
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="grid gap-2">
                      <Label htmlFor="edit-optionD" className="text-sm">
                        D.
                      </Label>
                      <Input
                        id="edit-optionD"
                        placeholder="Đáp án D"
                        value={optionD}
                        onChange={(e) => setOptionD(e.target.value)}
                      />
                      <div className="flex items-center gap-2">
                        <input
                          type="file"
                          ref={optionDImageRef}
                          accept="image/*"
                          className="hidden"
                          onChange={(e) => handleImageUpload(e.target.files?.[0] || null, setOptionDImage, 'optionD')}
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() => optionDImageRef.current?.click()}
                          disabled={uploading === 'optionD'}
                          className="gap-2"
                        >
                          <ImageIcon className="h-3 w-3" />
                          {uploading === 'optionD' ? 'Đang tải...' : 'Hình'}
                        </Button>
                        {optionDImage && (
                          <div className="relative">
                            <img src={optionDImage} alt="Option D" className="h-12 w-12 object-cover rounded border" />
                            <button
                              type="button"
                              onClick={() => setOptionDImage('')}
                              className="absolute -top-1 -right-1 bg-destructive text-destructive-foreground rounded-full p-0.5 hover:bg-destructive/90"
                            >
                              <X className="h-2.5 w-2.5" />
                            </button>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Correct Answer */}
                  <div className="grid gap-2">
                    <Label htmlFor="edit-correctAnswer">Đáp án đúng *</Label>
                    <Select
                      value={correctAnswer}
                      onValueChange={(value) => setCorrectAnswer(value as 'A' | 'B' | 'C' | 'D')}
                    >
                      <SelectTrigger id="edit-correctAnswer" className="w-32">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="A">A</SelectItem>
                        <SelectItem value="B">B</SelectItem>
                        <SelectItem value="C">C</SelectItem>
                        <SelectItem value="D">D</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <Button
                    type="button"
                    onClick={handleAddQuiz}
                    disabled={isHTMLEmpty(question) || !optionA.trim() || !optionB.trim()}
                    className="w-full gap-2"
                    variant="outline"
                  >
                    <Plus className="h-4 w-4" />
                    {editingQuizId ? 'Cập nhật câu hỏi' : 'Thêm câu hỏi vào danh sách'}
                  </Button>
                  {editingQuizId && (
                    <Button
                      type="button"
                      onClick={resetForm}
                      className="w-full"
                      variant="ghost"
                    >
                      Hủy chỉnh sửa
                    </Button>
                  )}
                </div>
              </div>

              {/* Quiz List */}
              {quizItems.length > 0 && (
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <h4 className="font-semibold text-sm">
                      Danh sách câu hỏi ({quizItems.length})
                    </h4>
                  </div>

                  <div className="space-y-3">
                    {quizItems.map((item, index) => (
                      <div
                        key={item.id}
                        className="rounded-lg border border-border p-3 space-y-2 bg-card"
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="flex-1 min-w-0">
                            <p className="font-medium text-sm mb-2 truncate" title={item.question.replace(/<[^>]*>/g, '')}>
                              {index + 1}. {getPlainTextPreview(item.question, 50)}
                            </p>
                            <div className="grid grid-cols-2 gap-1.5 text-xs text-muted-foreground">
                              <div className="flex gap-1 min-w-0">
                                <span className={`truncate ${item.correctAnswer === 'A' ? 'font-semibold text-green-600' : ''}`} title={`A. ${item.optionA}`}>
                                  A. {item.optionA}
                                </span>
                              </div>
                              <div className="flex gap-1 min-w-0">
                                <span className={`truncate ${item.correctAnswer === 'B' ? 'font-semibold text-green-600' : ''}`} title={`B. ${item.optionB}`}>
                                  B. {item.optionB}
                                </span>
                              </div>
                              <div className="flex gap-1 min-w-0">
                                <span className={`truncate ${item.correctAnswer === 'C' ? 'font-semibold text-green-600' : ''}`} title={`C. ${item.optionC}`}>
                                  C. {item.optionC}
                                </span>
                              </div>
                              <div className="flex gap-1 min-w-0">
                                <span className={`truncate ${item.correctAnswer === 'D' ? 'font-semibold text-green-600' : ''}`} title={`D. ${item.optionD}`}>
                                  D. {item.optionD}
                                </span>
                              </div>
                            </div>
                          </div>
                          <div className="flex gap-1">
                            <Button
                              type="button"
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7 flex-shrink-0"
                              onClick={() => handleEditQuiz(item)}
                              title="Sửa câu hỏi"
                            >
                              <Edit2 className="h-3.5 w-3.5" />
                            </Button>
                            <Button
                              type="button"
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7 flex-shrink-0 text-destructive hover:text-destructive"
                              onClick={() => handleRemoveQuiz(item.id)}
                              title="Xóa câu hỏi"
                            >
                              <Trash2 className="h-3.5 w-3.5" />
                            </Button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </ScrollArea>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setQuizItems([])
                resetForm()
                setEditDialogOpen(false)
              }}
            >
              Hủy
            </Button>
            <Button
              type="button"
              onClick={handleUpdate}
              disabled={quizItems.length === 0}
            >
              Cập nhật ({quizItems.length} câu)
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
