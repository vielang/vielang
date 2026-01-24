"use client"

import { createContext, useContext, useEffect, useState } from "react"
import { useRouter } from "next/navigation"

import { portalAuthApi } from "@/lib/api/vielang-index"
import type { SysMember } from "@/lib/api/vielang-types"

// Token Manager for localStorage
const TOKEN_KEY = "member_auth_token"
const TOKEN_HEAD_KEY = "member_token_head"
const USER_KEY = "member_user"

const tokenManager = {
  getToken: () => {
    if (typeof window === "undefined") return null
    return localStorage.getItem(TOKEN_KEY)
  },
  setToken: (token: string, tokenHead: string) => {
    if (typeof window === "undefined") return
    localStorage.setItem(TOKEN_KEY, token)
    localStorage.setItem(TOKEN_HEAD_KEY, tokenHead)
  },
  removeToken: () => {
    if (typeof window === "undefined") return
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(TOKEN_HEAD_KEY)
  },
  getUser: (): SysMember | null => {
    if (typeof window === "undefined") return null
    const userStr = localStorage.getItem(USER_KEY)
    return userStr ? (JSON.parse(userStr) as SysMember) : null
  },
  setUser: (user: SysMember) => {
    if (typeof window === "undefined") return
    localStorage.setItem(USER_KEY, JSON.stringify(user))
  },
  removeUser: () => {
    if (typeof window === "undefined") return
    localStorage.removeItem(USER_KEY)
  },
}

interface AuthState {
  user: SysMember | null
  token: string | null
  isLoading: boolean
}

interface AuthContextType extends AuthState {
  signIn: (username: string, password: string) => Promise<SysMember>
  signUp: (username: string, password: string, email: string) => Promise<void>
  signOut: () => Promise<void>
  refresh: () => Promise<void>
}

const AuthContext = createContext<AuthContextType | undefined>(
  undefined
)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<SysMember | null>(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()

  useEffect(() => {
    // Restore auth state from localStorage on page load
    const initAuth = () => {
      try {
        const savedToken = tokenManager.getToken()
        const savedUser = tokenManager.getUser()

        console.log(
          "[Auth] Initializing auth, token found:",
          !!savedToken,
          "user found:",
          !!savedUser
        )

        if (savedToken && savedUser) {
          setToken(savedToken)
          setUser(savedUser)
          console.log("[Auth] Auth state restored successfully")
        } else {
          console.log("[Auth] No saved auth state found")
        }
      } catch (error) {
        console.error("[Auth] Auth initialization error:", error)
      } finally {
        setIsLoading(false)
      }
    }

    initAuth()
  }, [])

  const signIn = async (
    username: string,
    password: string
  ): Promise<SysMember> => {
    try {
      const { token: newToken, tokenHead } = await portalAuthApi.login({
        username,
        password,
      })

      setToken(newToken)
      tokenManager.setToken(newToken, tokenHead)

      // Fetch user info after successful login
      const currentUser = await portalAuthApi.getCurrentMember()
      setUser(currentUser)
      tokenManager.setUser(currentUser)

      return currentUser
    } catch (error) {
      throw error
    }
  }

  const signUp = async (
    username: string,
    password: string,
    email: string
  ): Promise<void> => {
    try {
      await portalAuthApi.register({
        username,
        password,
        email,
      })
    } catch (error) {
      throw error
    }
  }

  const signOut = async (): Promise<void> => {
    try {
      await portalAuthApi.logout()
    } catch (error) {
      console.error("Sign out error:", error)
    } finally {
      // Clear local state and storage regardless of API call result
      setUser(null)
      setToken(null)
      tokenManager.removeToken()
      tokenManager.removeUser()

      router.push("/signin")
    }
  }

  const refresh = async (): Promise<void> => {
    try {
      if (token) {
        const { token: newToken, tokenHead } =
          await portalAuthApi.refreshToken()
        setToken(newToken)
        tokenManager.setToken(newToken, tokenHead)

        // Fetch updated user info
        const currentUser = await portalAuthApi.getCurrentMember()
        setUser(currentUser)
        tokenManager.setUser(currentUser)
      }
    } catch (error) {
      console.error("Auth refresh failed:", error)
      await signOut()
    }
  }

  return (
    <AuthContext.Provider
      value={{
        user,
        token,
        isLoading,
        signIn,
        signUp,
        signOut,
        refresh,
      }}
    >
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error("useAuth must be used within a AuthProvider")
  }
  return context
}
