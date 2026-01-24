/**
 * Secure Axios Instance with Auto-Refresh and CSRF Protection
 * Sử dụng BFF endpoints thay vì direct backend calls
 */

import axios, { type AxiosInstance } from "axios"

import { getCsrfToken, refreshTokenViaBFF } from "@/lib/auth-client"

// Create secure axios instance for client-side API calls
export const secureApi: AxiosInstance = axios.create({
  baseURL: "/api",
  withCredentials: true, // Send cookies automatically
})

// Request interceptor - Add CSRF token to state-changing requests
secureApi.interceptors.request.use(
  async (config) => {
    // Add CSRF token for state-changing requests
    const method = config.method?.toUpperCase()
    if (["POST", "PUT", "DELETE", "PATCH"].includes(method || "")) {
      try {
        const csrfToken = await getCsrfToken()
        config.headers["X-CSRF-Token"] = csrfToken
      } catch (error) {
        console.error("Failed to get CSRF token:", error)
      }
    }

    return config
  },
  (error) => Promise.reject(error)
)

// Response interceptor - Handle token refresh on 401
let isRefreshing = false
let failedQueue: Array<{
  resolve: (value?: unknown) => void
  reject: (reason?: unknown) => void
}> = []

const processQueue = (error: unknown = null) => {
  failedQueue.forEach((prom) => {
    if (error) {
      prom.reject(error)
    } else {
      prom.resolve()
    }
  })

  failedQueue = []
}

secureApi.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config

    // If error is not 401 or request already retried, reject
    if (error.response?.status !== 401 || originalRequest._retry) {
      return Promise.reject(error)
    }

    // If currently refreshing, queue this request
    if (isRefreshing) {
      return new Promise((resolve, reject) => {
        failedQueue.push({ resolve, reject })
      })
        .then(() => secureApi(originalRequest))
        .catch((err) => Promise.reject(err))
    }

    originalRequest._retry = true
    isRefreshing = true

    try {
      // Attempt to refresh token via BFF
      const refreshResult = await refreshTokenViaBFF()

      if (refreshResult.success) {
        // Token refreshed successfully
        processQueue()
        return secureApi(originalRequest)
      } else {
        // Refresh failed - logout user
        processQueue(new Error("Token refresh failed"))

        // Redirect to login page
        if (typeof window !== "undefined") {
          window.location.href = "/signin?session_expired=true"
        }

        return Promise.reject(error)
      }
    } catch (refreshError) {
      processQueue(refreshError)

      // Redirect to login page
      if (typeof window !== "undefined") {
        window.location.href = "/signin?session_expired=true"
      }

      return Promise.reject(refreshError)
    } finally {
      isRefreshing = false
    }
  }
)

// For backward compatibility - proxy requests to backend through BFF
export const createProxyApi = (baseURL: string): AxiosInstance => {
  const proxyApi = axios.create({
    baseURL,
    withCredentials: true,
  })

  // Similar interceptors for proxy API
  proxyApi.interceptors.request.use(
    async (config) => {
      const method = config.method?.toUpperCase()
      if (["POST", "PUT", "DELETE", "PATCH"].includes(method || "")) {
        try {
          const csrfToken = await getCsrfToken()
          config.headers["X-CSRF-Token"] = csrfToken
        } catch (error) {
          console.error("Failed to get CSRF token:", error)
        }
      }

      return config
    },
    (error) => Promise.reject(error)
  )

  proxyApi.interceptors.response.use(
    (response) => response,
    async (error) => {
      const originalRequest = error.config

      if (error.response?.status !== 401 || originalRequest._retry) {
        return Promise.reject(error)
      }

      originalRequest._retry = true

      try {
        const refreshResult = await refreshTokenViaBFF()

        if (refreshResult.success) {
          return proxyApi(originalRequest)
        } else {
          if (typeof window !== "undefined") {
            window.location.href = "/signin?session_expired=true"
          }
          return Promise.reject(error)
        }
      } catch (refreshError) {
        if (typeof window !== "undefined") {
          window.location.href = "/signin?session_expired=true"
        }
        return Promise.reject(refreshError)
      }
    }
  )

  return proxyApi
}

export default secureApi
