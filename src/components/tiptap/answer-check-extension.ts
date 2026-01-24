import { Node, mergeAttributes } from '@tiptap/core'

export interface AnswerCheckOptions {
  HTMLAttributes: Record<string, any>
}

export interface QuestionItem {
  prompt: string // VD: "🇫🇷 프랑스 →"
  correctAnswer: string
}

export interface AnswerCheckAttributes {
  id?: string
  title?: string // Tiêu đề chung cho nhóm câu hỏi
  questions: QuestionItem[] // Mảng các câu hỏi
  placeholder?: string
  hint?: string
  passingScore?: number // Điểm tối thiểu để đạt (mặc định 70)
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    answerCheck: {
      /**
       * Insert an answer check component with multiple questions
       */
      setAnswerCheck: (attributes: AnswerCheckAttributes) => ReturnType
    }
  }
}

export const AnswerCheck = Node.create<AnswerCheckOptions>({
  name: 'answerCheck',

  group: 'block',

  content: '', // No child content

  atom: true, // Treat as single unit

  addOptions() {
    return {
      HTMLAttributes: {},
    }
  },

  addAttributes() {
    return {
      id: {
        default: null,
        parseHTML: (element) => element.getAttribute('data-id'),
        renderHTML: (attributes) => ({
          'data-id': attributes.id,
        }),
      },
      title: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-title'),
        renderHTML: (attributes) => ({
          'data-title': attributes.title,
        }),
      },
      questions: {
        default: [],
        parseHTML: (element) => {
          const questionsAttr = element.getAttribute('data-questions')
          try {
            return questionsAttr ? JSON.parse(questionsAttr) : []
          } catch {
            return []
          }
        },
        renderHTML: (attributes) => ({
          'data-questions': JSON.stringify(attributes.questions || []),
        }),
      },
      placeholder: {
        default: 'Nhập câu trả lời...',
        parseHTML: (element) => element.getAttribute('data-placeholder'),
        renderHTML: (attributes) => ({
          'data-placeholder': attributes.placeholder,
        }),
      },
      hint: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-hint'),
        renderHTML: (attributes) => ({
          'data-hint': attributes.hint,
        }),
      },
      passingScore: {
        default: 70,
        parseHTML: (element) => {
          const score = element.getAttribute('data-passing-score')
          return score ? parseInt(score, 10) : 70
        },
        renderHTML: (attributes) => ({
          'data-passing-score': attributes.passingScore,
        }),
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="answer-check"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'div',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        'data-type': 'answer-check',
        class: 'answer-check-node my-4',
      }),
    ]
  },

  addCommands() {
    return {
      setAnswerCheck:
        (attributes) =>
        ({ commands }) => {
          return commands.insertContent({
            type: this.name,
            attrs: {
              id: attributes.id || `ac-${Date.now()}`,
              title: attributes.title || '',
              questions: attributes.questions || [],
              placeholder: attributes.placeholder || 'Nhập câu trả lời...',
              hint: attributes.hint || '',
              passingScore: attributes.passingScore || 70,
            },
          })
        },
    }
  },

  addNodeView() {
    return ({ node, editor, getPos }) => {
      const dom = document.createElement('div')
      dom.setAttribute('data-type', 'answer-check')
      dom.className = 'answer-check-node my-4 p-4 border border-border rounded-lg bg-card'

      const render = () => {
        const isEditable = editor.isEditable
        const { title, questions, placeholder, hint, passingScore } = node.attrs

        if (isEditable) {
          // Admin mode: Show configuration
          dom.innerHTML = `
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-semibold text-primary">✍️ Kiểm tra câu trả lời (Multi)</span>
                  <span class="text-xs text-muted-foreground">(Admin mode)</span>
                </div>
              </div>
              ${title ? `
                <div class="text-sm">
                  <span class="font-medium text-muted-foreground">Tiêu đề:</span>
                  <p class="mt-1">${title}</p>
                </div>
              ` : ''}
              <div class="text-sm">
                <span class="font-medium text-muted-foreground">Số câu hỏi:</span>
                <span class="ml-2 font-semibold">${questions.length}</span>
              </div>
              ${questions.length > 0 ? `
                <div class="text-sm">
                  <span class="font-medium text-muted-foreground">Câu hỏi:</span>
                  <div class="mt-2 space-y-1 rounded bg-muted/50 p-2">
                    ${questions.map((q: QuestionItem, i: number) => `
                      <div class="text-xs">
                        <span class="text-muted-foreground">(${i + 1})</span>
                        <span class="ml-1">${q.prompt}</span>
                        <span class="ml-2 font-mono text-green-600 dark:text-green-400">${q.correctAnswer}</span>
                      </div>
                    `).join('')}
                  </div>
                </div>
              ` : `
                <p class="text-xs text-muted-foreground italic">Chưa có câu hỏi nào</p>
              `}
              <div class="grid gap-2 md:grid-cols-2">
                <div class="text-sm">
                  <span class="font-medium text-muted-foreground">Điểm đạt:</span>
                  <p class="mt-1">${passingScore}%</p>
                </div>
              </div>
              ${hint ? `
                <div class="text-sm">
                  <span class="font-medium text-muted-foreground">Gợi ý:</span>
                  <p class="mt-1 text-xs italic">${hint}</p>
                </div>
              ` : ''}
              <div class="text-xs text-muted-foreground">
                Click vào component này để chỉnh sửa cấu hình
              </div>
            </div>
          `
        } else {
          // User mode: Show interactive answer checker with multiple inputs
          dom.innerHTML = `
            <div class="space-y-4" data-answer-check-id="${node.attrs.id}">
              ${title ? `
                <div class="text-base font-semibold">
                  ${title}
                </div>
              ` : ''}
              <div class="space-y-3">
                ${questions.map((q: QuestionItem, index: number) => `
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium whitespace-nowrap">${q.prompt}</span>
                    <input
                      type="text"
                      class="answer-check-input flex-1 rounded-lg border border-input bg-background px-3 py-2 text-sm focus:border-primary focus:outline-none"
                      placeholder="${placeholder}"
                      data-answer-index="${index}"
                    />
                  </div>
                `).join('')}
              </div>
              <div class="flex gap-2">
                <button
                  class="answer-check-button bg-primary text-primary-foreground hover:bg-primary/90 rounded-lg px-4 py-2 text-sm font-medium transition-colors disabled:opacity-50"
                  data-answer-check-button
                >
                  Kiểm tra
                </button>
                ${hint ? `
                  <button
                    class="answer-check-hint-button border border-input bg-background hover:bg-accent rounded-lg px-4 py-2 text-sm font-medium transition-colors"
                    data-answer-check-hint-button
                  >
                    Xem gợi ý
                  </button>
                ` : ''}
              </div>
              <div class="answer-check-result hidden" data-answer-check-result></div>
            </div>
          `

          // Add event listeners for user interaction
          const inputs = Array.from(dom.querySelectorAll('[data-answer-index]')) as HTMLInputElement[]
          const button = dom.querySelector('[data-answer-check-button]') as HTMLButtonElement
          const resultDiv = dom.querySelector('[data-answer-check-result]') as HTMLDivElement
          const hintButton = dom.querySelector('[data-answer-check-hint-button]') as HTMLButtonElement

          if (button && inputs.length > 0) {
            // Hide result when user types in any input
            inputs.forEach((input) => {
              input.addEventListener('input', () => {
                resultDiv.classList.add('hidden')
              })
            })

            const handleCheck = async () => {
              // Get all user answers
              const userAnswers = inputs.map((input) => input.value.trim())

              // Check if any answer is empty
              if (userAnswers.some((answer) => !answer)) {
                resultDiv.className = 'answer-check-result rounded-lg border-2 border-orange-500 bg-orange-50 dark:bg-orange-950 p-3'
                resultDiv.innerHTML = `
                  <p class="text-sm text-orange-800 dark:text-orange-200">
                    Vui lòng điền tất cả các câu trả lời trước khi kiểm tra.
                  </p>
                `
                resultDiv.classList.remove('hidden')
                return
              }

              button.disabled = true
              button.textContent = 'Đang kiểm tra...'

              try {
                // Import and use API similarity checker
                const { checkAnswersAPI } = await import('@/lib/api-similarity')

                // Prepare questions for API call
                const apiQuestions = questions.map((q: QuestionItem, index: number) => ({
                  prompt: q.prompt,
                  userAnswer: userAnswers[index] || '',
                  correctAnswer: q.correctAnswer,
                }))

                // Call API to check answers
                const apiResponse = await checkAnswersAPI(apiQuestions, passingScore)

                // Map API response to expected format
                const results = apiResponse.results

                // Use values from API response
                const totalScore = apiResponse.overallScore
                const overallPassed = apiResponse.passed
                const correctCount = apiResponse.correctCount
                const evaluation = apiResponse.evaluation

                // Show result
                const bgColor = overallPassed
                  ? 'bg-green-50 dark:bg-green-950 border-green-500'
                  : 'bg-red-50 dark:bg-red-950 border-red-500'
                const textColor = overallPassed
                  ? 'text-green-800 dark:text-green-200'
                  : 'text-red-800 dark:text-red-200'
                const icon = overallPassed ? '✓' : '✗'

                resultDiv.className = `answer-check-result rounded-lg border-2 ${bgColor} p-4`
                resultDiv.innerHTML = `
                  <div class="space-y-3">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <span class="text-2xl">${icon}</span>
                        <div>
                          <p class="text-sm font-semibold ${textColor}">
                            ${evaluation.message}
                          </p>
                          <p class="text-xs ${textColor} mt-0.5">
                            Đúng ${correctCount}/${questions.length} câu
                          </p>
                        </div>
                      </div>
                      <span class="text-2xl font-bold ${textColor}">
                        ${Math.round(totalScore)}%
                      </span>
                    </div>

                    ${!overallPassed ? `
                      <div class="border-t border-current/20 pt-3 space-y-2">
                        <p class="text-xs font-medium ${textColor}">Chi tiết:</p>
                        ${results.map((r) => `
                          <div class="text-xs ${textColor} flex items-start gap-2">
                            <span class="${r.passed ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
                              ${r.passed ? '✓' : '✗'}
                            </span>
                            <div class="flex-1">
                              <span>${r.prompt}</span>
                              ${!r.passed ? `
                                <div class="mt-0.5">
                                  <span class="opacity-70">Bạn: </span>
                                  <span class="font-mono">${r.userAnswer}</span>
                                  <br>
                                  <span class="opacity-70">Đúng: </span>
                                  <span class="font-mono font-semibold">${r.correctAnswer}</span>
                                </div>
                              ` : ''}
                            </div>
                            <span class="font-mono text-xs">${r.score}%</span>
                          </div>
                        `).join('')}
                      </div>
                    ` : ''}
                  </div>
                `
                resultDiv.classList.remove('hidden')
              } catch (error) {
                console.error('Error checking answer:', error)
                resultDiv.className = 'answer-check-result rounded-lg border-2 border-red-500 bg-red-50 dark:bg-red-950 p-3'
                resultDiv.innerHTML = `
                  <p class="text-sm text-red-800 dark:text-red-200">
                    Có lỗi xảy ra khi kiểm tra câu trả lời. Vui lòng thử lại.
                  </p>
                `
                resultDiv.classList.remove('hidden')
              } finally {
                button.disabled = false
                button.textContent = 'Kiểm tra'
              }
            }

            button.addEventListener('click', handleCheck)
            inputs.forEach((input) => {
              input.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                  handleCheck()
                }
              })
            })
          }

          // Hint button
          if (hintButton && hint) {
            hintButton.addEventListener('click', () => {
              resultDiv.className = 'answer-check-result rounded-lg border-2 border-blue-500 bg-blue-50 dark:bg-blue-950 p-3'
              resultDiv.innerHTML = `
                <div class="text-sm text-blue-800 dark:text-blue-200">
                  <p class="font-medium mb-1">💡 Gợi ý:</p>
                  <p>${hint}</p>
                </div>
              `
              resultDiv.classList.remove('hidden')
            })
          }
        }
      }

      render()

      return {
        dom,
        update: (updatedNode) => {
          if (updatedNode.type !== this.type) {
            return false
          }
          render()
          return true
        },
      }
    }
  },
})
