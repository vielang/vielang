/**
 * Common API response types
 */

// Common result wrapper from vielang backend
export interface CommonResult<T = unknown> {
  code: number
  message: string
  data: T
}

// Paginated response
export interface CommonPage<T = unknown> {
  pageNum: number
  pageSize: number
  totalPage: number
  total: number
  list: T[]
}

// API error response
export interface ApiError {
  code: number
  message: string
  data?: unknown
}
