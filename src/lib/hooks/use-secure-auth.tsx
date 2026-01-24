"use client"

import * as React from "react"
import { useRouter } from "next/navigation"

import {
  getAuthCodeViaBFF,
  getCurrentUserViaBFF,
  getUserInfoFromCookie,
  isAuthenticated,
  loginViaBFF,
  logoutViaBFF,
  registerByEmailViaBFF,
  registerViaBFF,
  type UserInfo,
} from "@/lib/auth-client"

interface SecureAuthContextType {
  user: UserInfo | null
  isLoading: boolean
  isAuthenticated: boolean
  signIn: (username: string, password: string) => Promise<void>
  signOut: () => Promise<void>
  signUp: (
    username: string,
    password: string,
    telephone: string,
    authCode: string
  ) => Promise<void>
  signUpByEmail: (
    username: string,
    password: string,
    email: string
  ) => Promise<void>
  getAuthCode: (telephone: string) => Promise<string>
  refreshUser: () => Promise<void>
}

const SecureAuthContext = React.createContext<SecureAuthContextType | null>(
  null
)

export function SecureAuthProvider({
  children,
}: {
  children: React.ReactNode
}) {
  const [user, setUser] = React.useState<UserInfo | null>(null)
  const [isLoading, setIsLoading] = React.useState(true)
  const router = useRouter()

  // Initialize user from localStorage on mount
  React.useEffect(() => {
    const initUser = async () => {
      try {
        // Check if authenticated via localStorage
        if (isAuthenticated()) {
          // Try to get user info from localStorage first
          const localUser = getUserInfoFromCookie() // reads from localStorage
          if (localUser) {
            setUser(localUser)
          } else {
            // Fetch from server if localStorage doesn't have user info
            const result = await getCurrentUserViaBFF()
            if (result.success && result.user) {
              setUser({
                id: result.user.id,
                username: result.user.username,
                nickname: result.user.nickname,
                icon: result.user.icon,
              })
              // Save to localStorage
              localStorage.setItem("user", JSON.stringify(result.user))
            }
          }
        }
      } catch (error) {
        console.error("Failed to initialize user:", error)
      } finally {
        setIsLoading(false)
      }
    }

    initUser()
  }, [])

  const signIn = async (username: string, password: string) => {
    setIsLoading(true)
    try {
      const result = await loginViaBFF(username, password)

      if (!result.success) {
        throw new Error(result.error || "Login failed")
      }

      // Set user from response
      if (result.user) {
        setUser({
          id: result.user.id,
          username: result.user.username,
          nickname: result.user.nickname,
          icon: result.user.icon,
        })
      } else {
        // Fallback to cookie
        const cookieUser = getUserInfoFromCookie()
        setUser(cookieUser)
      }

      // Redirect to home
      router.push("/")
      router.refresh()
    } catch (error) {
      console.error("Sign in error:", error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const signOut = async () => {
    setIsLoading(true)
    try {
      await logoutViaBFF()
      setUser(null)

      // Redirect to signin
      router.push("/signin")
      router.refresh()
    } catch (error) {
      console.error("Sign out error:", error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const signUp = async (
    username: string,
    password: string,
    telephone: string,
    authCode: string
  ) => {
    setIsLoading(true)
    try {
      const result = await registerViaBFF(
        username,
        password,
        telephone,
        authCode
      )

      if (!result.success) {
        throw new Error(result.error || "Registration failed")
      }

      // Redirect to signin after successful registration
      router.push("/signin?registered=true")
    } catch (error) {
      console.error("Sign up error:", error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const signUpByEmail = async (
    username: string,
    password: string,
    email: string
  ) => {
    setIsLoading(true)
    try {
      const result = await registerByEmailViaBFF(username, password, email)

      if (!result.success) {
        throw new Error(result.error || "Registration failed")
      }

      // Redirect to verify-email page with email parameter
      router.push(`/signup/verify-email?email=${encodeURIComponent(email)}`)
    } catch (error) {
      console.error("Sign up by email error:", error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const getAuthCode = async (telephone: string): Promise<string> => {
    const result = await getAuthCodeViaBFF(telephone)

    if (!result.success) {
      throw new Error(result.error || "Failed to get auth code")
    }

    return result.code || ""
  }

  const refreshUser = async () => {
    try {
      const result = await getCurrentUserViaBFF()

      if (result.success && result.user) {
        setUser({
          id: result.user.id,
          username: result.user.username,
          nickname: result.user.nickname,
          icon: result.user.icon,
        })
      }
    } catch (error) {
      console.error("Failed to refresh user:", error)
    }
  }

  const value = {
    user,
    isLoading,
    isAuthenticated: !!user,
    signIn,
    signOut,
    signUp,
    signUpByEmail,
    getAuthCode,
    refreshUser,
  }

  return (
    <SecureAuthContext.Provider value={value}>
      {children}
    </SecureAuthContext.Provider>
  )
}

export function useSecureAuth() {
  const context = React.useContext(SecureAuthContext)

  if (!context) {
    throw new Error("useSecureAuth must be used within SecureAuthProvider")
  }

  return context
}
