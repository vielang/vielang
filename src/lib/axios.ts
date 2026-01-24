import { env } from "@/env"
import axios, { AxiosError, AxiosResponse } from "axios"

/**
 * Legacy axios instance - deprecated
 * Use vielang-portal.ts or vielang-admin.ts instead
 * This file is kept for compatibility with old code
 */
export const api = axios.create({
  baseURL: env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL + "/api/v1/portal",
  timeout: 10000,
  headers: {
    "Content-Type": "application/json",
  },
})

// Helper function to get cookie value
function getCookie(name: string): string | null {
  if (typeof window === "undefined") return null
  const match = document.cookie.match(new RegExp("(^| )" + name + "=([^;]+)"))
  return match?.[2] ?? null
}

// Request interceptor to add auth token (client-side only)
api.interceptors.request.use(
  (config) => {
    if (typeof window !== "undefined") {
      // List of public API endpoints that don't require authentication
      const publicEndpoints = [
        "/product/search",
        "/product/detail",
        "/product/categoryTreeList",
        "/home/",
        "/brand/recommend",
        "/lms/category/list",
        "/lms/course/list",
        "/lms/course/recommended",
      ]

      // Check if the current request is to a public endpoint
      const isPublicEndpoint = publicEndpoints.some((endpoint) =>
        config.url?.includes(endpoint)
      )

      // Only add token for non-public endpoints
      if (!isPublicEndpoint) {
        // Try to get token from readable cookies first (new secure auth)
        let token = getCookie("auth_token_readable")
        let tokenHead = getCookie("token_head_readable") || "Bearer "

        // Fallback to localStorage (old auth - backward compatibility)
        if (!token) {
          token = localStorage.getItem("auth_token")
          tokenHead = localStorage.getItem("token_head") || "Bearer "
        }

        if (token) {
          // Combine tokenHead and token for Authorization header
          config.headers.Authorization = `${tokenHead}${token}`
          console.log("[axios] Adding Authorization header for:", config.url)
        } else {
          console.warn(
            "[axios] No token found for protected endpoint:",
            config.url
          )
        }
      }
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor for error handling
api.interceptors.response.use(
  (response: AxiosResponse) => {
    return response
  },
  (error: AxiosError) => {
    // Handle 401 Unauthorized - token expired or invalid
    if (error.response?.status === 401) {
      console.error("[axios] 401 Unauthorized for:", error.config?.url)

      // Don't automatically logout/redirect for secure auth (cookies)
      // Let the component/hook handle it
      const hasCookieAuth =
        typeof window !== "undefined" && getCookie("auth_token_readable")

      if (!hasCookieAuth && typeof window !== "undefined") {
        // Only clear localStorage auth if using old auth system
        const hasLocalStorageAuth = localStorage.getItem("auth_token")

        if (hasLocalStorageAuth) {
          console.log("[axios] Clearing old localStorage auth")
          localStorage.removeItem("auth_token")
          localStorage.removeItem("token_head")
          localStorage.removeItem("user")

          // Redirect to signin only if not already on auth pages
          const currentPath = window.location.pathname
          if (
            !currentPath.includes("/signin") &&
            !currentPath.includes("/signup")
          ) {
            window.location.href = "/signin"
          }
        }
      }
    }
    return Promise.reject(error)
  }
)

// Vielang backend CommonResult response type
export interface CommonResult<T = any> {
  code: number
  message: string | null
  data: T
}

// Vielang backend CommonPage response type
export interface CommonPage<T = any> {
  pageNum: number
  pageSize: number
  totalPage: number
  total: number
  list: T[]
}

// Legacy API Response types (for compatibility)
export interface ApiResponse<T = any> {
  data: T
  message?: string
  success: boolean
}

export interface PaginatedResponse<T = any> {
  data: T[]
  total: number
  page: number
  perPage: number
  totalPages: number
}

// Error handling utility
export function handleApiError(error: any): string {
  if (error.response?.data?.message) {
    return error.response.data.message
  }
  if (error.message) {
    return error.message
  }
  return "An unexpected error occurred"
}

// Helper to check if response is successful (code 200 = success in vielang backend)
export function isSuccess<T>(response: CommonResult<T>): boolean {
  return response.code === 200
}

export default api
