import { Node, mergeAttributes } from '@tiptap/core'

export interface QuizGroupOptions {
  HTMLAttributes: Record<string, any>
  // Quiz tracking options
  lessonId?: number
  groupIndex?: number
  onSubmit?: (submission: {
    quizGroupIndex: number
    answers: {
      itemIndex: number
      selectedAnswer: 'A' | 'B' | 'C' | 'D'
      timeSpentSeconds?: number
    }[]
  }) => Promise<{
    results: {
      itemIndex: number
      isCorrect: boolean
      correctAnswer: 'A' | 'B' | 'C' | 'D'
    }[]
    totalItems: number
    correctCount: number
    quizCompletionPercentage: number
    earnedXp: number
    earnedCoins: number
  }>
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    quizGroup: {
      /**
       * Insert a quiz group with multiple quiz items
       */
      setQuizGroup: (quizItems: Array<{
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
      }>) => ReturnType
    }
  }
}

export const QuizGroup = Node.create<QuizGroupOptions>({
  name: 'quizGroup',

  group: 'block',

  content: 'quizItem+',

  addOptions() {
    return {
      HTMLAttributes: {},
      lessonId: undefined,
      groupIndex: undefined,
      onSubmit: undefined,
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="quiz-group"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'div',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'quiz-group',
        class: 'quiz-group my-6',
      }),
      0, // contentDOM placeholder - children will be rendered here
    ]
  },

  addCommands() {
    return {
      setQuizGroup:
        (quizItems) =>
        ({ commands }) => {
          // Create content with quiz items
          const content = quizItems.map((item) => ({
            type: 'quizItem',
            attrs: item,
          }))

          return commands.insertContent({
            type: this.name,
            content,
          })
        },
    }
  },

  addNodeView() {
    return ({ node: initialNode, editor, getPos }) => {
      let node = initialNode // Make node mutable so it can be updated

      const dom = document.createElement('div')
      dom.setAttribute('data-type', 'quiz-group')
      dom.className = 'quiz-group my-6'

      // Store selected answers in memory (not persisted to document)
      const selectedAnswers = new Map<number, string>()
      let isChecked = false

      // Content container for quiz items (custom rendered)
      const contentContainer = document.createElement('div')
      contentContainer.className = 'quiz-items-container space-y-4'

      // Hidden container for Tiptap to render children (for save/load)
      const contentDOM = document.createElement('div')
      contentDOM.style.display = 'none'
      dom.appendChild(contentDOM)

      // Render quiz items
      const renderQuizItems = () => {
        contentContainer.innerHTML = ''
        const quizItems: any[] = []

        // Extract quiz items from node content
        node.content.forEach((child, offset, index) => {
          if (child.type.name === 'quizItem') {
            quizItems.push({
              node: child,
              index: index,
            })
          }
        })

        quizItems.forEach(({ node: quizNode, index: itemIndex }) => {
          const quizItemDiv = document.createElement('div')
          quizItemDiv.className = 'quiz-item mb-4 rounded-lg border border-border bg-card/50 p-4'

          // Question - HTML content and optional image
          const questionContainer = document.createElement('div')
          questionContainer.className = 'mb-3'

          const questionText = document.createElement('div')
          questionText.className = 'font-medium text-sm mb-2 prose prose-sm max-w-none'
          questionText.innerHTML = quizNode.attrs.question || ''
          questionContainer.appendChild(questionText)

          // Question image if exists
          if (quizNode.attrs.questionImage) {
            const questionImg = document.createElement('img')
            questionImg.src = quizNode.attrs.questionImage
            questionImg.alt = 'Question image'
            questionImg.className = 'rounded-md max-w-full h-auto max-h-48 object-contain'
            questionContainer.appendChild(questionImg)
          }

          quizItemDiv.appendChild(questionContainer)

          // Options
          const optionsContainer = document.createElement('div')
          const layout = quizNode.attrs.layout || 'vertical'
          optionsContainer.className = layout === 'grid' ? 'grid grid-cols-2 gap-2' : 'space-y-1.5'

          const options = [
            { key: 'A', text: quizNode.attrs.optionA, image: quizNode.attrs.optionAImage },
            { key: 'B', text: quizNode.attrs.optionB, image: quizNode.attrs.optionBImage },
            { key: 'C', text: quizNode.attrs.optionC, image: quizNode.attrs.optionCImage },
            { key: 'D', text: quizNode.attrs.optionD, image: quizNode.attrs.optionDImage },
          ]

          const selectedAnswer = selectedAnswers.get(itemIndex)
          const correctAnswer = quizNode.attrs.correctAnswer

          options.forEach(({ key, text, image }) => {
            const button = document.createElement('button')
            button.type = 'button'
            button.className =
              'quiz-option w-full text-left px-3 py-2.5 rounded-md border-2 transition-all duration-200 text-sm flex items-start gap-2'
            button.setAttribute('data-option', key)

            // Create checkmark icon (hidden by default)
            const checkIcon = `
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="check-icon flex-shrink-0 mt-0.5">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            `

            const crossIcon = `
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="flex-shrink-0 text-red-600 mt-0.5">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            `

            // Build option content HTML
            const imageHTML = image ? `<img src="${image}" alt="Option ${key}" class="rounded max-w-full h-auto max-h-32 object-contain mt-1" />` : ''

            // Highlight selected answer BEFORE checking
            if (!isChecked && selectedAnswer === key) {
              button.classList.add('border-border', 'bg-background')
              button.innerHTML = `${checkIcon}<div class="flex-1"><div><span class="font-semibold mr-2">${key}.</span><span>${text}</span></div>${imageHTML}</div>`
            } else if (!isChecked) {
              button.classList.add('border-border', 'bg-background')
              button.innerHTML = `<div class="flex-1"><div><span class="font-semibold mr-2">${key}.</span><span>${text}</span></div>${imageHTML}</div>`
            }

            // After checking, highlight correct/incorrect
            if (isChecked) {
              if (key === correctAnswer) {
                button.classList.add('border-green-500', 'bg-green-100', 'dark:bg-green-900/30')
                button.innerHTML = `${checkIcon}<div class="flex-1"><div><span class="font-semibold mr-2 text-green-700 dark:text-green-400">${key}.</span><span class="text-green-700 dark:text-green-400">${text}</span></div>${imageHTML}</div>`
              } else if (selectedAnswer === key) {
                button.classList.add('border-red-500', 'bg-red-100', 'dark:bg-red-900/30')
                button.innerHTML = `${crossIcon}<div class="flex-1"><div><span class="font-semibold mr-2 text-red-700 dark:text-red-400">${key}.</span><span class="text-red-700 dark:text-red-400">${text}</span></div>${imageHTML}</div>`
              } else {
                button.classList.add('border-border', 'bg-muted/30', 'opacity-60')
                button.innerHTML = `<div class="flex-1"><div><span class="font-semibold mr-2">${key}.</span><span>${text}</span></div>${imageHTML}</div>`
              }
              button.disabled = true
              button.classList.add('cursor-not-allowed')
            } else {
              // Click handler for selection - ONLY when NOT in edit mode
              button.addEventListener('click', (e) => {
                e.preventDefault()
                e.stopPropagation()

                if (editor.isEditable) {
                  return
                }

                selectedAnswers.set(itemIndex, key)
                renderQuizItems()
              })
            }

            optionsContainer.appendChild(button)
          })

          quizItemDiv.appendChild(optionsContainer)
          contentContainer.appendChild(quizItemDiv)
        })
      }

      renderQuizItems()
      dom.appendChild(contentContainer)

      // Footer with check button
      const footer = document.createElement('div')
      footer.className = 'flex items-center justify-between pt-4 mt-4'

      const checkButton = document.createElement('button')
      checkButton.type = 'button'
      checkButton.className =
        'quiz-check-all-btn px-6 py-2.5 bg-primary text-primary-foreground rounded-md font-semibold hover:bg-primary/90 transition-colors shadow-sm'
      checkButton.textContent = 'Kiểm tra'

      const feedback = document.createElement('div')
      feedback.className = 'quiz-group-feedback hidden text-base font-semibold'

      checkButton.addEventListener('click', async (e) => {
        e.preventDefault()
        e.stopPropagation()

        if (editor.isEditable) {
          return
        }

        // Count quiz items and check answers
        let totalQuestions = 0
        let correctAnswers = 0
        let answeredQuestions = 0

        // Extract quiz items from node content
        node.content.forEach((child, offset, index) => {
          if (child.type.name === 'quizItem') {
            totalQuestions++

            const correctAnswer = child.attrs.correctAnswer
            const selectedAnswer = selectedAnswers.get(index)

            if (selectedAnswer) {
              answeredQuestions++

              if (selectedAnswer === correctAnswer) {
                correctAnswers++
              }
            }
          }
        })

        // Check if all questions were answered
        if (answeredQuestions < totalQuestions) {
          feedback.textContent = `Bạn cần trả lời tất cả ${totalQuestions} câu hỏi!`
          feedback.className = 'quiz-group-feedback text-base font-semibold text-yellow-600'
          feedback.classList.remove('hidden')
          return
        }

        // Mark as checked
        isChecked = true
        renderQuizItems()

        // Show result
        const percentage = Math.round((correctAnswers / totalQuestions) * 100)

        if (percentage === 100) {
          feedback.textContent = `🎉 Xuất sắc! ${correctAnswers}/${totalQuestions} câu đúng (${percentage}%)`
          feedback.className = 'quiz-group-feedback text-base font-semibold text-green-600'
        } else if (percentage >= 70) {
          feedback.textContent = `✓ Khá tốt! ${correctAnswers}/${totalQuestions} câu đúng (${percentage}%)`
          feedback.className = 'quiz-group-feedback text-base font-semibold text-blue-600'
        } else if (percentage >= 50) {
          feedback.textContent = `${correctAnswers}/${totalQuestions} câu đúng (${percentage}%) - Cần cố gắng thêm!`
          feedback.className = 'quiz-group-feedback text-base font-semibold text-orange-600'
        } else {
          feedback.textContent = `${correctAnswers}/${totalQuestions} câu đúng (${percentage}%) - Hãy học lại nhé!`
          feedback.className = 'quiz-group-feedback text-base font-semibold text-red-600'
        }

        feedback.classList.remove('hidden')

        // Disable check button
        checkButton.disabled = true
        checkButton.classList.add('opacity-50', 'cursor-not-allowed')
      })

      footer.appendChild(checkButton)
      footer.appendChild(feedback)
      dom.appendChild(footer)

      return {
        dom,
        contentDOM, // Important: allows Tiptap to render children for save/load
        update: (updatedNode) => {
          if (updatedNode.type.name !== 'quizGroup') return false
          // Update the node reference with the new node
          node = updatedNode
          // Re-render when content changes
          renderQuizItems()
          return true
        },
      }
    }
  },
})
