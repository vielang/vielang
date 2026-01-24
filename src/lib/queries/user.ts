import "server-only"

import { cache } from "react"
import { unstable_noStore as noStore } from "next/cache"
import { cookies } from "next/headers"

import type { SysMember } from "@/lib/api/vielang-types"

/**
 * User queries for Vielang Backend
 *
 * NOTE: Vielang is a social platform with different concepts than PocketBase:
 * - Users are members of the social platform
 * - No e-commerce stores or subscription plans
 * - User data is managed through SysMember
 */

export interface User {
  id: number
  username: string
  phone?: string
  email?: string
  icon?: string
}

/**
 * Get cached user from server-side
 * Cache is used with a data-fetching function like fetch to share a data snapshot between components.
 * @see https://react.dev/reference/react/cache#reference
 */
export const getCachedUser = cache(async (): Promise<User | null> => {
  try {
    const cookieStore = await cookies()
    const authToken = cookieStore.get("auth_token")
    const userCookie = cookieStore.get("user")

    if (!authToken?.value || !userCookie?.value) {
      return null
    }

    // Parse user data from cookie
    try {
      const userData = JSON.parse(decodeURIComponent(userCookie.value)) as User
      return userData
    } catch (error) {
      console.error("Failed to parse user data from cookie:", error)
      return null
    }
  } catch (error) {
    console.error("Server-side auth check failed:", error)
    return null
  }
})

/**
 * Get user usage metrics
 * Vielang backend doesn't have stores concept, so this returns default values
 */
export async function getUserUsageMetrics(input: { userId: string }) {
  noStore()
  try {
    // Vielang backend doesn't have stores or usage tracking
    // Return default values
    return {
      storeCount: 0,
      productCount: 0,
    }
  } catch (error) {
    console.error("Error fetching user usage metrics:", error)
    return {
      storeCount: 0,
      productCount: 0,
    }
  }
}

/**
 * Get user plan metrics
 * Vielang backend doesn't have subscription plans, returns default free plan
 */
export async function getUserPlanMetrics(input: { userId: string }) {
  noStore()

  const fallback = {
    storeCount: 0,
    storeLimit: 0,
    productCount: 0,
    productLimit: 0,
    storeLimitExceeded: false,
    productLimitExceeded: false,
    subscriptionPlan: null,
  }

  try {
    // Vielang backend doesn't have subscription plans
    // Return default values
    return fallback
  } catch (error) {
    console.error("Error fetching user plan metrics:", error)
    return fallback
  }
}
