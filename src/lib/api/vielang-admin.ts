/**
 * Vielang Admin API Client
 * For Admin dashboard (content management, moderation, user management, etc.)
 * Base URL: NEXT_PUBLIC_VIELANG_ADMIN_API_URL
 * Backend Module: vielang-admin (Port 8080)
 * API Prefix: /api/v1/admin
 */

import { env } from "@/env"
import axios, { AxiosInstance } from "axios"

import type { CommonPage, CommonResult } from "@/lib/axios"
import type {
  SysAdmin,
  SysAdminInfo,
  SysRole,
  SysPermission,
  ContentPost,
  ContentCategory,
  ContentTag,
  SocialComment,
  LoginRequest,
  LoginResponse,
  CreatePostRequest,
  UpdatePostRequest,
  PostSearchParams,
  CategoryQueryParam,
  AdminQueryParam,
  RoleQueryParam,
} from "./vielang-types"

// Create dedicated axios instance for admin API
export const adminApi: AxiosInstance = axios.create({
  baseURL: env.NEXT_PUBLIC_VIELANG_ADMIN_API_URL + "/api/v1/admin",
  timeout: 15000,
  headers: {
    "Content-Type": "application/json",
  },
})

// Request interceptor - add auth token
adminApi.interceptors.request.use(
  (config) => {
    if (typeof window !== "undefined") {
      const token = localStorage.getItem("admin_auth_token")
      
      if (token) {
        // Token already includes "Bearer " prefix from adminTokenManager.setToken()
        config.headers.Authorization = token
      }
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor - handle errors
adminApi.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      console.error("[adminApi] 401 Unauthorized")

      if (typeof window !== "undefined") {
        localStorage.removeItem("admin_auth_token")
        localStorage.removeItem("admin_user")

        const currentPath = window.location.pathname
        if (!currentPath.includes("/admin/login")) {
          window.location.href = "/admin/login"
        }
      }
    }
    return Promise.reject(error)
  }
)

// ============================================
// ADMIN USER MANAGEMENT API
// Base: /api/v1/admin
// ============================================

export const adminAuthApi = {
  /**
   * Admin Registration
   * POST /api/v1/admin/register
   */
  register: async (data: {
    username: string
    password: string
    email: string
    nickname?: string
  }): Promise<CommonResult<SysAdmin>> => {
    const response = await adminApi.post<CommonResult<SysAdmin>>(
      "/register",
      data
    )
    return response.data
  },

  /**
   * Admin Login
   * POST /api/v1/admin/login
   */
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    const response = await adminApi.post<CommonResult<LoginResponse>>(
      "/login",
      credentials
    )

    if (response.data.code === 200 && response.data.data) {
      const { token, tokenHead } = response.data.data

      if (typeof window !== "undefined") {
        localStorage.setItem("admin_auth_token", token)
        localStorage.setItem("admin_token_head", tokenHead)
      }
    }

    return response.data.data
  },

  /**
   * Refresh JWT Token
   * GET /api/v1/admin/refreshToken
   */
  refreshToken: async (): Promise<{ token: string; tokenHead: string }> => {
    const response = await adminApi.get<
      CommonResult<{ token: string; tokenHead: string }>
    >("/refreshToken")
    return response.data.data
  },

  /**
   * Get Current Admin Info
   * GET /api/v1/admin/info
   */
  getCurrentAdmin: async (): Promise<SysAdminInfo> => {
    const response = await adminApi.get<CommonResult<SysAdminInfo>>("/info")
    return response.data.data
  },

  /**
   * Admin Logout
   * POST /api/v1/admin/logout
   */
  logout: async (): Promise<void> => {
    await adminApi.post("/logout")

    if (typeof window !== "undefined") {
      localStorage.removeItem("admin_auth_token")
      localStorage.removeItem("admin_token_head")
      localStorage.removeItem("admin_user")
    }
  },

  /**
   * Update Admin Password
   * POST /api/v1/admin/updatePassword
   */
  updatePassword: async (data: {
    username: string
    oldPassword: string
    newPassword: string
  }): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/updatePassword",
      data
    )
    return response.data
  },
}

export const adminUserApi = {
  /**
   * List Admins (Paginated)
   * GET /api/v1/admin/list
   */
  getList: async (params: AdminQueryParam): Promise<CommonPage<SysAdmin>> => {
    const queryParams = new URLSearchParams()

    if (params.keyword) queryParams.append("keyword", params.keyword)
    if (params.status !== undefined)
      queryParams.append("status", params.status.toString())
    if (params.pageNum !== undefined)
      queryParams.append("pageNum", params.pageNum.toString())
    if (params.pageSize !== undefined)
      queryParams.append("pageSize", params.pageSize.toString())

    const response = await adminApi.get<CommonResult<CommonPage<SysAdmin>>>(
      `/list?${queryParams.toString()}`
    )
    return response.data.data
  },

  /**
   * Get Admin by ID
   * GET /api/v1/admin/{id}
   */
  getById: async (id: number): Promise<SysAdmin> => {
    const response = await adminApi.get<CommonResult<SysAdmin>>(`/${id}`)
    return response.data.data
  },

  /**
   * Update Admin Info
   * POST /api/v1/admin/update/{id}
   */
  update: async (
    id: number,
    data: Partial<SysAdmin>
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/update/${id}`,
      data
    )
    return response.data
  },

  /**
   * Delete Admin
   * POST /api/v1/admin/delete/{id}
   */
  delete: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(`/delete/${id}`)
    return response.data
  },

  /**
   * Update Admin Status
   * POST /api/v1/admin/updateStatus/{id}
   * Request body: { status: 0 | 1 }
   */
  updateStatus: async (
    id: number,
    status: number
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/updateStatus/${id}`,
      { status }
    )
    return response.data
  },

  /**
   * Assign Roles to Admin
   * POST /api/v1/admin/role/update
   * Request body: { adminId: number, roleIds: number[] }
   */
  assignRoles: async (
    adminId: number,
    roleIds: number[]
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>("/role/update", {
      adminId,
      roleIds,
    })
    return response.data
  },

  /**
   * Get Admin's Roles
   * GET /api/v1/admin/role/{adminId}
   */
  getRoles: async (adminId: number): Promise<SysRole[]> => {
    const response = await adminApi.get<CommonResult<SysRole[]>>(
      `/role/${adminId}`
    )
    return response.data.data
  },
}

// ============================================
// ROLE MANAGEMENT API (RBAC)
// Base: /api/v1/admin/role
// ============================================

export const adminRoleApi = {
  /**
   * Create Role
   * POST /api/v1/admin/role/create
   */
  create: async (data: {
    name: string
    code: string
    description?: string
    sort?: number
  }): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/role/create",
      data
    )
    return response.data
  },

  /**
   * Update Role
   * POST /api/v1/admin/role/update/{id}
   */
  update: async (
    id: number,
    data: Partial<SysRole>
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/role/update/${id}`,
      data
    )
    return response.data
  },

  /**
   * Delete Roles (Batch)
   * POST /api/v1/admin/role/delete
   * Request body: { ids: number[] }
   */
  delete: async (ids: number[]): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>("/role/delete", {
      ids,
    })
    return response.data
  },

  /**
   * Get All Roles
   * GET /api/v1/admin/role/listAll
   */
  getAll: async (): Promise<SysRole[]> => {
    const response = await adminApi.get<CommonResult<SysRole[]>>(
      "/role/listAll"
    )
    return response.data.data
  },

  /**
   * Get Paginated Roles
   * GET /api/v1/admin/role/list
   */
  getList: async (params: RoleQueryParam): Promise<CommonPage<SysRole>> => {
    const queryParams = new URLSearchParams()

    if (params.keyword) queryParams.append("keyword", params.keyword)
    if (params.status !== undefined)
      queryParams.append("status", params.status.toString())
    if (params.pageNum !== undefined)
      queryParams.append("pageNum", params.pageNum.toString())
    if (params.pageSize !== undefined)
      queryParams.append("pageSize", params.pageSize.toString())

    const response = await adminApi.get<CommonResult<CommonPage<SysRole>>>(
      `/role/list?${queryParams.toString()}`
    )
    return response.data.data
  },

  /**
   * Update Role Status
   * POST /api/v1/admin/role/updateStatus/{id}
   */
  updateStatus: async (
    id: number,
    status: number
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/role/updateStatus/${id}`,
      { status }
    )
    return response.data
  },

  /**
   * Get Role's Menu Permissions
   * GET /api/v1/admin/role/listMenu/{roleId}
   */
  getMenuPermissions: async (roleId: number): Promise<SysPermission[]> => {
    const response = await adminApi.get<CommonResult<SysPermission[]>>(
      `/role/listMenu/${roleId}`
    )
    return response.data.data
  },

  /**
   * Get Role's Resource Permissions
   * GET /api/v1/admin/role/listResource/{roleId}
   */
  getResourcePermissions: async (roleId: number): Promise<SysPermission[]> => {
    const response = await adminApi.get<CommonResult<SysPermission[]>>(
      `/role/listResource/${roleId}`
    )
    return response.data.data
  },

  /**
   * Assign Menus to Role
   * POST /api/v1/admin/role/allocMenu
   * Request body: { roleId: number, menuIds: number[] }
   */
  assignMenus: async (
    roleId: number,
    menuIds: number[]
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/role/allocMenu",
      { roleId, menuIds }
    )
    return response.data
  },

  /**
   * Assign Resources to Role
   * POST /api/v1/admin/role/allocResource
   * Request body: { roleId: number, resourceIds: number[] }
   */
  assignResources: async (
    roleId: number,
    resourceIds: number[]
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/role/allocResource",
      { roleId, resourceIds }
    )
    return response.data
  },
}

// ============================================
// CATEGORY MANAGEMENT API
// Base: /api/v1/admin/categories
// ============================================

export const adminCategoryApi = {
  /**
   * Create Category
   * POST /api/v1/admin/categories/create
   */
  create: async (data: {
    name: string
    slug: string
    parentId?: number
    description?: string
    icon?: string
    coverImage?: string
    sort?: number
  }): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/categories/create",
      data
    )
    return response.data
  },

  /**
   * Update Category
   * POST /api/v1/admin/categories/update/{id}
   */
  update: async (
    id: number,
    data: Partial<ContentCategory>
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/categories/update/${id}`,
      data
    )
    return response.data
  },

  /**
   * Delete Category
   * POST /api/v1/admin/categories/delete/{id}
   */
  delete: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/categories/delete/${id}`
    )
    return response.data
  },

  /**
   * Get Category by ID
   * GET /api/v1/admin/categories/{id}
   */
  getById: async (id: number): Promise<ContentCategory> => {
    const response = await adminApi.get<CommonResult<ContentCategory>>(
      `/categories/${id}`
    )
    return response.data.data
  },

  /**
   * Get All Categories
   * GET /api/v1/admin/categories/list
   */
  getAll: async (): Promise<ContentCategory[]> => {
    const response = await adminApi.get<CommonResult<ContentCategory[]>>(
      "/categories/list"
    )
    return response.data.data
  },

  /**
   * Get Categories by Parent ID
   * GET /api/v1/admin/categories/list/{parentId}
   */
  getByParentId: async (parentId: number): Promise<ContentCategory[]> => {
    const response = await adminApi.get<CommonResult<ContentCategory[]>>(
      `/categories/list/${parentId}`
    )
    return response.data.data
  },

  /**
   * Update Navigation Status
   * POST /api/v1/admin/categories/updateNavStatus?id={id}&isNav={isNav}
   * Uses query parameters, not request body
   */
  updateNavStatus: async (
    ids: number[],
    isNav: number
  ): Promise<CommonResult<number>> => {
    // Backend expects single id as query param, but we keep array for compatibility
    const id = ids[0]
    const response = await adminApi.post<CommonResult<number>>(
      `/categories/updateNavStatus?id=${id}&isNav=${isNav}`
    )
    return response.data
  },

  /**
   * Update Category Status
   * POST /api/v1/admin/categories/updateStatus?id={id}&status={status}
   * Uses query parameters, not request body
   */
  updateStatus: async (
    ids: number[],
    status: number
  ): Promise<CommonResult<number>> => {
    // Backend expects single id as query param, but we keep array for compatibility
    const id = ids[0]
    const response = await adminApi.post<CommonResult<number>>(
      `/categories/updateStatus?id=${id}&status=${status}`
    )
    return response.data
  },
}

// ============================================
// POST MANAGEMENT API
// Base: /api/v1/admin/posts
// ============================================

export const adminPostApi = {
  /**
   * Create Post
   * POST /api/v1/admin/posts/create
   */
  create: async (data: CreatePostRequest): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/posts/create",
      data
    )
    return response.data
  },

  /**
   * Update Post
   * POST /api/v1/admin/posts/update/{id}
   */
  update: async (
    id: number,
    data: Partial<CreatePostRequest>
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/posts/update/${id}`,
      data
    )
    return response.data
  },

  /**
   * Delete Post
   * POST /api/v1/admin/posts/delete/{id}
   */
  delete: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/posts/delete/${id}`
    )
    return response.data
  },

  /**
   * Get Post by ID
   * GET /api/v1/admin/posts/{id}
   */
  getById: async (id: number): Promise<ContentPost> => {
    const response = await adminApi.get<CommonResult<ContentPost>>(
      `/posts/${id}`
    )
    return response.data.data
  },

  /**
   * List Posts (Filtered & Paginated)
   * GET /api/v1/admin/posts/list
   * Query params: categoryId, status, keyword, pageNum, pageSize
   */
  getList: async (params: PostSearchParams): Promise<CommonPage<ContentPost>> => {
    const queryParams = new URLSearchParams()

    if (params.categoryId)
      queryParams.append("categoryId", params.categoryId.toString())
    if (params.status !== undefined)
      queryParams.append("status", params.status.toString())
    if (params.keyword) queryParams.append("keyword", params.keyword)
    if (params.pageNum !== undefined)
      queryParams.append("pageNum", params.pageNum.toString())
    if (params.pageSize !== undefined)
      queryParams.append("pageSize", params.pageSize.toString())

    const response = await adminApi.get<CommonResult<CommonPage<ContentPost>>>(
      `/posts/list?${queryParams.toString()}`
    )
    return response.data.data
  },

  /**
   * Publish Post
   * POST /api/v1/admin/posts/publish/{id}
   */
  publish: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/posts/publish/${id}`
    )
    return response.data
  },

  /**
   * Update Post Status
   * POST /api/v1/admin/posts/updateStatus
   * Request body: { ids: number[], status: 0 | 1 | 2 } (0:Draft, 1:Published, 2:Archived)
   */
  updateStatus: async (
    ids: number[],
    status: number
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/posts/updateStatus",
      { ids, status }
    )
    return response.data
  },

  /**
   * Update Featured Status
   * POST /api/v1/admin/posts/updateFeatured
   * Request body: { ids: number[], isFeatured: 0 | 1 }
   */
  updateFeatured: async (
    ids: number[],
    isFeatured: number
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/posts/updateFeatured",
      { ids, isFeatured }
    )
    return response.data
  },

  /**
   * Update Pinned Status
   * POST /api/v1/admin/posts/updatePinned
   * Request body: { ids: number[], isPinned: 0 | 1 }
   */
  updatePinned: async (
    ids: number[],
    isPinned: number
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/posts/updatePinned",
      { ids, isPinned }
    )
    return response.data
  },
}

// ============================================
// COMMENT MODERATION API
// Base: /api/v1/admin/comments
// ============================================

export const adminCommentApi = {
  /**
   * List Comments (Filtered & Paginated)
   * GET /api/v1/admin/comments/list
   * Query params: postId, status, pageNum, pageSize
   */
  getList: async (params: {
    postId?: number
    status?: number
    pageNum?: number
    pageSize?: number
  }): Promise<CommonPage<SocialComment>> => {
    const queryParams = new URLSearchParams()

    if (params.postId !== undefined)
      queryParams.append("postId", params.postId.toString())
    if (params.status !== undefined)
      queryParams.append("status", params.status.toString())
    if (params.pageNum !== undefined)
      queryParams.append("pageNum", params.pageNum.toString())
    if (params.pageSize !== undefined)
      queryParams.append("pageSize", params.pageSize.toString())

    const response = await adminApi.get<
      CommonResult<CommonPage<SocialComment>>
    >(`/comments/list?${queryParams.toString()}`)
    return response.data.data
  },

  /**
   * Get Comment by ID
   * GET /api/v1/admin/comments/{id}
   */
  getById: async (id: number): Promise<SocialComment> => {
    const response = await adminApi.get<CommonResult<SocialComment>>(
      `/comments/${id}`
    )
    return response.data.data
  },

  /**
   * Approve Comment (status=1)
   * POST /api/v1/admin/comments/approve/{id}
   */
  approve: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/comments/approve/${id}`
    )
    return response.data
  },

  /**
   * Hide Comment (status=0)
   * POST /api/v1/admin/comments/hide/{id}
   */
  hide: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/comments/hide/${id}`
    )
    return response.data
  },

  /**
   * Mark Comment as Reported (status=2)
   * POST /api/v1/admin/comments/report/{id}
   */
  markAsReported: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/comments/report/${id}`
    )
    return response.data
  },

  /**
   * Delete Comment
   * POST /api/v1/admin/comments/delete/{id}
   */
  delete: async (id: number): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      `/comments/delete/${id}`
    )
    return response.data
  },

  /**
   * Update Comment Status
   * POST /api/v1/admin/comments/updateStatus
   * Request body: { ids: number[], status: 0 | 1 | 2 }
   */
  updateStatus: async (
    ids: number[],
    status: number
  ): Promise<CommonResult<number>> => {
    const response = await adminApi.post<CommonResult<number>>(
      "/comments/updateStatus",
      { ids, status }
    )
    return response.data
  },
}

// ============================================
// UNIFIED ADMIN API EXPORT
// ============================================

export default {
  auth: adminAuthApi,
  user: adminUserApi,
  role: adminRoleApi,
  category: adminCategoryApi,
  post: adminPostApi,
  comment: adminCommentApi,
}
