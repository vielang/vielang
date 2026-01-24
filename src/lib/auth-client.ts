/**
 * Client-side Authentication Utilities
 * Sử dụng BFF endpoints thay vì direct backend calls
 */

// Get CSRF Token from cookie
export function getCsrfTokenFromCookie(): string | null {
  if (typeof window === "undefined") return null

  const match = document.cookie.match(/csrf_token=([^;]+)/)
  return match ? match[1] || null : null
}

// Fetch CSRF Token from server
let csrfTokenCache: string | null = null

export async function getCsrfToken(): Promise<string> {
  // Return cached token if exists
  if (csrfTokenCache) {
    return csrfTokenCache
  }

  // Check cookie first
  const cookieToken = getCsrfTokenFromCookie()
  if (cookieToken) {
    csrfTokenCache = cookieToken
    return cookieToken
  }

  // Fetch new token from server
  const response = await fetch("/api/auth/csrf")
  const data: any = await response.json()

  csrfTokenCache = data.csrfToken
  return data.csrfToken
}

// Clear CSRF token cache
export function clearCsrfTokenCache() {
  csrfTokenCache = null
}

// Get user info from cookie
export interface UserInfo {
  id: number
  username: string
  nickname?: string
  icon?: string
  phone?: string
  email?: string
}

// Get user info from localStorage
export function getUserInfoFromLocalStorage(): UserInfo | null {
  if (typeof window === "undefined") return null

  const userStr = localStorage.getItem("user")
  if (!userStr) return null

  try {
    return JSON.parse(userStr) as UserInfo
  } catch {
    return null
  }
}

// Legacy alias for backward compatibility
export const getUserInfoFromCookie = getUserInfoFromLocalStorage

// Check if user is authenticated (has token in localStorage)
export function isAuthenticated(): boolean {
  if (typeof window === "undefined") return false

  // Check if auth_token exists in localStorage
  return !!localStorage.getItem("auth_token")
}

// Login via BFF
export async function loginViaBFF(
  username: string,
  password: string
): Promise<{ success: boolean; user?: any; error?: string }> {
  try {
    const response = await fetch("/api/auth/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({ username, password }),
    })

    const data: any = await response.json()

    if (!response.ok) {
      return {
        success: false,
        error: data.error || "Login failed",
      }
    }

    // Store token and user in localStorage
    if (data.token && typeof window !== "undefined") {
      localStorage.setItem("auth_token", data.token)
      localStorage.setItem("token_head", data.tokenHead || "Bearer ")
      if (data.user) {
        localStorage.setItem("user", JSON.stringify(data.user))
      }
    }

    return {
      success: true,
      user: data.user,
    }
  } catch (error) {
    return {
      success: false,
      error: "Network error",
    }
  }
}

// Logout via BFF
export async function logoutViaBFF(): Promise<{ success: boolean }> {
  try {
    await fetch("/api/auth/logout", {
      method: "POST",
      credentials: "include",
    })

    clearCsrfTokenCache()

    return { success: true }
  } catch (error) {
    return { success: false }
  }
}

// Refresh token via BFF
export async function refreshTokenViaBFF(): Promise<{ success: boolean }> {
  try {
    const response = await fetch("/api/auth/refresh", {
      method: "POST",
      credentials: "include",
    })

    if (!response.ok) {
      return { success: false }
    }

    return { success: true }
  } catch (error) {
    return { success: false }
  }
}

// Get current user via BFF
export async function getCurrentUserViaBFF(): Promise<{
  success: boolean
  user?: any
  error?: string
}> {
  try {
    const response = await fetch("/api/auth/user", {
      credentials: "include",
    })

    const data: any = await response.json()

    if (!response.ok) {
      return {
        success: false,
        error: data.error || "Failed to fetch user",
      }
    }

    return {
      success: true,
      user: data.user,
    }
  } catch (error) {
    return {
      success: false,
      error: "Network error",
    }
  }
}

// Get auth code (directly from backend - public API)
export async function getAuthCodeViaBFF(
  telephone: string
): Promise<{ success: boolean; code?: string; error?: string }> {
  try {
    // This is a public endpoint, call backend directly
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL}/api/v1/portal/sso/getAuthCode?telephone=${telephone}`
    )

    const data: any = await response.json()

    if (!response.ok || data.code !== 200) {
      return {
        success: false,
        error: data.message || "Failed to get auth code",
      }
    }

    return {
      success: true,
      code: data.data,
    }
  } catch (error) {
    return {
      success: false,
      error: "Network error",
    }
  }
}

// Register via BFF (phone-based)
export async function registerViaBFF(
  username: string,
  password: string,
  telephone: string,
  authCode: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const response = await fetch("/api/auth/register", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({ username, password, telephone, authCode }),
    })

    const data: any = await response.json()

    if (!response.ok) {
      return {
        success: false,
        error: data.error || "Registration failed",
      }
    }

    return {
      success: true,
    }
  } catch (error) {
    return {
      success: false,
      error: "Network error",
    }
  }
}

// Register by email via BFF
export async function registerByEmailViaBFF(
  username: string,
  password: string,
  email: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const response = await fetch("/api/auth/registerByEmail", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({ username, password, email }),
    })

    const data: any = await response.json()

    if (!response.ok) {
      return {
        success: false,
        error: data.error || "Registration failed",
      }
    }

    return {
      success: true,
    }
  } catch (error) {
    return {
      success: false,
      error: "Network error",
    }
  }
}
