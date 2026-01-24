'use client'

import { useState, useEffect } from 'react'
import type { Editor } from '@tiptap/react'
import { Check, HelpCircle, Plus, Trash2, X } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import type { AnswerCheckAttributes, QuestionItem } from './answer-check-extension'

interface RichTextAnswerCheckProps {
  editor: Editor | null
}

export function RichTextAnswerCheck({ editor }: RichTextAnswerCheckProps) {
  const [open, setOpen] = useState(false)
  const [formData, setFormData] = useState<AnswerCheckAttributes>({
    title: '',
    questions: [],
    placeholder: 'Nhập câu trả lời...',
    hint: '',
    passingScore: 70,
  })

  useEffect(() => {
    console.log('[AnswerCheck] formData changed:', formData)
  }, [formData])

  const handleAddQuestion = () => {
    console.log('[AnswerCheck] Adding question, current count:', formData.questions.length)
    const newQuestions = [
      ...formData.questions,
      { prompt: '', correctAnswer: '' },
    ]
    console.log('[AnswerCheck] New questions count:', newQuestions.length)
    setFormData({
      ...formData,
      questions: newQuestions,
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

  const handleSubmit = () => {
    console.log('[AnswerCheck] handleSubmit called')
    console.log('[AnswerCheck] formData:', formData)

    if (!editor) {
      console.error('[AnswerCheck] No editor')
      return
    }

    if (formData.questions.length === 0) {
      console.error('[AnswerCheck] No questions')
      alert('Vui lòng thêm ít nhất một câu hỏi')
      return
    }

    // Validate all questions have both prompt and answer
    const isValid = formData.questions.every(
      (q) => q.prompt.trim() && q.correctAnswer.trim()
    )

    console.log('[AnswerCheck] isValid:', isValid)

    if (!isValid) {
      alert('Vui lòng điền đầy đủ câu hỏi và đáp án cho tất cả các mục')
      return
    }

    const attrs = {
      id: `ac-${Date.now()}`,
      title: formData.title?.trim() || '',
      questions: formData.questions.map((q) => ({
        prompt: q.prompt.trim(),
        correctAnswer: q.correctAnswer.trim(),
      })),
      placeholder: formData.placeholder || 'Nhập câu trả lời...',
      hint: formData.hint?.trim() || '',
      passingScore: formData.passingScore || 70,
    }

    console.log('[AnswerCheck] Inserting with attrs:', attrs)

    const result = editor
      .chain()
      .focus()
      .setAnswerCheck(attrs)
      .run()

    console.log('[AnswerCheck] Insert result:', result)

    if (result) {
      // Reset form after successful insert
      setFormData({
        title: '',
        questions: [],
        placeholder: 'Nhập câu trả lời...',
        hint: '',
        passingScore: 70,
      })
      setOpen(false)
    } else {
      alert('Không thể thêm component vào editor. Vui lòng thử lại.')
    }
  }

  if (!editor) {
    return null
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 gap-2 px-3"
          title="Thêm câu hỏi kiểm tra"
        >
          <Check className="h-4 w-4" />
          <span className="text-xs">Kiểm tra đáp án</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
        <DialogHeader className="flex-shrink-0">
          <DialogTitle className="flex items-center gap-2">
            <Check className="h-5 w-5" />
            Thêm câu hỏi kiểm tra với WASM
          </DialogTitle>
          <DialogDescription>
            Tạo nhiều câu hỏi cho phép học viên nhập đáp án và kiểm tra độ chính xác
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4 overflow-y-auto flex-1">
          {/* Title */}
          <div className="space-y-2">
            <Label htmlFor="title">
              Tiêu đề
              <span className="ml-1 text-xs text-muted-foreground">(Tùy chọn)</span>
            </Label>
            <Input
              id="title"
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
                          htmlFor={`prompt-${index}`}
                          className="text-xs"
                        >
                          Câu hỏi / Gợi ý
                        </Label>
                        <Input
                          id={`prompt-${index}`}
                          placeholder="VD: 🇫🇷 프랑스 →"
                          value={question.prompt}
                          onChange={(e) =>
                            handleUpdateQuestion(index, 'prompt', e.target.value)
                          }
                        />
                      </div>
                      <div className="space-y-1">
                        <Label
                          htmlFor={`answer-${index}`}
                          className="text-xs"
                        >
                          Câu trả lời đúng
                        </Label>
                        <Input
                          id={`answer-${index}`}
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
            <Label htmlFor="placeholder">Placeholder</Label>
            <Input
              id="placeholder"
              placeholder="Nhập câu trả lời..."
              value={formData.placeholder}
              onChange={(e) =>
                setFormData({ ...formData, placeholder: e.target.value })
              }
            />
          </div>

          {/* Hint */}
          <div className="space-y-2">
            <Label htmlFor="hint" className="flex items-center gap-2">
              <HelpCircle className="h-4 w-4" />
              Gợi ý chung
              <span className="ml-1 text-xs text-muted-foreground">(Tùy chọn)</span>
            </Label>
            <Textarea
              id="hint"
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
            <Label htmlFor="passingScore">Điểm đạt (%)</Label>
            <Input
              id="passingScore"
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
            <p className="text-xs text-muted-foreground">
              Điểm tối thiểu (0-100%) để coi là đạt. Mặc định: 70%
            </p>
          </div>

          {/* Example */}
          <div className="rounded-lg border bg-muted/50 p-3">
            <h4 className="mb-2 text-sm font-semibold">Ví dụ:</h4>
            <div className="space-y-1 text-xs">
              <p className="font-medium">Tiêu đề: Điền tên quốc gia bằng tiếng Việt</p>
              <p className="text-muted-foreground">Câu hỏi:</p>
              <ul className="ml-4 space-y-0.5 text-muted-foreground">
                <li>(1) 🇫🇷 프랑스 → <span className="font-mono">Pháp</span></li>
                <li>(2) 🇨🇳 중국 → <span className="font-mono">Trung Quốc</span></li>
                <li>(3) 🇰🇷 한국 → <span className="font-mono">Hàn Quốc</span></li>
              </ul>
            </div>
          </div>
        </div>

        <DialogFooter className="flex-shrink-0">
          <Button
            type="button"
            variant="outline"
            onClick={() => setOpen(false)}
          >
            <X className="mr-2 h-4 w-4" />
            Hủy
          </Button>
          <Button
            type="button"
            onClick={handleSubmit}
            disabled={formData.questions.length === 0}
          >
            <Check className="mr-2 h-4 w-4" />
            Thêm vào bài học
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
