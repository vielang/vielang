/**
 * Authentication Interceptors
 * Handle auth token injection for requests
 */

import type {
  AxiosError,
  AxiosResponse,
  InternalAxiosRequestConfig,
} from "axios"

import { getCookie } from "../client"

/**
 * Portal auth request interceptor
 * Adds auth token from localStorage or cookies
 */
export function portalAuthInterceptor(
  config: InternalAxiosRequestConfig
): InternalAxiosRequestConfig {
  if (typeof window === "undefined") return config

  // Try localStorage first (old auth)
  let token = localStorage.getItem("auth_token")
  let tokenHead = localStorage.getItem("token_head") || "Bearer "

  // Fallback to cookies (new secure auth)
  if (!token) {
    token = getCookie("auth_token_readable")
    tokenHead = getCookie("token_head_readable") || "Bearer "
  }

  if (token) {
    config.headers.Authorization = `${tokenHead}${token}`
  }

  return config
}

/**
 * Admin auth request interceptor
 * Adds admin auth token from localStorage or cookies
 */
export function adminAuthInterceptor(
  config: InternalAxiosRequestConfig
): InternalAxiosRequestConfig {
  if (typeof window === "undefined") return config

  // Try localStorage first (admin auth)
  let token = localStorage.getItem("admin_auth_token")

  // Fallback to cookies (user auth accessing admin)
  if (!token) {
    const cookieToken = getCookie("auth_token_readable")
    const tokenHead = getCookie("token_head_readable")
    if (cookieToken) {
      token = tokenHead ? `${tokenHead}${cookieToken}` : cookieToken
    }
  }

  if (token) {
    // Token from localStorage already includes "Bearer " prefix
    config.headers.Authorization = token
  }

  return config
}

/**
 * Success response interceptor
 */
export function successInterceptor(response: AxiosResponse): AxiosResponse {
  return response
}

/**
 * Error response interceptor
 */
export function errorInterceptor(error: AxiosError): Promise<AxiosError> {
  // Log errors for debugging
  if (error.response) {
    console.error("[API Error]", {
      status: error.response.status,
      url: error.config?.url,
      data: error.response.data,
    })
  } else if (error.request) {
    console.error("[Network Error]", error.message)
  } else {
    console.error("[Request Error]", error.message)
  }

  return Promise.reject(error)
}
