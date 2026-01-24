/**
 * Custom Extensions Registry
 *
 * This file manages custom Tiptap extensions (Audio, Quiz, Answer Check, etc.)
 */

import { Audio } from '../audio-extension'
import { RichTextAudio } from '../audio-toolbar'
import { AudioBubbleMenuV2 } from '../audio-bubble-menu-v2'
import { QuizGroup } from '../quiz-group-extension'
import { QuizItem } from '../quiz-item-extension'
import { QuizToolbar } from '../quiz-toolbar'
import { QuizBubbleMenu } from '../quiz-bubble-menu'
import { AnswerCheck } from '../answer-check-extension'
import { RichTextAnswerCheck } from '../answer-check-toolbar'
import { AnswerCheckBubbleMenu } from '../answer-check-bubble-menu'

export interface ExtensionRegistryItem {
  name: string
  extension: any
  toolbar?: React.ComponentType<any>
  bubbleMenu?: React.ComponentType<any>
}

export const CUSTOM_EXTENSIONS_REGISTRY: ExtensionRegistryItem[] = [
  {
    name: 'audio',
    extension: Audio,
    toolbar: RichTextAudio,
    bubbleMenu: AudioBubbleMenuV2,
  },
  {
    name: 'quizGroup',
    extension: QuizGroup,
    toolbar: QuizToolbar,
    bubbleMenu: QuizBubbleMenu,
  },
  {
    name: 'quizItem',
    extension: QuizItem,
  },
  {
    name: 'answerCheck',
    extension: AnswerCheck,
    toolbar: RichTextAnswerCheck,
    bubbleMenu: AnswerCheckBubbleMenu,
  },
]

export interface ExtensionConfigs {
  // Quiz config will be added in Phase 5
  quiz?: any
}

/**
 * Get all extension instances with optional configurations
 */
export function getExtensions(configs?: ExtensionConfigs) {
  return CUSTOM_EXTENSIONS_REGISTRY.map((item) => {
    // Apply configuration if available
    if (item.name === 'quiz' && configs?.quiz) {
      return item.extension.configure(configs.quiz)
    }
    return item.extension
  })
}

/**
 * Get all toolbar components
 */
export function getToolbarComponents() {
  return CUSTOM_EXTENSIONS_REGISTRY.filter((item) => item.toolbar).map(
    (item) => ({
      name: item.name,
      Component: item.toolbar!,
    })
  )
}

/**
 * Get all bubble menu components
 */
export function getBubbleMenuComponents() {
  return CUSTOM_EXTENSIONS_REGISTRY.filter((item) => item.bubbleMenu).map(
    (item) => ({
      name: item.name,
      Component: item.bubbleMenu!,
    })
  )
}
