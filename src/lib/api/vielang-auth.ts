import { api, CommonResult, handleApiError, isSuccess } from "@/lib/axios"
import { deleteCookie, setCookie } from "@/lib/utils/cookies"

import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  SysMember,
} from "./vielang-types"

// Vielang Backend Auth API
export const vielangAuthApi = {
  /**
   * Sign in with username and password
   * POST /sso/login
   */
  signIn: async (credentials: LoginRequest): Promise<LoginResponse> => {
    try {
      const params = new URLSearchParams()
      params.append("username", credentials.username)
      params.append("password", credentials.password)

      const response = await api.post<CommonResult<LoginResponse>>(
        "/sso/login",
        params,
        {
          headers: {
            "Content-Type": "application/x-www-form-urlencoded",
          },
        }
      )

      if (!isSuccess(response.data)) {
        throw new Error(response.data.message || "Login failed")
      }

      const { token, tokenHead } = response.data.data

      // Store token in localStorage
      if (typeof window !== "undefined") {
        localStorage.setItem("auth_token", token)
        localStorage.setItem("token_head", tokenHead)
      }

      return response.data.data
    } catch (error) {
      throw new Error(handleApiError(error))
    }
  },

  /**
   * Register new user
   * POST /sso/register
   */
  signUp: async (credentials: RegisterRequest): Promise<void> => {
    try {
      const params = new URLSearchParams()
      params.append("username", credentials.username)
      params.append("password", credentials.password)
      params.append("email", credentials.email)
      if (credentials.phone) {
        params.append("phone", credentials.phone)
      }

      const response = await api.post<CommonResult<null>>(
        "/sso/register",
        params,
        {
          headers: {
            "Content-Type": "application/x-www-form-urlencoded",
          },
        }
      )

      if (!isSuccess(response.data)) {
        throw new Error(response.data.message || "Registration failed")
      }
    } catch (error) {
      throw new Error(handleApiError(error))
    }
  },

  /**
   * Get auth code for phone number
   * GET /sso/getAuthCode?telephone=xxx
   */
  getAuthCode: async (telephone: string): Promise<string> => {
    try {
      const response = await api.get<CommonResult<string>>(
        `/sso/getAuthCode?telephone=${telephone}`
      )

      if (!isSuccess(response.data)) {
        throw new Error(response.data.message || "Failed to get auth code")
      }

      return response.data.data
    } catch (error) {
      throw new Error(handleApiError(error))
    }
  },

  /**
   * Get current user info
   * GET /sso/info
   * Requires Authorization header with Bearer token
   */
  getCurrentUser: async (): Promise<SysMember> => {
    try {
      const response = await api.get<CommonResult<SysMember>>("/sso/info")

      if (!isSuccess(response.data)) {
        throw new Error(response.data.message || "Failed to get user info")
      }

      // Cache user in localStorage
      if (typeof window !== "undefined") {
        localStorage.setItem("user", JSON.stringify(response.data.data))
      }

      return response.data.data
    } catch (error) {
      throw new Error(handleApiError(error))
    }
  },

  /**
   * Update password
   * POST /sso/updatePassword
   */
  updatePassword: async (
    oldPassword: string,
    newPassword: string
  ): Promise<void> => {
    try {
      const params = new URLSearchParams()
      params.append("oldPassword", oldPassword)
      params.append("newPassword", newPassword)

      const response = await api.post<CommonResult<null>>(
        "/sso/updatePassword",
        params,
        {
          headers: {
            "Content-Type": "application/x-www-form-urlencoded",
          },
        }
      )

      if (!isSuccess(response.data)) {
        throw new Error(response.data.message || "Password update failed")
      }
    } catch (error) {
      throw new Error(handleApiError(error))
    }
  },

  /**
   * Refresh token
   * GET /sso/refreshToken
   */
  refreshToken: async (): Promise<LoginResponse> => {
    try {
      const response =
        await api.get<CommonResult<LoginResponse>>("/sso/refreshToken")

      if (!isSuccess(response.data)) {
        throw new Error(response.data.message || "Token refresh failed")
      }

      const { token, tokenHead } = response.data.data

      // Update token in localStorage
      if (typeof window !== "undefined") {
        localStorage.setItem("auth_token", token)
        localStorage.setItem("token_head", tokenHead)
      }

      return response.data.data
    } catch (error) {
      throw new Error(handleApiError(error))
    }
  },

  /**
   * Sign out (client-side token cleanup)
   */
  signOut: async (): Promise<void> => {
    try {
      // Clear local storage
      if (typeof window !== "undefined") {
        localStorage.removeItem("auth_token")
        localStorage.removeItem("token_head")
        localStorage.removeItem("user")
        deleteCookie("auth_token")
        deleteCookie("token_head")
        deleteCookie("user")
      }
    } catch (error) {
      console.warn("Sign out cleanup failed:", error)
    }
  },
}

// Token management helpers
export const vielangTokenManager = {
  getToken: (): string | null => {
    if (typeof window !== "undefined") {
      return localStorage.getItem("auth_token")
    }
    return null
  },

  getTokenHead: (): string | null => {
    if (typeof window !== "undefined") {
      return localStorage.getItem("token_head") || "Bearer"
    }
    return "Bearer"
  },

  setToken: (token: string, tokenHead: string = "Bearer"): void => {
    if (typeof window !== "undefined") {
      localStorage.setItem("auth_token", token)
      localStorage.setItem("token_head", tokenHead)
      setCookie("auth_token", token, 7)
      setCookie("token_head", tokenHead, 7)
    }
  },

  removeToken: (): void => {
    if (typeof window !== "undefined") {
      localStorage.removeItem("auth_token")
      localStorage.removeItem("token_head")
      deleteCookie("auth_token")
      deleteCookie("token_head")
    }
  },

  getUser: (): SysMember | null => {
    if (typeof window !== "undefined") {
      const userStr = localStorage.getItem("user")
      try {
        return userStr ? (JSON.parse(userStr) as SysMember) : null
      } catch {
        return null
      }
    }
    return null
  },

  setUser: (user: SysMember): void => {
    if (typeof window !== "undefined") {
      localStorage.setItem("user", JSON.stringify(user))
      setCookie("user", JSON.stringify(user), 7)
    }
  },

  removeUser: (): void => {
    if (typeof window !== "undefined") {
      localStorage.removeItem("user")
      deleteCookie("user")
    }
  },
}
