import "server-only"

import { cookies } from "next/headers"
import { env } from "@/env"
import axios from "axios"

/**
 * Server-side API instance for Vielang Portal
 * Note: Store-related functionality has been disabled as vielang backend is a social platform
 */
export const createServerApi = async () => {
  const instance = axios.create({
    baseURL: env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL + "/api/v1/portal",
    timeout: 10000,
    headers: {
      "Content-Type": "application/json",
    },
  })

  // Get auth token from cookies
  try {
    const cookieStore = await cookies()
    const authToken = cookieStore.get("auth_token")
    const tokenHead = cookieStore.get("token_head")

    if (authToken?.value) {
      // Combine tokenHead and token for Authorization header
      const head = tokenHead?.value || "Bearer "
      instance.defaults.headers.Authorization = `${head}${authToken.value}`
    }
  } catch (error) {
    console.warn("Could not access cookies in server API:", error)
  }

  return instance
}

/**
 * Server-side stores API - Disabled
 * Vielang backend doesn't support multi-vendor/multi-store functionality
 * These functions are kept for compatibility but will return empty results
 */
export const serverStoresApi = {
  getUserStores: async (userId: string) => {
    console.warn(
      "Store functionality is disabled - vielang backend is a social platform"
    )
    return []
  },

  getStore: async (storeId: string) => {
    console.warn(
      "Store functionality is disabled - vielang backend is a social platform"
    )
    return null
  },

  getStoreProducts: async (
    storeId: string,
    page = 1,
    perPage = 10,
    filters: any = {}
  ) => {
    console.warn(
      "Store functionality is disabled - vielang backend is a social platform"
    )
    return { items: [], totalItems: 0, page: 1, perPage: 10, totalPages: 0 }
  },
}
