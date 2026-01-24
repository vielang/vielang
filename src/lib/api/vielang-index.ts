/**
 * Vielang Backend API - Main Export File
 *
 * This file provides a unified interface to interact with the Vielang backend APIs.
 * Supports both Portal (member-facing) and Admin (dashboard) APIs.
 *
 * Backend: Spring Boot + MyBatis (vielang-portal on port 8085, vielang-admin on port 8080)
 * Database: MySQL (social_platform schema with 17 tables)
 *
 * Usage:
 * ```typescript
 * // Portal API (Member-facing)
 * import { portalApi } from '@/lib/api/vielang-index'
 * const { token } = await portalApi.auth.login({ username: 'user', password: 'pass' })
 * const posts = await portalApi.post.getList({ pageNum: 1, pageSize: 20 })
 *
 * // Admin API (Dashboard)
 * import { adminApi } from '@/lib/api/vielang-index'
 * const { token } = await adminApi.auth.login({ username: 'admin', password: 'pass' })
 * const posts = await adminApi.post.getList({ pageNum: 1, pageSize: 10 })
 * ```
 */

import adminApiDefault from "./vielang-admin"
import portalApiDefault from "./vielang-portal"

// Export axios instance and utilities
export { api, handleApiError, isSuccess } from "@/lib/axios"
export type { CommonResult, CommonPage } from "@/lib/axios"

// ============================================
// PORTAL API EXPORTS (Member-facing)
// Base: /api/v1/portal
// ============================================

export {
  portalApi,
  portalAuthApi,
  portalPostApi,
  portalSocialApi,
  portalCategoryApi,
} from "./vielang-portal"

// ============================================
// ADMIN API EXPORTS (Dashboard)
// Base: /api/v1/admin
// ============================================

export {
  adminApi,
  adminAuthApi,
  adminUserApi,
  adminRoleApi,
  adminCategoryApi,
  adminPostApi,
  adminCommentApi,
} from "./vielang-admin"

// ============================================
// TYPE EXPORTS
// ============================================

// All types from vielang-types.ts
export type * from "./vielang-types"

// ============================================
// DEFAULT EXPORT
// ============================================

export default {
  portal: portalApiDefault,
  admin: adminApiDefault,
}
