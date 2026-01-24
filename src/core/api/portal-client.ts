/**
 * Portal API Client
 * For customer-facing features: products, cart, orders, user profile
 * Backend: vielang-portal (port 8080) - /api/v1/portal/
 */

import { env } from "@/env"

import {
  adminAuthInterceptor,
  errorInterceptor,
  portalAuthInterceptor,
  successInterceptor,
} from "./interceptors/auth.interceptor"
import { clearAuthData, createApiClient, getCookie } from "./client"

// Create portal API client
export const portalApiClient = createApiClient({
  baseURL: env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL + "/api/v1/portal",
  timeout: 15000,
})

// Add interceptors
portalApiClient.interceptors.request.use(portalAuthInterceptor, (error) =>
  Promise.reject(error)
)

portalApiClient.interceptors.response.use(successInterceptor, (error) => {
  // Handle 401 Unauthorized
  if (error.response?.status === 401) {
    console.error("[Portal API] 401 Unauthorized:", error.config?.url)

    // Clear auth and redirect to signin
    if (typeof window !== "undefined") {
      clearAuthData("portal")

      const currentPath = window.location.pathname
      if (!currentPath.includes("/signin") && !currentPath.includes("/signup")) {
        window.location.href = "/signin"
      }
    }
  }

  return errorInterceptor(error)
})

// Export for backward compatibility
export const portalApi = portalApiClient
