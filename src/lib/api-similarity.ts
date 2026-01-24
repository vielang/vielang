import axios from 'axios'

const API_BASE_URL = process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL || 'http://localhost:8085'

/**
 * Interface for question with user answer
 */
export interface QuestionAnswer {
  prompt: string
  userAnswer: string
  correctAnswer: string
}

/**
 * Interface for answer check request
 */
export interface AnswerCheckRequest {
  questions: QuestionAnswer[]
  passingScore?: number
}

/**
 * Interface for detailed similarity scores
 */
export interface SimilarityScores {
  jaroWinkler: number
  damerauLevenshtein: number
  levenshtein: number
}

/**
 * Interface for question result
 */
export interface QuestionResult {
  index: number
  prompt: string
  userAnswer: string
  correctAnswer: string
  score: number
  passed: boolean
  detailedScores: SimilarityScores
}

/**
 * Interface for evaluation
 */
export interface Evaluation {
  grade: string
  message: string
  passed: boolean
}

/**
 * Interface for answer check response
 */
export interface AnswerCheckResponse {
  overallScore: number
  passed: boolean
  correctCount: number
  totalQuestions: number
  evaluation: Evaluation
  results: QuestionResult[]
}

/**
 * Interface for detailed similarity result (backwards compatibility)
 */
export interface DetailedSimilarityResult {
  overallScore: number
  jaroWinklerScore: number
  damerauLevenshteinScore: number
  levenshteinScore: number
}

/**
 * Check answers using backend API
 *
 * @param questions - List of questions with user answers
 * @param passingScore - Passing score threshold (default: 70)
 * @returns Answer check response with scores and evaluation
 */
export async function checkAnswersAPI(
  questions: QuestionAnswer[],
  passingScore: number = 70
): Promise<AnswerCheckResponse> {
  try {
    const response = await axios.post<{
      code: number
      message: string
      data: AnswerCheckResponse
    }>(`${API_BASE_URL}/similarity/check-answers`, {
      questions,
      passingScore,
    })

    if (response.data.code === 200 && response.data.data) {
      return response.data.data
    } else {
      throw new Error(response.data.message || 'Failed to check answers')
    }
  } catch (error) {
    console.error('Error checking answers:', error)
    throw error
  }
}

/**
 * Calculate similarity between two strings using backend API
 *
 * @param str1 - First string
 * @param str2 - Second string
 * @returns Similarity score (0-100)
 */
export async function calculateSimilarity(
  str1: string,
  str2: string
): Promise<number> {
  try {
    const response = await axios.post<{
      code: number
      message: string
      data: number
    }>(`${API_BASE_URL}/similarity/calculate`, null, {
      params: { str1, str2 },
    })

    if (response.data.code === 200 && response.data.data !== undefined) {
      return response.data.data
    } else {
      throw new Error(response.data.message || 'Failed to calculate similarity')
    }
  } catch (error) {
    console.error('Error calculating similarity:', error)
    throw error
  }
}

/**
 * Calculate detailed similarity scores using backend API
 *
 * @param str1 - First string
 * @param str2 - Second string
 * @returns Detailed scores from different algorithms
 */
export async function calculateSimilarityDetailed(
  str1: string,
  str2: string
): Promise<DetailedSimilarityResult> {
  try {
    const response = await axios.post<{
      code: number
      message: string
      data: SimilarityScores
    }>(`${API_BASE_URL}/similarity/calculate-detailed`, null, {
      params: { str1, str2 },
    })

    if (response.data.code === 200 && response.data.data) {
      return {
        overallScore: calculateCombinedScore(response.data.data),
        jaroWinklerScore: response.data.data.jaroWinkler,
        damerauLevenshteinScore: response.data.data.damerauLevenshtein,
        levenshteinScore: response.data.data.levenshtein,
      }
    } else {
      throw new Error(response.data.message || 'Failed to calculate detailed similarity')
    }
  } catch (error) {
    console.error('Error calculating detailed similarity:', error)
    throw error
  }
}

/**
 * Calculate combined score from detailed scores
 */
function calculateCombinedScore(scores: SimilarityScores): number {
  return (
    scores.jaroWinkler * 0.4 +
    scores.damerauLevenshtein * 0.35 +
    scores.levenshtein * 0.25
  )
}

/**
 * Evaluate answer based on score
 *
 * @param score - Similarity score (0-100)
 * @returns Evaluation result
 */
export function evaluateAnswer(score: number): {
  passed: boolean
  grade: 'excellent' | 'very-good' | 'good' | 'fair' | 'needs-improvement'
  message: string
} {
  if (score >= 95) {
    return {
      passed: true,
      grade: 'excellent',
      message: 'Xuất sắc! Câu trả lời hoàn toàn chính xác.',
    }
  } else if (score >= 85) {
    return {
      passed: true,
      grade: 'very-good',
      message: 'Rất tốt! Câu trả lời gần như chính xác.',
    }
  } else if (score >= 70) {
    return {
      passed: true,
      grade: 'good',
      message: 'Tốt! Câu trả lời đúng phần lớn.',
    }
  } else if (score >= 50) {
    return {
      passed: false,
      grade: 'fair',
      message: 'Khá! Cần cải thiện thêm một chút.',
    }
  } else {
    return {
      passed: false,
      grade: 'needs-improvement',
      message: 'Cần cố gắng hơn! Hãy xem lại đáp án đúng.',
    }
  }
}
