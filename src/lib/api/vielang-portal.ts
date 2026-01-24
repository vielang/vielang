/**
 * Vielang Portal API Client
 * For Member-facing frontend (posts browsing, social interactions, profile, etc.)
 * Base URL: NEXT_PUBLIC_VIELANG_PORTAL_API_URL
 * Backend Module: vielang-portal (Port 8085)
 * API Prefix: /api/v1/portal
 */

import { env } from "@/env"
import axios, { AxiosInstance } from "axios"

import type { CommonPage, CommonResult } from "@/lib/axios"
import type {
  SysMember,
  ContentPost,
  ContentCategory,
  SocialComment,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  CreateCommentRequest,
  PostSearchParams,
} from "./vielang-types"

// Create dedicated axios instance for portal API
export const portalApi: AxiosInstance = axios.create({
  baseURL: env.NEXT_PUBLIC_VIELANG_PORTAL_API_URL + "/api/v1/portal",
  timeout: 15000,
  headers: {
    "Content-Type": "application/json",
  },
})

// Request interceptor - add auth token from localStorage
portalApi.interceptors.request.use(
  (config) => {
    if (typeof window !== "undefined") {
      const token = localStorage.getItem("member_auth_token")
      const tokenHead = localStorage.getItem("member_token_head") || "Bearer "

      if (token) {
        config.headers.Authorization = `${tokenHead}${token}`
        console.log("[portalApi] Adding Authorization header for:", config.url)
      }
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor - handle errors
portalApi.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      console.error("[portalApi] 401 Unauthorized for:", error.config?.url)

      const errorMessage = error.response?.data?.message || ""
      const hasToken = !!localStorage.getItem("member_auth_token")

      const isAuthenticationError =
        errorMessage.includes("token has expired") ||
        errorMessage.includes("token is invalid") ||
        errorMessage.includes("Not logged in") ||
        errorMessage.includes("未登录") ||
        !hasToken

      if (isAuthenticationError && typeof window !== "undefined") {
        localStorage.removeItem("member_auth_token")
        localStorage.removeItem("member_token_head")
        localStorage.removeItem("member_user")

        const currentPath = window.location.pathname
        if (
          !currentPath.includes("/signin") &&
          !currentPath.includes("/signup")
        ) {
          window.location.href = "/signin"
        }
      }
    }
    return Promise.reject(error)
  }
)

// ============================================
// MEMBER AUTHENTICATION API
// Base: /api/v1/portal/member
// ============================================

export const portalAuthApi = {
  /**
   * Member Registration
   * POST /api/v1/portal/member/register
   */
  register: async (data: RegisterRequest): Promise<CommonResult<SysMember>> => {
    const response = await portalApi.post<CommonResult<SysMember>>(
      "/member/register",
      data
    )
    return response.data
  },

  /**
   * Member Login
   * POST /api/v1/portal/member/login
   */
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    const response = await portalApi.post<CommonResult<LoginResponse>>(
      "/member/login",
      credentials
    )

    if (response.data.code === 200 && response.data.data) {
      const { token, tokenHead } = response.data.data

      if (typeof window !== "undefined") {
        localStorage.setItem("member_auth_token", token)
        localStorage.setItem("member_token_head", tokenHead)
      }
    }

    return response.data.data
  },

  /**
   * Refresh JWT Token
   * GET /api/v1/portal/member/refreshToken
   */
  refreshToken: async (): Promise<{ token: string; tokenHead: string }> => {
    const response = await portalApi.get<
      CommonResult<{ token: string; tokenHead: string }>
    >("/member/refreshToken")
    return response.data.data
  },

  /**
   * Get Current Member Info
   * GET /api/v1/portal/member/info
   */
  getCurrentMember: async (): Promise<SysMember> => {
    const response = await portalApi.get<CommonResult<SysMember>>(
      "/member/info"
    )
    return response.data.data
  },

  /**
   * Update Member Profile
   * POST /api/v1/portal/member/update
   */
  updateProfile: async (
    data: Partial<SysMember>
  ): Promise<CommonResult<number>> => {
    const response = await portalApi.post<CommonResult<number>>(
      "/member/update",
      data
    )
    return response.data
  },

  /**
   * Update Member Password
   * POST /api/v1/portal/member/updatePassword
   */
  updatePassword: async (data: {
    oldPassword: string
    newPassword: string
  }): Promise<CommonResult<number>> => {
    const response = await portalApi.post<CommonResult<number>>(
      "/member/updatePassword",
      data
    )
    return response.data
  },

  /**
   * Member Logout
   * POST /api/v1/portal/member/logout
   */
  logout: async (): Promise<void> => {
    await portalApi.post("/member/logout")

    if (typeof window !== "undefined") {
      localStorage.removeItem("member_auth_token")
      localStorage.removeItem("member_token_head")
      localStorage.removeItem("member_user")
    }
  },
}

// ============================================
// POST BROWSING API (PUBLIC)
// Base: /api/v1/portal/posts
// ============================================

export const portalPostApi = {
  /**
   * List Published Posts (PUBLIC - No auth required)
   * GET /api/v1/portal/posts/list
   * Query params: categoryId, keyword, sortBy, pageNum, pageSize
   */
  getList: async (params: PostSearchParams): Promise<CommonPage<ContentPost>> => {
    const queryParams = new URLSearchParams()

    if (params.categoryId)
      queryParams.append("categoryId", params.categoryId.toString())
    if (params.keyword) queryParams.append("keyword", params.keyword)
    if (params.sortBy) queryParams.append("sortBy", params.sortBy)
    if (params.pageNum !== undefined)
      queryParams.append("pageNum", params.pageNum.toString())
    if (params.pageSize !== undefined)
      queryParams.append("pageSize", params.pageSize.toString())

    const response = await portalApi.get<CommonResult<CommonPage<ContentPost>>>(
      `/posts/list?${queryParams.toString()}`
    )
    return response.data.data
  },

  /**
   * Get Post Detail (PUBLIC - increments view count)
   * GET /api/v1/portal/posts/{id}
   */
  getDetail: async (id: number): Promise<ContentPost> => {
    const response = await portalApi.get<CommonResult<ContentPost>>(
      `/posts/${id}`
    )
    return response.data.data
  },

  /**
   * Get Trending Posts
   * GET /api/v1/portal/posts/trending
   */
  getTrending: async (): Promise<ContentPost[]> => {
    const response = await portalApi.get<CommonResult<ContentPost[]>>(
      "/posts/trending"
    )
    return response.data.data
  },
}

// ============================================
// SOCIAL INTERACTIONS API (LIKE & COMMENT)
// Base: /api/v1/portal/social
// ============================================

export const portalSocialApi = {
  /**
   * Like a Post
   * POST /api/v1/portal/social/like/post/{postId}
   * Requires authentication
   */
  likePost: async (postId: number): Promise<CommonResult<void>> => {
    const response = await portalApi.post<CommonResult<void>>(
      `/social/like/post/${postId}`
    )
    return response.data
  },

  /**
   * Unlike a Post
   * POST /api/v1/portal/social/unlike/post/{postId}
   * Requires authentication
   */
  unlikePost: async (postId: number): Promise<CommonResult<void>> => {
    const response = await portalApi.post<CommonResult<void>>(
      `/social/unlike/post/${postId}`
    )
    return response.data
  },

  /**
   * Check if Post is Liked
   * GET /api/v1/portal/social/like/post/{postId}/status
   * Returns boolean (true if liked)
   */
  checkLikeStatus: async (postId: number): Promise<boolean> => {
    const response = await portalApi.get<CommonResult<boolean>>(
      `/social/like/post/${postId}/status`
    )
    return response.data.data
  },

  /**
   * Add Comment to Post
   * POST /api/v1/portal/social/comment/post/{postId}
   * Request body: { content: string }
   */
  addComment: async (
    postId: number,
    content: string
  ): Promise<CommonResult<SocialComment>> => {
    const response = await portalApi.post<CommonResult<SocialComment>>(
      `/social/comment/post/${postId}`,
      { content }
    )
    return response.data
  },

  /**
   * Reply to Comment
   * POST /api/v1/portal/social/comment/reply/{parentId}
   * Request body: { content: string }
   */
  replyToComment: async (
    parentId: number,
    content: string
  ): Promise<CommonResult<SocialComment>> => {
    const response = await portalApi.post<CommonResult<SocialComment>>(
      `/social/comment/reply/${parentId}`,
      { content }
    )
    return response.data
  },

  /**
   * Delete Own Comment
   * POST /api/v1/portal/social/comment/delete/{commentId}
   */
  deleteComment: async (commentId: number): Promise<CommonResult<void>> => {
    const response = await portalApi.post<CommonResult<void>>(
      `/social/comment/delete/${commentId}`
    )
    return response.data
  },

  /**
   * Get Post Comments (PUBLIC)
   * GET /api/v1/portal/social/comments/post/{postId}
   * Returns nested comment tree with replies
   */
  getPostComments: async (postId: number): Promise<SocialComment[]> => {
    const response = await portalApi.get<CommonResult<SocialComment[]>>(
      `/social/comments/post/${postId}`
    )
    return response.data.data
  },
}

// ============================================
// CATEGORY API (PUBLIC)
// Base: /api/v1/portal/categories
// ============================================

export const portalCategoryApi = {
  /**
   * List Active Categories (PUBLIC - No auth required)
   * GET /api/v1/portal/categories
   * Query params: limit (optional, default 20)
   */
  getList: async (limit?: number): Promise<ContentCategory[]> => {
    const params = limit ? { limit } : {}
    const response = await portalApi.get<CommonResult<ContentCategory[]>>(
      "/categories",
      { params }
    )
    return response.data.data
  },
}

// ============================================
// UNIFIED PORTAL API EXPORT
// ============================================

export default {
  auth: portalAuthApi,
  post: portalPostApi,
  social: portalSocialApi,
  category: portalCategoryApi,
}
