/**
 * Base API Client Factory
 * Creates axios instances with common configuration
 */

import axios, { type AxiosInstance, type AxiosRequestConfig } from "axios"

export interface ApiClientConfig {
  baseURL: string
  timeout?: number
  withCredentials?: boolean
}

/**
 * Create a configured axios instance
 */
export function createApiClient(config: ApiClientConfig): AxiosInstance {
  const client = axios.create({
    baseURL: config.baseURL,
    timeout: config.timeout || 15000,
    withCredentials: config.withCredentials || false,
    headers: {
      "Content-Type": "application/json",
    },
  })

  return client
}

/**
 * Helper to get cookie value (client-side only)
 */
export function getCookie(name: string): string | null {
  if (typeof window === "undefined") return null
  const match = document.cookie.match(new RegExp(`(^| )${name}=([^;]+)`))
  return match ? match[2] || null : null
}

/**
 * Helper to clear auth data
 */
export function clearAuthData(type: "portal" | "admin" = "portal") {
  if (typeof window === "undefined") return

  if (type === "portal") {
    // Clear portal auth
    localStorage.removeItem("auth_token")
    localStorage.removeItem("token_head")
    localStorage.removeItem("user")
    document.cookie =
      "auth_token_readable=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;"
    document.cookie =
      "token_head_readable=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;"
    document.cookie =
      "user_info=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;"
  } else {
    // Clear admin auth
    localStorage.removeItem("admin_auth_token")
    localStorage.removeItem("admin_user")
  }
}
