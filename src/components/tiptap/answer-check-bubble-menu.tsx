'use client'

import { useState, useEffect } from 'react'
import type { Editor } from '@tiptap/react'
import { Check, HelpCircle, Plus, Trash2, X } from 'lucide-react'
import { BaseBubbleMenu } from './base/BaseBubbleMenu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import type { AnswerCheckAttributes, QuestionItem } from './answer-check-extension'

interface AnswerCheckBubbleMenuProps {
  editor: Editor | null
}

export function AnswerCheckBubbleMenu({ editor }: AnswerCheckBubbleMenuProps) {
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [formData, setFormData] = useState<AnswerCheckAttributes>({
    title: '',
    questions: [],
    placeholder: 'Nhập câu trả lời...',
    hint: '',
    passingScore: 70,
  })

  // Load current node attributes when dialog opens
  useEffect(() => {
    if (showEditDialog && editor) {
      const attrs = editor.getAttributes('answerCheck')
      setFormData({
        title: attrs.title || '',
        questions: attrs.questions || [],
        placeholder: attrs.placeholder || 'Nhập câu trả lời...',
        hint: attrs.hint || '',
        passingScore: attrs.passingScore || 70,
      })
    }
  }, [showEditDialog, editor])

  const handleAddQuestion = () => {
    setFormData({
      ...formData,
      questions: [
        ...formData.questions,
        { prompt: '', correctAnswer: '' },
      ],
    })
  }

  const handleRemoveQuestion = (index: number) => {
    setFormData({
      ...formData,
      questions: formData.questions.filter((_, i) => i !== index),
    })
  }

  const handleUpdateQuestion = (
    index: number,
    field: keyof QuestionItem,
    value: string
  ) => {
    const newQuestions = [...formData.questions]
    const currentQuestion = newQuestions[index]
    if (currentQuestion) {
      newQuestions[index] = {
        prompt: currentQuestion.prompt,
        correctAnswer: currentQuestion.correctAnswer,
        [field]: value
      }
    }
    setFormData({ ...formData, questions: newQuestions })
  }

  const handleUpdate = () => {
    if (!editor || formData.questions.length === 0) {
      return
    }

    // Validate all questions have both prompt and answer
    const isValid = formData.questions.every(
      (q) => q.prompt.trim() && q.correctAnswer.trim()
    )

    if (!isValid) {
      alert('Vui lòng điền đầy đủ câu hỏi và đáp án cho tất cả các mục')
      return
    }

    editor
      .chain()
      .focus()
      .updateAttributes('answerCheck', {
        title: formData.title?.trim() || '',
        questions: formData.questions.map((q) => ({
          prompt: q.prompt.trim(),
          correctAnswer: q.correctAnswer.trim(),
        })),
        placeholder: formData.placeholder || 'Nhập câu trả lời...',
        hint: formData.hint?.trim() || '',
        passingScore: formData.passingScore || 70,
      })
      .run()

    setShowEditDialog(false)
  }

  const handleDelete = () => {
    if (!editor) return
    editor.chain().focus().deleteSelection().run()
  }

  return (
    <>
      <BaseBubbleMenu
        editor={editor}
        nodeName="answerCheck"
        onEdit={() => setShowEditDialog(true)}
        onDelete={handleDelete}
      />

      {/* Edit Dialog */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
          <DialogHeader className="flex-shrink-0">
            <DialogTitle className="flex items-center gap-2">
              <Check className="h-5 w-5" />
              Chỉnh sửa câu hỏi kiểm tra
            </DialogTitle>
            <DialogDescription>
              Cập nhật danh sách câu hỏi và đáp án
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4 overflow-y-auto flex-1">
            {/* Title */}
            <div className="space-y-2">
              <Label htmlFor="edit-title">
                Tiêu đề
                <span className="ml-1 text-xs text-muted-foreground">(Tùy chọn)</span>
              </Label>
              <Input
                id="edit-title"
                placeholder="VD: Điền tên quốc gia bằng tiếng Việt"
                value={formData.title}
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
              />
            </div>

            {/* Questions List */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <Label>Danh sách câu hỏi <span className="text-destructive">*</span></Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={handleAddQuestion}
                  className="h-8"
                >
                  <Plus className="mr-1 h-4 w-4" />
                  Thêm câu hỏi
                </Button>
              </div>

              {formData.questions.length === 0 ? (
                <div className="rounded-lg border border-dashed border-muted-foreground/25 p-8 text-center">
                  <p className="text-sm text-muted-foreground">
                    Chưa có câu hỏi nào. Click "Thêm câu hỏi" để bắt đầu.
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  {formData.questions.map((question, index) => (
                    <div
                      key={index}
                      className="rounded-lg border bg-muted/30 p-3 space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">
                          Câu hỏi #{index + 1}
                        </span>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => handleRemoveQuestion(index)}
                          className="h-7 text-destructive hover:text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                      <div className="grid gap-2 md:grid-cols-2">
                        <div className="space-y-1">
                          <Label
                            htmlFor={`edit-prompt-${index}`}
                            className="text-xs"
                          >
                            Câu hỏi / Gợi ý
                          </Label>
                          <Input
                            id={`edit-prompt-${index}`}
                            placeholder="VD: 🇫🇷 프랑스 →"
                            value={question.prompt}
                            onChange={(e) =>
                              handleUpdateQuestion(index, 'prompt', e.target.value)
                            }
                          />
                        </div>
                        <div className="space-y-1">
                          <Label
                            htmlFor={`edit-answer-${index}`}
                            className="text-xs"
                          >
                            Câu trả lời đúng
                          </Label>
                          <Input
                            id={`edit-answer-${index}`}
                            placeholder="VD: Pháp"
                            value={question.correctAnswer}
                            onChange={(e) =>
                              handleUpdateQuestion(
                                index,
                                'correctAnswer',
                                e.target.value
                              )
                            }
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Placeholder */}
            <div className="space-y-2">
              <Label htmlFor="edit-placeholder">Placeholder</Label>
              <Input
                id="edit-placeholder"
                placeholder="Nhập câu trả lời..."
                value={formData.placeholder}
                onChange={(e) =>
                  setFormData({ ...formData, placeholder: e.target.value })
                }
              />
            </div>

            {/* Hint */}
            <div className="space-y-2">
              <Label htmlFor="edit-hint" className="flex items-center gap-2">
                <HelpCircle className="h-4 w-4" />
                Gợi ý chung
                <span className="ml-1 text-xs text-muted-foreground">(Tùy chọn)</span>
              </Label>
              <Textarea
                id="edit-hint"
                placeholder="VD: Hãy dịch tên quốc gia sang tiếng Việt..."
                value={formData.hint}
                onChange={(e) =>
                  setFormData({ ...formData, hint: e.target.value })
                }
                rows={2}
              />
            </div>

            {/* Passing Score */}
            <div className="space-y-2">
              <Label htmlFor="edit-passingScore">Điểm đạt (%)</Label>
              <Input
                id="edit-passingScore"
                type="number"
                min="0"
                max="100"
                value={formData.passingScore}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    passingScore: parseInt(e.target.value, 10) || 70,
                  })
                }
              />
            </div>
          </div>

          <DialogFooter className="flex-shrink-0">
            <Button
              type="button"
              variant="outline"
              onClick={() => setShowEditDialog(false)}
            >
              <X className="mr-2 h-4 w-4" />
              Hủy
            </Button>
            <Button
              type="button"
              onClick={handleUpdate}
              disabled={formData.questions.length === 0}
            >
              <Check className="mr-2 h-4 w-4" />
              Cập nhật
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
