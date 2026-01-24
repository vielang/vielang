/**
 * Admin API Client
 * For admin dashboard features: product/order/member management
 * Backend: vielang-admin (port 8085) - /api/v1/admin/
 */

import { env } from "@/env"

import {
  adminAuthInterceptor,
  errorInterceptor,
  successInterceptor,
} from "./interceptors/auth.interceptor"
import { clearAuthData, createApiClient } from "./client"

// Create admin API client
export const adminApiClient = createApiClient({
  baseURL: env.NEXT_PUBLIC_VIELANG_ADMIN_API_URL + "/api/v1/admin",
  timeout: 15000,
})

// Add interceptors
adminApiClient.interceptors.request.use(adminAuthInterceptor, (error) =>
  Promise.reject(error)
)

adminApiClient.interceptors.response.use(successInterceptor, (error) => {
  // Handle 401 Unauthorized
  if (error.response?.status === 401) {
    console.error("[Admin API] 401 Unauthorized:", error.config?.url)

    // Clear auth and redirect
    if (typeof window !== "undefined") {
      clearAuthData("admin")

      // Redirect to admin login if on admin route, else signin
      const isAdminRoute = window.location.pathname.startsWith("/admin")
      window.location.href = isAdminRoute ? "/admin/login" : "/signin"
    }
  }

  return errorInterceptor(error)
})

// Export for backward compatibility
export const adminApi = adminApiClient
