"use client"

import { createContext, useContext, useEffect, useState } from "react"
import { useRouter } from "next/navigation"

import { adminAuthApi } from "@/lib/api/vielang-index"
import type { SysAdmin, SysPermission } from "@/lib/api/vielang-types"

interface AdminAuthState {
  admin: (SysAdmin & { roles?: string[]; menus?: SysPermission[] }) | null
  token: string | null
  isLoading: boolean
}

interface AdminAuthContextType extends AdminAuthState {
  signIn: (username: string, password: string) => Promise<void>
  signOut: () => Promise<void>
  refresh: () => Promise<void>
}

const AdminAuthContext = createContext<AdminAuthContextType | undefined>(
  undefined
)

// Token management utilities
const ADMIN_TOKEN_KEY = "admin_auth_token"
const ADMIN_USER_KEY = "admin_user"

const adminTokenManager = {
  getToken: (): string | null => {
    if (typeof window === "undefined") return null
    return localStorage.getItem(ADMIN_TOKEN_KEY)
  },

  setToken: (token: string, tokenHead: string = "Bearer "): void => {
    if (typeof window === "undefined") return
    const fullToken = tokenHead + token
    localStorage.setItem(ADMIN_TOKEN_KEY, fullToken)
  },

  removeToken: (): void => {
    if (typeof window === "undefined") return
    localStorage.removeItem(ADMIN_TOKEN_KEY)
  },

  getAdmin: (): (SysAdmin & { roles?: string[]; menus?: SysPermission[] }) | null => {
    if (typeof window === "undefined") return null
    const adminStr = localStorage.getItem(ADMIN_USER_KEY)
    return adminStr
      ? (JSON.parse(adminStr) as SysAdmin & {
          roles?: string[]
          menus?: SysPermission[]
        })
      : null
  },

  setAdmin: (
    admin: SysAdmin & { roles?: string[]; menus?: SysPermission[] }
  ): void => {
    if (typeof window === "undefined") return
    localStorage.setItem(ADMIN_USER_KEY, JSON.stringify(admin))
  },

  removeAdmin: (): void => {
    if (typeof window === "undefined") return
    localStorage.removeItem(ADMIN_USER_KEY)
  },
}

export function AdminAuthProvider({ children }: { children: React.ReactNode }) {
  const [admin, setAdmin] = useState<
    (SysAdmin & { roles?: string[]; menus?: SysPermission[] }) | null
  >(null)
  const [token, setToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const router = useRouter()

  useEffect(() => {
    // Restore auth state from localStorage on page load
    const initAuth = () => {
      try {
        const savedToken = adminTokenManager.getToken()
        const savedAdmin = adminTokenManager.getAdmin()

        if (savedToken && savedAdmin) {
          // Restore token and admin without validation
          // Token will be validated on next API call
          setToken(savedToken)
          setAdmin(savedAdmin)
        }
      } catch (error) {
        console.error("Auth initialization error:", error)
      } finally {
        setIsLoading(false)
      }
    }

    initAuth()
  }, [])

  const signIn = async (username: string, password: string): Promise<void> => {
    try {
      // Step 1: Login to get token
      const { token: newToken, tokenHead } = await adminAuthApi.login({
        username,
        password,
      })

      setToken(newToken)
      adminTokenManager.setToken(newToken, tokenHead)

      // Step 2: Get admin info (including roles and menus)
      const adminInfo = await adminAuthApi.getCurrentAdmin()

      const adminData: SysAdmin & { roles?: string[]; menus?: SysPermission[] } = {
        id: 0, // Will be filled from actual response if available
        username: adminInfo.username,
        email: "", // Not provided by /info endpoint
        status: 1,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        avatar: adminInfo.icon,
        roles: adminInfo.roles,
        menus: adminInfo.menus,
      }

      setAdmin(adminData)
      adminTokenManager.setAdmin(adminData)
    } catch (error) {
      // Clean up on error
      adminTokenManager.removeToken()
      adminTokenManager.removeAdmin()
      setAdmin(null)
      setToken(null)
      throw error
    }
  }

  const signOut = async (): Promise<void> => {
    try {
      await adminAuthApi.logout()
    } catch (error) {
      console.error("Sign out error:", error)
    } finally {
      // Clear local state and storage regardless of API call result
      setAdmin(null)
      setToken(null)
      adminTokenManager.removeToken()
      adminTokenManager.removeAdmin()
      router.push("/admin/login")
    }
  }

  const refresh = async (): Promise<void> => {
    try {
      if (token) {
        const { token: newToken, tokenHead } = await adminAuthApi.refreshToken()
        setToken(newToken)
        adminTokenManager.setToken(newToken, tokenHead)

        // Fetch updated admin info
        const adminInfo = await adminAuthApi.getCurrentAdmin()
        const updatedAdmin: SysAdmin & { roles?: string[]; menus?: SysPermission[] } = {
          ...admin,
          id: admin?.id || 0,
          email: admin?.email || "",
          status: admin?.status || 1,
          createdAt: admin?.createdAt || new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          username: adminInfo.username,
          avatar: adminInfo.icon,
          roles: adminInfo.roles,
          menus: adminInfo.menus,
        }

        setAdmin(updatedAdmin)
        adminTokenManager.setAdmin(updatedAdmin)
      }
    } catch (error) {
      console.error("Auth refresh failed:", error)
      await signOut()
    }
  }

  return (
    <AdminAuthContext.Provider
      value={{
        admin,
        token,
        isLoading,
        signIn,
        signOut,
        refresh,
      }}
    >
      {children}
    </AdminAuthContext.Provider>
  )
}

export function useAdminAuth() {
  const context = useContext(AdminAuthContext)
  if (context === undefined) {
    throw new Error("useAdminAuth must be used within an AdminAuthProvider")
  }
  return context
}

// Export token manager for use in API interceptors
export { adminTokenManager }
