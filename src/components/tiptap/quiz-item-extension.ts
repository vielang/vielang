import { Node, mergeAttributes } from '@tiptap/core'

export interface QuizItemOptions {
  HTMLAttributes: Record<string, any>
}

export const QuizItem = Node.create<QuizItemOptions>({
  name: 'quizItem',

  group: 'block',

  atom: true,

  addOptions() {
    return {
      HTMLAttributes: {},
    }
  },

  addAttributes() {
    return {
      question: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-question'),
        renderHTML: (attributes) => ({
          'data-question': attributes.question,
        }),
      },
      questionImage: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-question-image'),
        renderHTML: (attributes) => ({
          'data-question-image': attributes.questionImage,
        }),
      },
      optionA: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-a'),
        renderHTML: (attributes) => ({
          'data-option-a': attributes.optionA,
        }),
      },
      optionAImage: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-a-image'),
        renderHTML: (attributes) => ({
          'data-option-a-image': attributes.optionAImage,
        }),
      },
      optionB: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-b'),
        renderHTML: (attributes) => ({
          'data-option-b': attributes.optionB,
        }),
      },
      optionBImage: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-b-image'),
        renderHTML: (attributes) => ({
          'data-option-b-image': attributes.optionBImage,
        }),
      },
      optionC: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-c'),
        renderHTML: (attributes) => ({
          'data-option-c': attributes.optionC,
        }),
      },
      optionCImage: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-c-image'),
        renderHTML: (attributes) => ({
          'data-option-c-image': attributes.optionCImage,
        }),
      },
      optionD: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-d'),
        renderHTML: (attributes) => ({
          'data-option-d': attributes.optionD,
        }),
      },
      optionDImage: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-option-d-image'),
        renderHTML: (attributes) => ({
          'data-option-d-image': attributes.optionDImage,
        }),
      },
      correctAnswer: {
        default: 'A',
        parseHTML: (element) => element.getAttribute('data-correct-answer'),
        renderHTML: (attributes) => ({
          'data-correct-answer': attributes.correctAnswer,
        }),
      },
      layout: {
        default: 'vertical',
        parseHTML: (element) => element.getAttribute('data-layout') || 'vertical',
        renderHTML: (attributes) => ({
          'data-layout': attributes.layout || 'vertical',
        }),
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="quiz-item"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    // Render as a hidden div with all data attributes
    // This ensures the data is preserved when saving/loading
    return [
      'div',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'quiz-item',
        class: 'quiz-item-data',
        style: 'display: none;',
      }),
    ]
  },
})
