"use client"

import { createContext, useContext, useEffect, useState } from "react"
import { useRouter } from "next/navigation"

import { vielangAuthApi, vielangTokenManager } from "@/lib/api/vielang-auth"
import type { SysMember } from "@/lib/api/vielang-types"

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

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<SysMember | null>(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()

  useEffect(() => {
    // Restore auth state from localStorage on page load
    const initAuth = () => {
      try {
        const savedToken = vielangTokenManager.getToken()
        const savedUser = vielangTokenManager.getUser()

        if (savedToken && savedUser) {
          // Restore token and user without validation
          // Token will be validated on next API call
          setToken(savedToken)
          setUser(savedUser)
        }
      } catch (error) {
        console.error("Auth initialization error:", error)
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
      const authData = await vielangAuthApi.signIn({ username, password })
      const userData = await vielangAuthApi.getCurrentUser()

      setUser(userData)
      setToken(authData.token)
      vielangTokenManager.setToken(authData.token, authData.tokenHead)
      vielangTokenManager.setUser(userData)

      return userData
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
      await vielangAuthApi.signUp({
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
      await vielangAuthApi.signOut()
    } catch (error) {
      console.error("Sign out API error:", error)
    } finally {
      // Clear local state and storage regardless of API call result
      setUser(null)
      setToken(null)
      vielangTokenManager.removeToken()
      vielangTokenManager.removeUser()
      router.push("/signin")
    }
  }

  const refresh = async (): Promise<void> => {
    try {
      if (token) {
        const authData = await vielangAuthApi.refreshToken()
        const userData = await vielangAuthApi.getCurrentUser()
        setUser(userData)
        setToken(authData.token)
        vielangTokenManager.setToken(authData.token, authData.tokenHead)
        vielangTokenManager.setUser(userData)
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
    throw new Error("useAuth must be used within an AuthProvider")
  }
  return context
}
